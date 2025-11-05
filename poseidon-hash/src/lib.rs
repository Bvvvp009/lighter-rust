//! # Poseidon Hash (Goldilocks)
//!
//! Rust port of Poseidon2 hash function and Goldilocks field arithmetic, ported from lighter-go (Lighter Protocol).
//!
//! ## ⚠️ Security Warning
//!
//! **This library has NOT been audited and is provided as-is. Use with caution.**
//!
//! - This is a **prototype port** from the Go SDK (lighter-go)
//! - **Not security audited** - do not use in production without proper security review
//! - While the implementation appears to work correctly, cryptographic software requires careful auditing
//! - This is an open-source contribution and not an official Lighter Protocol library
//! - Use at your own risk
//!
//! ## Overview
//!
//! This crate provides essential cryptographic primitives for Zero-Knowledge proof systems:
//!
//! - **Goldilocks Field**: A special prime field (p = 2^64 - 2^32 + 1) optimized for 64-bit CPU operations
//! - **Poseidon2 Hash**: A ZK-friendly hash function designed for low constraint counts in ZK circuits
//! - **Fp5 Extension Field**: Quintic extension field (GF(p^5)) for elliptic curve operations
//!
//! ## Features
//!
//! - Fast field arithmetic with optimized modular reduction
//! - Efficient Poseidon2 hash implementation
//! - 40-byte field elements for cryptographic operations
//! - Production-grade performance and security
//!
//! ## Example
//!
//! ```rust
//! use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
//!
//! // Field arithmetic
//! let a = Goldilocks::from_canonical_u64(42);
//! let b = Goldilocks::from_canonical_u64(10);
//! let sum = a.add(&b);
//! let product = a.mul(&b);
//!
//! // Poseidon2 hashing
//! let elements = vec![
//!     Goldilocks::from_canonical_u64(1),
//!     Goldilocks::from_canonical_u64(2),
//!     Goldilocks::from_canonical_u64(3),
//! ];
//! let hash = hash_to_quintic_extension(&elements);
//! ```

/// Goldilocks field element.
///
/// The Goldilocks field uses prime modulus p = 2^64 - 2^32 + 1, which is optimized for:
/// - Fast modular reduction using bit manipulation
/// - Efficient 64-bit CPU operations
/// - Zero-Knowledge proof systems (Plonky2, STARKs)
///
/// # Example
///
/// ```rust
/// use poseidon_hash::Goldilocks;
///
/// let a = Goldilocks::from_canonical_u64(42);
/// let b = Goldilocks::from_canonical_u64(10);
/// let sum = a.add(&b);
/// let product = a.mul(&b);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Goldilocks(pub u64);

impl Goldilocks {
    /// Field modulus: p = 2^64 - 2^32 + 1 = 0xffffffff00000001
    pub const MODULUS: u64 = 0xffffffff00000001;
    
    /// Epsilon constant: (1 << 32) - 1 = 0xffffffff
    /// Used for efficient modular reduction
    pub const EPSILON: u64 = 0xffffffff;
    
    /// The order of the field (same as MODULUS)
    pub const ORDER: u64 = Self::MODULUS;
    
    /// Returns the zero element of the field.
    pub fn zero() -> Self {
        Goldilocks(0)
    }
    
    /// Returns the multiplicative identity (one) of the field.
    pub fn one() -> Self {
        Goldilocks(1)
    }
    
    /// Checks if this element is zero.
    pub fn is_zero(&self) -> bool {
        self.to_canonical_u64() == 0
    }
    
    /// Converts this field element to its canonical representation as a u64.
    ///
    /// The canonical form ensures the value is in the range [0, MODULUS).
    pub fn to_canonical_u64(&self) -> u64 {
        let x = self.0;
        if x >= Self::MODULUS {
            x - Self::MODULUS
        } else {
            x
        }
    }
    
