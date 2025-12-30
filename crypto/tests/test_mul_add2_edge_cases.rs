//! Test edge cases for mul_add2 to identify bugs

use goldilocks_crypto::{ScalarField, Point};

#[test]
fn test_mul_add2_edge_cases() {
    println!("\n=== Testing mul_add2 Edge Cases ===\n");
    
    let generator = Point::generator();
    
    // Test 1: One scalar is zero
    println!("Test 1: mul_add2(G, P, 0, 1) should equal P");
    let p = generator.mul(&ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 5;
        bytes
    }).unwrap());
    let zero = ScalarField::from_bytes_le(&[0u8; 40]).unwrap();
    let one = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 1;
        bytes
    }).unwrap();
    
    let result1 = Point::mul_add2(&generator, &p, &zero, &one);
    let expected1 = p.encode();
    let computed1 = result1.encode();
    let match1 = expected1.0.iter().zip(computed1.0.iter()).all(|(a, b)| a.0 == b.0);
    println!("  Expected: {}", hex::encode(&expected1.to_bytes_le()));
    println!("  Computed: {}", hex::encode(&computed1.to_bytes_le()));
    println!("  Match: {}\n", match1);
    
    // Test 2: Other scalar is zero
    println!("Test 2: mul_add2(G, P, 1, 0) should equal G");
    let result2 = Point::mul_add2(&generator, &p, &one, &zero);
    let expected2 = generator.encode();
    let computed2 = result2.encode();
    let match2 = expected2.0.iter().zip(computed2.0.iter()).all(|(a, b)| a.0 == b.0);
    println!("  Expected: {}", hex::encode(&expected2.to_bytes_le()));
    println!("  Computed: {}", hex::encode(&computed2.to_bytes_le()));
    println!("  Match: {}\n", match2);
    
    // Test 3: Both scalars are zero
    println!("Test 3: mul_add2(G, P, 0, 0) should equal neutral");
    let result3 = Point::mul_add2(&generator, &p, &zero, &zero);
    let neutral = Point::neutral().encode();
    let computed3 = result3.encode();
    let match3 = neutral.0.iter().zip(computed3.0.iter()).all(|(a, b)| a.0 == b.0);
    println!("  Expected: {}", hex::encode(&neutral.to_bytes_le()));
    println!("  Computed: {}", hex::encode(&computed3.to_bytes_le()));
    println!("  Match: {}\n", match3);
    
    // Test 4: Simple case s*G + 0*P
    println!("Test 4: mul_add2(G, P, s, 0) where s=2 should equal 2*G");
    let two = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 2;
        bytes
    }).unwrap();
    let result4 = Point::mul_add2(&generator, &p, &two, &zero);
    let expected4 = generator.mul(&two).encode();
    let computed4 = result4.encode();
    let match4 = expected4.0.iter().zip(computed4.0.iter()).all(|(a, b)| a.0 == b.0);
    println!("  Expected: {}", hex::encode(&expected4.to_bytes_le()));
    println!("  Computed: {}", hex::encode(&computed4.to_bytes_le()));
    println!("  Match: {}\n", match4);
    
    assert!(match1 && match2 && match3 && match4, "Some edge case tests failed");
}













