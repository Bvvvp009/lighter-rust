//! Compare Go vs Rust Signature Generation - Byte-for-Byte Analysis
//!
//! This tool generates signatures with identical inputs in both Rust and Go,
//! then compares them byte-for-byte along with all intermediate values.
//!
//! Usage:
//!   1. Ensure Go is installed and lighter-go is built
//!   2. Run: cargo run --example compare_go_rust_signatures --release
//!
//! This tool will:
//!   - Generate signatures with fixed nonces for deterministic comparison
//!   - Compare R point encodings (Point vs WeierstrassPoint)
//!   - Compare e (challenge) values
//!   - Compare s (response) values
//!   - Compare final signatures byte-for-byte
//!   - Check encoding consistency

use goldilocks_crypto::{
    ScalarField, Point, WeierstrassPoint, Fp5Element, Goldilocks,
};
use poseidon_hash::hash_to_quintic_extension;
use hex;
use std::process::Command;

struct SignatureComponents {
    r_point_encoded: Fp5Element,
    r_point_weierstrass_encoded: Fp5Element,
    e_fp5: Fp5Element,
    e_scalar: ScalarField,
    s: ScalarField,
    signature: Vec<u8>,
}

fn generate_rust_signature_with_details(
    private_key_bytes: &[u8; 40],
    message_bytes: &[u8; 40],
    nonce_bytes: &[u8; 40],
) -> Result<SignatureComponents, Box<dyn std::error::Error>> {
    // Convert private key
    let private_scalar = ScalarField::from_bytes_le(private_key_bytes)
        .map_err(|e| format!("Failed to parse private key: {:?}", e))?;
    
    // Convert nonce
    let nonce_scalar = ScalarField::from_bytes_le(nonce_bytes)
        .map_err(|e| format!("Failed to parse nonce: {:?}", e))?;
    
    // Convert message to Fp5Element
    let message_fp5 = Fp5Element::from_bytes_le(message_bytes)
        .map_err(|e| format!("Failed to parse message: {:?}", e))?;
    
    // Step 1: Compute R = nonce * generator_point
    let generator = Point::generator();
    let r_point = generator.mul(&nonce_scalar);
    let r_encoded = r_point.encode();
    
    // Also encode using WeierstrassPoint for comparison
    let r_weierstrass = WeierstrassPoint::decode_fp5_as_weierstrass(&r_encoded)
        .ok_or("Failed to decode R as WeierstrassPoint")?;
    let r_weierstrass_encoded = r_weierstrass.encode();
    
    // Step 2: Compute challenge e = H(R || message)
    let mut pre_image = [Goldilocks::zero(); 10];
    pre_image[..5].copy_from_slice(&r_encoded.0);
    pre_image[5..].copy_from_slice(&message_fp5.0);
    
    let e_fp5 = hash_to_quintic_extension(&pre_image);
    let e_scalar = ScalarField::from_fp5_element(&e_fp5);
    
    // Step 3: Compute response s = nonce - e * private_key
    // CRITICAL: e*private_key from mul() is in Montgomery form, must convert to canonical
    let e_times_private = e_scalar.mul(&private_scalar);
    let e_times_private_canonical = e_times_private.to_canonical();
    let s = nonce_scalar.sub(e_times_private_canonical);
    
    // Step 4: Assemble signature
    let mut signature = [0u8; 80];
    let s_bytes = s.to_bytes_le();
    signature[..40].copy_from_slice(&s_bytes);
    let e_bytes = e_scalar.to_bytes_le();
    signature[40..].copy_from_slice(&e_bytes);
    
    Ok(SignatureComponents {
        r_point_encoded: r_encoded,
        r_point_weierstrass_encoded: r_weierstrass_encoded,
        e_fp5,
        e_scalar,
        s,
        signature: signature.to_vec(),
    })
}

fn print_fp5_element(label: &str, fp5: &Fp5Element) {
    println!("  {}:", label);
    println!("    Elements: [{}, {}, {}, {}, {}]",
        fp5.0[0].0, fp5.0[1].0, fp5.0[2].0, fp5.0[3].0, fp5.0[4].0);
    println!("    Bytes (hex): {}", hex::encode(&fp5.to_bytes_le()));
}

