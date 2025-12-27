//! Debug test for window creation

use goldilocks_crypto::{ScalarField, Point};
use hex;

#[test]
fn test_window_creation() {
    println!("\n=== Debug Window Creation ===\n");
    
    let generator = Point::generator();
    
    // Create window using make_window_affine
    let window = generator.make_window_affine();
    
    println!("Window size: {}", window.len());
    
    // Manually compute what each window entry should be
    for i in 0..std::cmp::min(8, window.len()) {
        let window_point_x = hex::encode(&window[i].x.to_bytes_le());
        let window_point_u = hex::encode(&window[i].u.to_bytes_le());
        
        // Compute manually using simple multiplication
        let expected = generator.mul_simple((i + 1) as u64);
        let expected_affine_x = expected.x.mul(&expected.t).mul(&expected.z.mul(&expected.t).inverse());
        let expected_affine_u = expected.u.mul(&expected.z).mul(&expected.z.mul(&expected.t).inverse());
        
        let expected_x = hex::encode(&expected_affine_x.to_bytes_le());
        let expected_u = hex::encode(&expected_affine_u.to_bytes_le());
        
        println!("\nWindow[{}]:", i);
        println!("  Got x: {}...", &window_point_x[..32]);
        println!("  Exp x: {}...", &expected_x[..32]);
        println!("  Match: {}", window_point_x == expected_x);
        
        if window_point_x != expected_x {
            println!("  ❌ MISMATCH at window[{}]!", i);
        }
    }
}
