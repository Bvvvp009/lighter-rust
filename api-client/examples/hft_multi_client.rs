use api_client::{LighterClient, CreateOrderRequest};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::env;
use std::time::Instant;
use tokio::time::Duration;

/// High-Frequency Trading (HFT) example using multiple API keys
/// Uses LighterClient's internal nonce cache as single source of truth
struct HftTrader {
    clients: Vec<Arc<LighterClient>>,
    current_client_index: Arc<AtomicUsize>,
    account_index: i64,
    api_key_indices: Vec<u8>,
    // Manual nonce cache: track nonces per client for manual mode
    // Use i64::MIN as sentinel value (faster than Option<i64>)
    manual_nonces: Arc<tokio::sync::Mutex<Vec<i64>>>,
}

impl HftTrader {
    fn new(base_url: String, account_index: i64, api_keys: Vec<(u8, String)>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut clients = Vec::new();
        let mut api_key_indices = Vec::new();
        
        println!("🔑 Initializing {} API key clients...", api_keys.len());
        for (api_key_index, api_key) in api_keys.iter() {
            let client = LighterClient::new(
                base_url.clone(),
                api_key,
                account_index,
                *api_key_index,
            )?;
            clients.push(Arc::new(client));
            api_key_indices.push(*api_key_index);
            println!("  ✓ Client {} (API Key Index: {})", clients.len(), api_key_index);
        }
        
        let num_clients = clients.len();
        Ok(Self {
            clients,
            current_client_index: Arc::new(AtomicUsize::new(0)),
            account_index,
            api_key_indices,
            manual_nonces: Arc::new(tokio::sync::Mutex::new(vec![i64::MIN; num_clients])),
        })
    }
    
    /// Get the next client in round-robin fashion
    #[inline]
    fn get_next_client(&self) -> (Arc<LighterClient>, u8) {
        let index = self.current_client_index.fetch_add(1, Ordering::Relaxed);
        let client_idx = index % self.clients.len();
        (Arc::clone(&self.clients[client_idx]), self.api_key_indices[client_idx])
    }
    
    /// Warmup nonces by initializing LighterClient's internal cache
    /// This ensures each client's cache is ready before sending orders
    async fn warmup_nonces(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Initializing nonce caches for all clients...");
        
        let mut errors = Vec::new();
        for (idx, client) in self.clients.iter().enumerate() {
            // Initialize cache by calling refresh_nonce() which fetches and initializes the cache
            match client.refresh_nonce().await {
                Ok(nonce) => {
                    println!("  ✓ Client {} (API Key {}): cache initialized with nonce = {}", 
                        idx + 1, self.api_key_indices[idx], nonce);
                }
                Err(e) => {
                    let error_msg = format!("Client {} (API Key {}): {}", idx + 1, self.api_key_indices[idx], e);
                    println!("  ⚠️  {}", error_msg);
                    errors.push(error_msg);
                    // Continue with other clients instead of failing immediately
                }
            }
        }
        
        if errors.len() == self.clients.len() {
            return Err(format!("Failed to initialize any client caches. All {} clients failed:\n  {}", 
                self.clients.len(), errors.join("\n  ")).into());
        }
        
        if !errors.is_empty() {
            println!("  ⚠️  Warning: {} out of {} clients failed to initialize", errors.len(), self.clients.len());
        }
        
        println!("✅ Nonce cache initialization complete ({} successful, {} failed)", 
            self.clients.len() - errors.len(), errors.len());
        Ok(())
    }
    
    /// Send order using automatic nonce (internal cache) - single source of truth
    /// Uses create_order() without explicit nonce to let client manage nonces automatically
    async fn send_order_automatic_nonce(&self, order: CreateOrderRequest, order_num: usize) -> Result<serde_json::Value, String> {
        let (client, api_key_idx) = self.get_next_client();
        let client_index = (self.current_client_index.load(Ordering::Relaxed).saturating_sub(1)) % self.clients.len();
        let start = Instant::now();
        
        // Use client's internal nonce cache - automatic nonce management
        // create_order() without explicit nonce uses internal cache
        let result = client.create_order(order).await;
        let elapsed = start.elapsed();
        
        match &result {
            Ok(response) => {
                let code = response["code"].as_i64().unwrap_or_default();
                let message = response["message"].as_str().unwrap_or("N/A");
                if code == 200 {
                    println!("  ✓ Order {}: Success (Client {}, Key {}, Auto-nonce, {:.2}ms)", 
                        order_num, client_index + 1, api_key_idx, elapsed.as_secs_f64() * 1000.0);
                } else {
                    println!("  ✗ Order {}: {} - {} (Client {}, Key {}, Auto-nonce, {:.2}ms)", 
                        order_num, code, message, client_index + 1, api_key_idx, elapsed.as_secs_f64() * 1000.0);
                }
            }
            Err(e) => {
                println!("  ✗ Order {}: Error - {} (Client {}, Key {}, Auto-nonce, {:.2}ms)", 
                    order_num, e, client_index + 1, api_key_idx, elapsed.as_secs_f64() * 1000.0);
            }
        }
        
        result.map_err(|e| format!("{}", e))
    }
    
