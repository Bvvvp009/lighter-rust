//! Debug Montgomery vs canonical form

use goldilocks_crypto::ScalarField;
use hex;

fn limbs_to_bytes(limbs: [u64; 5]) -> [u8; 40] {
    let mut bytes = [0u8; 40];
    for (i, &limb) in limbs.iter().enumerate() {
        let start = i * 8;
        bytes[start..start + 8].copy_from_slice(&limb.to_le_bytes());
    }
    bytes
}

#[test]
fn test_montgomery_canonical() {
    println!("\n=== Montgomery vs Canonical ===\n");
    
    let two_bytes = limbs_to_bytes([2, 0, 0, 0, 0]);
    let two = ScalarField::from_bytes_le(&two_bytes).unwrap();
    
    println!("Original: {:?}", two.0);
    
    let two_canonical = two.to_canonical();
    println!("After to_canonical: {:?}", two_canonical.0);
    
    // Check if they're the same
    println!("\nAre they the same? {}", two.0 == two_canonical.0);
    
    // What about the bytes?
    let two_bytes_after = two_canonical.to_bytes_le();
    println!("\nOriginal bytes: {}", hex::encode(&two_bytes));
    println!("After canonical: {}", hex::encode(&two_bytes_after));
}
