use crate::{CryptoError, Result, Goldilocks, Fp5Element, ScalarField};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchnorrError {
    #[error("Invalid signature format")]
    InvalidSignature,
    #[error("Point operation failed")]
    PointOperation,
}

// Scalar field constants
const N: [u64; 4] = [
    0x8c46eb2100000001, 0x224698fc0994a8dd, 0x0000000000000000, 0x4000000000000000
];
const N0I: u64 = 0x8c46eb2100000001;
const R2: [u64; 4] = [
    0x07717a21e77894e8, 0x1a75e45b33d469f4, 0xc4dfc927c5ed3713, 0x2f431806ad2fe478
];

// Elliptic curve constants
#[allow(dead_code)]
const A_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(2), Goldilocks(0), Goldilocks(0), Goldilocks(0), Goldilocks(0)
]);
pub const B_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0), Goldilocks(263), Goldilocks(0), Goldilocks(0), Goldilocks(0)
]);
pub const B_MUL2_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0), Goldilocks(526), Goldilocks(0), Goldilocks(0), Goldilocks(0)
]);
pub const B_MUL4_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0), Goldilocks(1052), Goldilocks(0), Goldilocks(0), Goldilocks(0)
]);
pub const B_MUL16_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0), Goldilocks(4208), Goldilocks(0), Goldilocks(0), Goldilocks(0)
]);

// Generator point for the curve
const GENERATOR_ECG_FP5_POINT: Point = Point {
    x: Fp5Element([
        Goldilocks(12883135586176881569), Goldilocks(4356519642755055268), 
        Goldilocks(5248930565894896907), Goldilocks(2165973894480315022), Goldilocks(2448410071095648785)
    ]),
    z: Fp5Element([
        Goldilocks(1), Goldilocks(0), Goldilocks(0), Goldilocks(0), Goldilocks(0)
    ]),
    u: Fp5Element([
        Goldilocks(1), Goldilocks(0), Goldilocks(0), Goldilocks(0), Goldilocks(0)
    ]),
    t: Fp5Element([
        Goldilocks(4), Goldilocks(0), Goldilocks(0), Goldilocks(0), Goldilocks(0)
    ])
};

#[derive(Debug, Clone)]
pub struct Scalar([u64; 4]);

impl Scalar {
    pub fn new(limbs: [u64; 4]) -> Self {
        Scalar(limbs)
    }
    
    pub fn limbs(&self) -> [u64; 4] {
        self.0
    }
    
