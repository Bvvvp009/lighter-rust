//! Binary Merkle tree built with the Poseidon2 hash over the Goldilocks field.
//!
//! Leaves are padded to the next power of two with zero hashes so the tree is
//! always complete and proofs have a fixed size (`depth` sibling nodes).
//!
//! # Example
//!
//! ```rust
//! use poseidon_hash::{Goldilocks, hash_no_pad};
//! use poseidon_hash::merkle::MerkleTree;
//!
//! let leaves: Vec<_> = (1u64..=8)
//!     .map(|i| hash_no_pad(&[Goldilocks::from_canonical_u64(i)]))
//!     .collect();
//!
//! let tree = MerkleTree::build(&leaves);
//! assert_eq!(tree.depth(), 3);           // 8 leaves → depth 3
//! assert_eq!(tree.leaf_count(), 8);
//!
//! // Generate and verify an inclusion proof.
//! let proof = tree.prove(2).unwrap();
//! assert!(MerkleTree::verify(tree.root(), &proof, leaves[2]));
//! ```

use crate::{Goldilocks, HashOut, hash_no_pad};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Combines two `HashOut` values into one using Poseidon2.
pub fn hash_two_to_one(left: HashOut, right: HashOut) -> HashOut {
    let combined = [
        left[0],  left[1],  left[2],  left[3],
        right[0], right[1], right[2], right[3],
    ];
    hash_no_pad(&combined)
}

/// The canonical *zero* (empty) leaf hash used to pad a tree to a power of two.
pub fn zero_hash() -> HashOut {
    [Goldilocks::zero(); 4]
}

// ---------------------------------------------------------------------------
// MerkleProof
// ---------------------------------------------------------------------------

/// A Merkle inclusion proof for a single leaf.
///
/// Produced by [`MerkleTree::prove`] and verified by [`MerkleTree::verify`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    /// Position of the proven leaf in the *original* (unpadded) leaf list.
    pub leaf_index: usize,
    /// Sibling hashes from leaf level up to (but not including) the root.
    /// `siblings[0]` is the sibling at the leaf level; `siblings[last]` is
    /// the sibling just below the root.
    pub siblings: Vec<HashOut>,
}

// ---------------------------------------------------------------------------
// MerkleTree
// ---------------------------------------------------------------------------

/// A complete binary Merkle tree built with Poseidon2.
///
/// Internal layout: `layers[0]` are the padded leaves,
/// `layers[depth()]` is a single-element vec containing the root.
pub struct MerkleTree {
    layers: Vec<Vec<HashOut>>,
    /// Number of *original* (unpadded) leaves supplied to `build`.
    original_leaf_count: usize,
}

impl MerkleTree {
    /// Builds a Merkle tree from the given leaf hashes.
    ///
    /// If `leaves` is empty the tree has a single zero-hash root.
    /// Otherwise leaves are padded to the next power of two and the tree
    /// is built bottom-up.
    pub fn build(leaves: &[HashOut]) -> Self {
        if leaves.is_empty() {
            return Self {
                layers: vec![vec![zero_hash()], vec![zero_hash()]],
                original_leaf_count: 0,
            };
        }

        let original_leaf_count = leaves.len();

        // Pad to next power of two, with a minimum of 2 so every non-empty
        // tree has at least one level of hashing (depth ≥ 1).
        let padded_len = leaves.len().next_power_of_two().max(2);
        let mut current: Vec<HashOut> = Vec::with_capacity(padded_len);
        current.extend_from_slice(leaves);
        while current.len() < padded_len {
            current.push(zero_hash());
        }

        let mut layers: Vec<Vec<HashOut>> = vec![current];

        // Build layers upward.
        loop {
            let prev = layers.last().unwrap();
            if prev.len() == 1 {
                break;
            }
            let next: Vec<HashOut> = prev
                .chunks(2)
                .map(|pair| hash_two_to_one(pair[0], pair[1]))
                .collect();
            layers.push(next);
        }

        Self { layers, original_leaf_count }
    }

    /// Returns the Merkle root hash.
    pub fn root(&self) -> HashOut {
        *self.layers.last().unwrap().first().unwrap()
    }

    /// Returns the depth of the tree (the number of sibling nodes in a proof).
    ///
    /// A tree with one or two leaves has depth 1; with 3–4 leaves depth 2; etc.
    /// The empty tree (built from an empty slice) has depth 1 (one hashing level).
    pub fn depth(&self) -> usize {
        self.layers.len().saturating_sub(1)
    }

    /// Returns the number of original (unpadded) leaves.
    pub fn leaf_count(&self) -> usize {
        self.original_leaf_count
    }

    /// Returns the padded leaf at `index` (including zero-padded leaves).
    pub fn padded_leaf(&self, index: usize) -> Option<HashOut> {
        self.layers.first()?.get(index).copied()
    }

