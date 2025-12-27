use goldilocks_crypto::{ScalarField, schnorr::Point};
use hex;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <private_key_hex>", args[0]);
        std::process::exit(1);
    }

    let private_key_hex = &args[1];
    let private_key_hex = private_key_hex.strip_prefix("0x").unwrap_or(private_key_hex);
    
    let private_key_bytes = hex::decode(private_key_hex).expect("Invalid hex");
    if private_key_bytes.len() != 40 {
        eprintln!("Private key must be 40 bytes (got {})", private_key_bytes.len());
        std::process::exit(1);
    }

    let private_scalar = ScalarField::from_bytes_le(&private_key_bytes).expect("Invalid scalar");
    
    let generator = Point::generator();
    let public_point = generator.mul(&private_scalar);
    let public_fp5 = public_point.encode();
    let public_bytes = public_fp5.to_bytes_le();
    
    println!("Private key: {}", private_key_hex);
    println!("Public key:  {}", hex::encode(&public_bytes));
}
