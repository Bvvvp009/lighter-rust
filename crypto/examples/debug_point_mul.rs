use goldilocks_crypto::{Point, ScalarField};

fn main() {
    println!("=== Testing scalar multiplication with simple values ===\n");
    
    let g = Point::generator();
    
    // Test scalar = 1
    println!("Scalar = 1:");
    let sk1 = ScalarField::new([1, 0, 0, 0, 0]);
    let pk1 = g.mul(&sk1);
    let encoded1 = pk1.encode().to_bytes_le();
    println!("  G*1: {}", hex::encode(&encoded1));
    
    // Test scalar = 2
    println!("\nScalar = 2:");
    let sk2 = ScalarField::new([2, 0, 0, 0, 0]);
    let pk2 = g.mul(&sk2);
    let encoded2 = pk2.encode().to_bytes_le();
    println!("  G*2: {}", hex::encode(&encoded2));
    
    // Test scalar = 3
    println!("\nScalar = 3:");
    let sk3 = ScalarField::new([3, 0, 0, 0, 0]);
    let pk3 = g.mul(&sk3);
    let encoded3 = pk3.encode().to_bytes_le();
    println!("  G*3: {}", hex::encode(&encoded3));
    
    // Test the actual private key
    println!("\nActual private key:");
    let privbytes = [
        0xc5u8, 0x23, 0x0d, 0x52, 0x49, 0x2a, 0x60, 0x89,
        0x54, 0x47, 0x6c, 0x66, 0xf3, 0xbe, 0x44, 0x55,
        0x94, 0x60, 0xd1, 0x01, 0xdc, 0xce, 0xc8, 0xd4,
        0xe2, 0xe8, 0xd2, 0xca, 0xf4, 0xf3, 0xb9, 0x83,
        0xe7, 0x73, 0x89, 0x56, 0x3d, 0xf7, 0x2f, 0x51,
    ];
    let sk = ScalarField::from_bytes_le(&privbytes).expect("Invalid private key");
    println!("  Loaded scalar limbs: {:?}", sk.limbs());
    let sk_canon = sk.to_canonical();
    println!("  Canonical form limbs: {:?}", sk_canon.limbs());
    
    let pk = g.mul(&sk);
    let encoded_pk = pk.encode().to_bytes_le();
    println!("  G*privkey: {}", hex::encode(&encoded_pk));
    
    // Debug: Try multiplying by canonical version too
    let pk_canon = g.mul(&sk_canon);
    let encoded_pk_canon = pk_canon.encode().to_bytes_le();
    println!("  G*privkey_canonical: {}", hex::encode(&encoded_pk_canon));
}
