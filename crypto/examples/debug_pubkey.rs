use goldilocks_crypto::{ScalarField, schnorr::Point};
use hex;

fn main() {
    let priv_hex = "c5230d52492a608954476c66f3be44559460d101dccec8d4e2e8d2caf4f3b983e77389563df72f51";
    let priv_bytes = hex::decode(priv_hex).unwrap();
    
    let sk = ScalarField::from_bytes_le(&priv_bytes).unwrap();
    let gen = Point::generator();
    let pk_point = gen.mul(&sk);
    let pk_fp5 = pk_point.encode();
    let pk_bytes = pk_fp5.to_bytes_le();
    
    println!("Private key: {}", priv_hex);
    println!("Public key:  {}", hex::encode(&pk_bytes));
    println!("Expected:    a0791a9d534a0b9dbf61ae438b25dae38f35229083a73a75d532e855cfc78cfaf50a3dbc2856a6dc");
}