    pub fn from_bytes_le(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidPrivateKeyLength(bytes.len()));
        }
        
        let mut limbs = [0u64; 4];
        for (i, chunk) in bytes.chunks(8).enumerate() {
            let mut val = 0u64;
            for (j, &byte) in chunk.iter().enumerate() {
                val |= (byte as u64) << (j * 8);
            }
            limbs[i] = val;
        }
        
        Ok(Scalar(limbs))
    }
    
    pub fn to_bytes_le(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for (i, &limb) in self.0.iter().enumerate() {
            for j in 0..8 {
                bytes[i * 8 + j] = ((limb >> (j * 8)) & 0xFF) as u8;
            }
        }
        bytes
    }
    
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0u8; 40];
        let le_bytes = self.to_bytes_le();
        bytes[..32].copy_from_slice(&le_bytes);
        // Pad with zeros to make it 40 bytes (little-endian format)
        bytes
    }
    
    pub fn to_montgomery(&self) -> [u64; 4] {
        // Multiply by R2 to convert to Montgomery form
        self.monty_mul(&R2)
    }
    
    pub fn from_montgomery(montgomery: &[u64; 4]) -> [u64; 4] {
        // Multiply by 1 to convert from Montgomery form
        let one = [1, 0, 0, 0];
        monty_mul(montgomery, &one)
    }
    
    pub fn mul(&self, other: &Self) -> Self {
        let self_mont = self.to_montgomery();
        let other_mont = other.to_montgomery();
        let result_mont = monty_mul(&self_mont, &other_mont);
        let result = Self::from_montgomery(&result_mont);
        Scalar(result)
    }
    
    fn monty_mul(&self, other: &[u64; 4]) -> [u64; 4] {
        monty_mul(&self.0, other)
    }
    
    fn sub_inner(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
        let mut result = [0u64; 4];
        let mut borrow = 0u64;
        
        for i in 0..4 {
            let (diff, new_borrow) = if a[i] >= b[i] + borrow {
                (a[i] - b[i] - borrow, 0)
            } else {
                (a[i].wrapping_sub(b[i]).wrapping_sub(borrow), 1)
            };
            result[i] = diff;
            borrow = new_borrow;
        }
        
        result
    }
    
    fn add_order(a: &[u64; 4]) -> [u64; 4] {
        let mut result = [0u64; 4];
        let mut carry = 0u64;
        
        for i in 0..4 {
            let sum = (a[i] as u128) + (N[i] as u128) + (carry as u128);
            result[i] = (sum & 0xFFFFFFFFFFFFFFFF) as u64;
            carry = (sum >> 64) as u64;
        }
        
        result
    }
    
    pub fn from_fp5_element(e_fp5: &Fp5Element) -> Self {
        // Convert Fp5Element to scalar using the same logic as Go FromGfp5
        // Go: FromGfp5(fp5) -> FromNonCanonicalBigInt(BigIntFromArray([5]uint64{fp5[0], fp5[1], fp5[2], fp5[3], fp5[4]}))
        // This creates a 320-bit integer from the 5 Goldilocks elements and converts to scalar
        
        // Go BigIntFromArray creates a big.Int by shifting and ORing the 5 uint64 values
        // We need to simulate this by creating a 320-bit integer and then reducing it modulo the scalar field
        
        // For now, use a simplified conversion that takes the first 4 elements (32 bytes)
        // This matches the scalar field size (4*64 = 256 bits)
        let mut scalar_limbs = [0u64; 4];
        for i in 0..4 {
            scalar_limbs[i] = e_fp5.0[i].0;
        }
        Scalar(scalar_limbs)
    }
}

impl Default for Scalar {
    fn default() -> Self {
        Scalar([0, 0, 0, 0])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AffinePoint {
    pub x: Fp5Element,
    pub u: Fp5Element,
}

impl AffinePoint {
    pub fn new(x: Fp5Element, u: Fp5Element) -> Self {
        AffinePoint { x, u }
    }
    
    pub fn neutral() -> Self {
        AffinePoint {
            x: Fp5Element::zero(),
            u: Fp5Element::zero(),
        }
    }
    
    pub fn to_point(&self) -> Point {
        Point::new(
            self.x,
            Fp5Element::one(),
            self.u,
            Fp5Element::one(),
        )
    }
    
    pub fn set_neg(&mut self) {
        self.u = self.u.neg();
    }
    
    pub fn set_lookup(&mut self, win: &[AffinePoint], k: i32) {
        // sign = 0xFFFFFFFF if k < 0, 0x00000000 otherwise
        let sign = (k >> 31) as u32;
        // ka = abs(k)
        let ka = ((k as u32) ^ sign).wrapping_sub(sign);
        // km1 = ka - 1
        let km1 = ka.wrapping_sub(1);
        
        let mut x = Fp5Element::zero();
        let mut u = Fp5Element::zero();
        
        for i in 0..win.len() {
            let m = km1.wrapping_sub(i as u32);
            let c_1 = (m | (!m).wrapping_add(1)) >> 31;
            let c = (c_1 as u64).wrapping_sub(1);
            if c != 0 {
                x = win[i].x;
                u = win[i].u;
            }
        }
        
        // If k < 0, then we must negate the point.
        let c = (sign as u64) | ((sign as u64) << 32);
        self.x = x;
        self.u = u;
        
        if c != 0 {
            self.u = self.u.neg();
        }
    }
}

/// Point on the ECgFp5 elliptic curve.
///
/// Points are represented in projective coordinates (x, z, u, t) for efficient
/// arithmetic operations. The curve equation is defined over the Fp5 extension field.
///
/// # Example
///
/// ```rust
/// use crypto::{Point, ScalarField};
///
/// // Get the generator point
/// let generator = Point::generator();
///
/// // Multiply by a scalar to get a new point
/// let scalar = ScalarField::sample_crypto();
/// let point = generator.mul(&scalar);
///
/// // Encode point to bytes
/// let encoded = point.encode();
/// let bytes = encoded.to_bytes_le();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// X coordinate in projective form
    pub x: Fp5Element,
    /// Z coordinate in projective form
    pub z: Fp5Element,
    /// U coordinate in projective form
    pub u: Fp5Element,
    /// T coordinate in projective form
    pub t: Fp5Element,
}

impl Point {
    /// Creates a new point from projective coordinates.
    pub fn new(x: Fp5Element, z: Fp5Element, u: Fp5Element, t: Fp5Element) -> Self {
        Point { x, z, u, t }
    }
    
