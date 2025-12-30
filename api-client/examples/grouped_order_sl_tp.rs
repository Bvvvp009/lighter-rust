use api_client::CreateOrderRequest;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!(">> GROUPED ORDER WITH ATTACHED SL/TP");
    println!("{}", "=".repeat(80));
    println!();

    println!("📋 This example shows the structure for grouped orders");
    println!();

    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64 + 300;
    let account_index: i64 = 361816; // example value

    println!("📝 Creating grouped order structure with SL/TP...");
    println!("  Primary: IoC Sell order for 0.1 ETH @ $2500");
    println!("  SL: Stop Loss @ $5000 (limit: $5050)");
    println!("  TP: Take Profit @ $1500 (limit: $1550)");
    println!();

    // Main IoC order (Immediate or Cancel - fills immediately or cancels)
    let ioc_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,      // ETH-USD
        client_order_index: 500,  // unique identifier
        base_amount: 1000,        // 0.1 ETH
        price: 2500_00,           // $2500
        is_ask: true,             // sell
        order_type: 0,            // limit order
        time_in_force: 2,         // immediate or cancel (IoC)
        reduce_only: false,
        trigger_price: 0,
    };

    // Take Profit order (triggers when price drops to $1500)
    let take_profit_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 501,
        base_amount: 0,           // 0 = entire executed size
        price: 1550_00,           // limit price to buy at
        is_ask: false,            // buy to close
        order_type: 3,            // take profit type
        time_in_force: 1,         // good till time
        reduce_only: true,        // only close position
        trigger_price: 1500_00,   // trigger when price = $1500
    };

    // Stop Loss order (triggers when price rises to $5000)
    let stop_loss_order = CreateOrderRequest {
        account_index,
        order_book_index: 0,
        client_order_index: 502,
        base_amount: 0,           // 0 = entire executed size
        price: 5050_00,           // limit price to buy at
        is_ask: false,            // buy to close
        order_type: 2,            // stop loss type
        time_in_force: 1,         // good till time
        reduce_only: true,        // only close position
        trigger_price: 5000_00,   // trigger when price = $5000
    };

    println!("✅ Example order structures created:");
    println!("  Primary Order: {:?}", ioc_order.order_type);
    println!("  SL Order: {:?}", stop_loss_order.order_type);
    println!("  TP Order: {:?}", take_profit_order.order_type);
    println!();
    println!("📝 To submit as grouped orders, use send_tx_batch example");
    println!("   or create_grouped_orders API method when available");

    Ok(())
}
