use poseidon_hash::{Goldilocks, hash_to_quintic_extension, Fp5Element};

fn main() {
    println!("Testing poseidon-hash library exports...");
    
    // Test Goldilocks
    let a = Goldilocks::from_canonical_u64(42);
    let b = Goldilocks::from_canonical_u64(10);
    let sum = a.add(&b);
    let product = a.mul(&b);
    println!("Goldilocks: {} + {} = {}", a.to_canonical_u64(), b.to_canonical_u64(), sum.to_canonical_u64());
    println!("Goldilocks: {} * {} = {}", a.to_canonical_u64(), b.to_canonical_u64(), product.to_canonical_u64());
    
    // Test Fp5Element
    let elem1 = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
    let elem2 = Fp5Element::one();
    let product = elem1.mul(&elem2);
    let bytes = product.to_bytes_le();
    println!("Fp5Element bytes length: {}", bytes.len());
    
    // Test Poseidon2 hash
    let elements = vec![
        Goldilocks::from_canonical_u64(1),
        Goldilocks::from_canonical_u64(2),
        Goldilocks::from_canonical_u64(3),
    ];
    let hash = hash_to_quintic_extension(&elements);
    println!("Poseidon2 hash computed successfully");
    
    println!("All poseidon-hash exports work correctly!");
}






