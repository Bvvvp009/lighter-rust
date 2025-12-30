// Debug to check r#type field serialization
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct TestStruct {
    r#type: u8,
}

fn main() {
    let test = TestStruct { r#type: 1 };
    println!("{}", serde_json::to_string(&test).unwrap());
    
    // Also test with json! macro
    let json = serde_json::json!({
        "Type": 1u8
    });
    println!("{}", serde_json::to_string(&json).unwrap());
}