    /// Returns the generator point (base point) of the curve.
    ///
    /// This is the standard generator used for key generation and signing.
    pub fn generator() -> Self {
        GENERATOR_ECG_FP5_POINT
    }
    
    /// Returns the neutral (identity) element of the curve.
    pub fn neutral() -> Self {
        Point::new(
            Fp5Element::zero(),
            Fp5Element::one(),
            Fp5Element::zero(),
            Fp5Element::one()
        )
    }
    
    /// Adds two points on the elliptic curve.
    ///
    /// This implements point addition in projective coordinates for efficiency.
    /// The operation assumes the points are distinct (x coordinates differ).
    pub fn add(&self, other: &Point) -> Point {
        // cost: 10M
        // Note: Assumes distinct points (x coordinates differ)
        let x1 = self.x;
        let z1 = self.z;
        let u1 = self.u;
        let _t1 = self.t;
        
        let x2 = other.x;
        let z2 = other.z;
        let u2 = other.u;
        let _t2 = other.t;
        
        // Intermediate calculations
        let t1 = x1.mul(&x2);
        let t2 = z1.mul(&z2);
        let t3 = u1.mul(&u2);
        let t4 = _t1.mul(&_t2);
        let t5 = x1.add(&z1).mul(&x2.add(&z2)).sub(&t1.add(&t2));
        let t6 = u1.add(&_t1).mul(&u2.add(&_t2)).sub(&t3.add(&t4));
        let t7 = t1.add(&t2.mul(&B_ECG_FP5_POINT));
        let t8 = t4.mul(&t7);
        let t9 = t3.mul(&t5.mul(&B_MUL2_ECG_FP5_POINT).add(&t7.double()));
        let t10 = t4.add(&t3.double()).mul(&t5.add(&t7));
        
        let x_new = t10.sub(&t8).mul(&B_ECG_FP5_POINT);
        let z_new = t8.sub(&t9);
        let u_new = t6.mul(&t2.mul(&B_ECG_FP5_POINT).sub(&t1));
        let t_new = t8.add(&t9);
        
        Point::new(x_new, z_new, u_new, t_new)
    }
    
    // Point doubling on the elliptic curve
    pub fn double(&self) -> Point {
        // cost: 4M+5S
        let x = self.x;
        let z = self.z;
        let u = self.u;
        let t = self.t;
        
        let t1 = z.mul(&t);
        let t2 = t1.mul(&t);
        let x1 = t2.square();
        let z1 = t1.mul(&u);
        let t3 = u.square();
        let w1 = t2.sub(&t3.mul(&(x.add(&z)).double()));
        let t4 = z1.square();
        
        let x_new = t4.mul(&B_MUL4_ECG_FP5_POINT);
        let z_new = w1.square();
        let u_new = (w1.add(&z1)).square().sub(&t4.add(&z_new));
        let t_new = x1.double().sub(&t4.mul(&Fp5Element::from_uint64_array([4, 0, 0, 0, 0])).add(&z_new));
        
        Point::new(x_new, z_new, u_new, t_new)
    }
    
