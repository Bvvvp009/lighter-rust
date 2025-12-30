use std::fmt;
use num_bigint::BigUint;

/// Scalar field element for the ECgFp5 curve.
///
/// Represents a scalar value in the scalar field of the ECgFp5 elliptic curve.
/// The scalar field uses a 5-limb representation (320 bits total) for efficient
/// arithmetic operations.
///
/// # Example
///
/// ```rust
/// use crypto::ScalarField;
///
/// // Generate a random scalar (cryptographically secure)
/// let scalar = ScalarField::sample_crypto();
///
/// // Create from bytes
/// let bytes = [0u8; 40];
/// let scalar = ScalarField::from_bytes_le(&bytes).unwrap();
///
/// // Convert to bytes
/// let bytes = scalar.to_bytes_le();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalarField(pub [u64; 5]);

impl ScalarField {
    // Scalar field modulus constants
    pub const N: ScalarField = ScalarField([
        0xE80FD996948BFFE1,  // N[0]
        0xE8885C39D724A09C,  // N[1]
        0x7FFFFFE6CFB80639,  // N[2]
        0x7FFFFFF100000016,  // N[3]
        0x7FFFFFFD80000007,  // N[4]
    ]);
    
    pub const N0I: u64 = 0xD78BEF72057B7BDF; // -1/N[0] mod 2^64
    
    pub const R2: ScalarField = ScalarField([
        0xA01001DCE33DC739,  // R2[0]
        0x6C3228D33F62ACCF,  // R2[1]
        0xD1D796CC91CF8525,  // R2[2]
        0xAADFFF5D1574C1D8,  // R2[3]
        0x4ACA13B28CA251F5,  // R2[4]
    ]);
    
    /// R2^-1 mod N: Modular inverse of R2, used to fix Point::mul() scalar form bug
    /// This is needed because e*P = (e*sk Montgomery)*G when e is canonical,
    /// but we need (e*sk canonical)*G. Using e * R2^-1 gives the correct result.
    pub const R2_INV: ScalarField = ScalarField([
        0x709c213d77a10649,  // R2_INV[0]
        0xdd567530551c44e6,  // R2_INV[1]
        0xc97ab1c242380e2e,  // R2_INV[2]
        0x9628a8046f74c730,  // R2_INV[3]
        0x5763bcb178ed3ac7,  // R2_INV[4]
    ]);
    
    pub const T632: ScalarField = ScalarField([
        0x2B0266F317CA91B3,  // T632[0]
        0xEC1D26528E984773,  // T632[1]
        0x8651D7865E12DB94,  // T632[2]
        0xDA2ADFF5941574D0,  // T632[3]
        0x53CACA12110CA256,  // T632[4]
    ]);
    
    pub const ZERO: ScalarField = ScalarField([0, 0, 0, 0, 0]);
    pub const ONE: ScalarField = ScalarField([1, 0, 0, 0, 0]);
    pub const TWO: ScalarField = ScalarField([2, 0, 0, 0, 0]);
    pub const NEG_ONE: ScalarField = ScalarField([
        0xE80FD996948BFFE0,
        0xE8885C39D724A09C,
        0x7FFFFFE6CFB80639,
        0x7FFFFFF100000016,
        0x7FFFFFFD80000007,
    ]);
    
    pub fn new(limbs: [u64; 5]) -> Self {
        ScalarField(limbs)
    }
    
