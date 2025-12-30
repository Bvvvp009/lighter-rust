//! Test to verify select() function behavior

use goldilocks_crypto::ScalarField;

#[test]
fn test_select_function() {
    println!("\n=== Testing select() Function ===\n");
    
    let a0 = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 10;
        bytes
    }).unwrap();
    
    let a1 = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 20;
        bytes
    }).unwrap();
    
    println!("a0: {:?}", a0.0);
    println!("a1: {:?}", a1.0);
    
    // Test select(0, a0, a1) - should return a0
    let result0 = ScalarField::select(0, &a0, &a1);
    println!("\nselect(0, a0, a1): {:?}", result0.0);
    println!("  Should equal a0: {}", result0.0 == a0.0);
    assert_eq!(result0.0, a0.0, "select(0, a0, a1) should return a0");
    
    // Test select(1, a0, a1) - should return a1
    let result1 = ScalarField::select(1, &a0, &a1);
    println!("select(1, a0, a1): {:?}", result1.0);
    println!("  Should equal a1: {}", result1.0 == a1.0);
    assert_eq!(result1.0, a1.0, "select(1, a0, a1) should return a1");
    
    // Test select(0xFFFFFFFFFFFFFFFF, a0, a1) - should return a1
    let result_ff = ScalarField::select(0xFFFFFFFFFFFFFFFF, &a0, &a1);
    println!("select(0xFFFFFFFFFFFFFFFF, a0, a1): {:?}", result_ff.0);
    println!("  Should equal a1: {}", result_ff.0 == a1.0);
    assert_eq!(result_ff.0, a1.0, "select(0xFFFFFFFFFFFFFFFF, a0, a1) should return a1");
    
    println!("\n✅ select() function works correctly");
    println!("  select(0, a0, a1) = a0");
    println!("  select(1, a0, a1) = a1");
    println!("  select(0xFFFFFFFFFFFFFFFF, a0, a1) = a1");
}












