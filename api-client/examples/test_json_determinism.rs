// Test 1: JSON Serialization Determinism Test
// This is CRITICAL - if JSON serialization is non-deterministic,
// it will cause different hashes and signature failures

use serde_json::json;
use std::collections::HashSet;

fn main() {
    println!("🔬 Test 1: JSON Serialization Determinism");
    println!("==========================================");
    println!("Testing if serde_json produces identical output for same input\n");

    // Create a transaction structure matching api-client
    let tx_info = json!({
        "AccountIndex": 12345i64,
        "ApiKeyIndex": 0u8,
        "MarketIndex": 2u8,
        "ClientOrderIndex": 1234567890u64,
        "BaseAmount": -1_000_000i64,
        "Price": 3200_000_000i64,
        "IsAsk": 1,
        "Type": 0u8,
        "TimeInForce": 1u8,
        "ReduceOnly": 0,
        "TriggerPrice": 0i64,
        "OrderExpiry": 1735598000000i64,
        "ExpiredAt": 1735000000000i64,
        "Nonce": 100i64,
        "Sig": ""
    });

    println!("Test Data:");
    println!("{}", serde_json::to_string_pretty(&tx_info).unwrap());
    println!();

    // Test 1: Serialize 10,000 times and check if all identical
    println!("🧪 Serializing 10,000 times...");
    let mut outputs = Vec::new();
    let mut unique_outputs = HashSet::new();

    for i in 0..10_000 {
        let json_str = serde_json::to_string(&tx_info)
            .expect("Serialization should not fail");
        
        if i < 5 {
            println!("  [{}] {} bytes: {}", i + 1, json_str.len(), 
                &json_str[..json_str.len().min(80)]);
        }
        
        outputs.push(json_str.clone());
        unique_outputs.insert(json_str);
    }

    println!();
    println!("📊 Results:");
    println!("  Total serializations: {}", outputs.len());
    println!("  Unique outputs: {}", unique_outputs.len());
    
    if unique_outputs.len() == 1 {
        println!("  ✅ PASS: JSON serialization is DETERMINISTIC");
        println!();
        println!("  This is GOOD! serde_json produces consistent output.");
        println!("  JSON field order is NOT the root cause of signature failures.");
    } else {
        println!("  ❌ FAIL: JSON serialization is NON-DETERMINISTIC!");
        println!();
        println!("  🚨 ROOT CAUSE FOUND!");
        println!("  serde_json produces different outputs for the same data.");
        println!("  This causes different hashes → different signatures.");
        println!();
        println!("  Sample variations:");
        for (i, variant) in unique_outputs.iter().take(3).enumerate() {
            println!("    Variant {}: {}", i + 1, variant);
        }
    }

    // Test 2: Check if field order is stable
    println!();
    println!("🧪 Testing field order stability...");
    
    let first_output = &outputs[0];
    let last_output = &outputs[outputs.len() - 1];
    
    // Extract keys from JSON
    let first_obj: serde_json::Value = serde_json::from_str(first_output).unwrap();
    let last_obj: serde_json::Value = serde_json::from_str(last_output).unwrap();
    
    if let (Some(first_map), Some(last_map)) = (first_obj.as_object(), last_obj.as_object()) {
        let first_keys: Vec<_> = first_map.keys().collect();
        let last_keys: Vec<_> = last_map.keys().collect();
        
        if first_keys == last_keys {
            println!("  ✅ Field order is stable");
            println!("  Keys: {:?}", first_keys);
        } else {
            println!("  ❌ Field order varies!");
            println!("  First: {:?}", first_keys);
            println!("  Last:  {:?}", last_keys);
        }
    }

    // Test 3: Byte-level comparison
    println!();
    println!("🧪 Byte-level comparison...");
    let all_identical = outputs.windows(2).all(|w| w[0] == w[1]);
    
    if all_identical {
        println!("  ✅ All outputs are byte-for-byte identical");
    } else {
        println!("  ❌ Outputs differ at byte level");
        
        // Find first difference
        for i in 1..outputs.len() {
            if outputs[i] != outputs[0] {
                println!();
                println!("  First difference at iteration {}:", i + 1);
                println!("  Original: {}", outputs[0]);
                println!("  Different: {}", outputs[i]);
                break;
            }
        }
    }

    // Test 4: Hash stability (simulate signature hashing)
    println!();
    println!("🧪 Testing hash stability (simulated Poseidon2 input)...");
    
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hashes = Vec::new();
    for output in &outputs {
        let mut hasher = DefaultHasher::new();
        output.hash(&mut hasher);
        hashes.push(hasher.finish());
    }
    
    let unique_hashes: HashSet<_> = hashes.iter().collect();
    
    if unique_hashes.len() == 1 {
        println!("  ✅ All serializations produce the same hash");
        println!("  Hash: {:016x}", hashes[0]);
    } else {
        println!("  ❌ Different serializations produce different hashes!");
        println!("  Unique hashes: {}", unique_hashes.len());
        
        for (i, hash) in unique_hashes.iter().take(3).enumerate() {
            println!("    Hash {}: {:016x}", i + 1, hash);
        }
    }

    println!();
    println!("================================");
    println!("📋 Summary");
    println!("================================");
    
    if unique_outputs.len() == 1 && all_identical {
        println!("✅ JSON serialization is fully deterministic");
        println!("   → This is NOT the root cause of signature failures");
        println!("   → Continue to Test 2 (signature determinism)");
    } else {
        println!("❌ JSON serialization has issues");
        println!("   → This IS the root cause of signature failures");
        println!("   → FIX: Use ordered HashMap or custom serializer");
        println!();
        println!("Recommended fix:");
        println!("  1. Use serde_json with preserve_order feature");
        println!("  2. Or use a struct with #[serde(rename_all = \"PascalCase\")]");
        println!("  3. Or serialize fields manually in specific order");
    }
}
