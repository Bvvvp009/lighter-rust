use goldilocks_crypto::{ScalarField};
use hex;

fn main() {
    let priv_hex = "c5230d52492a608954476c66f3be44559460d101dccec8d4e2e8d2caf4f3b983e77389563df72f51";
    let priv_bytes = hex::decode(priv_hex).unwrap();
    
    let sk = ScalarField::from_bytes_le(&priv_bytes).unwrap();
    println!("Private key hex: {}", priv_hex);
    println!("Scalar limbs: {:?}", sk.0);
    
    let canonical = sk.to_canonical();
    println!("Canonical limbs: {:?}", canonical.0);
    
    let back_to_bytes = sk.to_bytes_le();
    println!("Back to bytes: {}", hex::encode(back_to_bytes));
}