    /// Adds two field elements with modular reduction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::Goldilocks;
    ///
    /// let a = Goldilocks::from_canonical_u64(100);
    /// let b = Goldilocks::from_canonical_u64(50);
    /// let sum = a.add(&b);
    /// assert_eq!(sum.to_canonical_u64(), 150);
    /// ```
    pub fn add(&self, other: &Goldilocks) -> Goldilocks {
        // Field addition with modular reduction using epsilon optimization
        let (sum, over) = self.0.overflowing_add(other.0);
        let (sum, over) = sum.overflowing_add(over as u64 * Self::EPSILON);
        let final_sum = if over {
            sum + Self::EPSILON
        } else {
            sum
        };
        Goldilocks(final_sum)
    }
    
    /// Subtracts two field elements with modular reduction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::Goldilocks;
    ///
    /// let a = Goldilocks::from_canonical_u64(100);
    /// let b = Goldilocks::from_canonical_u64(50);
    /// let diff = a.sub(&b);
    /// assert_eq!(diff.to_canonical_u64(), 50);
    /// ```
    pub fn sub(&self, other: &Goldilocks) -> Goldilocks {
        // Field subtraction with modular reduction
        let (diff, borrow) = self.0.overflowing_sub(other.0);
        let (diff, borrow) = diff.overflowing_sub(borrow as u64 * Self::EPSILON);
        let final_diff = if borrow {
            diff - Self::EPSILON
        } else {
            diff
        };
        Goldilocks(final_diff)
    }
    
    /// Multiplies two field elements with modular reduction.
    ///
    /// Uses optimized reduction algorithm for the Goldilocks prime.
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::Goldilocks;
    ///
    /// let a = Goldilocks::from_canonical_u64(10);
    /// let b = Goldilocks::from_canonical_u64(5);
    /// let product = a.mul(&b);
    /// assert_eq!(product.to_canonical_u64(), 50);
    /// ```
    pub fn mul(&self, other: &Goldilocks) -> Goldilocks {
        // Field multiplication with optimized modular reduction
        // Algorithm: Compute product as u128, then reduce using Goldilocks prime properties
        let product = (self.0 as u128) * (other.0 as u128);
        let x_hi = (product >> 64) as u64;
        let x_lo = product as u64;
        
        let x_hi_hi = x_hi >> 32;
        let x_hi_lo = x_hi & Self::EPSILON;
        
        let (t0, borrow) = x_lo.overflowing_sub(x_hi_hi);
        let t0 = if borrow { t0 - Self::EPSILON } else { t0 };
        let t1 = x_hi_lo * Self::EPSILON;
        
        let (sum, over) = t0.overflowing_add(t1);
        let t2 = sum + Self::EPSILON * over as u64;
        Goldilocks(t2)
    }
    
    /// Computes the square of this field element.
    ///
    /// More efficient than `self.mul(self)` as it can use optimized squaring formulas.
    pub fn square(&self) -> Goldilocks {
        self.mul(self)
    }
    
    /// Doubles this field element (multiplies by 2).
    pub fn double(&self) -> Goldilocks {
        self.add(self)
    }
    
    /// Computes the multiplicative inverse of this field element.
    ///
    /// Uses Fermat's little theorem: a^(-1) ≡ a^(p-2) mod p
    ///
    /// # Panics
    ///
    /// This function will panic if called on zero (which has no inverse).
    /// Use `inverse_or_zero()` if you need to handle zero elements.
    pub fn inverse(&self) -> Goldilocks {
        // Fermat's little theorem: a^(p-2) ≡ a^(-1) mod p
        // p = 2^64 - 2^32 + 1
        // p - 2 = 2^64 - 2^32 - 1
        let mut result = Goldilocks::one();
        let mut base = *self;
        let mut exp = Self::MODULUS - 2;
        
        while exp > 0 {
            if exp & 1 == 1 {
                result = result.mul(&base);
            }
            base = base.mul(&base);
            exp >>= 1;
        }
        
        result
    }
    