fn print_scalar(label: &str, scalar: &ScalarField) {
    println!("  {}:", label);
    let limbs = scalar.to_bytes_le();
    println!("    Limbs: {:?}", limbs.chunks(8).map(|c| {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(c);
        u64::from_le_bytes(arr)
    }).collect::<Vec<_>>());
    println!("    Bytes (hex): {}", hex::encode(&limbs));
}

fn compare_components(
    rust: &SignatureComponents,
    go: &SignatureComponents,
    test_num: usize,
) {
    println!("\n{}", "=".repeat(80));
    println!("COMPARISON #{}", test_num);
    println!("{}", "=".repeat(80));
    
    // Compare R point encodings
    println!("\n📊 R Point Encoding Comparison:");
    print_fp5_element("Rust (Point::encode)", &rust.r_point_encoded);
    print_fp5_element("Rust (WeierstrassPoint::encode)", &rust.r_point_weierstrass_encoded);
    print_fp5_element("Go (expected)", &go.r_point_encoded);
    
    let r_match_point = rust.r_point_encoded.0.iter()
        .zip(go.r_point_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    let r_match_weierstrass = rust.r_point_weierstrass_encoded.0.iter()
        .zip(go.r_point_encoded.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    println!("\n  R encoding match (Point): {}", if r_match_point { "✅ YES" } else { "❌ NO" });
    println!("  R encoding match (Weierstrass): {}", if r_match_weierstrass { "✅ YES" } else { "❌ NO" });
    
    if !r_match_point && !r_match_weierstrass {
        println!("  ⚠️  Neither encoding matches Go!");
        println!("  This suggests an issue with R point computation or encoding.");
    }
    
    // Compare e (challenge) values
    println!("\n📊 Challenge (e) Comparison:");
    print_fp5_element("Rust e (Fp5)", &rust.e_fp5);
    print_fp5_element("Go e (Fp5)", &go.e_fp5);
    print_scalar("Rust e (Scalar)", &rust.e_scalar);
    print_scalar("Go e (Scalar)", &go.e_scalar);
    
    let e_fp5_match = rust.e_fp5.0.iter()
        .zip(go.e_fp5.0.iter())
        .all(|(a, b)| a.0 == b.0);
    
    let e_scalar_match = rust.e_scalar.to_bytes_le() == go.e_scalar.to_bytes_le();
    
    println!("\n  e (Fp5) match: {}", if e_fp5_match { "✅ YES" } else { "❌ NO" });
    println!("  e (Scalar) match: {}", if e_scalar_match { "✅ YES" } else { "❌ NO" });
    
    if !e_fp5_match {
        println!("  ⚠️  e values differ! This could be due to:");
        println!("     - Different R encoding used in hash");
        println!("     - Different message encoding");
        println!("     - Different hash implementation");
    }
    
    // Compare s (response) values
    println!("\n📊 Response (s) Comparison:");
    print_scalar("Rust s", &rust.s);
    print_scalar("Go s", &go.s);
    
    let s_match = rust.s.to_bytes_le() == go.s.to_bytes_le();
    println!("\n  s match: {}", if s_match { "✅ YES" } else { "❌ NO" });
    
    if !s_match {
        println!("  ⚠️  s values differ! This could be due to:");
        println!("     - Different e values (see above)");
        println!("     - Different arithmetic in s = k - e*sk");
        println!("     - Montgomery form issues in subtraction");
    }
    
    // Compare final signatures
    println!("\n📊 Final Signature Comparison:");
    println!("  Rust signature (hex): {}", hex::encode(&rust.signature));
    println!("  Go signature (hex):   {}", hex::encode(&go.signature));
    
    let sig_match = rust.signature == go.signature;
    println!("\n  Signature match: {}", if sig_match { "✅ YES" } else { "❌ NO" });
    
    if sig_match {
        println!("\n  ✅✅✅ SIGNATURES MATCH PERFECTLY! ✅✅✅");
    } else {
        println!("\n  ❌❌❌ SIGNATURES DO NOT MATCH ❌❌❌");
        
        // Show byte-by-byte differences
        println!("\n  Byte-by-byte differences:");
        let mut diff_count = 0;
        for (i, (r_byte, g_byte)) in rust.signature.iter().zip(go.signature.iter()).enumerate() {
            if r_byte != g_byte {
                diff_count += 1;
                if diff_count <= 10 {  // Show first 10 differences
                    println!("    Byte {}: Rust=0x{:02x}, Go=0x{:02x}", i, r_byte, g_byte);
                }
            }
        }
        if diff_count > 10 {
            println!("    ... and {} more differences", diff_count - 10);
        }
        
        // Check if s or e parts differ
        let s_match = rust.signature[..40] == go.signature[..40];
        let e_match = rust.signature[40..] == go.signature[40..];
        
        println!("\n  s part (bytes 0-39) match: {}", if s_match { "✅ YES" } else { "❌ NO" });
        println!("  e part (bytes 40-79) match: {}", if e_match { "✅ YES" } else { "❌ NO" });
    }
}

fn run_go_signature_test(
    private_key_hex: &str,
    message_hex: &str,
    nonce_hex: &str,
) -> Result<SignatureComponents, Box<dyn std::error::Error>> {
    // Run the Go helper script
    let go_helper_path = "lighter-rust/signer/examples/go_signature_helper.go";
    
    // Check if Go helper exists
    if !std::path::Path::new(go_helper_path).exists() {
        return Err(format!("Go helper not found at: {}. Please ensure it exists.", go_helper_path).into());
    }
    
    // Run Go helper
    let output = Command::new("go")
        .arg("run")
        .arg(go_helper_path)
        .arg(private_key_hex)
        .arg(message_hex)
        .arg(nonce_hex)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Go helper failed: {}", stderr).into());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse output
    let mut r_encoded_elements = Vec::new();
    let mut e_fp5_elements = Vec::new();
    let mut e_scalar_hex = String::new();
    let mut s_scalar_hex = String::new();
    let mut signature_hex = String::new();
    
    for line in stdout.lines() {
        if line.starts_with("R_ENCODED:") {
            let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
            for part in parts {
                r_encoded_elements.push(part.parse::<u64>()?);
            }
        } else if line.starts_with("E_FP5:") {
            let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
            for part in parts {
                e_fp5_elements.push(part.parse::<u64>()?);
            }
        } else if line.starts_with("E_SCALAR:") {
            e_scalar_hex = line.split(':').nth(1).unwrap_or("").to_string();
        } else if line.starts_with("S_SCALAR:") {
            s_scalar_hex = line.split(':').nth(1).unwrap_or("").to_string();
        } else if line.starts_with("SIGNATURE:") {
            signature_hex = line.split(':').nth(1).unwrap_or("").to_string();
        }
    }
    
    // Convert to Rust types
    if r_encoded_elements.len() != 5 {
        return Err(format!("Expected 5 R encoded elements, got {}", r_encoded_elements.len()).into());
    }
    if e_fp5_elements.len() != 5 {
        return Err(format!("Expected 5 e Fp5 elements, got {}", e_fp5_elements.len()).into());
    }
    
    let r_encoded = Fp5Element([
        Goldilocks(r_encoded_elements[0]),
        Goldilocks(r_encoded_elements[1]),
        Goldilocks(r_encoded_elements[2]),
        Goldilocks(r_encoded_elements[3]),
        Goldilocks(r_encoded_elements[4]),
    ]);
    
    let e_fp5 = Fp5Element([
        Goldilocks(e_fp5_elements[0]),
        Goldilocks(e_fp5_elements[1]),
        Goldilocks(e_fp5_elements[2]),
        Goldilocks(e_fp5_elements[3]),
        Goldilocks(e_fp5_elements[4]),
    ]);
    
    let e_scalar_bytes = hex::decode(e_scalar_hex.trim())?;
    let e_scalar = ScalarField::from_bytes_le(&e_scalar_bytes.try_into().map_err(|_| "Invalid e scalar length")?)?;
    
    let s_bytes = hex::decode(s_scalar_hex.trim())?;
    let s = ScalarField::from_bytes_le(&s_bytes.try_into().map_err(|_| "Invalid s scalar length")?)?;
    
    let signature_bytes = hex::decode(signature_hex.trim())?;
    
    Ok(SignatureComponents {
        r_point_encoded: r_encoded,
        r_point_weierstrass_encoded: r_encoded, // Will be computed separately if needed
        e_fp5,
        e_scalar,
        s,
        signature: signature_bytes,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Go vs Rust Signature Comparison Tool");
    println!("{}", "=".repeat(80));
    println!("This tool compares signature generation between Rust and Go implementations");
    println!("using identical inputs (private key, message, nonce).\n");
    
    // Test vectors with fixed nonces for deterministic comparison
    let test_cases = vec![
        (
            "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
            "0000000000000000000000000000000000000000000000000000000000000000000000000000",
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "Test 1: All zeros message with fixed nonce"
        ),
        (
            "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456",
            "Test 2: All ones message with fixed nonce"
        ),
        (
            "a7ba74373af1bacbfc6d635e6a21df5fcfb0c16cac4cfe1c8a7467bb0f224addeacfcd5dd1a0fa6c",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876",
            "Test 3: Pattern message with fixed nonce"
        ),
    ];
    
    for (i, (private_key_hex, message_hex, nonce_hex, description)) in test_cases.iter().enumerate() {
        println!("\n\n{}", "🔄".repeat(40));
        println!("{}", description);
        println!("{}", "🔄".repeat(40));
        
        // Parse hex strings
        let private_key_bytes = hex::decode(private_key_hex)?
            .try_into()
            .map_err(|_| "Invalid private key length")?;
        let message_bytes = hex::decode(message_hex)?
            .try_into()
            .map_err(|_| "Invalid message length")?;
        let nonce_bytes = hex::decode(nonce_hex)?
            .try_into()
            .map_err(|_| "Invalid nonce length")?;
        
        println!("\n📝 Inputs:");
        println!("  Private key (hex): {}", private_key_hex);
        println!("  Message (hex):     {}", message_hex);
        println!("  Nonce (hex):       {}", nonce_hex);
        
        // Generate Rust signature with details
        println!("\n🔧 Generating Rust signature...");
        let rust_components = generate_rust_signature_with_details(
            &private_key_bytes,
            &message_bytes,
            &nonce_bytes,
        )?;
        
        println!("✅ Rust signature generated");
        println!("  Signature (hex): {}", hex::encode(&rust_components.signature));
        
        // Try to generate Go signature (placeholder for now)
        println!("\n🔧 Generating Go signature...");
        match run_go_signature_test(private_key_hex, message_hex, nonce_hex) {
            Ok(go_components) => {
                compare_components(&rust_components, &go_components, i + 1);
            }
            Err(e) => {
                println!("⚠️  Go comparison not available: {}", e);
                println!("\n📊 Rust-only analysis:");
                println!("  R (Point encoding): {}", hex::encode(&rust_components.r_point_encoded.to_bytes_le()));
                println!("  R (Weierstrass encoding): {}", hex::encode(&rust_components.r_point_weierstrass_encoded.to_bytes_le()));
                println!("  e (Fp5): {}", hex::encode(&rust_components.e_fp5.to_bytes_le()));
                println!("  e (Scalar): {}", hex::encode(&rust_components.e_scalar.to_bytes_le()));
                println!("  s: {}", hex::encode(&rust_components.s.to_bytes_le()));
                println!("  Signature: {}", hex::encode(&rust_components.signature));
                
                // Check encoding consistency
                let encoding_match = rust_components.r_point_encoded.0.iter()
                    .zip(rust_components.r_point_weierstrass_encoded.0.iter())
                    .all(|(a, b)| a.0 == b.0);
                
                println!("\n  Encoding consistency:");
                println!("    Point::encode() == WeierstrassPoint::encode(): {}", 
                    if encoding_match { "✅ YES" } else { "❌ NO" });
                
                if !encoding_match {
                    println!("    ⚠️  Encodings differ - this is expected but may cause verification issues");
                    println!("    Point uses: t/u (Montgomery form)");
                    println!("    Weierstrass uses: Y/(A/3-X)");
                }
            }
        }
    }
    
    println!("\n\n{}", "=".repeat(80));
    println!("COMPARISON COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\nNext steps:");
    println!("  1. Implement Go test infrastructure for byte-for-byte comparison");
    println!("  2. Compare R point encodings between implementations");
    println!("  3. Verify e computation matches exactly");
    println!("  4. Check s computation for arithmetic issues");
    println!("  5. Test with real Go signatures from the API");
    
    Ok(())
}