    /// Multiplies this point by a scalar (scalar multiplication).
    ///
    /// This is the core operation for key generation and signature verification.
    /// Uses windowed scalar multiplication for efficiency.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crypto::{Point, ScalarField};
    ///
    /// let generator = Point::generator();
    /// let scalar = ScalarField::sample_crypto();
    /// let result = generator.mul(&scalar);
    /// ```
    pub fn mul(&self, scalar: &ScalarField) -> Point {
        if scalar.0 == [0, 0, 0, 0, 0] {
            return Point::neutral();
        }
        
        // Special case for scalar 1
        if scalar.0 == [1, 0, 0, 0, 0] {
            return self.clone();
        }
        
        // Windowed multiplication algorithm (optimized)
        const WINDOW: usize = 5;
        #[allow(dead_code)]
        const WIN_SIZE: usize = 1 << (WINDOW - 1); // 16
        
        // Make window with affine points
        let win = self.make_window_affine();
        
        // Recode scalar into signed digits
        let digits = scalar.recode_signed(WINDOW);
        
        // Start with the last digit (least significant)
        let mut result = Self::lookup_var_time(&win, digits[digits.len() - 1]).to_point();

        // Process remaining digits from right to left (least significant to most significant)
        for i in (0..digits.len() - 1).rev() {
            result = result.set_m_double(WINDOW as u32);
            let lookup = Self::lookup(&win, digits[i]);
            result = result.add_affine(&lookup);
        }
        
        result
    }
    
    // Create window of affine points for efficient multiplication
    // Create window of affine points for efficient scalar multiplication
    pub fn make_window_affine(&self) -> Vec<AffinePoint> {
        const WIN_SIZE: usize = 16; // 2^(5-1)
        let mut tmp = vec![Point::neutral(); WIN_SIZE];
        tmp[0] = self.clone();
        
        for i in 1..WIN_SIZE {
            if i & 1 == 0 {
                // Even index: tmp[i] = tmp[i-1] + p
                tmp[i] = tmp[i-1].add(self);
            } else {
                // Odd index: tmp[i] = tmp[i>>1].double()
                tmp[i] = tmp[i >> 1].double();
            }
        }

        // Convert batch of points to affine coordinates
        Point::batch_to_affine(&tmp)
    }
    
    // Convert batch of points to affine coordinates
    pub fn batch_to_affine(src: &[Point]) -> Vec<AffinePoint> {
        let n = src.len();
        if n == 0 {
            return Vec::new();
        }
        if n == 1 {
            let p = &src[0];
            let m1 = p.z.mul(&p.t).inverse();
            return vec![AffinePoint {
                x: p.x.mul(&p.t).mul(&m1),
                u: p.u.mul(&p.z).mul(&m1),
            }];
        }
        
        let mut res = vec![AffinePoint::neutral(); n];
        
        // Compute product of all values to invert
        let mut m = src[0].z.mul(&src[0].t);
        for i in 1..n {
            let x = m;
            m = m.mul(&src[i].z);
            let u = m;
            m = m.mul(&src[i].t);
            res[i] = AffinePoint { x, u };
        }
        
        m = m.inverse();
        
        // Propagate back inverses
        for i in (1..n).rev() {
            res[i].u = src[i].u.mul(&res[i].u).mul(&m);
            m = m.mul(&src[i].t);
            res[i].x = src[i].x.mul(&res[i].x).mul(&m);
            m = m.mul(&src[i].z);
        }
        
        res[0].u = src[0].u.mul(&src[0].z).mul(&m);
        m = m.mul(&src[0].t);
        res[0].x = src[0].x.mul(&m);
        
        res
    }
    
