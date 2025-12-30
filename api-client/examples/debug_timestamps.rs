// Debug utility to print actual TX JSON and signature details
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Simulate what's being generated
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
    let expired_at = now + 599_000;
    
    println!("Current time: {}", now);
    println!("ExpiredAt: {}", expired_at);
    println!("Difference: {} ms", expired_at - now);
    
    // Check multiple timestamps to see variance
    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        println!("Time sample {}: {}", i, t);
    }
}