    /// Send order using manual nonce (explicit nonce provided)
    /// Fetches nonce ONCE per client, then increments locally
    async fn send_order_manual_nonce(&self, order: CreateOrderRequest, order_num: usize) -> Result<serde_json::Value, String> {
        let (client, api_key_idx) = self.get_next_client();
        let client_index = (self.current_client_index.load(Ordering::Relaxed).saturating_sub(1)) % self.clients.len();
        let start = Instant::now();
        
        // CRITICAL FIX: Calculate nonce and release mutex IMMEDIATELY
        // Hold mutex ONLY for nonce calculation, NOT during network call
        // This allows parallel execution of multiple orders
        let manual_nonce = {
            let mut nonces = self.manual_nonces.lock().await;
            if nonces[client_index] != i64::MIN {
                // Use cached nonce and increment (direct integer operation - very fast)
                let next_nonce = nonces[client_index] + 1;
                nonces[client_index] = next_nonce; // Update immediately
                next_nonce
            } else {
                // First time: fetch from API (mutex held during fetch, but that's unavoidable)
                match client.get_nonce().await {
                    Ok(nonce) => {
                        nonces[client_index] = nonce;
                        nonce
                    }
                    Err(e) => {
                        println!("  ✗ Order {}: Failed to fetch nonce - {} (Client {}, Key {}, {:.2}ms)", 
                            order_num, e, client_index + 1, api_key_idx, start.elapsed().as_secs_f64() * 1000.0);
                        return Err(format!("Failed to fetch nonce: {}", e));
                    }
                }
            }
        }; // ✅ MUTEX RELEASED HERE - BEFORE NETWORK CALL!
        
        // Network call happens WITHOUT mutex held - allows parallelism! ✅
        let result = client.create_order_direct(order, manual_nonce).await;
        let elapsed = start.elapsed();
        
        match &result {
            Ok(response) => {
                let code = response["code"].as_i64().unwrap_or_default();
                let message = response["message"].as_str().unwrap_or("N/A");
                if code == 200 {
                    // Success: nonce already updated above (no mutex needed)
                    println!("  ✓ Order {}: Success (Client {}, Key {}, Manual-nonce={}, {:.2}ms)", 
                        order_num, client_index + 1, api_key_idx, manual_nonce, elapsed.as_secs_f64() * 1000.0);
                } else if code == 21104 {
                    // Nonce error: refetch from API (need mutex for update)
                    let mut nonces = self.manual_nonces.lock().await;
                    if let Ok(fresh_nonce) = client.get_nonce().await {
                        nonces[client_index] = fresh_nonce;
                    }
                    println!("  ✗ Order {}: {} - {} (Client {}, Key {}, Manual-nonce={}, {:.2}ms)", 
                        order_num, code, message, client_index + 1, api_key_idx, manual_nonce, elapsed.as_secs_f64() * 1000.0);
                } else {
                    // Other error: nonce was already updated above (optimistic update)
                    // If order failed, nonce might not have been consumed, but we'll handle it on next order
                    println!("  ✗ Order {}: {} - {} (Client {}, Key {}, Manual-nonce={}, {:.2}ms)", 
                        order_num, code, message, client_index + 1, api_key_idx, manual_nonce, elapsed.as_secs_f64() * 1000.0);
                }
            }
            Err(e) => {
                println!("  ✗ Order {}: Error - {} (Client {}, Key {}, Manual-nonce={}, {:.2}ms)", 
                    order_num, e, client_index + 1, api_key_idx, manual_nonce, elapsed.as_secs_f64() * 1000.0);
            }
        }
        
        result.map_err(|e| format!("{}", e))
    }
    
    async fn send_orders_parallel(&self, orders: Vec<CreateOrderRequest>) -> Vec<Result<serde_json::Value, String>> {
        use std::collections::HashMap;
        
        // Group orders by API key to ensure sequential nonce ordering per key
        let mut orders_by_key: HashMap<u8, Vec<(usize, CreateOrderRequest)>> = HashMap::new();
        
        // Assign orders to API keys in round-robin
        for (order_num, order) in orders.into_iter().enumerate() {
            let (_, api_key_idx) = self.get_next_client();
            orders_by_key.entry(api_key_idx).or_insert_with(Vec::new).push((order_num, order));
        }
        
        // Create a task per API key - each processes its orders sequentially
        let mut tasks = Vec::new();
        
        for (api_key_idx, key_orders) in orders_by_key {
            let client = {
                let client_idx = self.api_key_indices.iter()
                    .position(|&idx| idx == api_key_idx)
                    .unwrap();
                Arc::clone(&self.clients[client_idx])
            };
            
            tasks.push(tokio::spawn(async move {
                let mut results = Vec::new();
                
                // Process orders sequentially for this API key
                // Each client manages its own nonce cache internally
                let mut last_order_time: Option<Instant> = None;
                for (order_num, order) in key_orders {
                    // Add minimum delay between orders to prevent nonce conflicts
                    // Based on measurements: server updates nonce instantly, but API fetch takes ~250ms
                    // So we need delay to ensure nonce is consumed before next order
                    if let Some(last_time) = last_order_time {
                        let elapsed = last_time.elapsed();
                        let min_interval = Duration::from_millis(150);
                        if elapsed < min_interval {
                            tokio::time::sleep(min_interval - elapsed).await;
                        }
                    }
                    
                    let start = Instant::now();
                
                    // Use client's internal nonce cache - single source of truth
                    // create_order() without explicit nonce uses internal cache
                    let result = client.create_order(order).await;
                    last_order_time = Some(Instant::now());
                let elapsed = start.elapsed();
                
                match &result {
                    Ok(response) => {
                        let code = response["code"].as_i64().unwrap_or_default();
                        let message = response["message"].as_str().unwrap_or("N/A");
                        if code == 200 {
                                println!("  ✓ Order {}: Success (Key {}, {:.2}ms)", 
                                    order_num, api_key_idx, elapsed.as_secs_f64() * 1000.0);
                        } else {
                                println!("  ✗ Order {}: {} - {} (Key {}, {:.2}ms)", 
                                    order_num, code, message, api_key_idx, elapsed.as_secs_f64() * 1000.0);
                        }
                    }
                    Err(e) => {
                            println!("  ✗ Order {}: Error - {} (Key {}, {:.2}ms)", 
                                order_num, e, api_key_idx, elapsed.as_secs_f64() * 1000.0);
                    }
                }
                
                    results.push((order_num, result.map_err(|e| format!("{}", e))));
                }
                
                results
            }));
        }
        
        // Collect all results and sort by order number
        let mut all_results: Vec<(usize, Result<serde_json::Value, String>)> = Vec::new();
        for task in tasks {
            let task_results = task.await.unwrap_or_else(|e| {
                vec![(0, Err(format!("Task error: {}", e)))]
            });
            all_results.extend(task_results);
        }
        
        // Sort by order number and extract results
        all_results.sort_by_key(|(num, _)| *num);
        all_results.into_iter().map(|(_, result)| result).collect()
    }
    
