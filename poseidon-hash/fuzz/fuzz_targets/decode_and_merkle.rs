#![no_main]

use libfuzzer_sys::fuzz_target;
use poseidon_hash::merkle::MerkleTree;
use poseidon_hash::{hash_no_pad, hash_out_from_bytes_le, hash_to_quintic_extension, Fp5Element, Goldilocks};

fuzz_target!(|data: &[u8]| {
    let _ = Goldilocks::from_bytes_le(data);
    let _ = Fp5Element::from_bytes_le(data);
    let _ = hash_out_from_bytes_le(data);

    let elements: Vec<Goldilocks> = data
        .chunks(8)
        .take(32)
        .map(|chunk| {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            Goldilocks::from_noncanonical_u64(u64::from_le_bytes(bytes))
        })
        .collect();

    let _ = hash_no_pad(&elements);
    let _ = hash_to_quintic_extension(&elements);

    let leaves: Vec<_> = elements
        .chunks(4)
        .map(|chunk| {
            let mut leaf = [Goldilocks::zero(); 4];
            for (index, value) in chunk.iter().enumerate() {
                leaf[index] = *value;
            }
            hash_no_pad(&leaf)
        })
        .collect();

    let tree = MerkleTree::build(&leaves);
    if !leaves.is_empty() {
        let index = data[0] as usize % leaves.len();
        if let Some(proof) = tree.prove(index) {
            let _ = MerkleTree::verify(tree.root(), &proof, leaves[index]);
        }
    }
});