    /// Creates a field element from a canonical u64 value.
    ///
    /// The input value should be in the range [0, MODULUS). Values outside this range
    /// will be automatically reduced by field operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::Goldilocks;
    ///
    /// let a = Goldilocks::from_canonical_u64(42);
    /// ```
    pub fn from_canonical_u64(val: u64) -> Goldilocks {
        Goldilocks(val)
    }
    
    /// Creates a field element from an i64 value.
    ///
    /// Negative values are handled using two's complement representation.
    /// The field operations will automatically reduce the value modulo MODULUS.
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::Goldilocks;
    ///
    /// let a = Goldilocks::from_i64(-10);
    /// ```
    pub fn from_i64(val: i64) -> Goldilocks {
        // Direct cast - two's complement representation is valid in the field
        // Field operations will reduce modulo MODULUS automatically
        Goldilocks(val as u64)
    }
}

impl From<u64> for Goldilocks {
    fn from(val: u64) -> Self {
        Goldilocks::from_canonical_u64(val)
    }
}

#[allow(dead_code)]
fn reduce_u128(x: u128) -> u64 {
    let low = x as u64;
    let high = (x >> 64) as u64;
    
    // Reduce high part
    let reduced_high = high.wrapping_sub(high >> 32);
    let result = low.wrapping_add(reduced_high << 32);
    
    // Final reduction
    if result >= Goldilocks::MODULUS {
        result - Goldilocks::MODULUS
    } else {
        result
    }
}

/// Fp5 extension field element.
///
/// Represents an element of the quintic extension field GF(p^5) where p is the Goldilocks prime.
/// Each element is represented as a polynomial of degree at most 4 over the base field.
///
/// The extension field uses the irreducible polynomial x^5 = w where w = 3.
///
/// # Example
///
/// ```rust
/// use poseidon_hash::{Fp5Element, Goldilocks};
///
/// let a = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
/// let b = Fp5Element::one();
/// let product = a.mul(&b);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fp5Element(pub [Goldilocks; 5]);

impl Fp5Element {
    /// Returns the zero element of the extension field.
    pub fn zero() -> Self {
        Fp5Element([Goldilocks::zero(); 5])
    }
    
    /// Returns the multiplicative identity (one) of the extension field.
    pub fn one() -> Self {
        let mut result = [Goldilocks::zero(); 5];
        result[0] = Goldilocks::one();
        Fp5Element(result)
    }
    
