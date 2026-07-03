use crate::{CryptoError, Fp5Element, Goldilocks, Result, ScalarField};

// Elliptic curve constants
const A_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(2),
    Goldilocks(0),
    Goldilocks(0),
    Goldilocks(0),
    Goldilocks(0),
]);
pub const B_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0),
    Goldilocks(263),
    Goldilocks(0),
    Goldilocks(0),
    Goldilocks(0),
]);
pub const B_MUL2_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0),
    Goldilocks(526),
    Goldilocks(0),
    Goldilocks(0),
    Goldilocks(0),
]);
pub const B_MUL4_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0),
    Goldilocks(1052),
    Goldilocks(0),
    Goldilocks(0),
    Goldilocks(0),
]);
pub const B_MUL16_ECG_FP5_POINT: Fp5Element = Fp5Element([
    Goldilocks(0),
    Goldilocks(4208),
    Goldilocks(0),
    Goldilocks(0),
    Goldilocks(0),
]);

// Generator point for the curve
const GENERATOR_ECG_FP5_POINT: Point = Point {
    x: Fp5Element([
        Goldilocks(12883135586176881569),
        Goldilocks(4356519642755055268),
        Goldilocks(5248930565894896907),
        Goldilocks(2165973894480315022),
        Goldilocks(2448410071095648785),
    ]),
    z: Fp5Element([
        Goldilocks(1),
        Goldilocks(0),
        Goldilocks(0),
        Goldilocks(0),
        Goldilocks(0),
    ]),
    u: Fp5Element([
        Goldilocks(1),
        Goldilocks(0),
        Goldilocks(0),
        Goldilocks(0),
        Goldilocks(0),
    ]),
    t: Fp5Element([
        Goldilocks(4),
        Goldilocks(0),
        Goldilocks(0),
        Goldilocks(0),
        Goldilocks(0),
    ]),
};

#[derive(Debug, Clone, Copy)]
pub struct AffinePoint {
    pub x: Fp5Element,
    pub u: Fp5Element,
}

fn ct_mask_eq_u32(left: u32, right: u32) -> u64 {
    let diff = left ^ right;
    let is_equal = ((diff | diff.wrapping_neg()) >> 31) ^ 1;
    0u64.wrapping_sub(is_equal as u64)
}

fn ct_select_goldilocks(mask: u64, when_false: Goldilocks, when_true: Goldilocks) -> Goldilocks {
    Goldilocks(when_false.0 ^ (mask & (when_false.0 ^ when_true.0)))
}

fn ct_select_fp5(mask: u64, when_false: &Fp5Element, when_true: &Fp5Element) -> Fp5Element {
    let mut result = [Goldilocks::zero(); 5];
    for (index, slot) in result.iter_mut().enumerate() {
        *slot = ct_select_goldilocks(mask, when_false.0[index], when_true.0[index]);
    }
    Fp5Element(result)
}

