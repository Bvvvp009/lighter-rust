fn main() {
    let value1 = 1766863675u64;
    let value2 = 1766863675i64;
    
    // Using json! macro
    let json1 = serde_json::json!({"value": value1});
    let json2 = serde_json::json!({"value": value2});
    
    let str1 = serde_json::to_string(&json1).unwrap();
    let str2 = serde_json::to_string(&json2).unwrap();
    
    println!("u64: {}", str1);
    println!("i64: {}", str2);
    println!("Equal: {}", str1 == str2);
}
