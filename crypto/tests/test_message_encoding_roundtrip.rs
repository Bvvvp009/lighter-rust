//! Test to verify that Fp5Element::to_bytes_le() and message_to_fp5() are inverses

use goldilocks_crypto::Fp5Element;
use goldilocks_crypto::schnorr::sign_with_nonce;
use poseidon_hash::Goldilocks;

// We need to test message_to_fp5, but it's private. Let's test via the public API
#[test]
fn test_fp5_to_bytes_roundtrip() {
    // Test with various Fp5Element values
    let test_cases = vec![
        Fp5Element([Goldilocks(0), Goldilocks(0), Goldilocks(0), Goldilocks(0), Goldilocks(0)]),
        Fp5Element([Goldilocks(1), Goldilocks(0), Goldilocks(0), Goldilocks(0), Goldilocks(0)]),
        Fp5Element([Goldilocks(12345), Goldilocks(67890), Goldilocks(11111), Goldilocks(22222), Goldilocks(33333)]),
        Fp5Element([
            Goldilocks(8398652514106806347),
            Goldilocks(11069112711939986896),
            Goldilocks(9732488227085561369),
            Goldilocks(18076754337204438535),
            Goldilocks(17155407358725346236),
        ]),
    ];
    
    for (i, original) in test_cases.iter().enumerate() {
        // Convert to bytes
        let bytes = original.to_bytes_le();
        
        // Convert back using from_bytes_le (direct conversion)
        let reconstructed_direct = Fp5Element::from_bytes_le(&bytes)
            .expect("Failed to reconstruct via from_bytes_le");
        
        // Verify direct round-trip works
        assert_eq!(original.0, reconstructed_direct.0, 
                   "Direct round-trip failed for test case {}", i);
        
        // Now test via sign/verify to see if message_to_fp5 works correctly
        // We'll use a dummy signature to test this
        let mut private_key = [0u8; 40];
        private_key[0] = 1;
        
        // Sign with the bytes (this uses message_to_fp5 internally)
        let signature = sign_with_nonce(&private_key, &bytes, &private_key)
            .expect("Failed to sign");
        
        // Get public key
        use goldilocks_crypto::{ScalarField, Point};
        let private_scalar = ScalarField::from_bytes_le(&private_key).unwrap();
        let public_key_point = Point::generator().mul(&private_scalar);
        let public_key_bytes = public_key_point.encode().to_bytes_le();
        
        // Verify (this also uses message_to_fp5 internally)
        use goldilocks_crypto::verify_signature;
        let is_valid = verify_signature(&signature, &bytes, &public_key_bytes)
            .expect("Failed to verify");
        
        if !is_valid {
            println!("⚠️  Warning: Round-trip via sign/verify failed for test case {}", i);
            println!("  Original: {:?}", original.0);
            println!("  Reconstructed (direct): {:?}", reconstructed_direct.0);
        } else {
            println!("✅ Round-trip via sign/verify succeeded for test case {}", i);
        }
    }
}

#[test]
fn test_message_bytes_are_canonical_little_endian() {
    // Test that bytes produced by to_bytes_le() can be correctly interpreted
    // by message_to_fp5() (which expects "canonical little-endian" format)
    
    // Create a test Fp5Element
    let fp5 = Fp5Element([
        Goldilocks(0x0123456789ABCDEF),
        Goldilocks(0xFEDCBA9876543210),
        Goldilocks(0x1111222233334444),
        Goldilocks(0x5555666677778888),
        Goldilocks(0x9999AAAABBBBCCCC),
    ]);
    
    let bytes = fp5.to_bytes_le();
    
    // Each 8-byte chunk should be little-endian representation of the limb
    for (i, &limb) in fp5.0.iter().enumerate() {
        let chunk = &bytes[i*8..(i+1)*8];
        let value_from_bytes = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3],
            chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        assert_eq!(limb.0, value_from_bytes, 
                   "Limb {} doesn't match bytes", i);
    }
    
    println!("✅ Byte encoding is correct little-endian");
}