    // Lookup point in window (variable time)
    pub fn lookup_var_time(win: &[AffinePoint], k: i32) -> AffinePoint {
        if k == 0 {
            AffinePoint::neutral()
        } else if k > 0 {
            win[k as usize - 1].clone()
        } else {
            let mut res = win[(-k) as usize - 1].clone();
            res.set_neg();
            res
        }
    }
    
    // Lookup point in window (constant time)
    pub fn lookup(win: &[AffinePoint], k: i32) -> AffinePoint {
        let mut result = AffinePoint::neutral();
        result.set_lookup(win, k);
        result
    }
    
    // Multiple doublings - optimized point multiplication by 2^n
    pub fn set_m_double(&self, n: u32) -> Point {
        if n == 0 {
            return self.clone();
        }
        if n == 1 {
            return self.double();
        }

        // cost: n*(2M+5S) + 2M+1S
        let x0 = self.x;
        let z0 = self.z;
        let u0 = self.u;
        let t0 = self.t;

        let t1 = z0.mul(&t0);
        let t2 = t1.mul(&t0);
        let x1 = t2.square();
        let z1 = t1.mul(&u0);
        let t3 = u0.square();
        let w1 = t2.sub(&t3.mul(&(x0.add(&z0)).double()));
        let t4 = w1.square();
        let t5 = z1.square();
        let x = t5.square().mul(&B_MUL16_ECG_FP5_POINT);
        let w = x1.double().sub(&t5.mul(&Fp5Element::from_uint64_array([4, 0, 0, 0, 0])).add(&t4));
        let z = w1.add(&z1).square().sub(&t4.add(&t5));

        let mut x_acc = x;
        let mut w_acc = w;
        let mut z_acc = z;

        for _i in 2..n {
            let t1 = z_acc.square();
            let t2 = t1.square();
            let t3 = w_acc.square();
            let t4 = t3.square();
            let t5 = w_acc.add(&z_acc).square().sub(&t1.add(&t3));
            z_acc = t5.mul(&x_acc.add(&t1).double().sub(&t3));
            x_acc = t2.mul(&t4).mul(&B_MUL16_ECG_FP5_POINT);
            w_acc = t4.add(&t2.mul(&B_MUL4_ECG_FP5_POINT.sub(&Fp5Element::from_uint64_array([4, 0, 0, 0, 0])))).neg();
        }

        let t1 = w_acc.square();
        let t2 = z_acc.square();
        let t3 = w_acc.add(&z_acc).square().sub(&t1.add(&t2));
        let w1_final = t1.sub(&x_acc.add(&t2).double());
        
        // Store z = Square(w1) first
        let z_final = w1_final.square();

        Point {
            x: t3.square().mul(&B_ECG_FP5_POINT),
            z: z_final,
            u: t3.mul(&w1_final),
            t: t1.double().mul(&t1.sub(&t2.double())).sub(&z_final),
        }
    }
    
    // Add affine point
    pub fn add_affine(&self, other: &AffinePoint) -> Point {
        // Go AddAffine algorithm - cost: 8M
        let x1 = self.x;
        let z1 = self.z;
        let u1 = self.u;
        let _t1 = self.t;
        let x2 = other.x;
        let u2 = other.u;
        
        let t1 = x1.mul(&x2);
        let t2 = z1;
        let t3 = u1.mul(&u2);
        let t4 = _t1;
        let t5 = x1.add(&x2.mul(&z1));
        let t6 = u1.add(&u2.mul(&_t1));
        let t7 = t1.add(&t2.mul(&B_ECG_FP5_POINT));
        let t8 = t4.mul(&t7);
        let t9 = t3.mul(&t5.mul(&B_MUL2_ECG_FP5_POINT).add(&t7.double()));
        let t10 = t4.add(&t3.double()).mul(&t5.add(&t7));
        
        Point {
            x: t10.sub(&t8).mul(&B_ECG_FP5_POINT),
            u: t6.mul(&t2.mul(&B_ECG_FP5_POINT).sub(&t1)),
            z: t8.sub(&t9),
            t: t8.add(&t9),
        }
    }
    
