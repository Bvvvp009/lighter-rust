use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct TestTxInfo {
    account_index: i64,
    sig: String,
}

fn main() {
    let tx_info = TestTxInfo {
        account_index: 123,
        sig: String::new(),
    };
    
    println!("Original: {}", serde_json::to_string(&tx_info).unwrap());
    
    // Using spread operator
    let final_tx_info = TestTxInfo {
        sig: "signature_data".to_string(),
        ..tx_info.clone()
    };
    
    println!("After spread: {}", serde_json::to_string(&final_tx_info).unwrap());
    
    // Verify values are correct
    println!("account_index: {}, sig: {}", final_tx_info.account_index, final_tx_info.sig);
}