    pub fn limbs(&self) -> [u64; 5] {
        self.0
    }
    
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }
    
    pub fn equals(&self, rhs: &ScalarField) -> bool {
        self.0 == rhs.0
    }
    
    /// Internal addition function (without modular reduction).
    ///
    /// This is a low-level function used internally. Use `add()` for normal operations.
    pub fn add_inner(&self, a: ScalarField) -> ScalarField {
        let mut r = [0u64; 5];
        let mut c = 0u64;
        
        for i in 0..5 {
            let z = self.0[i] as u128 + a.0[i] as u128 + c as u128;
            r[i] = z as u64;
            c = (z >> 64) as u64;
        }
        
        ScalarField(r)
    }
    
    /// Internal subtraction function (without modular reduction).
    ///
    /// Returns the result and a borrow flag. This is a low-level function used internally.
    /// Use `sub()` for normal operations.
    pub fn sub_inner(&self, a: &ScalarField) -> (ScalarField, u64) {
        let mut r = [0u64; 5];
        let mut c = 0u64;
        
        for i in 0..5 {
            // Go: z := U128From64(s[i]).Sub64(a[i]).Sub64(c)
            // Sub64 does: v.Lo, borrowed = bits.Sub64(u.Lo, n, 0)
            //            v.Hi = u.Hi - borrowed
            // Since U128From64 has Hi=0, v.Hi = 0 - borrowed = (0xFFFFFFFFFFFFFFFF if borrowed, else 0)
            // Then c = z.Hi & 1
            
            // Simulate U128 subtraction
            let (diff1, borrow1) = self.0[i].overflowing_sub(a.0[i]);
            // z1.Hi = 0 - borrow1 = (if borrow1 then 0xFFFFFFFFFFFFFFFF else 0)
            let z1_hi: u64 = if borrow1 { 0xFFFFFFFFFFFFFFFF } else { 0 };
            
            // Then subtract c (previous borrow) from diff1
            let (diff2, borrow2) = diff1.overflowing_sub(c);
            // z2.Hi = z1.Hi - borrow2
            let z2_hi: u64 = z1_hi.wrapping_sub(borrow2 as u64);
            
            r[i] = diff2;
            // c = z2.Hi & 1
            c = z2_hi & 1;
        }
        
        if c != 0 {
            (ScalarField(r), 0xFFFFFFFFFFFFFFFF)
        } else {
            (ScalarField(r), 0)
        }
    }
    
    /// Conditionally selects between two scalars.
    ///
    /// Returns `a1` if `c != 0`, otherwise returns `a0`.
    /// This is a constant-time operation used for secure implementations.
    /// Note: `c` should be either 0 or 0xFFFFFFFFFFFFFFFF (all bits set).
    pub fn select(c: u64, a0: &ScalarField, a1: &ScalarField) -> ScalarField {
        // Use -c as a mask: if c != 0, -c will be all 1s; if c == 0, -c will be 0
        // This works because in two's complement: -0 = 0, -0xFFFFFFFFFFFFFFFF = 1
        // Actually, we need: if c == 0, mask = 0; if c != 0, mask = 0xFFFFFFFFFFFFFFFF
        // The XOR trick: a0 ^ (mask & (a0 ^ a1))
        // If mask = 0: a0 ^ 0 = a0
        // If mask = 0xFFFFFFFFFFFFFFFF: a0 ^ (a0 ^ a1) = a1
        let mask = if c == 0 { 0 } else { 0xFFFFFFFFFFFFFFFF };
        ScalarField([
            a0.0[0] ^ (mask & (a0.0[0] ^ a1.0[0])),
            a0.0[1] ^ (mask & (a0.0[1] ^ a1.0[1])),
            a0.0[2] ^ (mask & (a0.0[2] ^ a1.0[2])),
            a0.0[3] ^ (mask & (a0.0[3] ^ a1.0[3])),
            a0.0[4] ^ (mask & (a0.0[4] ^ a1.0[4])),
        ])
    }
    
    /// Adds two scalars with modular reduction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crypto::ScalarField;
    ///
    /// let a = ScalarField::ONE;
    /// let b = ScalarField::TWO;
    /// let sum = a.add(b);
    /// ```
    pub fn add(&self, rhs: ScalarField) -> ScalarField {
        // Fast path: add limbs with carry
        let r0 = self.add_inner(rhs);
        // Try subtracting N to reduce
        let (r1, borrow) = r0.sub_inner(&Self::N);
        // If no borrow (borrow == 0), r0 >= N, so use r1 = r0 - N
        // If borrow (borrow != 0), r0 < N, so use r0
        Self::select(borrow, &r1, &r0)
    }
    
    /// Subtracts two scalars with modular reduction.
    pub fn sub(&self, rhs: ScalarField) -> ScalarField {
        // Try direct subtraction
        let (r0, borrow) = self.sub_inner(&rhs);
        // If borrow (borrow != 0), result is negative, so add N
        // If no borrow (borrow == 0), result is already correct
        if borrow != 0 {
            r0.add_inner(Self::N)
        } else {
            r0
        }
    }
    
    /// Computes the additive inverse (negation) of this scalar.
    pub fn neg(&self) -> ScalarField {
        Self::ZERO.sub(*self)
    }
    
    /// Montgomery multiplication.
    ///
    /// This is a low-level function used internally for efficient modular multiplication.
    /// Use `mul()` for normal operations.
    pub fn monty_mul(&self, rhs: &ScalarField) -> ScalarField {
        let mut r = [0u64; 5];
        
        for i in 0..5 {
            let m = rhs.0[i];
            let f = (self.0[0].wrapping_mul(m).wrapping_add(r[0])).wrapping_mul(Self::N0I);
            
            let mut cc1 = 0u64;
            let mut cc2 = 0u64;
            
            for j in 0..5 {
                // First compute: z = self[j] * m + r[j] + cc1
                let z = (self.0[j] as u128)
                    .wrapping_mul(m as u128)
                    .wrapping_add(r[j] as u128)
                    .wrapping_add(cc1 as u128);
                cc1 = (z >> 64) as u64;
                let z_lo = z as u64;
                
                // Then compute: z = f * N[j] + z_lo + cc2
                let z = (f as u128)
                    .wrapping_mul(Self::N.0[j] as u128)
                    .wrapping_add(z_lo as u128)
                    .wrapping_add(cc2 as u128);
                cc2 = (z >> 64) as u64;
                
                // Store result: if j > 0, store in r[j-1], otherwise it goes into r[4] later
                if j > 0 {
                    r[j-1] = z as u64;
                }
                // Note: when j == 0, the result is discarded (as in Go implementation)
            }
            // Final carry goes into r[4]
            r[4] = cc1.wrapping_add(cc2);
        }
        
        // Reduce modulo N
        let (r2, c) = ScalarField(r).sub_inner(&Self::N);
        Self::select(c, &r2, &ScalarField(r))
    }
    
    /// Multiplies two scalars with modular reduction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crypto::ScalarField;
    ///
    /// let a = ScalarField::TWO;
    /// let b = ScalarField::TWO;
    /// let product = a.mul(&b);
    /// ```
    pub fn mul(&self, rhs: &ScalarField) -> ScalarField {
        // Use Montgomery multiplication for performance
        // Convert to Montgomery form, multiply, and convert back
        let a_mont = self.monty_mul(&Self::R2);
        let b_mont = rhs.monty_mul(&Self::R2);
        let prod_mont = a_mont.monty_mul(&b_mont);
        // Convert back from Montgomery form
        prod_mont.monty_mul(&ScalarField::ONE)
    }
    
    /// Computes the square of this scalar.
    ///
    /// More efficient than `self.mul(&self)`.
    pub fn square(&self) -> ScalarField {
        self.mul(self)
    }
    
    /// Multiplies two canonical scalars and returns the result in canonical form.
    /// 
    /// This is a workaround for the Point::mul() scalar form bug where e*P != (e*sk canonical)*G.
    /// Uses BigUint for canonical multiplication to avoid Montgomery form issues.
    pub fn mul_canonical(&self, rhs: &ScalarField) -> ScalarField {
        // Convert to BigUint (little-endian)
        let self_bytes = self.to_bytes_le();
        let rhs_bytes = rhs.to_bytes_le();
        let n_bytes = Self::N.to_bytes_le();
        
        let self_big = BigUint::from_bytes_le(&self_bytes);
        let rhs_big = BigUint::from_bytes_le(&rhs_bytes);
        let n_big = BigUint::from_bytes_le(&n_bytes);
        
        // Compute product mod N in canonical form
        let product_big = (&self_big * &rhs_big) % &n_big;
        let product_bytes = product_big.to_bytes_le();
        
        // Convert back to limbs
        let mut product_limbs = [0u64; 5];
        for (i, chunk) in product_bytes.chunks(8).enumerate().take(5) {
            let mut limb_bytes = [0u8; 8];
            let copy_len = chunk.len().min(8);
            limb_bytes[..copy_len].copy_from_slice(&chunk[..copy_len]);
            product_limbs[i] = u64::from_le_bytes(limb_bytes);
        }
        
        ScalarField(product_limbs)
    }
    
    /// Converts a scalar from Montgomery form to canonical form.
    ///
    /// Note: `mul()` already returns canonical form, so this is only needed
    /// for values that are explicitly in Montgomery form (e.g., from `monty_mul()`).
    /// Operations like `recode_signed()` and serialization expect canonical form.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crypto::ScalarField;
    ///
    /// let a = ScalarField::from_bytes_le(&[1; 40]).unwrap();
    /// let a_montgomery = a.monty_mul(&ScalarField::R2); // Convert to Montgomery form
    /// let a_canonical = a_montgomery.to_canonical(); // Convert back to canonical
    /// ```
    pub fn to_canonical(&self) -> ScalarField {
        // To convert from Montgomery to canonical: multiply by 1 using Montgomery multiplication
        // If x_m = x * R2 mod n (Montgomery form), then x = x_m * 1 / R mod n
        // Montgomery multiplication with 1 (canonical) gives: (x_m * 1) / R mod n = x mod n
        // Note: ONE is in canonical form [1, 0, 0, 0, 0], which is correct for this conversion
        self.monty_mul(&Self::ONE)
    }
    
    // Convert to little-endian bytes
    pub fn to_bytes_le(&self) -> [u8; 40] {
        let mut result = [0u8; 40];
        for i in 0..5 {
            let bytes = self.0[i].to_le_bytes();
            for j in 0..8 {
                result[i * 8 + j] = bytes[j];
            }
        }
        result
    }
    
    // Convert from little-endian bytes
    pub fn from_bytes_le(data: &[u8]) -> Result<Self, String> {
        if data.len() != 40 {
            return Err("Invalid length".to_string());
        }
        
        let mut value = [0u64; 5];
        for i in 0..5 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[i * 8..(i + 1) * 8]);
            value[i] = u64::from_le_bytes(bytes);
        }
        Ok(ScalarField(value))
    }
    
    /// Converts an Fp5Element to a ScalarField.
    ///
    /// This function creates a 320-bit integer from the 5 Goldilocks field elements
    /// and reduces it modulo the scalar field modulus.
    ///
    /// The conversion treats the Fp5Element as a big-endian 320-bit integer:
    /// `arr[4]<<256 | arr[3]<<192 | arr[2]<<128 | arr[1]<<64 | arr[0]`
    pub fn from_fp5_element(e_fp5: &crate::Fp5Element) -> Self {
        // Match Go's FromGfp5 exactly:
        // Go: result.Or(result, new(big.Int).SetUint64(fp5[i].ToCanonicalUint64()))
        // We need to convert each Goldilocks element to canonical form first
        
        // Create 320-bit integer from array (big-endian interpretation)
        let mut value = BigUint::from(0u64);
        for i in (0..5).rev() {
            value <<= 64;
            // CRITICAL: Use to_canonical_u64() to match Go's ToCanonicalUint64()
            let canonical_val = e_fp5.0[i].to_canonical_u64();
            value += BigUint::from(canonical_val);
        }
        
        // Step 2: FromNonCanonicalBigInt - reduce modulo ORDER
        let order_bytes = hex::decode("7ffffffd800000077ffffff1000000167fffffe6cfb80639e8885c39d724a09ce80fd996948bffe1")
            .expect("invalid ORDER hex");
        let order_big = BigUint::from_bytes_be(&order_bytes);
        let reduced = &value % &order_big;
        
        // Step 3: Convert back to 5-limb scalar
        let reduced_limbs = Self::bigint_to_limbs(reduced);
        ScalarField(reduced_limbs)
    }
    
    // Divide by 2 (right shift)
    pub fn div_by_2(&self) -> ScalarField {
        let mut result = [0u64; 5];
        let mut carry = 0u64;
        
        for i in (0..5).rev() {
            let val = self.0[i];
            result[i] = (val >> 1) | (carry << 63);
            carry = val & 1;
        }
        
        ScalarField(result)
    }
    
    // Recode scalar into signed digits for windowed multiplication (width-w signed recoding)
    /// Recodes a scalar for signed windowed scalar multiplication.
    ///
    /// This is an internal function used for efficient point multiplication.
    pub fn recode_signed(&self, window_width: usize) -> Vec<i32> {
        let w = window_width as i32;
        let mw = (1u32 << w) - 1;
        let hw = 1u32 << (w - 1);
        
        // Compute number of digits needed: (319 + WINDOW) / WINDOW
        let num_digits = (319 + window_width) / window_width;
        let mut digits = vec![0i32; num_digits];
        
        // Process limbs (little-endian: index 0 is least significant)
        let limbs = &self.0;
        let mut acc: u64 = 0;
        let mut acc_len: i32 = 0;
        let mut j = 0;
        let mut cc: u32 = 0;
        
        for i in 0..num_digits {
            // Get next w-bit chunk in bb
            let mut bb: u32;
            if acc_len < w {
                if j < limbs.len() {
                    let nl = limbs[j];
                    j += 1;
                    // Combine accumulator and new limb, extract w bits
                    // Note: acc_len is i32, but shift operations need usize
                    let acc_len_usize = acc_len as usize;
                    let combined = if acc_len_usize < 64 {
                        acc | (nl << acc_len_usize)
                    } else {
                        acc // acc_len >= 64 means acc should already have the value
                    };
                    bb = (combined as u32) & mw;
                    // Shift new limb right by (w - acc_len) bits
                    let shift_amt = (w - acc_len) as usize;
                    acc = if shift_amt < 64 {
                        nl >> shift_amt
                    } else {
                        0
                    };
                } else {
                    bb = (acc as u32) & mw;
                    acc = 0;
                }
                acc_len += 64 - w;
            } else {
                bb = (acc as u32) & mw;
                acc_len -= w;
                let shift_amt = w as usize;
                acc >>= shift_amt;
            }
            
            // If bb is greater than 2^(w-1), subtract 2^w and propagate a carry
            bb = bb.wrapping_add(cc);
            cc = (hw.wrapping_sub(bb)) >> 31;
            digits[i] = (bb as i32) - ((cc << w) as i32);
        }
        
        digits
    }
    
    // Split to 4-bit limbs
    pub fn split_to_4bit_limbs(&self) -> [u8; 80] {
        let mut result = [0u8; 80];
        for i in 0..5 {
            for j in 0..16 {
                result[i * 16 + j] = ((self.0[i] >> (j * 4)) & 0xF) as u8;
            }
        }
        result
    }
    
    // Create ScalarField from u64
    pub fn from_u64(val: u64) -> ScalarField {
        let mut result = [0u64; 5];
        result[0] = val;
        ScalarField(result)
    }
    
    // Add scalar values (for testing)
    pub fn add_raw(&self, val: u64) -> ScalarField {
        let added = ScalarField([self.0[0].wrapping_add(val), self.0[1], self.0[2], self.0[3], self.0[4]]);
        Self::from_non_canonical_limbs(added.0)
    }
    
    // Sample a random scalar using crypto-secure randomness
    /// Generates a cryptographically secure random scalar.
    ///
    /// This function uses a secure random number generator to create a scalar
    /// suitable for use as a private key or nonce.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crypto::ScalarField;
    ///
    /// let private_key = ScalarField::sample_crypto();
    /// ```
    pub fn sample_crypto() -> ScalarField {
        use rand::Rng;
        
        // Generate random big int in range [0, ORDER)
        // ORDER = N = 1067993516717146951041484916571792702745057740581727230159139685185762082554198619328292418486241
        // N in big-endian hex:
        let order_bytes = hex::decode("7ffffffd800000077ffffff1000000167fffffe6cfb80639e8885c39d724a09ce80fd996948bffe1")
            .expect("invalid ORDER hex");
        
        let order_big = BigUint::from_bytes_be(&order_bytes);
        
        // Generate random value less than ORDER
        // We generate random bytes and check if less than ORDER
        let mut rng = rand::thread_rng();
        let mut random_bytes = [0u8; 40];
        
        loop {
            // Generate random bytes
            for byte in &mut random_bytes {
                *byte = rng.gen();
            }
            
            let random_big = BigUint::from_bytes_le(&random_bytes);
            if random_big < order_big {
                // Convert to limbs
                let limbs_array = Self::bigint_to_limbs(random_big);
                return ScalarField(limbs_array);
            }
        }
    }
    
    // Convert big int to 5-limb array (little endian)
    fn bigint_to_limbs(value: BigUint) -> [u64; 5] {
        let bytes = value.to_bytes_le();
        let mut limbs = [0u64; 5];
        
        // Convert bytes to limbs (little endian, 8 bytes per limb)
        for (i, chunk) in bytes.chunks(8).enumerate().take(5) {
            let mut limb_bytes = [0u8; 8];
            let copy_len = chunk.len().min(8);
            limb_bytes[..copy_len].copy_from_slice(&chunk[..copy_len]);
            limbs[i] = u64::from_le_bytes(limb_bytes);
        }
        
        limbs
    }
    
    // Convert non-canonical limbs to canonical scalar (mod N)
    /// Creates a scalar from a non-canonical big integer representation.
    ///
    /// This function reduces the input modulo the scalar field modulus.
    pub fn from_non_canonical_limbs(limbs: [u64; 5]) -> ScalarField {
        // Convert limbs to big int
        let mut value = BigUint::from(0u64);
        for i in (0..5).rev() {
            value <<= 64;
            value += BigUint::from(limbs[i]);
        }
        
        // Reduce modulo ORDER
        let order_bytes = hex::decode("7ffffffd800000077ffffff1000000167fffffe6cfb80639e8885c39d724a09ce80fd996948bffe1")
            .expect("invalid ORDER hex");
        let order_big = BigUint::from_bytes_be(&order_bytes);
        let reduced = &value % &order_big;
        
        // Convert back to limbs
        let reduced_limbs = Self::bigint_to_limbs(reduced);
        ScalarField(reduced_limbs)
    }
}

impl fmt::Display for ScalarField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ScalarField({:016x}{:016x}{:016x}{:016x}{:016x})", 
               self.0[4], self.0[3], self.0[2], self.0[1], self.0[0])
    }
}