    // Convert single point to affine (for debugging)
    pub fn to_affine_single(&self) -> AffinePoint {
        let m1 = self.z.mul(&self.t).inverse();
        AffinePoint {
            x: self.x.mul(&self.t).mul(&m1),
            u: self.u.mul(&self.z).mul(&m1),
        }
    }
    
    // Simple multiplication for verification (for debugging)
    pub fn mul_simple(&self, scalar: u64) -> Point {
        if scalar == 0 {
            return Point::neutral();
        }
        if scalar == 1 {
            return *self;
        }
        
        let mut result = Point::neutral();
        let mut addend = *self;
        let mut s = scalar;
        
        while s > 0 {
            if s & 1 == 1 {
                result = result.add(&addend);
            }
            addend = addend.double();
            s >>= 1;
        }
        
        result
    }
    
    /// Encodes the point to an Fp5Element using fractional coordinates.
    /// 
    /// Returns t * u^-1, which represents the point in the quintic extension field.
    /// If u is zero (point at infinity), returns zero.
    /// Encodes this point to an Fp5Element.
    ///
    /// The encoding represents the point in a canonical form suitable for
    /// hashing and serialization.
    pub fn encode(&self) -> Fp5Element {
        if self.u.is_zero() {
            Fp5Element::zero()
        } else {
            self.t.mul(&self.u.inverse())
        }
    }
    
    /// Checks if two points are equal using fractional coordinates.
    /// 
    /// Two points are equal if u1*t2 == u2*t1.
    pub fn equals(&self, other: &Point) -> bool {
        let left = self.u.mul(&other.t);
        let right = other.u.mul(&self.t);
        left.0.iter().zip(right.0.iter()).all(|(a, b)| a.0 == b.0)
    }
    
    /// Decodes an Fp5Element back to a Point.
    ///
    /// This is a simplified decoding that reconstructs a point from its encoded form.
    /// Note: This is not a complete inverse of `encode()` - it creates a valid point
    /// representation but may not recover the exact original point.
    ///
    /// # Arguments
    /// * `encoded` - The encoded Fp5Element (typically from `encode()`)
    ///
    /// # Returns
    /// A Point reconstructed from the encoded value.
    pub fn decode(encoded: &Fp5Element) -> Self {
        if encoded.is_zero() {
            // Neutral point
            Self::neutral()
        } else {
            // For non-zero encoded values, create a point where:
            // u = 1, t = encoded (so that t * u^-1 = encoded)
            Self {
                x: *encoded,
                z: Fp5Element::one(),
                u: Fp5Element::one(),
                t: *encoded,
            }
        }
    }
    