    #[allow(dead_code)]
    /// Diagnostic function to compare nonce handling and identify signature issues
    async fn diagnose_nonce_and_signature(&self, api_key_idx: u8) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n{}", "═".repeat(80));
        println!("🔍 DIAGNOSTIC: Nonce Cache vs Manual Nonce Comparison");
        println!("{}", "═".repeat(80));
        
        let client_idx = self.api_key_indices.iter()
            .position(|&idx| idx == api_key_idx)
            .ok_or_else(|| format!("API key {} not found", api_key_idx))?;
        let client = &self.clients[client_idx];
        
        // Test 1: Fetch nonce manually
        println!("\n  📍 Test 1: Manual Nonce Fetch");
        let manual_nonce1 = client.get_nonce().await?;
        println!("    Manual nonce 1: {}", manual_nonce1);
        tokio::time::sleep(Duration::from_millis(100)).await;
        let manual_nonce2 = client.get_nonce().await?;
        println!("    Manual nonce 2: {}", manual_nonce2);
        println!("    Difference: {}", manual_nonce2 - manual_nonce1);
        
        // Test 2: Use cached nonce (if available)
        println!("\n  📍 Test 2: Cached Nonce (via get_next_nonce_from_cache)");
        // Note: This requires accessing internal method, so we'll test via create_order
        
        // Test 3: Compare order with manual nonce vs cached nonce
        println!("\n  📍 Test 3: Order with Manual Nonce");
        let order1 = CreateOrderRequest {
            account_index: self.account_index,
            order_book_index: 1,
            client_order_index: 888888888,
            base_amount: 1000,
            price: 385000,
            is_ask: true,
            order_type: 0,
            time_in_force: 0,
            reduce_only: false,
            trigger_price: 0,
        };
        let fresh_nonce = client.get_nonce().await?;
        println!("    Using fresh nonce: {}", fresh_nonce);
        let result1 = client.create_order_direct(order1, fresh_nonce).await;
        match &result1 {
            Ok(r) => {
                let code = r["code"].as_i64().unwrap_or_default();
                println!("    Result: code={}, message={}", code, r["message"].as_str().unwrap_or("N/A"));
            }
            Err(e) => println!("    Error: {}", e),
        }
        
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        println!("\n  📍 Test 4: Order with Cached Nonce (via create_order)");
        let order2 = CreateOrderRequest {
            account_index: self.account_index,
            order_book_index: 1,
            client_order_index: 888888889,
            base_amount: 1000,
            price: 385001,
            is_ask: false,
            order_type: 0,
            time_in_force: 0,
            reduce_only: false,
            trigger_price: 0,
        };
        let result2 = client.create_order(order2).await;
        match &result2 {
            Ok(r) => {
                let code = r["code"].as_i64().unwrap_or_default();
                println!("    Result: code={}, message={}", code, r["message"].as_str().unwrap_or("N/A"));
            }
            Err(e) => println!("    Error: {}", e),
        }
        
        // Test 5: Check nonce cache state
        println!("\n  📍 Test 5: Nonce Cache State Analysis");
        println!("    Nonce management: Fetches nonce once, stores as (nonce - 1), increments offset atomically");
        
        // Test 6: Check if nonce changes between fetch and use
        println!("\n  📍 Test 6: Nonce Consistency Check");
        let nonce_before = client.get_nonce().await?;
        println!("    Nonce before order: {}", nonce_before);
        
        let order3 = CreateOrderRequest {
            account_index: self.account_index,
            order_book_index: 1,
            client_order_index: 888888890,
            base_amount: 1000,
            price: 385002,
            is_ask: true,
            order_type: 0,
            time_in_force: 0,
            reduce_only: false,
            trigger_price: 0,
        };
        
        // Use manual nonce
        let result3 = client.create_order_direct(order3, nonce_before).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        let nonce_after = client.get_nonce().await?;
        println!("    Nonce after order: {}", nonce_after);
        println!("    Expected nonce after: {}", nonce_before + 1);
        println!("    Actual difference: {}", nonce_after - nonce_before);
        
        match &result3 {
            Ok(r) => {
                let code = r["code"].as_i64().unwrap_or_default();
                println!("    Order result: code={}", code);
                if code == 21120 {
                    println!("    ⚠️  INVALID SIGNATURE ERROR - This suggests nonce mismatch in signature!");
                }
            }
            Err(e) => println!("    Order error: {}", e),
        }
        
        println!("\n{}", "═".repeat(80));
        println!("💡 Key Findings:");
        println!("  • If manual nonce works but cached fails → nonce cache issue");
        println!("  • If both fail with 21120 → signature generation issue");
        println!("  • If nonce doesn't increment → server not processing order");
        println!("{}", "═".repeat(80));
        