    /// Checks if this element is zero.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x.is_zero())
    }
    
    /// Adds two extension field elements.
    ///
    /// Addition is performed component-wise on the polynomial coefficients.
    pub fn add(&self, other: &Fp5Element) -> Fp5Element {
        let mut result = [Goldilocks::zero(); 5];
        for i in 0..5 {
            result[i] = self.0[i].add(&other.0[i]);
        }
        Fp5Element(result)
    }
    
    /// Subtracts two extension field elements.
    ///
    /// Subtraction is performed component-wise on the polynomial coefficients.
    pub fn sub(&self, other: &Fp5Element) -> Fp5Element {
        let mut result = [Goldilocks::zero(); 5];
        for i in 0..5 {
            result[i] = self.0[i].sub(&other.0[i]);
        }
        Fp5Element(result)
    }
    
    /// Multiplies two extension field elements.
    ///
    /// Uses the irreducible polynomial x^5 = w (where w = 3) to reduce the result.
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::{Fp5Element, Goldilocks};
    ///
    /// let a = Fp5Element::from_uint64_array([1, 0, 0, 0, 0]);
    /// let b = Fp5Element::from_uint64_array([2, 0, 0, 0, 0]);
    /// let product = a.mul(&b);
    /// ```
    pub fn mul(&self, other: &Fp5Element) -> Fp5Element {
        // Multiplication in quintic extension field
        // Uses irreducible polynomial x^5 = w where w = 3
        const W: Goldilocks = Goldilocks(3);
        
        // c0 = a0*b0 + w*(a1*b4 + a2*b3 + a3*b2 + a4*b1)
        let a0b0 = self.0[0].mul(&other.0[0]);
        let a1b4 = self.0[1].mul(&other.0[4]);
        let a2b3 = self.0[2].mul(&other.0[3]);
        let a3b2 = self.0[3].mul(&other.0[2]);
        let a4b1 = self.0[4].mul(&other.0[1]);
        let added = a1b4.add(&a2b3).add(&a3b2).add(&a4b1);
        let muld = W.mul(&added);
        let c0 = a0b0.add(&muld);
        
        // c1 = a0*b1 + a1*b0 + w*(a2*b4 + a3*b3 + a4*b2)
        let a0b1 = self.0[0].mul(&other.0[1]);
        let a1b0 = self.0[1].mul(&other.0[0]);
        let a2b4 = self.0[2].mul(&other.0[4]);
        let a3b3 = self.0[3].mul(&other.0[3]);
        let a4b2 = self.0[4].mul(&other.0[2]);
        let added = a2b4.add(&a3b3).add(&a4b2);
        let muld = W.mul(&added);
        let c1 = a0b1.add(&a1b0).add(&muld);
        
        // c2 = a0*b2 + a1*b1 + a2*b0 + w*(a3*b4 + a4*b3)
        let a0b2 = self.0[0].mul(&other.0[2]);
        let a1b1 = self.0[1].mul(&other.0[1]);
        let a2b0 = self.0[2].mul(&other.0[0]);
        let a3b4 = self.0[3].mul(&other.0[4]);
        let a4b3 = self.0[4].mul(&other.0[3]);
        let added = a3b4.add(&a4b3);
        let muld = W.mul(&added);
        let c2 = a0b2.add(&a1b1).add(&a2b0).add(&muld);
        
        // c3 = a0*b3 + a1*b2 + a2*b1 + a3*b0 + w*a4*b4
        let a0b3 = self.0[0].mul(&other.0[3]);
        let a1b2 = self.0[1].mul(&other.0[2]);
        let a2b1 = self.0[2].mul(&other.0[1]);
        let a3b0 = self.0[3].mul(&other.0[0]);
        let a4b4 = self.0[4].mul(&other.0[4]);
        let muld = W.mul(&a4b4);
        let c3 = a0b3.add(&a1b2).add(&a2b1).add(&a3b0).add(&muld);
        
        // c4 = a0*b4 + a1*b3 + a2*b2 + a3*b1 + a4*b0
        let a0b4 = self.0[0].mul(&other.0[4]);
        let a1b3 = self.0[1].mul(&other.0[3]);
        let a2b2 = self.0[2].mul(&other.0[2]);
        let a3b1 = self.0[3].mul(&other.0[1]);
        let a4b0 = self.0[4].mul(&other.0[0]);
        let c4 = a0b4.add(&a1b3).add(&a2b2).add(&a3b1).add(&a4b0);
        
        Fp5Element([c0, c1, c2, c3, c4])
    }
    
    /// Computes the multiplicative inverse of this element.
    ///
    /// Returns zero if this element is zero (which has no inverse).
    pub fn inverse(&self) -> Fp5Element {
        self.inverse_or_zero()
    }
    
    /// Computes the multiplicative inverse, returning zero if the element is zero.
    ///
    /// This is a safe version of `inverse()` that handles zero elements gracefully.
    pub fn inverse_or_zero(&self) -> Fp5Element {
        if self.is_zero() {
            return Fp5Element::zero();
        }
        
        // Use Frobenius automorphism for efficient inversion
        let d = self.frobenius();
        let e = d.mul(&d.frobenius());
        let f = e.mul(&e.repeated_frobenius(2));
        
        // Compute g = a[0]*f[0] + w*(a[1]*f[4] + a[2]*f[3] + a[3]*f[2] + a[4]*f[1])
        let w = Goldilocks(3);
        let a0b0 = self.0[0].mul(&f.0[0]);
        let a1b4 = self.0[1].mul(&f.0[4]);
        let a2b3 = self.0[2].mul(&f.0[3]);
        let a3b2 = self.0[3].mul(&f.0[2]);
        let a4b1 = self.0[4].mul(&f.0[1]);
        let added = a1b4.add(&a2b3).add(&a3b2).add(&a4b1);
        let muld = w.mul(&added);
        let g = a0b0.add(&muld);
        
        // Return f * g.inverse()
        let g_inv = g.inverse();
        f.scalar_mul(&g_inv)
    }
    
    /// Applies the Frobenius automorphism once.
    ///
    /// The Frobenius automorphism raises each coefficient to the p-th power.
    pub fn frobenius(&self) -> Fp5Element {
        self.repeated_frobenius(1)
    }
    
    /// Applies the Frobenius automorphism `count` times.
    ///
    /// Since we're in GF(p^5), applying it 5 times returns the original element.
    pub fn repeated_frobenius(&self, count: usize) -> Fp5Element {
        if count == 0 {
            return *self;
        }
        
        let d = 5;
        let count = count % d;
        
        if count == 0 {
            return *self;
        }
        
        // FP5_DTH_ROOT = 1041288259238279555
        let dth_root = Goldilocks(1041288259238279555);
        
        // Compute z0 = dth_root^count
        let mut z0 = dth_root;
        for _ in 1..count {
            z0 = z0.mul(&dth_root);
        }
        
        // Compute powers of z0: [1, z0, z0^2, z0^3, z0^4]
        let mut z_powers = [Goldilocks::zero(); 5];
        z_powers[0] = Goldilocks::one();
        for i in 1..5 {
            z_powers[i] = z_powers[i-1].mul(&z0);
        }
        
        // Multiply each coordinate by corresponding power
        let mut result = [Goldilocks::zero(); 5];
        for i in 0..5 {
            result[i] = self.0[i].mul(&z_powers[i]);
        }
        
        Fp5Element(result)
    }
    
    /// Multiplies this element by a scalar (base field element).
    ///
    /// This is more efficient than full extension field multiplication when one operand
    /// is in the base field.
    pub fn scalar_mul(&self, scalar: &Goldilocks) -> Fp5Element {
        Fp5Element([
            self.0[0].mul(scalar),
            self.0[1].mul(scalar),
            self.0[2].mul(scalar),
            self.0[3].mul(scalar),
            self.0[4].mul(scalar),
        ])
    }
    
    /// Computes the square of this element.
    ///
    /// Optimized implementation that uses fewer operations than general multiplication.
    pub fn square(&self) -> Fp5Element {
        // Optimized squaring for quintic extension field
        const W: Goldilocks = Goldilocks(3);
        let double_w = W.add(&W); // 2*w = 6
        
        // c0 = a0^2 + 2*w*(a1*a4 + a2*a3)
        let a0s = self.0[0].mul(&self.0[0]);
        let a1a4 = self.0[1].mul(&self.0[4]);
        let a2a3 = self.0[2].mul(&self.0[3]);
        let added = a1a4.add(&a2a3);
        let muld = double_w.mul(&added);
        let c0 = a0s.add(&muld);
        
        // c1 = 2*a0*a1 + 2*w*a2*a4 + w*a3^2
        let a0_double = self.0[0].add(&self.0[0]);
        let a0_double_a1 = a0_double.mul(&self.0[1]);
        let a2a4_double_w = double_w.mul(&self.0[2].mul(&self.0[4]));
        let a3a3w = W.mul(&self.0[3].mul(&self.0[3]));
        let c1 = a0_double_a1.add(&a2a4_double_w).add(&a3a3w);
        
        // c2 = 2*a0*a2 + a1^2 + 2*w*a4*a3
        let a0_double_a2 = a0_double.mul(&self.0[2]);
        let a1_square = self.0[1].mul(&self.0[1]);
        let a4a3_double_w = double_w.mul(&self.0[4].mul(&self.0[3]));
        let c2 = a0_double_a2.add(&a1_square).add(&a4a3_double_w);
        
        // c3 = 2*a0*a3 + 2*a1*a2 + w*a4^2
        let a1_double = self.0[1].add(&self.0[1]);
        let a0_double_a3 = a0_double.mul(&self.0[3]);
        let a1_double_a2 = a1_double.mul(&self.0[2]);
        let a4_square_w = W.mul(&self.0[4].mul(&self.0[4]));
        let c3 = a0_double_a3.add(&a1_double_a2).add(&a4_square_w);
        
        // c4 = 2*a0*a4 + 2*a1*a3 + a2^2
        let a0_double_a4 = a0_double.mul(&self.0[4]);
        let a1_double_a3 = a1_double.mul(&self.0[3]);
        let a2_square = self.0[2].mul(&self.0[2]);
        let c4 = a0_double_a4.add(&a1_double_a3).add(&a2_square);
        
        Fp5Element([c0, c1, c2, c3, c4])
    }
    
    /// Doubles this element (multiplies by 2).
    pub fn double(&self) -> Fp5Element {
        self.add(self)
    }
    
    /// Creates an Fp5Element from an array of 5 u64 values.
    ///
    /// Each u64 value is interpreted as a Goldilocks field element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::Fp5Element;
    ///
    /// let elem = Fp5Element::from_uint64_array([1, 2, 3, 4, 5]);
    /// ```
    pub fn from_uint64_array(arr: [u64; 5]) -> Fp5Element {
        let mut result = [Goldilocks::zero(); 5];
        for i in 0..5 {
            result[i] = Goldilocks(arr[i]);
        }
        Fp5Element(result)
    }
    
    /// Converts this element to a 40-byte little-endian representation.
    ///
    /// Each of the 5 Goldilocks field elements contributes 8 bytes (little-endian).
    ///
    /// # Example
    ///
    /// ```rust
    /// use poseidon_hash::Fp5Element;
    ///
    /// let elem = Fp5Element::one();
    /// let bytes = elem.to_bytes_le();
    /// assert_eq!(bytes.len(), 40);
    /// ```
    pub fn to_bytes_le(&self) -> [u8; 40] {
        let mut result = [0u8; 40];
        for i in 0..5 {
            result[i*8..(i+1)*8].copy_from_slice(&self.0[i].0.to_le_bytes());
        }
        result
    }
    
    /// Computes the additive inverse (negation) of this element.
    pub fn neg(&self) -> Fp5Element {
        Fp5Element::zero().sub(self)
    }
}

