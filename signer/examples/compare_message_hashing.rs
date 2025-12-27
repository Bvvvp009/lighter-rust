//! Compare Message Hashing - Test ArrayFromCanonicalLittleEndianBytes behavior
//!
//! This tool tests the message hashing implementation to ensure it matches Go's
//! ArrayFromCanonicalLittleEndianBytes behavior exactly.
//!
//! Usage:
//!   cargo run --example compare_message_hashing --release

use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
use hex;

/// Test different message lengths to understand padding behavior
fn test_message_hashing(message: &str, expected_elements: Option<Vec<u64>>) {
    println!("\n{}", "=".repeat(80));
    println!("Testing message: \"{}\"", message);
    println!("Message length: {} bytes", message.len());
    println!("{}", "=".repeat(80));
    
    let auth_bytes = message.as_bytes();
    println!("\nMessage bytes: {:?}", auth_bytes);
    println!("Message bytes (hex): {}", hex::encode(auth_bytes));
    
    // Calculate padding
    let missing = (8 - auth_bytes.len() % 8) % 8;
    println!("\nPadding calculation:");
    println!("  Length % 8: {}", auth_bytes.len() % 8);
    println!("  Missing bytes: {}", missing);
    
    let mut elements = Vec::new();
    let mut i = 0;
    let mut chunk_idx = 0;
    
    println!("\nProcessing chunks:");
    while i < auth_bytes.len() {
        let next_start = (i + 8).min(auth_bytes.len());
        let chunk = &auth_bytes[i..next_start];
        
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        
        println!("\n  Chunk {}:", chunk_idx);
        println!("    Original bytes: {:?}", chunk);
        println!("    Original bytes (hex): {}", hex::encode(chunk));
        
        // Pad only the last chunk if needed
        if chunk.len() < 8 && missing > 0 {
            bytes[chunk.len()..].fill(0);
            println!("    After padding: {:?}", bytes);
            println!("    After padding (hex): {}", hex::encode(&bytes));
        }
        
        // CRITICAL: Match Go's FromCanonicalLittleEndianBytes behavior
        // Go reverses bytes before calling SetBytesCanonical (which expects big-endian)
        bytes.reverse();
        let val = u64::from_be_bytes(bytes);
        let goldi = Goldilocks::from_canonical_u64(val);
        elements.push(goldi);
        
        println!("    After reverse: {:?}", bytes);
        println!("    After reverse (hex): {}", hex::encode(&bytes));
        println!("    u64 value: {} (0x{:x})", val, val);
        println!("    Goldilocks element: {} (0x{:x})", goldi.0, goldi.0);
        
        i = next_start;
        chunk_idx += 1;
    }
    
    println!("\nAll Goldilocks elements:");
    for (idx, elem) in elements.iter().enumerate() {
        println!("  [{}]: {} (0x{:x})", idx, elem.0, elem.0);
    }
    
    // Hash using Poseidon2
    let hash_fp5 = hash_to_quintic_extension(&elements);
    println!("\nPoseidon2 Hash (Fp5Element):");
    println!("  Elements: [{}, {}, {}, {}, {}]",
        hash_fp5.0[0].0, hash_fp5.0[1].0, hash_fp5.0[2].0,
        hash_fp5.0[3].0, hash_fp5.0[4].0);
    
    let message_bytes = hash_fp5.to_bytes_le();
    println!("  Hash bytes (hex): {}", hex::encode(&message_bytes));
    println!("  Hash bytes length: {} bytes", message_bytes.len());
    
    // Compare with expected if provided
    if let Some(ref expected) = expected_elements {
        println!("\nComparison with expected:");
        if elements.len() == expected.len() {
            let mut all_match = true;
            for (idx, (actual, exp)) in elements.iter().zip(expected.iter()).enumerate() {
                let matches = actual.0 == *exp;
                if !matches {
                    all_match = false;
                }
                println!("  [{}]: {} {} {} (expected: {})",
                    idx,
                    actual.0,
                    if matches { "==" } else { "!=" },
                    exp,
                    if matches { "✓" } else { "✗ MISMATCH" }
                );
            }
            if all_match {
                println!("\n✅ All elements match expected values!");
            } else {
                println!("\n❌ Some elements do NOT match expected values!");
            }
        } else {
            println!("  Length mismatch: got {} elements, expected {}", elements.len(), expected.len());
        }
    }
    
    println!("{}", "=".repeat(80));
}

fn main() {
    println!("🔍 Message Hashing Comparison Tool");
    println!("This tool tests the message hashing implementation to ensure it matches Go's");
    println!("ArrayFromCanonicalLittleEndianBytes behavior exactly.\n");
    
    // Test case 1: Typical auth token message
    let deadline = 1766426073i64;
    let account_index = 361816i64;
    let api_key_index = 5u8;
    let message1 = format!("{}:{}:{}", deadline, account_index, api_key_index);
    test_message_hashing(&message1, None);
    
    // Test case 2: Short message (to test padding)
    let message2 = "123:456:7";
    test_message_hashing(message2, None);
    
    // Test case 3: Very short message
    let message3 = "1:2:3";
    test_message_hashing(message3, None);
    
    // Test case 4: Exactly 8 bytes (no padding needed)
    let message4 = "12345678";
    test_message_hashing(message4, None);
    
    // Test case 5: Exactly 16 bytes (2 full chunks)
    let message5 = "1234567812345678";
    test_message_hashing(message5, None);
    
    // Test case 6: 17 bytes (2 chunks + 1 byte padding)
    let message6 = "12345678123456789";
    test_message_hashing(message6, None);
    
    println!("\n\n{}", "=".repeat(80));
    println!("ANALYSIS");
    println!("{}", "=".repeat(80));
    println!("\nKey observations:");
    println!("1. Check if padding is applied correctly");
    println!("2. Verify byte reversal matches Go's behavior");
    println!("3. Compare Goldilocks element values with Go output");
    println!("4. Verify Poseidon2 hash output matches Go");
    println!("\nNext steps:");
    println!("- Compare these outputs with Go's ArrayFromCanonicalLittleEndianBytes");
    println!("- Test with actual auth token generation");
    println!("- Verify signatures match Go implementation");
}