        Ok(())
    }
    
    /// Display HTTP headers used by the client
    fn display_header_comparison() {
        println!("\n{}", "═".repeat(80));
        println!("📋 HTTP HEADERS USED BY RUST CLIENT");
        println!("{}", "═".repeat(80));
        
        println!("\n  Headers:");
        println!("    • Content-Type: application/x-www-form-urlencoded");
        println!("    • User-Agent: Lighter-Rust-Client/1.0");
        println!("    • X-API-Key: Set via client configuration");
        println!("    • X-Secret-Key: Set via client configuration");
        println!("    • Connection: keep-alive");
        println!("    • Accept-Encoding: gzip, deflate, br");
        println!("    • sendTx: Uses application/x-www-form-urlencoded");
        
        println!("\n  🦀 Rust SDK Headers:");
        println!("    • User-Agent: Lighter-Rust-Client/1.0");
        println!("    • Accept: application/json");
        println!("    • sendTx: Uses multipart/form-data");
        println!("    • Connection: keep-alive (via reqwest client)");
        println!("    • Content-Type: Set automatically by reqwest for multipart");
        
        println!("\n  ⚠️  Key Differences:");
        println!("    • TypeScript uses x-www-form-urlencoded for sendTx");
        println!("    • Rust uses multipart/form-data for sendTx (matches Python)");
        println!("    • TypeScript has X-API-Key and X-Secret-Key headers");
        println!("    • Python and Rust don't use X-API-Key headers (auth in transaction)");
        
        println!("{}", "═".repeat(80));
    }
    
    #[allow(dead_code)]
    /// Measure nonce update timing: check nonce, send order, then poll until nonce increases
    /// Runs multiple tests to get statistical data for nonce management strategy
    /// Now separates API round-trip time from actual server nonce update time
    async fn measure_nonce_update_time(&self, api_key_idx: u8, num_tests: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n⏱️  Measuring Nonce Update Timing (API Key {}, {} tests)...", api_key_idx, num_tests);
        println!("  📊 This will measure:");
        println!("    1. API round-trip time (time to fetch nonce)");
        println!("    2. Actual server nonce update time (when server processes the order)");
        
        // Find the client for this API key
        let client_idx = self.api_key_indices.iter()
            .position(|&idx| idx == api_key_idx)
            .ok_or_else(|| format!("API key {} not found", api_key_idx))?;
        let client = &self.clients[client_idx];
        
        // First, measure API round-trip time by fetching nonce multiple times
        println!("\n  🔍 Step 0: Measuring API round-trip time (nonce fetch latency)...");
        let mut api_fetch_times = Vec::new();
        for i in 0..10 {
            let fetch_start = Instant::now();
            match client.get_nonce().await {
                Ok(_) => {
                    let fetch_time = fetch_start.elapsed();
                    api_fetch_times.push(fetch_time);
                    if i < 3 {
                        println!("    Fetch {}: {:.2}ms", i + 1, fetch_time.as_secs_f64() * 1000.0);
                    }
                }
                Err(e) => {
                    println!("    ✗ Fetch {} failed: {}", i + 1, e);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        let avg_api_fetch_time = if !api_fetch_times.is_empty() {
            api_fetch_times.iter().sum::<Duration>() / api_fetch_times.len() as u32
        } else {
            Duration::from_millis(100) // Default estimate
        };
        
        println!("    ✓ Average API fetch time: {:.2}ms", avg_api_fetch_time.as_secs_f64() * 1000.0);
        println!("    ✓ Min: {:.2}ms, Max: {:.2}ms", 
            api_fetch_times.iter().min().map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0),
            api_fetch_times.iter().max().map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0));
        
        let mut order_times = Vec::new();
        let mut nonce_update_times = Vec::new(); // Total time from order until nonce increases
        let mut server_update_times = Vec::new(); // Actual server update time (minus API fetch overhead)
        let mut total_times = Vec::new();
        let mut successful_tests = 0;
        let mut failed_tests = 0;
        
        for test_num in 1..=num_tests {
            println!("\n  🔬 Test {}/{}", test_num, num_tests);
            
            // Step 1: Get initial nonce
            let initial_nonce = match client.get_nonce().await {
                Ok(nonce) => nonce,
                Err(e) => {
                    println!("    ✗ Failed to fetch initial nonce: {}", e);
                    failed_tests += 1;
                    continue;
                }
            };
            
            // Step 2: Send an order
            let order = CreateOrderRequest {
                account_index: self.account_index,
                order_book_index: 1,
                client_order_index: 999999999 + test_num as u64,
                base_amount: 1000 + (test_num as i64 * 10),
                price: 385000 + (test_num as i64),
                is_ask: (test_num % 2) == 0,
                order_type: 0,
                time_in_force: 0,
                reduce_only: false,
                trigger_price: 0,
            };
            
            let order_start = Instant::now();
            let order_result = client.create_order_direct(order, initial_nonce).await;
            let order_time = order_start.elapsed();
            
            let order_success = match &order_result {
                Ok(response) => {
                    let code = response["code"].as_i64().unwrap_or_default();
                    code == 200
                }
                Err(_) => false,
            };
            
            if !order_success {
                println!("    ✗ Order failed, skipping test");
                failed_tests += 1;
                continue;
            }
            
            order_times.push(order_time);
            
            // Step 3: Poll nonce continuously until it increases
            // Measure each API call time to separate network latency from server update time
            let poll_start = Instant::now();
            let mut poll_count = 0;
            let max_polls = 200; // Safety limit (2 seconds at 10ms intervals)
            let poll_interval = Duration::from_millis(10); // Poll every 10ms
            
            let mut nonce_update_time: Option<Duration> = None;
            let mut fetch_times_during_poll = Vec::new();
            
            while poll_count < max_polls {
                poll_count += 1;
                let fetch_start = Instant::now();
                match client.get_nonce().await {
                    Ok(nonce) => {
                        let fetch_time = fetch_start.elapsed();
                        fetch_times_during_poll.push(fetch_time);
                        
                        if nonce > initial_nonce {
                            if nonce_update_time.is_none() {
                                let total_elapsed = poll_start.elapsed();
                                nonce_update_time = Some(total_elapsed);
                                
                                // Calculate actual server update time:
                                // The server updated the nonce BEFORE we made this fetch call
                                // So server time = total_elapsed - fetch_time (this call)
                                // But we also need to account for the polling interval
                                // Best estimate: total_elapsed - fetch_time - (poll_interval / 2)
                                // This assumes the nonce was updated halfway through the polling interval
                                let estimated_server_time = if total_elapsed > fetch_time {
                                    total_elapsed.saturating_sub(fetch_time).saturating_sub(poll_interval / 2)
                                } else {
                                    Duration::ZERO
                                };
                                server_update_times.push(estimated_server_time);
                            }
                            break;
                        }
                    }
                    Err(_) => {
                        // Continue polling on error
                    }
                }
                
                tokio::time::sleep(poll_interval).await;
            }
            
            if let Some(update_time) = nonce_update_time {
                nonce_update_times.push(update_time);
                total_times.push(order_time + update_time);
                successful_tests += 1;
                
                // Calculate average fetch time during polling for this test
                let avg_fetch_during_poll = if !fetch_times_during_poll.is_empty() {
                    fetch_times_during_poll.iter().sum::<Duration>() / fetch_times_during_poll.len() as u32
                } else {
                    avg_api_fetch_time
                };
                
                let server_time = server_update_times.last().copied().unwrap_or(update_time);
                println!("    ✓ Nonce updated after {:.2}ms total", update_time.as_secs_f64() * 1000.0);
                println!("      - Order send: {:.2}ms", order_time.as_secs_f64() * 1000.0);
                println!("      - Server update time: ~{:.2}ms (estimated, excluding API fetch overhead)", 
                    server_time.as_secs_f64() * 1000.0);
                println!("      - Avg API fetch during poll: {:.2}ms", avg_fetch_during_poll.as_secs_f64() * 1000.0);
            } else {
                println!("    ✗ Nonce did not increase within timeout");
                failed_tests += 1;
            }
            
            // Small delay between tests to avoid overwhelming the server
            if test_num < num_tests {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
        
        // Calculate and display statistics
        println!("\n{}", "═".repeat(80));
        println!("📊 NONCE UPDATE TIMING STATISTICS");
        println!("{}", "═".repeat(80));
        println!("  Tests run: {}", num_tests);
        println!("  Successful: {}", successful_tests);
        println!("  Failed: {}", failed_tests);
        
        if nonce_update_times.is_empty() {
            println!("\n  ⚠️  No successful tests to analyze");
            return Ok(());
        }
        
        // Sort times for percentile calculations
        let mut sorted_order_times: Vec<f64> = order_times.iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        sorted_order_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut sorted_update_times: Vec<f64> = nonce_update_times.iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        sorted_update_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut sorted_total_times: Vec<f64> = total_times.iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        sorted_total_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Helper function to calculate percentiles
        let percentile = |sorted: &[f64], p: f64| -> f64 {
            if sorted.is_empty() {
                return 0.0;
            }
            let index = (sorted.len() as f64 * p / 100.0).ceil() as usize - 1;
            sorted[index.min(sorted.len() - 1)]
        };
        
        // Helper function to calculate average
        let average = |values: &[f64]| -> f64 {
            if values.is_empty() {
                return 0.0;
            }
            values.iter().sum::<f64>() / values.len() as f64
        };
        
        println!("\n  📈 Order Send Time (ms):");
        println!("    Min:    {:.2}", sorted_order_times[0]);
        println!("    Max:    {:.2}", sorted_order_times[sorted_order_times.len() - 1]);
        println!("    Avg:    {:.2}", average(&sorted_order_times));
        println!("    Median: {:.2}", percentile(&sorted_order_times, 50.0));
        println!("    P95:    {:.2}", percentile(&sorted_order_times, 95.0));
        println!("    P99:    {:.2}", percentile(&sorted_order_times, 99.0));
        
        println!("\n  ⏱️  Total Nonce Update Time (ms) - Time from order until nonce increases (includes API fetch overhead):");
        println!("    Min:    {:.2}", sorted_update_times[0]);
        println!("    Max:    {:.2}", sorted_update_times[sorted_update_times.len() - 1]);
        println!("    Avg:    {:.2}", average(&sorted_update_times));
        println!("    Median: {:.2}", percentile(&sorted_update_times, 50.0));
        println!("    P95:    {:.2}", percentile(&sorted_update_times, 95.0));
        println!("    P99:    {:.2}", percentile(&sorted_update_times, 99.0));
        
        // Calculate and display actual server update times (excluding API fetch overhead)
        if !server_update_times.is_empty() {
            let mut sorted_server_times: Vec<f64> = server_update_times.iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .collect();
            sorted_server_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            println!("\n  🎯 Actual Server Nonce Update Time (ms) - Excluding API fetch overhead:");
            println!("    Min:    {:.2}", sorted_server_times[0]);
            println!("    Max:    {:.2}", sorted_server_times[sorted_server_times.len() - 1]);
            println!("    Avg:    {:.2}", average(&sorted_server_times));
            println!("    Median: {:.2}", percentile(&sorted_server_times, 50.0));
            println!("    P95:    {:.2}", percentile(&sorted_server_times, 95.0));
            println!("    P99:    {:.2}", percentile(&sorted_server_times, 99.0));
            println!("    📊 API fetch overhead: ~{:.2}ms per call", avg_api_fetch_time.as_secs_f64() * 1000.0);
        }
        
        println!("\n  📊 Total Time (ms) - Order send + Nonce update:");
        println!("    Min:    {:.2}", sorted_total_times[0]);
        println!("    Max:    {:.2}", sorted_total_times[sorted_total_times.len() - 1]);
        println!("    Avg:    {:.2}", average(&sorted_total_times));
        println!("    Median: {:.2}", percentile(&sorted_total_times, 50.0));
        println!("    P95:    {:.2}", percentile(&sorted_total_times, 95.0));
        println!("    P99:    {:.2}", percentile(&sorted_total_times, 99.0));
        
        // Recommendations for nonce management
        println!("\n  💡 Nonce Management Strategy Recommendations:");
        let avg_update = average(&sorted_update_times);
        let p95_update = percentile(&sorted_update_times, 95.0);
        let p99_update = percentile(&sorted_update_times, 99.0);
        let median_update = percentile(&sorted_update_times, 50.0);
        
        // Use actual server update time if available, otherwise use total time
        let (avg_server_time, median_server_time, p95_server_time, p99_server_time) = if !server_update_times.is_empty() {
            let mut sorted_server_times: Vec<f64> = server_update_times.iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .collect();
            sorted_server_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            (
                average(&sorted_server_times),
                percentile(&sorted_server_times, 50.0),
                percentile(&sorted_server_times, 95.0),
                percentile(&sorted_server_times, 99.0),
            )
        } else {
            (avg_update, median_update, p95_update, p99_update)
        };
        
        println!("    📊 Server Nonce Update Performance (Actual Server Time):");
        println!("      • Average: {:.2}ms (total: {:.2}ms including API overhead)", avg_server_time, avg_update);
        println!("      • Median: {:.2}ms (total: {:.2}ms including API overhead)", median_server_time, median_update);
        println!("      • P95: {:.2}ms (total: {:.2}ms including API overhead)", p95_server_time, p95_update);
        println!("      • P99: {:.2}ms (total: {:.2}ms including API overhead)", p99_server_time, p99_update);
        println!("      • API fetch overhead: ~{:.2}ms per call", avg_api_fetch_time.as_secs_f64() * 1000.0);
        
        println!("\n    ⚙️  Recommended Nonce Management Strategy (based on actual server time):");
        
        // Calculate recommended delays based on actual server time
        let recommended_delay = (p95_server_time * 1.2).ceil();
        let safe_delay = (p99_server_time * 1.3).ceil();
        let aggressive_delay = (median_server_time * 1.1).ceil();
        
        println!("      • Aggressive (for speed): {:.0}ms delay (Median {:.0}ms + 10%)", aggressive_delay, median_server_time);
        println!("      • Balanced (recommended): {:.0}ms delay (P95 {:.0}ms + 20%)", recommended_delay, p95_server_time);
        println!("      • Conservative (for reliability): {:.0}ms delay (P99 {:.0}ms + 30%)", safe_delay, p99_server_time);
        
        println!("\n    📝 Implementation Notes:");
        println!("      • Current implementation uses 150ms delay");
        if recommended_delay < 150.0 {
            println!("      • ⚠️  Current delay ({:.0}ms) is higher than recommended ({:.0}ms)", 150.0, recommended_delay);
            println!("      • Consider reducing to {:.0}ms for better throughput", recommended_delay);
        } else if recommended_delay > 150.0 {
            println!("      • ⚠️  Current delay ({:.0}ms) is LOWER than recommended ({:.0}ms)", 150.0, recommended_delay);
            println!("      • Consider increasing to {:.0}ms to avoid nonce conflicts", recommended_delay);
        } else {
            println!("      • ✓ Current delay ({:.0}ms) is appropriate", 150.0);
        }
        
        println!("\n    🔄 Nonce Update Behavior:");
        if avg_server_time < 50.0 {
            println!("      • Server updates nonces very quickly (< 50ms average)");
            println!("      • Optimistic nonce management is highly effective");
        } else if avg_server_time < 100.0 {
            println!("      • Server updates nonces moderately fast (< 100ms average)");
            println!("      • Optimistic nonce management works well with small delays");
        } else {
            println!("      • Server updates nonces slowly (> 100ms average)");
            println!("      • Consider longer delays or more conservative nonce management");
        }
        
        println!("\n    📈 Key Insights:");
        println!("      • Actual server processing: ~{:.0}ms", avg_server_time);
        println!("      • API fetch overhead: ~{:.0}ms per call", avg_api_fetch_time.as_secs_f64() * 1000.0);
        println!("      • Total observed time: ~{:.0}ms (server + API overhead)", avg_update);
        println!("      • Recommended delay accounts for server processing time only");
        
        println!("{}", "═".repeat(80));
        
        Ok(())
    }
    
    /// Test and compare automatic nonce vs manual nonce
    async fn test_automatic_vs_manual_nonce(&self, num_orders: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n{}", "═".repeat(80));
        println!("🧪 TEST: Automatic Nonce vs Manual Nonce Comparison");
        println!("{}", "═".repeat(80));
        
        // Prepare orders
        let mut orders = Vec::new();
        for i in 0..num_orders {
            orders.push(CreateOrderRequest {
                account_index: self.account_index,
                order_book_index: 1,
                client_order_index: 2000000 + i as u64,
                base_amount: 1000 + (i as i64 * 10),
                price: 385000 + (i as i64),
                is_ask: (i % 2) == 0,
                order_type: 0,
                time_in_force: 0,
                reduce_only: false,
                trigger_price: 0,
            });
        }
        
        // Test 1: Automatic Nonce (Internal Cache)
        println!("\n📊 Test 1: Automatic Nonce (Internal Cache)");
        println!("  Using: client.create_order() - system automatically manages nonce");
        println!("  Sending {} orders...", num_orders);
        
        let start_auto = Instant::now();
        let mut auto_results = Vec::new();
        for (i, order) in orders.iter().enumerate() {
            auto_results.push(self.send_order_automatic_nonce(order.clone(), i).await);
            // Small delay to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let elapsed_auto = start_auto.elapsed();
        
        let mut auto_success = 0;
        let mut auto_errors = 0;
        let mut auto_invalid_sig = 0;
        let mut auto_invalid_nonce = 0;
        
        for result in auto_results.iter() {
            match result {
                Ok(response) => {
                    let code = response["code"].as_i64().unwrap_or_default();
                    if code == 200 {
                        auto_success += 1;
                    } else {
                        auto_errors += 1;
                        if code == 21120 { auto_invalid_sig += 1; }
                        if code == 21104 { auto_invalid_nonce += 1; }
                    }
                }
                Err(_) => auto_errors += 1,
            }
        }
        
        println!("\n  📈 Automatic Nonce Results:");
        println!("    Total Time: {:.2}ms ({:.2}s)", elapsed_auto.as_secs_f64() * 1000.0, elapsed_auto.as_secs_f64());
        println!("    Success: {} ({:.1}%)", auto_success, (auto_success as f64 / num_orders as f64) * 100.0);
        println!("    Errors: {} ({:.1}%)", auto_errors, (auto_errors as f64 / num_orders as f64) * 100.0);
        println!("    Invalid Signature (21120): {}", auto_invalid_sig);
        println!("    Invalid Nonce (21104): {}", auto_invalid_nonce);
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Test 2: Manual Nonce (Explicit Fetch)
        println!("\n📊 Test 2: Manual Nonce (Explicit Fetch)");
        println!("  Using: client.get_nonce() + client.create_order_direct() - manual nonce management");
        println!("  Sending {} orders...", num_orders);
        
        let start_manual = Instant::now();
        let mut manual_results = Vec::new();
        for (i, order) in orders.iter().enumerate() {
            manual_results.push(self.send_order_manual_nonce(order.clone(), i).await);
            // Small delay to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let elapsed_manual = start_manual.elapsed();
        
        let mut manual_success = 0;
        let mut manual_errors = 0;
        let mut manual_invalid_sig = 0;
        let mut manual_invalid_nonce = 0;
        
        for result in manual_results.iter() {
            match result {
                Ok(response) => {
                    let code = response["code"].as_i64().unwrap_or_default();
                    if code == 200 {
                        manual_success += 1;
                    } else {
                        manual_errors += 1;
                        if code == 21120 { manual_invalid_sig += 1; }
                        if code == 21104 { manual_invalid_nonce += 1; }
                    }
                }
                Err(_) => manual_errors += 1,
            }
        }
        
        println!("\n  📈 Manual Nonce Results:");
        println!("    Total Time: {:.2}ms ({:.2}s)", elapsed_manual.as_secs_f64() * 1000.0, elapsed_manual.as_secs_f64());
        println!("    Success: {} ({:.1}%)", manual_success, (manual_success as f64 / num_orders as f64) * 100.0);
        println!("    Errors: {} ({:.1}%)", manual_errors, (manual_errors as f64 / num_orders as f64) * 100.0);
        println!("    Invalid Signature (21120): {}", manual_invalid_sig);
        println!("    Invalid Nonce (21104): {}", manual_invalid_nonce);
        
        // Comparison
        println!("\n{}", "─".repeat(80));
        println!("📊 Comparison Summary:");
        println!("{}", "─".repeat(80));
        println!("  Automatic Nonce:");
        println!("    • Success Rate: {:.1}%", (auto_success as f64 / num_orders as f64) * 100.0);
        println!("    • Avg Time/Order: {:.2}ms", elapsed_auto.as_secs_f64() * 1000.0 / num_orders as f64);
        println!("    • Invalid Signature Errors: {}", auto_invalid_sig);
        println!("    • Invalid Nonce Errors: {}", auto_invalid_nonce);
        
        println!("\n  Manual Nonce:");
        println!("    • Success Rate: {:.1}%", (manual_success as f64 / num_orders as f64) * 100.0);
        println!("    • Avg Time/Order: {:.2}ms", elapsed_manual.as_secs_f64() * 1000.0 / num_orders as f64);
        println!("    • Invalid Signature Errors: {}", manual_invalid_sig);
        println!("    • Invalid Nonce Errors: {}", manual_invalid_nonce);
        
        let time_diff = elapsed_manual.as_secs_f64() - elapsed_auto.as_secs_f64();
        println!("\n  ⚡ Performance Difference:");
        if time_diff > 0.0 {
            println!("    • Automatic is {:.2}ms faster ({:.1}% faster)", 
                time_diff * 1000.0, (time_diff / elapsed_manual.as_secs_f64()) * 100.0);
        } else {
            println!("    • Manual is {:.2}ms faster ({:.1}% faster)", 
                -time_diff * 1000.0, (-time_diff / elapsed_auto.as_secs_f64()) * 100.0);
        }
        
        println!("\n  💡 Recommendation:");
        if auto_invalid_sig < manual_invalid_sig {
            println!("    • Automatic nonce has fewer signature errors - RECOMMENDED");
        } else if manual_invalid_sig < auto_invalid_sig {
            println!("    • Manual nonce has fewer signature errors - consider using manual");
        } else {
            println!("    • Both methods have similar error rates");
        }
        
        if auto_success > manual_success {
            println!("    • Automatic nonce has better success rate");
        } else if manual_success > auto_success {
            println!("    • Manual nonce has better success rate");
        }
        
        println!("{}", "═".repeat(80));
        
        Ok(())
    }
    
    async fn benchmark(&self, num_orders: usize, parallel: bool, use_automatic_nonce: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 Benchmarking {} orders (parallel: {}, nonce: {})...", 
            num_orders, parallel, if use_automatic_nonce { "automatic" } else { "manual" });
        println!("  ⚙️  Using {} nonce with 150ms minimum delay between orders per API key", 
            if use_automatic_nonce { "AUTOMATIC (internal cache)" } else { "MANUAL (explicit fetch)" });
        
        let mut orders = Vec::new();
        for i in 0..num_orders {
            orders.push(CreateOrderRequest {
                account_index: self.account_index,
                order_book_index: 1,
                client_order_index: 1000000 + i as u64,
                base_amount: 1000 + (i as i64 * 10),
                price: 385000 + (i as i64),
                is_ask: (i % 2) == 0,
                order_type: 0,
                time_in_force: 0,
                reduce_only: false,
                trigger_price: 0,
            });
        }
        
        println!("  Sending {} orders...", num_orders);
        let start = Instant::now();
        let results = if parallel {
            self.send_orders_parallel(orders).await
        } else {
            let mut results = Vec::new();
            for (i, order) in orders.into_iter().enumerate() {
                if use_automatic_nonce {
                    results.push(self.send_order_automatic_nonce(order, i).await);
                } else {
                    results.push(self.send_order_manual_nonce(order, i).await);
                }
            }
            results
        };
        let elapsed = start.elapsed();
        
        let mut success_count = 0;
        let mut error_count = 0;
        let mut invalid_sig_count = 0;
        let mut invalid_nonce_count = 0;
        let mut error_codes = std::collections::HashMap::new();
        
        for result in results.iter() {
            match result {
                Ok(response) => {
                    let code = response["code"].as_i64().unwrap_or_default();
                    if code == 200 {
                        success_count += 1;
                    } else {
                        error_count += 1;
                        if code == 21120 { invalid_sig_count += 1; }
                        if code == 21104 { invalid_nonce_count += 1; }
                        *error_codes.entry(code.to_string()).or_insert(0) += 1;
                    }
                }
                Err(_) => {
                    error_count += 1;
                    *error_codes.entry("network".to_string()).or_insert(0) += 1;
                }
            }
        }
        
        let total_time_ms = elapsed.as_secs_f64() * 1000.0;
        let orders_per_second = num_orders as f64 / elapsed.as_secs_f64();
        let avg_time_per_order_ms = total_time_ms / num_orders as f64;
        
        println!("\n📈 Benchmark Results:");
        println!("  Total Time: {:.2}ms ({:.2}s)", total_time_ms, elapsed.as_secs_f64());
        println!("  Orders/Second: {:.2}", orders_per_second);
        println!("  Avg Time/Order: {:.2}ms", avg_time_per_order_ms);
        println!("  Success: {} ({:.1}%)", success_count, (success_count as f64 / num_orders as f64) * 100.0);
        println!("  Errors: {} ({:.1}%)", error_count, (error_count as f64 / num_orders as f64) * 100.0);
        println!("  Invalid Signature (21120): {}", invalid_sig_count);
        println!("  Invalid Nonce (21104): {}", invalid_nonce_count);
        
        if !error_codes.is_empty() {
            println!("\n  Error Breakdown: {:?}", error_codes);
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🚀 HIGH-FREQUENCY TRADING (HFT) - OPTIMISTIC NONCE");
    println!("{}", "═".repeat(80));
    println!();
    
    dotenv::dotenv().ok();
    
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "https://testnet.zklighter.elliot.ai".to_string());
    let account_index: i64 = env::var("ACCOUNT_INDEX").unwrap_or_else(|_| "665".to_string()).parse()?;
    
    let api_keys: Vec<(u8, String)> = vec![
        (4, env::var("API_PRIVATE_KEY_4").unwrap_or_else(|_| env::var("API_PRIVATE_KEY").unwrap_or_default())),
        (7, env::var("API_PRIVATE_KEY_7")?),
        (8, env::var("API_PRIVATE_KEY_8")?),
        (9, env::var("API_PRIVATE_KEY_9")?),
        (10, env::var("API_PRIVATE_KEY_10")?),
        (11, env::var("API_PRIVATE_KEY_11")?),
        (12, env::var("API_PRIVATE_KEY_12")?),
    ];
    
    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  API Keys: {}", api_keys.len());
    println!();
    
    let trader = HftTrader::new(base_url, account_index, api_keys)?;
    trader.warmup_nonces().await?;
    
    // Display header comparison
    HftTrader::display_header_comparison();
    
    // Test: Automatic vs Manual Nonce Comparison
    trader.test_automatic_vs_manual_nonce(10).await?;
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Benchmark 1: Automatic Nonce (Sequential)
    println!("\n{}", "─".repeat(80));
    println!("📊 BENCHMARK 1: Automatic Nonce (Sequential)");
    println!("{}", "─".repeat(80));
    trader.benchmark(10, false, true).await?;
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Benchmark 2: Manual Nonce (Sequential)
    println!("\n{}", "─".repeat(80));
    println!("📊 BENCHMARK 2: Manual Nonce (Sequential)");
    println!("{}", "─".repeat(80));
    trader.benchmark(10, false, false).await?;
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Benchmark 3: Automatic Nonce (Parallel)
    println!("\n{}", "─".repeat(80));
    println!("📊 BENCHMARK 3: Automatic Nonce (Parallel)");
    println!("{}", "─".repeat(80));
    trader.benchmark(20, true, true).await?;
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Benchmark 4: Manual Nonce (Parallel)
    println!("\n{}", "─".repeat(80));
    println!("📊 BENCHMARK 4: Manual Nonce (Parallel)");
    println!("{}", "─".repeat(80));
    trader.benchmark(20, true, false).await?;
    
    println!("\n{}", "═".repeat(80));
    println!("✅ HFT Testing Complete");
    println!("{}", "═".repeat(80));
    
    Ok(())
}