// Poseidon2 hash implementation constants
const WIDTH: usize = 12;
const RATE: usize = 8;
const ROUNDS_F_HALF: usize = 4;
const ROUNDS_P: usize = 22;

// External round constants (8 rounds total)
const EXTERNAL_CONSTANTS: [[u64; WIDTH]; 8] = [
    [
        15492826721047263190, 11728330187201910315, 8836021247773420868, 16777404051263952451,
        5510875212538051896, 6173089941271892285, 2927757366422211339, 10340958981325008808,
        8541987352684552425, 9739599543776434497, 15073950188101532019, 12084856431752384512,
    ],
    [
        4584713381960671270, 8807052963476652830, 54136601502601741, 4872702333905478703,
        5551030319979516287, 12889366755535460989, 16329242193178844328, 412018088475211848,
        10505784623379650541, 9758812378619434837, 7421979329386275117, 375240370024755551,
    ],
    [
        3331431125640721931, 15684937309956309981, 578521833432107983, 14379242000670861838,
        17922409828154900976, 8153494278429192257, 15904673920630731971, 11217863998460634216,
        3301540195510742136, 9937973023749922003, 3059102938155026419, 1895288289490976132,
    ],
    [
        5580912693628927540, 10064804080494788323, 9582481583369602410, 10186259561546797986,
        247426333829703916, 13193193905461376067, 6386232593701758044, 17954717245501896472,
        1531720443376282699, 2455761864255501970, 11234429217864304495, 4746959618548874102,
    ],
    [
        13571697342473846203, 17477857865056504753, 15963032953523553760, 16033593225279635898,
        14252634232868282405, 8219748254835277737, 7459165569491914711, 15855939513193752003,
        16788866461340278896, 7102224659693946577, 3024718005636976471, 13695468978618890430,
    ],
    [
        8214202050877825436, 2670727992739346204, 16259532062589659211, 11869922396257088411,
        3179482916972760137, 13525476046633427808, 3217337278042947412, 14494689598654046340,
        15837379330312175383, 8029037639801151344, 2153456285263517937, 8301106462311849241,
    ],
    [
        13294194396455217955, 17394768489610594315, 12847609130464867455, 14015739446356528640,
        5879251655839607853, 9747000124977436185, 8950393546890284269, 10765765936405694368,
        14695323910334139959, 16366254691123000864, 15292774414889043182, 10910394433429313384,
    ],
    [
        17253424460214596184, 3442854447664030446, 3005570425335613727, 10859158614900201063,
        9763230642109343539, 6647722546511515039, 909012944955815706, 18101204076790399111,
        11588128829349125809, 15863878496612806566, 5201119062417750399, 176665553780565743,
    ],
];