    /// Generates an inclusion proof for the leaf at `index`.
    ///
    /// Returns `None` if `index` is out of the range `[0, leaf_count())`.
    pub fn prove(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.original_leaf_count {
            return None;
        }

        let mut siblings = Vec::with_capacity(self.depth());
        let mut idx = index;

        for layer in &self.layers[..self.layers.len().saturating_sub(1)] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            siblings.push(
                layer
                    .get(sibling_idx)
                    .copied()
                    .unwrap_or_else(zero_hash),
            );
            idx /= 2;
        }

        Some(MerkleProof { leaf_index: index, siblings })
    }

    /// Verifies that `leaf` at `proof.leaf_index` is committed to by `root`.
    ///
    /// Returns `true` iff the proof is valid.
    pub fn verify(root: HashOut, proof: &MerkleProof, leaf: HashOut) -> bool {
        let mut current = leaf;
        let mut idx = proof.leaf_index;

        for &sibling in &proof.siblings {
            current = if idx % 2 == 0 {
                hash_two_to_one(current, sibling)
            } else {
                hash_two_to_one(sibling, current)
            };
            idx /= 2;
        }

        current == root
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Goldilocks;

    fn leaf(n: u64) -> HashOut {
        hash_no_pad(&[Goldilocks::from_canonical_u64(n)])
    }

    #[test]
    fn empty_tree_has_zero_root() {
        let tree = MerkleTree::build(&[]);
        assert_eq!(tree.root(), zero_hash());
        assert_eq!(tree.depth(), 1);
        assert_eq!(tree.leaf_count(), 0);
    }

    #[test]
    fn single_leaf_tree() {
        let l = leaf(42);
        let tree = MerkleTree::build(&[l]);
        // Padded to 1 leaf (already power of two), root == hash(leaf, zero)
        // depth is 1 because we pad to 2 then hash once.
        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(tree.depth(), 1);
        // Root should be deterministic
        let expected_root = hash_two_to_one(l, zero_hash());
        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn two_leaf_tree_root() {
        let l0 = leaf(1);
        let l1 = leaf(2);
        let tree = MerkleTree::build(&[l0, l1]);
        assert_eq!(tree.depth(), 1);
        assert_eq!(tree.root(), hash_two_to_one(l0, l1));
    }

    #[test]
    fn eight_leaf_tree_depth() {
        let leaves: Vec<_> = (1u64..=8).map(leaf).collect();
        let tree = MerkleTree::build(&leaves);
        assert_eq!(tree.depth(), 3);
        assert_eq!(tree.leaf_count(), 8);
    }

    #[test]
    fn prove_and_verify_all_leaves() {
        let leaves: Vec<_> = (0u64..8).map(leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let root = tree.root();

        for i in 0..leaves.len() {
            let proof = tree.prove(i).expect("proof must exist for valid index");
            assert_eq!(proof.leaf_index, i);
            assert_eq!(proof.siblings.len(), tree.depth());
            assert!(
                MerkleTree::verify(root, &proof, leaves[i]),
                "proof for leaf {i} must verify"
            );
        }
    }

    #[test]
    fn proof_out_of_range_returns_none() {
        let leaves: Vec<_> = (0u64..4).map(leaf).collect();
        let tree = MerkleTree::build(&leaves);
        assert!(tree.prove(4).is_none());
        assert!(tree.prove(100).is_none());
    }

    #[test]
    fn tampered_leaf_fails_verification() {
        let leaves: Vec<_> = (0u64..4).map(leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let root = tree.root();
        let proof = tree.prove(1).unwrap();
        let wrong_leaf = leaf(999);
        assert!(!MerkleTree::verify(root, &proof, wrong_leaf));
    }

    #[test]
    fn tampered_sibling_fails_verification() {
        let leaves: Vec<_> = (0u64..4).map(leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let root = tree.root();
        let mut proof = tree.prove(0).unwrap();
        proof.siblings[0] = leaf(999); // corrupt first sibling
        assert!(!MerkleTree::verify(root, &proof, leaves[0]));
    }

    #[test]
    fn non_power_of_two_leaves() {
        // 5 leaves → padded to 8
        let leaves: Vec<_> = (0u64..5).map(leaf).collect();
        let tree = MerkleTree::build(&leaves);
        assert_eq!(tree.depth(), 3);
        assert_eq!(tree.leaf_count(), 5);

        // All 5 original leaves should have valid proofs.
        let root = tree.root();
        for i in 0..5 {
            let proof = tree.prove(i).unwrap();
            assert!(MerkleTree::verify(root, &proof, leaves[i]));
        }
        // Index 5..8 are out of the original range.
        assert!(tree.prove(5).is_none());
    }

    #[test]
    fn different_orderings_produce_different_roots() {
        let fwd: Vec<_> = (0u64..4).map(leaf).collect();
        let rev: Vec<_> = (0u64..4).rev().map(leaf).collect();
        let t1 = MerkleTree::build(&fwd);
        let t2 = MerkleTree::build(&rev);
        assert_ne!(t1.root(), t2.root());
    }

    #[test]
    fn root_is_deterministic() {
        let leaves: Vec<_> = (0u64..4).map(leaf).collect();
        let t1 = MerkleTree::build(&leaves);
        let t2 = MerkleTree::build(&leaves);
        assert_eq!(t1.root(), t2.root());
    }
}