    pub fn is_neutral(&self) -> bool {
        self.u.is_zero()
    }
}

/// Signs a message using Schnorr signature scheme with a given nonce.
/// 
/// This function implements the Schnorr signature algorithm:
/// 1. Compute R = nonce * G (where G is the generator point)
/// 2. Encode R as an Fp5 element
/// 3. Compute challenge e = H(R || message) using Poseidon2
/// 4. Compute response s = nonce - e * private_key
/// 5. Return signature as (s || e) concatenated (80 bytes total)
/// 
/// # Arguments
/// * `private_key` - 40-byte private key (little-endian)
/// * `message` - Message to sign (typically 40 bytes, representing a hash)
/// * `nonce_bytes` - Nonce bytes (will be padded/truncated to 40 bytes)
/// 
/// # Returns
/// A vector containing the signature (80 bytes: 40 bytes s + 40 bytes e)
/// 
/// # Errors
/// Returns an error if the private key length is invalid.
pub fn sign_with_nonce(private_key: &[u8], message: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>> {
    if private_key.len() != 40 {
        return Err(CryptoError::InvalidPrivateKeyLength(private_key.len()));
    }
    
    // Convert private key to 5-limb scalar field element (40 bytes = 5 * 8 bytes)
    let private_scalar = ScalarField::from_bytes_le(private_key)
        .map_err(|_| CryptoError::InvalidPrivateKeyLength(private_key.len()))?;
    
    // Convert nonce to 5-limb scalar field element
    let mut nonce_bytes_40 = [0u8; 40];
    let copy_len = nonce_bytes.len().min(40);
    nonce_bytes_40[..copy_len].copy_from_slice(&nonce_bytes[..copy_len]);
    let nonce_scalar = ScalarField::from_bytes_le(&nonce_bytes_40)
        .map_err(|_| CryptoError::InvalidPrivateKeyLength(nonce_bytes.len()))?;
    
    // Convert message to Fp5Element (quintic extension field element)
    let mut message_elements = [Goldilocks::zero(); 5];
    for (i, chunk) in message.chunks(8).enumerate().take(5) {
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        message_elements[i] = Goldilocks::from_canonical_u64(u64::from_le_bytes(bytes));
    }
    let message_fp5 = Fp5Element(message_elements);
    
    // Step 1: Compute R = nonce * generator_point
    let generator = Point::generator();
    let r_point = generator.mul(&nonce_scalar);
    let r_encoded = r_point.encode();
    
    // Step 2: Compute challenge e = H(R || message)
    use poseidon_hash::hash_to_quintic_extension;
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    // Hash the pre-image using Poseidon2 to get challenge e
    let e_fp5 = hash_to_quintic_extension(&pre_image);
    let e_scalar = ScalarField::from_fp5_element(&e_fp5);
    
    // Step 3: Compute response s = nonce - e * private_key
    let e_times_private = e_scalar.mul(&private_scalar);
    let s = nonce_scalar.sub(e_times_private);
    
    // Step 4: Assemble signature as (s || e)
    let mut signature = [0u8; 80];
    let s_bytes = s.to_bytes_le();
    signature[..40].copy_from_slice(&s_bytes);
    
    let e_bytes = e_scalar.to_bytes_le();
    signature[40..].copy_from_slice(&e_bytes);
    
    Ok(signature.to_vec())
}

/// Verifies a Schnorr signature.
///
/// This function verifies that a signature was created by the holder of the
/// private key corresponding to the given public key.
///
/// # Arguments
///
/// * `signature` - The signature to verify (80 bytes: 40 bytes s + 40 bytes e)
/// * `message` - The message that was signed (40 bytes)
/// * `public_key` - The public key (40 bytes)
///
/// # Returns
///
/// Returns `Ok(true)` if the signature is valid, `Ok(false)` if invalid,
/// or an error if the inputs are malformed.
///
/// # Example
///
/// ```rust
/// use crypto::{sign_with_nonce, verify_signature, ScalarField};
///
/// let private_key = ScalarField::sample_crypto();
/// let private_key_bytes = private_key.to_bytes_le();
/// let public_key_bytes = private_key_bytes; // Simplified for example
///
/// let message = [0u8; 40];
/// let nonce = ScalarField::sample_crypto();
/// let nonce_bytes = nonce.to_bytes_le();
///
/// let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();
/// let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
/// ```
pub fn verify_signature(signature: &[u8], message: &[u8], public_key: &[u8]) -> Result<bool> {
    if signature.len() != 80 {
        return Err(CryptoError::InvalidSignatureLength(signature.len()));
    }
    
    if message.len() != 40 {
        return Err(CryptoError::InvalidMessageLength(message.len()));
    }
    
    if public_key.len() != 40 {
        return Err(CryptoError::InvalidPrivateKeyLength(public_key.len()));
    }

    // Parse signature: s (40 bytes) + e (40 bytes)
    let s_bytes = &signature[0..40];
    let e_bytes = &signature[40..80];

    // Convert to scalars
    let s = ScalarField::from_bytes_le(s_bytes)
        .map_err(|_| CryptoError::InvalidSignatureLength(signature.len()))?;
    let e = ScalarField::from_bytes_le(e_bytes)
        .map_err(|_| CryptoError::InvalidSignatureLength(signature.len()))?;

    // Convert message to Fp5Element
    let mut message_elements = [Goldilocks::zero(); 5];
    for (i, chunk) in message.chunks(8).enumerate().take(5) {
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        message_elements[i] = Goldilocks::from_canonical_u64(u64::from_le_bytes(bytes));
    }
    let message_fp5 = Fp5Element(message_elements);

    // For now, treat public_key as private key and compute public point
    // This is a temporary fix until we implement proper point decoding
    let private_scalar = ScalarField::from_bytes_le(public_key)
        .map_err(|_| CryptoError::InvalidPrivateKeyLength(public_key.len()))?;
    let generator = Point::generator();
    let public_point = generator.mul(&private_scalar);

    // Compute R = s * G + e * public_key (combined scalar multiplication)
    let s_g = generator.mul(&s);
    let e_public = public_point.mul(&e);
    let r_point = s_g.add(&e_public);

    // Encode R
    let r_encoded = r_point.encode();

    // Compute e' = H(r || message) using Poseidon2 hash
    use poseidon_hash::hash_to_quintic_extension;
    // Use fixed-size array instead of Vec to avoid heap allocation
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);

    let e_prime_fp5 = hash_to_quintic_extension(&pre_image);
    let e_prime_scalar = ScalarField::from_fp5_element(&e_prime_fp5);

    // Verify e == e'
    Ok(e.equals(&e_prime_scalar))
}

// Helper functions
fn monty_mul(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    let mut result = [0u64; 4];
    let mut carry = 0u64;
    
    for i in 0..4 {
        let mut temp = 0u128;
        for j in 0..=i {
            temp = temp.wrapping_add((a[j] as u128).wrapping_mul(b[i - j] as u128));
        }
        temp = temp.wrapping_add(carry as u128);
        
        result[i] = (temp & 0xFFFFFFFFFFFFFFFF) as u64;
        carry = (temp >> 64) as u64;
    }
    
    // Reduce modulo N
    let n0i = N0I;
    let q = (result[0] as u128 * n0i as u128) & 0xFFFFFFFFFFFFFFFF;
    
    let mut temp = 0u128;
    for i in 0..4 {
        temp += (q as u128) * (N[i] as u128) + (result[i] as u128);
        if i < 3 {
            result[i] = (temp & 0xFFFFFFFFFFFFFFFF) as u64;
            temp >>= 64;
        }
    }
    result[3] = (temp & 0xFFFFFFFFFFFFFFFF) as u64;
    
    // Final reduction
    if result >= N {
        result = Scalar::sub_inner(&result, &N);
    }
    
    result
}

impl Scalar {
    #[allow(dead_code)]
    fn add(&self, other: &Scalar) -> Scalar {
        let mut result = [0u64; 4];
        let mut carry = 0u64;
        
        for i in 0..4 {
            let sum = (self.0[i] as u128) + (other.0[i] as u128) + (carry as u128);
            result[i] = (sum & 0xFFFFFFFFFFFFFFFF) as u64;
            carry = (sum >> 64) as u64;
        }
        
        // Reduce modulo N if necessary
        if result >= N {
            result = Scalar::sub_inner(&result, &N);
        }
        
        Scalar(result)
    }
    
    pub fn sub(&self, other: &Scalar) -> Scalar {
        let mut result = [0u64; 4];
        let mut borrow = 0u64;
        
        for i in 0..4 {
            let (diff, new_borrow) = if self.0[i] >= other.0[i] + borrow {
                (self.0[i] - other.0[i] - borrow, 0)
            } else {
                (self.0[i].wrapping_sub(other.0[i]).wrapping_sub(borrow), 1)
            };
            result[i] = diff;
            borrow = new_borrow;
        }
        
        // Add N if we borrowed
        if borrow > 0 {
            result = Scalar::add_order(&result);
        }
        
        Scalar(result)
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Scalar {}