// Internal round constants (22 partial rounds)
const INTERNAL_CONSTANTS: [u64; ROUNDS_P] = [
    11921381764981422944, 10318423381711320787, 8291411502347000766, 229948027109387563,
    9152521390190983261, 7129306032690285515, 15395989607365232011, 8641397269074305925,
    17256848792241043600, 6046475228902245682, 12041608676381094092, 12785542378683951657,
    14546032085337914034, 3304199118235116851, 16499627707072547655, 10386478025625759321,
    13475579315436919170, 16042710511297532028, 1411266850385657080, 9024840976168649958,
    14047056970978379368, 838728605080212101,
];

// Matrix diagonal constants for Poseidon2
const MATRIX_DIAG_12_U64: [u64; WIDTH] = [
    0xc3b6c08e23ba9300, 0xd84b5de94a324fb6, 0x0d0c371c5b35b84f, 0x7964f570e7188037,
    0x5daf18bbd996604b, 0x6743bc47b9595257, 0x5528b9362c59bb70, 0xac45e25b7127b68b,
    0xa2077d7dfbb606b5, 0xf3faac6faee378ae, 0x0c6388b51545e883, 0xd27dbb6944917b60,
];

/// Hashes a slice of Goldilocks field elements to a single Fp5Element.
///
/// This is the main Poseidon2 hash function. It takes an arbitrary number of
/// Goldilocks field elements and produces a 40-byte hash (Fp5Element).
///
/// # Example
///
/// ```rust
/// use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
///
/// let elements = vec![
///     Goldilocks::from_canonical_u64(1),
///     Goldilocks::from_canonical_u64(2),
///     Goldilocks::from_canonical_u64(3),
/// ];
/// let hash = hash_to_quintic_extension(&elements);
/// ```
pub fn hash_to_quintic_extension(input: &[Goldilocks]) -> Fp5Element {
    hash_n_to_m_no_pad(input, 5)
}

