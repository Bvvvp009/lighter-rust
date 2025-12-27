//! Test to debug add_affine neutral handling

use goldilocks_crypto::Point;
use goldilocks_crypto::schnorr::AffinePoint;

#[test]
fn test_add_affine_neutral_debug() {
    println!("\n=== Debugging add_affine Neutral Handling ===\n");
    
    let generator = Point::generator();
    let neutral_pt = Point::neutral();
    
    println!("Generator point:");
    println!("  is_neutral(): {}", generator.is_neutral());
    println!("  encoded: {}", hex::encode(&generator.encode().to_bytes_le()));
    
    println!("\nNeutral point:");
    println!("  is_neutral(): {}", neutral_pt.is_neutral());
    println!("  encoded: {}", hex::encode(&neutral_pt.encode().to_bytes_le()));
    
    // Convert to affine
    let g_affine = {
        let m1 = generator.z.mul(&generator.t).inverse();
        AffinePoint {
            x: generator.x.mul(&generator.t).mul(&m1),
            u: generator.u.mul(&generator.z).mul(&m1),
        }
    };
    
    let neutral_affine = AffinePoint::neutral();
    
    println!("\nAffine neutral:");
    println!("  u.is_zero(): {}", neutral_affine.u.is_zero());
    
    // Test the actual addition
    println!("\nTesting generator.add_affine(&neutral_affine):");
    let result = generator.add_affine(&neutral_affine);
    println!("  Result is_neutral(): {}", result.is_neutral());
    println!("  Result encoded: {}", hex::encode(&result.encode().to_bytes_le()));
    println!("  Generator encoded: {}", hex::encode(&generator.encode().to_bytes_le()));
    let match_result = result.encode().0.iter().zip(generator.encode().0.iter()).all(|(a, b)| a.0 == b.0);
    println!("  Match: {}", match_result);
}









