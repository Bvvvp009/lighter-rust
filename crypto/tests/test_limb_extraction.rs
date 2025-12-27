//! Test to verify 4-bit limb extraction order and format

use goldilocks_crypto::ScalarField;

#[test]
fn test_limb_extraction_order() {
    println!("\n=== Testing 4-bit Limb Extraction ===\n");
    
    // Test with a known scalar value
    // Create scalar = 1 (all zeros except first byte = 1)
    let scalar = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 1;
        bytes
    }).unwrap();
    
    println!("Scalar value (limbs):");
    for (i, &limb) in scalar.0.iter().enumerate() {
        println!("  limb[{}] = 0x{:016x}", i, limb);
    }
    
    let limbs = scalar.split_to_4bit_limbs();
    
    println!("\n4-bit limbs (80 total):");
    println!("  First 20 limbs: {:?}", &limbs[0..20]);
    println!("  Last 20 limbs: {:?}", &limbs[60..80]);
    
    // Find where the 1 is
    let mut one_positions = Vec::new();
    for (i, &limb) in limbs.iter().enumerate() {
        if limb != 0 {
            one_positions.push((i, limb));
        }
    }
    println!("\nNon-zero limbs:");
    for (idx, val) in &one_positions {
        println!("  limbs[{}] = {}", idx, val);
    }
    
    // Verify: if scalar = 1, then limbs[0] should be 1
    assert_eq!(limbs[0], 1, "First limb should be 1 for scalar=1");
    assert_eq!(one_positions.len(), 1, "Should have exactly one non-zero limb for scalar=1");
    
    // Test with scalar = 0x1234 (in first limb)
    let scalar2 = ScalarField::from_bytes_le(&{
        let mut bytes = [0u8; 40];
        bytes[0] = 0x34;
        bytes[1] = 0x12;
        bytes
    }).unwrap();
    
    println!("\n=== Testing with scalar = 0x1234 ===\n");
    println!("Scalar value (limbs):");
    for (i, &limb) in scalar2.0.iter().enumerate() {
        println!("  limb[{}] = 0x{:016x}", i, limb);
    }
    
    let limbs2 = scalar2.split_to_4bit_limbs();
    println!("\n4-bit limbs:");
    println!("  First 8 limbs: {:?}", &limbs2[0..8]);
    
    // Verify: 0x1234 = 0b0001_0010_0011_0100
    // In little-endian 4-bit chunks: [0x4, 0x3, 0x2, 0x1]
    assert_eq!(limbs2[0], 0x4, "limbs[0] should be 0x4");
    assert_eq!(limbs2[1], 0x3, "limbs[1] should be 0x3");
    assert_eq!(limbs2[2], 0x2, "limbs[2] should be 0x2");
    assert_eq!(limbs2[3], 0x1, "limbs[3] should be 0x1");
    
    println!("\n✅ Limb extraction order verified (little-endian within each 64-bit limb)");
}

#[test]
fn test_limb_extraction_with_random() {
    println!("\n=== Testing Limb Extraction with Random Scalar ===\n");
    
    let scalar = ScalarField::sample_crypto();
    let limbs = scalar.split_to_4bit_limbs();
    
    println!("Random scalar (first 3 limbs):");
    for i in 0..3 {
        println!("  limb[{}] = 0x{:016x}", i, scalar.0[i]);
    }
    
    println!("\n4-bit limbs (first 32):");
    for i in 0..32 {
        if i % 16 == 0 {
            print!("\n  [{}..{}]: ", i, i+15);
        }
        print!("{:x} ", limbs[i]);
    }
    println!("\n");
    
    // Verify we have 80 limbs total
    assert_eq!(limbs.len(), 80, "Should have exactly 80 4-bit limbs");
    
    // Verify each limb is in range [0, 15]
    for (i, &limb) in limbs.iter().enumerate() {
        assert!(limb <= 0xF, "Limb {} should be <= 0xF, got {}", i, limb);
    }
    
    println!("✅ Random scalar limb extraction verified");
}