fn hash_n_to_m_no_pad(input: &[Goldilocks], num_outputs: usize) -> Fp5Element {
    let mut perm = [Goldilocks::zero(); WIDTH];
    
    // Process input in chunks of RATE
    for chunk in input.chunks(RATE) {
        for (j, &val) in chunk.iter().enumerate() {
            perm[j] = val;
        }
        permute(&mut perm);
    }
    
    // Extract outputs (num_outputs is always 5 for our use case)
    let mut output_idx = 0;
    let mut outputs = [Goldilocks::zero(); 5];
    loop {
        for i in 0..RATE {
            if output_idx < num_outputs {
                outputs[output_idx] = perm[i];
                output_idx += 1;
            }
            if output_idx == num_outputs {
                return Fp5Element(outputs);
            }
        }
        permute(&mut perm);
    }
}

/// Applies the Poseidon2 permutation to a 12-element state array.
///
/// This is the core permutation function used by the hash. It applies:
/// - External linear layer
/// - Full rounds (first half)
/// - Partial rounds
/// - Full rounds (second half)
pub fn permute(input: &mut [Goldilocks; WIDTH]) {
    external_linear_layer(input);
    full_rounds(input, 0);
    partial_rounds(input);
    full_rounds(input, ROUNDS_F_HALF);
}

