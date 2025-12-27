//! Debug Signature Components - Detailed Analysis Tool
//!
//! This tool generates signatures and provides detailed analysis of all components:
//! - R point encoding (Point vs WeierstrassPoint)
//! - Challenge e computation
//! - Response s computation
//! - Encoding consistency checks
//! - Montgomery form analysis
//!
//! Usage:
//!   cargo run --example debug_signature_components --release

use goldilocks_crypto::{
    verify_signature, ScalarField, Point, WeierstrassPoint, 
    Fp5Element, Goldilocks,
};
use poseidon_hash::hash_to_quintic_extension;
use hex;

struct DetailedSignatureAnalysis {
    // Inputs
    private_key_bytes: [u8; 40],
    message_bytes: [u8; 40],
    nonce_bytes: [u8; 40],
    
    // Scalar values
    private_scalar: ScalarField,
    nonce_scalar: ScalarField,
    message_fp5: Fp5Element,
    
    // R point computation
    r_point: Point,
    r_encoded_point: Fp5Element,
    r_weierstrass: WeierstrassPoint,
    r_encoded_weierstrass: Fp5Element,
    
    // Challenge computation
    pre_image: [Goldilocks; 10],
    e_fp5: Fp5Element,
    e_scalar: ScalarField,
    
    // Response computation
    e_times_private: ScalarField,
    s: ScalarField,
    
    // Final signature
    signature: Vec<u8>,
    
    // Verification
    verification_result: bool,
}

fn analyze_signature_generation(
    private_key_bytes: &[u8; 40],
    message_bytes: &[u8; 40],
    nonce_bytes: &[u8; 40],
) -> Result<DetailedSignatureAnalysis, Box<dyn std::error::Error>> {
    // Convert inputs
    let private_scalar = ScalarField::from_bytes_le(private_key_bytes)
        .map_err(|e| format!("Failed to parse private key: {:?}", e))?;
    
    let nonce_scalar = ScalarField::from_bytes_le(nonce_bytes)
        .map_err(|e| format!("Failed to parse nonce: {:?}", e))?;
    
    let message_fp5 = Fp5Element::from_bytes_le(message_bytes)
        .map_err(|e| format!("Failed to parse message: {:?}", e))?;
    
    // Step 1: Compute R = nonce * generator_point
    let generator = Point::generator();
    let r_point = generator.mul(&nonce_scalar);
    let r_encoded_point = r_point.encode();
    
    // Also encode using WeierstrassPoint for comparison
    let r_weierstrass = WeierstrassPoint::decode_fp5_as_weierstrass(&r_encoded_point)
        .ok_or("Failed to decode R as WeierstrassPoint")?;
    let r_encoded_weierstrass = r_weierstrass.encode();
    
    // Step 2: Compute challenge e = H(R || message)
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded_point.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    let e_fp5 = hash_to_quintic_extension(&pre_image);
    let e_scalar = ScalarField::from_fp5_element(&e_fp5);
    
    // Step 3: Compute response s = nonce - e * private_key
    let e_times_private = e_scalar.mul(&private_scalar);
    let s = nonce_scalar.sub(e_times_private);
    
    // Step 4: Assemble signature
    let mut signature = [0u8; 80];
    let s_bytes = s.to_bytes_le();
    signature[..40].copy_from_slice(&s_bytes);
    let e_bytes = e_scalar.to_bytes_le();
    signature[40..].copy_from_slice(&e_bytes);
    
    // Verify signature
    let public_key = generator.mul(&private_scalar).encode().to_bytes_le();
    let verification_result = verify_signature(&signature, message_bytes, &public_key)
        .unwrap_or(false);
    
    Ok(DetailedSignatureAnalysis {
        private_key_bytes: *private_key_bytes,
        message_bytes: *message_bytes,
        nonce_bytes: *nonce_bytes,
        private_scalar,
        nonce_scalar,
        message_fp5,
        r_point,
        r_encoded_point,
        r_weierstrass,
        r_encoded_weierstrass,
        pre_image,
        e_fp5,
        e_scalar,
        e_times_private,
        s,
        signature: signature.to_vec(),
        verification_result,
    })
}

