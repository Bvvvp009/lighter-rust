use goldilocks_crypto::{ScalarField, sign_hashed_message};
use hex;

fn main() {
    let privkey_hex = "c5230d52492a608954476c66f3be44559460d101dccec8d4e2e8d2caf4f3b983e77389563df72f51";
    let privkey_bytes = hex::decode(privkey_hex).unwrap();
    
    // Go's hash
    let hash_hex = "b63f2674b8fd44b604c142d0c8c1bdf8e3a2d3dab4ad8e6b15e64803becef90d59e7f02c9807fb4c";
    let hash_bytes = hex::decode(hash_hex).unwrap();
    
    // Nonce as scalar 2
    let nonce_scalar = ScalarField::new([2, 0, 0, 0, 0]);
    let nonce_bytes = nonce_scalar.to_bytes_le();
    
    let signature = sign_hashed_message(&privkey_bytes, &hash_bytes, &nonce_bytes).unwrap();
    let sig_hex = hex::encode(&signature);
    
    println!("Nonce scalar [2, 0, 0, 0, 0] -> bytes: {}", hex::encode(&nonce_bytes));
    println!("Rust signature:   {}", sig_hex);
    println!("Go signature:     eeb76862d2f64fa1f2b5e4e1bbee455ea45b7cb2dddf603456d829044ff0c339c101f10e8fa7a74bff6bdc4971eeb11a9dcc05b854867553ccb4309ca188141fde370e00b52849515fc31c4af8f2f714");
    
    if sig_hex == "eeb76862d2f64fa1f2b5e4e1bbee455ea45b7cb2dddf603456d829044ff0c339c101f10e8fa7a74bff6bdc4971eeb11a9dcc05b854867553ccb4309ca188141fde370e00b52849515fc31c4af8f2f714" {
        println!("✅ Signatures match!");
    } else {
        println!("❌ Signatures differ!");
    }
}
