//! Test to check how mul_add2 handles neutral points in the initial setup

use goldilocks_crypto::Point;
use goldilocks_crypto::schnorr::AffinePoint;

#[test]
fn test_neutral_point_addition() {
    println!("\n=== Testing Neutral Point Addition ===\n");
    
    let generator = Point::generator();
    let g_affine = {
        let m1 = generator.z.mul(&generator.t).inverse();
        AffinePoint {
            x: generator.x.mul(&generator.t).mul(&m1),
            u: generator.u.mul(&generator.z).mul(&m1),
        }
    };
    
    let neutral = AffinePoint::neutral();
    
    println!("Testing neutral.to_point().add_affine(&neutral):");
    let neutral_pt = neutral.to_point();
    let result1 = neutral_pt.add_affine(&neutral);
    let result1_encoded = result1.encode();
    println!("  Result encoded: {}", hex::encode(&result1_encoded.to_bytes_le()));
    println!("  Is neutral: {}", result1.is_neutral());
    
    println!("\nTesting g_affine.to_point().add_affine(&neutral):");
    let g_pt = g_affine.to_point();
    let result2 = g_pt.add_affine(&neutral);
    let result2_encoded = result2.encode();
    let g_encoded = generator.encode();
    println!("  Result encoded: {}", hex::encode(&result2_encoded.to_bytes_le()));
    println!("  G encoded: {}", hex::encode(&g_encoded.to_bytes_le()));
    println!("  Match: {}", result2_encoded.0.iter().zip(g_encoded.0.iter()).all(|(a, b)| a.0 == b.0));
    
    println!("\nTesting neutral.to_point().add_affine(&g_affine):");
    let result3 = neutral_pt.add_affine(&g_affine);
    let result3_encoded = result3.encode();
    println!("  Result encoded: {}", hex::encode(&result3_encoded.to_bytes_le()));
    println!("  Match: {}", result3_encoded.0.iter().zip(g_encoded.0.iter()).all(|(a, b)| a.0 == b.0));
}