fn print_analysis(analysis: &DetailedSignatureAnalysis, test_num: usize) {
    println!("\n{}", "=".repeat(80));
    println!("DETAILED SIGNATURE ANALYSIS #{}", test_num);
    println!("{}", "=".repeat(80));
    
    // Inputs
    println!("\n📝 INPUTS:");
    println!("  Private key (hex): {}", hex::encode(&analysis.private_key_bytes));
    println!("  Message (hex):     {}", hex::encode(&analysis.message_bytes));
    println!("  Nonce (hex):       {}", hex::encode(&analysis.nonce_bytes));
    
    // R Point Analysis
    println!("\n🔵 R POINT ANALYSIS:");
    println!("  R = nonce * G");
    println!("  R (Point encoding - t/u):");
    println!("    Elements: [{}, {}, {}, {}, {}]",
        analysis.r_encoded_point.0[0].0,
        analysis.r_encoded_point.0[1].0,
        analysis.r_encoded_point.0[2].0,
        analysis.r_encoded_point.0[3].0,
        analysis.r_encoded_point.0[4].0);
    println!("    Bytes (hex): {}", hex::encode(&analysis.r_encoded_point.to_bytes_le()));
    
    println!("  R (WeierstrassPoint encoding - Y/(A/3-X)):");
    println!("    Elements: [{}, {}, {}, {}, {}]",
        analysis.r_encoded_weierstrass.0[0].0,
        analysis.r_encoded_weierstrass.0[1].0,
        analysis.r_encoded_weierstrass.0[2].0,
        analysis.r_encoded_weierstrass.0[3].0,
        analysis.r_encoded_weierstrass.0[4].0);
    println!("    Bytes (hex): {}", hex::encode(&analysis.r_encoded_weierstrass.to_bytes_le()));
    
    let encoding_match = analysis.r_encoded_point.0.iter()
        .zip(analysis.r_encoded_weierstrass.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("  Encoding match: {}", if encoding_match { "✅ YES" } else { "❌ NO" });
    if !encoding_match {
        println!("    ⚠️  Point and WeierstrassPoint encodings differ!");
        println!("    This is expected (different formulas), but verification uses WeierstrassPoint.");
        println!("    If signing uses Point encoding but verification expects WeierstrassPoint,");
        println!("    signatures will fail verification!");
    }
    
    // Challenge e Analysis
    println!("\n🟢 CHALLENGE (e) ANALYSIS:");
    println!("  e = H(R || message)");
    println!("  Pre-image (10 Goldilocks elements):");
    for (i, elem) in analysis.pre_image.iter().enumerate() {
        println!("    [{}]: {} (0x{:x})", i, elem.0, elem.0);
    }
    
    println!("  e (Fp5):");
    println!("    Elements: [{}, {}, {}, {}, {}]",
        analysis.e_fp5.0[0].0,
        analysis.e_fp5.0[1].0,
        analysis.e_fp5.0[2].0,
        analysis.e_fp5.0[3].0,
        analysis.e_fp5.0[4].0);
    println!("    Bytes (hex): {}", hex::encode(&analysis.e_fp5.to_bytes_le()));
    
    println!("  e (Scalar):");
    println!("    Bytes (hex): {}", hex::encode(&analysis.e_scalar.to_bytes_le()));
    
    // Response s Analysis
    println!("\n🟡 RESPONSE (s) ANALYSIS:");
    println!("  s = nonce - e * private_key");
    println!("  e * private_key:");
    println!("    Bytes (hex): {}", hex::encode(&analysis.e_times_private.to_bytes_le()));
    println!("  s:");
    println!("    Bytes (hex): {}", hex::encode(&analysis.s.to_bytes_le()));
    
    // Check for potential arithmetic issues
    let nonce_bytes = analysis.nonce_scalar.to_bytes_le();
    let e_times_private_bytes = analysis.e_times_private.to_bytes_le();
    let s_bytes = analysis.s.to_bytes_le();
    
    println!("\n  Arithmetic check (s = k - e*sk):");
    println!("    k (hex):        {}", hex::encode(&nonce_bytes));
    println!("    e*sk (hex):     {}", hex::encode(&e_times_private_bytes));
    println!("    s (hex):        {}", hex::encode(&s_bytes));
    
    // Try to reconstruct k from s + e*sk
    let k_reconstructed = analysis.s.add(analysis.e_times_private);
    let k_reconstructed_bytes = k_reconstructed.to_bytes_le();
    let k_match = nonce_bytes == k_reconstructed_bytes;
    
    println!("    k_reconstructed: {}", hex::encode(&k_reconstructed_bytes));
    println!("    k matches: {}", if k_match { "✅ YES" } else { "❌ NO" });
    
    if !k_match {
        println!("    ⚠️  Arithmetic error detected! s + e*sk != k");
        println!("    This suggests an issue with subtraction or Montgomery form.");
    }
    
    // Final Signature
    println!("\n🔴 FINAL SIGNATURE:");
    println!("  Signature (hex): {}", hex::encode(&analysis.signature));
    println!("  s part (bytes 0-39):  {}", hex::encode(&analysis.signature[..40]));
    println!("  e part (bytes 40-79): {}", hex::encode(&analysis.signature[40..]));
    
    // Verification
    println!("\n✅ VERIFICATION:");
    println!("  Local verification: {}", 
        if analysis.verification_result { "✅ PASSED" } else { "❌ FAILED" });
    
    if !analysis.verification_result {
        println!("\n  ⚠️  SIGNATURE FAILS LOCAL VERIFICATION!");
        println!("  Possible causes:");
        println!("    1. R encoding mismatch (Point vs WeierstrassPoint)");
        println!("    2. Arithmetic error in s = k - e*sk");
        println!("    3. Montgomery form issues in subtraction");
        println!("    4. Encoding inconsistency between signing and verification");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Signature Components Debug Tool");
    println!("{}", "=".repeat(80));
    println!("This tool provides detailed analysis of signature generation components.\n");
    
    // Test cases with fixed nonces
    let test_cases = vec![
        (
            "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
            "0000000000000000000000000000000000000000000000000000000000000000000000000000",
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "Test 1: All zeros message"
        ),
        (
            "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456",
            "Test 2: All ones message"
        ),
        (
            "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876",
            "Test 3: Pattern message"
        ),
    ];
    
    let mut total_tests = 0;
    let mut passed_tests = 0;
    
    for (i, (private_key_hex, message_hex, nonce_hex, description)) in test_cases.iter().enumerate() {
        println!("\n\n{}", "🔄".repeat(40));
        println!("{}", description);
        println!("{}", "🔄".repeat(40));
        
        // Parse hex strings
        let private_key_bytes: [u8; 40] = hex::decode(private_key_hex)?
            .try_into()
            .map_err(|_| "Invalid private key length")?;
        let message_bytes: [u8; 40] = hex::decode(message_hex)?
            .try_into()
            .map_err(|_| "Invalid message length")?;
        let nonce_bytes: [u8; 40] = hex::decode(nonce_hex)?
            .try_into()
            .map_err(|_| "Invalid nonce length")?;
        
        // Analyze signature generation
        match analyze_signature_generation(&private_key_bytes, &message_bytes, &nonce_bytes) {
            Ok(analysis) => {
                print_analysis(&analysis, i + 1);
                total_tests += 1;
                if analysis.verification_result {
                    passed_tests += 1;
                }
            }
            Err(e) => {
                println!("❌ Error analyzing signature: {}", e);
            }
        }
    }
    
    // Summary
    println!("\n\n{}", "=".repeat(80));
    println!("SUMMARY");
    println!("{}", "=".repeat(80));
    println!("  Total tests: {}", total_tests);
    println!("  Passed verification: {}", passed_tests);
    println!("  Failed verification: {}", total_tests - passed_tests);
    
    if passed_tests == total_tests {
        println!("\n✅ All signatures pass local verification!");
    } else {
        let failure_rate = ((total_tests - passed_tests) as f64 / total_tests as f64) * 100.0;
        println!("\n❌ Some signatures fail verification!");
        println!("  Failure rate: {:.1}%", failure_rate);
        println!("\n  Next steps:");
        println!("    1. Compare R encodings - check if Point vs WeierstrassPoint is the issue");
        println!("    2. Check arithmetic in s = k - e*sk - verify Montgomery form handling");
        println!("    3. Compare with Go implementation using compare_go_rust_signatures tool");
        println!("    4. Check if verification uses different encoding than signing");
    }
    
    Ok(())
}