fn ct_select_affine(mask: u64, when_false: &AffinePoint, when_true: &AffinePoint) -> AffinePoint {
    AffinePoint {
        x: ct_select_fp5(mask, &when_false.x, &when_true.x),
        u: ct_select_fp5(mask, &when_false.u, &when_true.u),
    }
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
        Point::new(self.x, Fp5Element::one(), self.u, Fp5Element::one())
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

        let mut selected = AffinePoint::neutral();
        for (index, point) in win.iter().enumerate() {
            let mask = ct_mask_eq_u32(km1, index as u32);
            selected = ct_select_affine(mask, &selected, point);
        }

        // If k < 0, negate the selected point without branching.
        let sign_mask = (sign as u64) | ((sign as u64) << 32);
        let negated = AffinePoint {
            x: selected.x,
            u: selected.u.neg(),
        };
        *self = ct_select_affine(sign_mask, &selected, &negated);
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
/// use goldilocks_crypto::{Point, ScalarField};
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

/// Two `Point` values are equal when they represent the same affine point.
///
/// Equality is tested projectively:
/// - `X₁/Z₁ == X₂/Z₂`  ⟺  `X₁·Z₂ == X₂·Z₁`
/// - `U₁/T₁ == U₂/T₂`  ⟺  `U₁·T₂ == U₂·T₁`
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        let x_eq = self.x.mul(&other.z) == other.x.mul(&self.z);
        let u_eq = self.u.mul(&other.t) == other.u.mul(&self.t);
        x_eq && u_eq
    }
}
impl Eq for Point {}

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
            Fp5Element::one(),
        )
    }

    /// Adds two distinct points on the elliptic curve using projective coordinates.
    ///
    /// # Safety invariant
    ///
    /// This operation is defined only when the two points have **distinct affine
    /// x-coordinates** (`X₁/Z₁ ≠ X₂/Z₂`).  Specifically:
    ///
    /// - The neutral element plus any point is safe (projective neutral has Z=0
    ///   which is handled correctly by the formula).
    /// - **Point doubling (P + P) is NOT safe** — use [`Point::double`] instead.
    ///   Calling `add` with equal non-neutral inputs produces an incorrect result.
    ///
    /// In the internal windowed scalar multiplication both `mul()` and `mul_add2()`
    /// always invoke `double()` for the doubling step, so this invariant is never
    /// violated in production code paths.
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
        let t_new = x1.double().sub(
            &t4.mul(&Fp5Element::from_uint64_array([4, 0, 0, 0, 0]))
                .add(&z_new),
        );

        Point::new(x_new, z_new, u_new, t_new)
    }

    /// Combined scalar multiplication: computes scalarA * a + scalarB * b efficiently.
    ///
    /// This is equivalent to a.mul(&scalarA).add(&b.mul(&scalarB)) but more efficient.
    /// Used for verification: R = s * G + e * P
    ///
    /// # Example
    ///
    /// ```rust
    /// use goldilocks_crypto::{Point, ScalarField};
    ///
    /// let generator = Point::generator();
    /// let s = ScalarField::sample_crypto();
    /// let e = ScalarField::sample_crypto();
    /// let public_key = generator.mul(&s);
    /// let result = Point::mul_add2(&generator, &public_key, &s, &e);
    /// ```
    pub fn mul_add2(a: &Point, b: &Point, scalar_a: &ScalarField, scalar_b: &ScalarField) -> Point {
        // Use 4-bit window for efficiency (matches Go implementation)
        // Go's PrecomputeWindow creates: multiples[0]=neutral, multiples[1]=p, multiples[2]=2*p, etc.
        // We need to create a similar structure
        const WINDOW_SIZE: usize = 16; // 2^4

        // Create windows matching Go's structure: win[0]=neutral, win[1]=self, win[2]=2*self, etc.
        // Helper to convert point to affine
        let to_affine_single = |p: &Point| -> AffinePoint {
            let m1 = p.z.mul(&p.t).inverse();
            AffinePoint {
                x: p.x.mul(&p.t).mul(&m1),
                u: p.u.mul(&p.z).mul(&m1),
            }
        };

        let mut a_window = vec![AffinePoint::neutral(); WINDOW_SIZE];
        a_window[1] = to_affine_single(a);
        a_window[2] = to_affine_single(&a.double());
        for i in 3..WINDOW_SIZE {
            let prev = a_window[i - 1].to_point();
            a_window[i] = to_affine_single(&prev.add(a));
        }

        let mut b_window = vec![AffinePoint::neutral(); WINDOW_SIZE];
        b_window[1] = to_affine_single(b);
        b_window[2] = to_affine_single(&b.double());
        for i in 3..WINDOW_SIZE {
            let prev = b_window[i - 1].to_point();
            b_window[i] = to_affine_single(&prev.add(b));
        }

        // Split scalars into 4-bit limbs (80 limbs of 4 bits each, little-endian).
        // Each limb value is in [0, 15].  Window indexing:
        //   limb == 0  →  neutral element  (window[0] = neutral)
        //   limb == k  →  win[k] = k·P  for k in [1, 15]
        // This matches the Go implementation's window layout.
        let a_limbs = scalar_a.split_to_4bit_limbs();
        let b_limbs = scalar_b.split_to_4bit_limbs();

        let num_limbs = a_limbs.len();

        let a_idx = a_limbs[num_limbs - 1] as usize;
        let b_idx = b_limbs[num_limbs - 1] as usize;

        // Add the two affine points: window[a_idx] + window[b_idx]
        let a_lookup = a_window[a_idx];
        let b_lookup = b_window[b_idx];

        // Add the two affine points: convert first to point, then add second as affine
        let mut result = a_lookup.to_point();
        result = result.add_affine(&b_lookup);

        // Process remaining limbs from right to left (most significant to least)
        for i in (0..num_limbs - 1).rev() {
            // Double 4 times (since each limb is 4 bits)
            result = result.set_m_double(4);

            // Add corresponding window entries
            // res = res.Add(aWindow[i].Add(bWindow[i]))
            let a_idx = a_limbs[i] as usize;
            let b_idx = b_limbs[i] as usize;

            let a_lookup = a_window[a_idx];
            let b_lookup = b_window[b_idx];

            // Add the two affine lookups first, then add to result
            let a_pt = a_lookup.to_point();
            let combined = a_pt.add_affine(&b_lookup);
            result = result.add(&combined);
        }

        result
    }

    /// Multiplies this point by a scalar (scalar multiplication).
    ///
    /// This is the core operation for key generation and signature verification.
    /// Uses windowed scalar multiplication for efficiency.
    ///
    /// Note: The scalar should be in canonical form. If you have a scalar in Montgomery
    /// form (e.g., from `mul()`), convert it to canonical first using `monty_mul(&ScalarField::ONE)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goldilocks_crypto::{Point, ScalarField};
    ///
    /// let generator = Point::generator();
    /// let scalar = ScalarField::sample_crypto();
    /// let result = generator.mul(&scalar);
    /// ```
    pub fn mul(&self, scalar: &ScalarField) -> Point {
        const WINDOW_BITS: usize = 4;

        let win = self.make_window_affine();

        let digits = scalar.split_to_4bit_limbs();
        let mut result = Point::neutral();

        for digit in digits.iter().rev() {
            for _ in 0..WINDOW_BITS {
                result = result.double();
            }
            let lookup = Self::lookup_unsigned(&win, *digit);
            result = result.add_affine(&lookup);
        }

        result
    }

    /// Precomputes the 4-bit signed window for [`mul`](Point::mul).
    ///
    /// Returns 15 affine points `[P, 2P, 3P, …, 15P]` (indices 0–14).
    /// The 16th multiple (16P) is never needed because 4-bit limbs range over
    /// `0..=15`; `lookup_unsigned` uses `k == 0` for the neutral element and
    /// indices 1–15 for the window entries.
    pub fn make_window_affine(&self) -> Vec<AffinePoint> {
        // 15 entries: win[i] = (i+1)·P  for i in 0..15
        const WIN_SIZE: usize = 15;
        let mut tmp = vec![Point::neutral(); WIN_SIZE];
        tmp[0] = *self;

        for i in 1..WIN_SIZE {
            if i & 1 == 0 {
                // Even: tmp[i] = tmp[i-1] + P  (next odd multiple)
                tmp[i] = tmp[i - 1].add(self);
            } else {
                // Odd: tmp[i] = tmp[i>>1].double()  (double a smaller multiple)
                tmp[i] = tmp[i >> 1].double();
            }
        }

        // Batch-convert to affine coordinates (single field inversion)
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

    /// Looks up a signed index `k` in the precomputed window — **variable time**.
    ///
    /// Returns `k·P` for `k ∈ [-WIN_SIZE, WIN_SIZE]` using direct array indexing
    /// (branches on `k`).  This is **not** constant-time and must only be used
    /// with public, non-secret indices such as the loop counter in
    /// [`mul_add2`](Point::mul_add2) where the scalar bits are not secret at
    /// the call site.
    ///
    /// For secret indices use [`lookup`](Point::lookup), which is constant-time.
    #[allow(dead_code)]
    pub(crate) fn lookup_var_time(win: &[AffinePoint], k: i32) -> AffinePoint {
        if k == 0 {
            AffinePoint::neutral()
        } else if k > 0 {
            win[k as usize - 1]
        } else {
            let mut res = win[(-k) as usize - 1];
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

    fn lookup_unsigned(win: &[AffinePoint], k: u8) -> AffinePoint {
        let mut result = AffinePoint::neutral();
        for (index, point) in win.iter().enumerate().take(15) {
            let mask = ct_mask_eq_u32(k as u32, (index as u32) + 1);
            result = ct_select_affine(mask, &result, point);
        }
        result
    }

    // Multiple doublings - optimized point multiplication by 2^n
    pub fn set_m_double(&self, n: u32) -> Point {
        if n == 0 {
            return *self;
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
        let w = x1.double().sub(
            &t5.mul(&Fp5Element::from_uint64_array([4, 0, 0, 0, 0]))
                .add(&t4),
        );
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
            w_acc = t4
                .add(&t2.mul(
                    &B_MUL4_ECG_FP5_POINT.sub(&Fp5Element::from_uint64_array([4, 0, 0, 0, 0])),
                ))
                .neg();
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
        left == right
    }

    /// Decodes an Fp5Element back to a Point.
    ///
    /// This implements the proper decoding algorithm from Go's Decode() function.
    /// Curve equation: y^2 = x*(x^2 + a*x + b); encoded value is w = y/x.
    /// Solving: x^2 - (w^2 - a)*x + b = 0
    ///
    /// # Arguments
    /// * `encoded` - The encoded Fp5Element (typically from `encode()`)
    ///
    /// # Returns
    /// `Some(Point)` if decoding succeeds, `None` if the encoded value is invalid.
    pub fn decode(encoded: &Fp5Element) -> Option<Self> {
        use poseidon_hash::Fp5Element as Fp5;

        // If w == 0, return neutral point
        if encoded.is_zero() {
            return Some(Self::neutral());
        }

        // Step 1: Compute e = w^2 - a
        let w_squared = encoded.square();
        let e = w_squared.sub(&A_ECG_FP5_POINT);

        // Step 2: Compute delta = e^2 - 4*b
        let e_squared = e.square();
        let delta = e_squared.sub(&B_MUL4_ECG_FP5_POINT);

        // Step 3: Compute canonical square root of delta
        let (r, success) = delta.canonical_sqrt();
        let r = if success { r } else { Fp5::zero() };

        // Step 4: Solve quadratic: x1 = (e + r) / 2, x2 = (e - r) / 2
        // Division by 2: multiply by inverse of 2
        let fp5_two = Fp5::from_uint64_array([2, 0, 0, 0, 0]);
        let two_inv = fp5_two.inverse();
        let x1 = e.add(&r).mul(&two_inv);
        let x2 = e.sub(&r).mul(&two_inv);

        // Step 5: Choose x based on Legendre symbol
        // We want the solution that is NOT a square
        // If x1 is a square (Legendre == 1), use x2; otherwise use x1
        let x1_legendre = x1.legendre();
        let one_goldi = Goldilocks::one();
        let mut x = x1;
        if x1_legendre.equals(&one_goldi) {
            // x1 is a square, so use x2 (which is not a square)
            x = x2;
        }
        // else: x1 is not a square, so use x1 (already set)

        // Step 6: Set coordinates based on success
        let final_x = if success { x } else { Fp5::zero() };
        let z = Fp5::one();
        let u = if success { Fp5::one() } else { Fp5::zero() };
        let t = if success { *encoded } else { Fp5::one() };

        // Step 7: Return point if decoding succeeded
        if success || encoded.is_zero() {
            Some(Point {
                x: final_x,
                z,
                u,
                t,
            })
        } else {
            None
        }
    }

    pub fn is_neutral(&self) -> bool {
        self.u.is_zero()
    }
}

/// Helper function to convert message bytes to Fp5Element consistently.
/// This ensures the same conversion is used in both signing and verification.
///
/// Matches Go's `FromCanonicalLittleEndianBytes` behaviour:
/// - Message must be exactly 40 bytes (5 × 8 bytes).
/// - Each 8-byte chunk is interpreted as a little-endian u64.
/// - Every chunk **must** be a canonical Goldilocks element, i.e. < MODULUS.
///   If any chunk is ≥ MODULUS the function returns
///   `Err(CryptoError::NonCanonicalMessage { index, value })`.
///
/// # Security note
/// Silently reducing non-canonical chunks would allow two distinct 40-byte
/// messages to map to the same Fp5 element, breaking signature binding.
/// Callers must ensure each 8-byte chunk is in `[0, MODULUS)`.
pub(crate) fn message_to_fp5(message: &[u8]) -> Result<Fp5Element> {
    if message.len() != 40 {
        return Err(CryptoError::InvalidMessageLength(message.len()));
    }

    let mut message_elements = [Goldilocks::zero(); 5];
    for (i, chunk) in message.chunks(8).enumerate().take(5) {
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        let val = u64::from_le_bytes(bytes);
        if val >= Goldilocks::MODULUS {
            return Err(CryptoError::NonCanonicalMessage {
                index: i,
                value: val,
            });
        }
        message_elements[i] = Goldilocks::from_canonical_u64(val);
    }
    Ok(Fp5Element(message_elements))
}

fn parse_canonical_private_scalar(bytes: &[u8], field: &'static str) -> Result<ScalarField> {
    if bytes.len() != 40 {
        return Err(CryptoError::InvalidPrivateKeyLength(bytes.len()));
    }

    let scalar = ScalarField::from_canonical_bytes_le(bytes)
        .map_err(|_| CryptoError::NonCanonicalScalar { field })?;
    if scalar.is_zero() {
        return Err(CryptoError::ZeroScalar { field });
    }
    Ok(scalar)
}

fn parse_canonical_nonce_scalar(nonce_bytes: &[u8]) -> Result<ScalarField> {
    if nonce_bytes.len() != 40 {
        return Err(CryptoError::InvalidNonceLength(nonce_bytes.len()));
    }

    let scalar = ScalarField::from_canonical_bytes_le(nonce_bytes)
        .map_err(|_| CryptoError::NonCanonicalScalar { field: "nonce" })?;
    if scalar.is_zero() {
        return Err(CryptoError::ZeroScalar { field: "nonce" });
    }
    Ok(scalar)
}

fn parse_canonical_signature_scalar(bytes: &[u8], field: &'static str) -> Result<ScalarField> {
    if bytes.len() != 40 {
        return Err(CryptoError::InvalidSignatureLength(bytes.len()));
    }

    ScalarField::from_canonical_bytes_le(bytes)
        .map_err(|_| CryptoError::NonCanonicalScalar { field })
}

fn parse_valid_public_key_point(public_key: &[u8]) -> Result<Point> {
    if public_key.len() != 40 {
        return Err(CryptoError::InvalidPublicKeyLength(public_key.len()));
    }

    let public_key_fp5 =
        Fp5Element::from_bytes_le(public_key).map_err(|_| CryptoError::InvalidPublicKey)?;
    let point = Point::decode(&public_key_fp5).ok_or(CryptoError::InvalidPublicKey)?;

    if point.is_neutral() {
        return Err(CryptoError::InvalidPublicKey);
    }

    Ok(point)
}

/// Validates that a public key is a valid encoded point.
///
/// This function checks if the public key bytes can be decoded as a valid point
/// on the elliptic curve. Returns `Ok(())` if valid, `Err` otherwise.
///
/// # Arguments
/// * `public_key` - 40-byte public key to validate
///
/// # Returns
/// `Ok(())` if the public key is valid, `Err(CryptoError::InvalidPublicKey)` otherwise
///
/// # Example
///
/// ```rust
/// use goldilocks_crypto::{ScalarField, Point, validate_public_key};
///
/// let private_key = ScalarField::sample_crypto();
/// let public_key = Point::generator().mul(&private_key);
/// let public_key_bytes = public_key.encode().to_bytes_le();
///
/// // Validate the public key
/// validate_public_key(&public_key_bytes).unwrap();
/// ```
pub fn validate_public_key(public_key: &[u8]) -> Result<()> {
    parse_valid_public_key_point(public_key).map(|_| ())
}

/// Signs a message using Schnorr signature scheme with a randomly generated nonce.
///
/// This function generates a random nonce and calls `sign_with_nonce` internally.
///
/// # Arguments
/// * `private_key` - 40-byte private key (little-endian)
/// * `message` - Message to sign (typically 40 bytes, representing a hash)
///
/// # Returns
/// A vector containing the signature (80 bytes: 40 bytes s + 40 bytes e)
///
/// # Errors
/// Returns an error if the private key length is invalid.
pub fn sign(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    // Generate a random nonce
    let nonce = ScalarField::sample_crypto();
    let nonce_bytes = nonce.to_bytes_le();

    // Call sign_with_nonce with the random nonce
    sign_with_nonce(private_key, message, &nonce_bytes)
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
    // Convert private key to a canonical non-zero scalar field element.
    let private_scalar = parse_canonical_private_scalar(private_key, "private key")?;

    // Convert nonce to a canonical non-zero scalar field element.
    let nonce_scalar = parse_canonical_nonce_scalar(nonce_bytes)?;

    // Convert message to Fp5Element (quintic extension field element)
    // Use helper function to ensure consistency with verification
    let message_fp5 = message_to_fp5(message)?;

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

/// Signs a Poseidon2-hashed message (already as Fp5Element).
///
/// This is used when the message is already hashed by Poseidon2 (e.g., transaction hash).
/// The input is the Poseidon2 hash output (Fp5Element) as 40 little-endian bytes.
///
/// This matches Go's behavior: construct transaction → hash with Poseidon2 → sign the hash.
/// NOTE: Unlike sign_with_nonce, this does NOT hash the message again.
///
/// # Arguments
/// * `private_key` - 40-byte private key (little-endian)
/// * `hashed_message_fp5` - 40-byte Poseidon2 hash output (little-endian Fp5Element) - NOT hashed again
/// * `nonce_bytes` - Nonce bytes (will be padded/truncated to 40 bytes)
///
/// # Returns
/// A vector containing the signature (80 bytes: 40 bytes s + 40 bytes e)
pub fn sign_hashed_message(
    private_key: &[u8],
    hashed_message_fp5: &[u8],
    nonce_bytes: &[u8],
) -> Result<Vec<u8>> {
    let private_scalar = parse_canonical_private_scalar(private_key, "private key")?;

    if hashed_message_fp5.len() != 40 {
        return Err(CryptoError::InvalidMessageLength(hashed_message_fp5.len()));
    }

    let nonce_scalar = parse_canonical_nonce_scalar(nonce_bytes)?;

    // The hashed_message_fp5 is already a Poseidon2 output - convert it to Fp5Element
    // This is the m (message hash) that we use directly in signing
    let message_fp5 = Fp5Element::from_bytes_le(hashed_message_fp5)
        .map_err(|_| CryptoError::InvalidMessageLength(hashed_message_fp5.len()))?;

    // Step 1: Compute R = nonce * G
    let generator = Point::generator();
    let r_point = generator.mul(&nonce_scalar);
    let r_encoded = r_point.encode();

    // Step 2: Compute challenge e = H(R || m) where m is the already-hashed message
    // This is different from sign_with_nonce which hashes the raw message first
    use poseidon_hash::hash_to_quintic_extension;
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);

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
/// use goldilocks_crypto::{sign_with_nonce, verify_signature, ScalarField, Point};
///
/// let private_key = ScalarField::sample_crypto();
/// let private_key_bytes = private_key.to_bytes_le();
/// let public_key_bytes = Point::generator().mul(&private_key).encode().to_bytes_le();
///
/// let message = [0u8; 40];
/// let nonce = ScalarField::sample_crypto();
/// let nonce_bytes = nonce.to_bytes_le();
///
/// let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();
/// let is_valid = verify_signature(&signature, &message, &public_key_bytes).unwrap();
/// assert!(is_valid);
/// ```
pub fn verify_signature(signature: &[u8], message: &[u8], public_key: &[u8]) -> Result<bool> {
    if signature.len() != 80 {
        return Err(CryptoError::InvalidSignatureLength(signature.len()));
    }

    if message.len() != 40 {
        return Err(CryptoError::InvalidMessageLength(message.len()));
    }

    if public_key.len() != 40 {
        return Err(CryptoError::InvalidPublicKeyLength(public_key.len()));
    }

    // Parse signature: s (40 bytes) + e (40 bytes)
    let s_bytes = &signature[0..40];
    let e_bytes = &signature[40..80];

    // Convert to canonical scalars.
    let s = parse_canonical_signature_scalar(s_bytes, "signature.s")?;
    let e = parse_canonical_signature_scalar(e_bytes, "signature.e")?;

    // Convert message to Fp5Element
    // Use helper function to ensure consistency with signing
    let message_fp5 = message_to_fp5(message)?;

    let public_point = parse_valid_public_key_point(public_key)?;

    // Compute R = s * G + e * public_key
    let generator = Point::generator();
    let r_point = Point::mul_add2(&generator, &public_point, &s, &e);

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
    let is_valid = e.equals(&e_prime_scalar);

    // Debug logging for failures
    #[cfg(debug_assertions)]
    if !is_valid {
        use std::println;
        println!("\n=== VERIFICATION FAILURE DEBUG ===");
        println!("  e bytes (from signature): {:?}", e.to_bytes_le());
        println!(
            "  e_prime bytes (computed): {:?}",
            e_prime_scalar.to_bytes_le()
        );
        println!("  r_encoded (computed R): {:?}", r_encoded.to_bytes_le());
        println!("  message_fp5: {:?}", message_fp5.to_bytes_le());
        println!(
            "  public_point encoded: {:?}",
            public_point.encode().to_bytes_le()
        );
        println!("  s bytes: {:?}", s.to_bytes_le());
        println!("  R = s*G + e*P encoded: {:?}", r_encoded.to_bytes_le());
        println!("===============================\n");
    }

    Ok(is_valid)
}