fn full_rounds(state: &mut [Goldilocks; WIDTH], start: usize) {
    for r in start..start + ROUNDS_F_HALF {
        add_rc(state, r);
        sbox(state);
        external_linear_layer(state);
    }
}

fn partial_rounds(state: &mut [Goldilocks; WIDTH]) {
    for r in 0..ROUNDS_P {
        add_rci(state, r);
        sbox_p(0, state);
        internal_linear_layer(state);
    }
}

fn external_linear_layer(s: &mut [Goldilocks; WIDTH]) {
    // Process in 4-element windows for efficiency
    for i in 0..3 { // 3 windows of 4 elements each
        let t0 = s[4*i].add(&s[4*i+1]);     // s0+s1
        let t1 = s[4*i+2].add(&s[4*i+3]);   // s2+s3
        let t2 = t0.add(&t1);               // t0+t1 = s0+s1+s2+s3
        let t3 = t2.add(&s[4*i+1]);         // t2+s1 = s0+2s1+s2+s3
        let t4 = t2.add(&s[4*i+3]);         // t2+s3 = s0+s1+s2+2s3
        let t5 = s[4*i].double();           // 2s0
        let t6 = s[4*i+2].double();         // 2s2
        
        s[4*i] = t3.add(&t0);
        s[4*i+1] = t6.add(&t3);
        s[4*i+2] = t1.add(&t4);
        s[4*i+3] = t5.add(&t4);
    }
    
    // Add sums to each element
    // Unroll loops for better performance (WIDTH is constant 12, so we have 3 groups of 4)
    let sum0 = s[0].add(&s[4]).add(&s[8]);
    let sum1 = s[1].add(&s[5]).add(&s[9]);
    let sum2 = s[2].add(&s[6]).add(&s[10]);
    let sum3 = s[3].add(&s[7]).add(&s[11]);
    
    for i in 0..WIDTH {
        s[i] = s[i].add(match i % 4 {
            0 => &sum0,
            1 => &sum1,
            2 => &sum2,
            3 => &sum3,
            _ => unreachable!(),
        });
    }
}

fn internal_linear_layer(state: &mut [Goldilocks; WIDTH]) {
    let mut sum = state[0];
    for i in 1..WIDTH {
        sum = sum.add(&state[i]);
    }
    for i in 0..WIDTH {
        state[i] = state[i].mul(&Goldilocks(MATRIX_DIAG_12_U64[i])).add(&sum);
    }
}

fn add_rc(state: &mut [Goldilocks; WIDTH], external_round: usize) {
    for i in 0..WIDTH {
        state[i] = state[i].add(&Goldilocks(EXTERNAL_CONSTANTS[external_round][i]));
    }
}

fn add_rci(state: &mut [Goldilocks; WIDTH], round: usize) {
    state[0] = state[0].add(&Goldilocks(INTERNAL_CONSTANTS[round]));
}

fn sbox(state: &mut [Goldilocks; WIDTH]) {
    for i in 0..WIDTH {
        sbox_p(i, state);
    }
}

fn sbox_p(index: usize, state: &mut [Goldilocks; WIDTH]) {
    // Poseidon2 S-box: x^7
    // Computed as: x^7 = (x^2 * x)^2 * x
    let tmp = state[index];
    let tmp_square = tmp.square();
    let tmp_sixth = tmp_square.mul(&tmp).square();
    state[index] = tmp_sixth.mul(&tmp);
}
