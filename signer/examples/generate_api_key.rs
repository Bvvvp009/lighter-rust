//! Generate a new API key pair
//!
//! This example generates a new random API key pair (private/public key).
//!
//! Usage:
//!   cargo run --example generate_api_key

use signer::KeyManager;
use hex;

fn main() {
    println!("🔑 Generating new API key pair...\n");
    
    // Generate a new random key pair
    let key_manager = KeyManager::generate();
    
    // Get private and public keys
    let private_key = key_manager.private_key_bytes();
    let public_key = key_manager.public_key_bytes();
    
    println!("✅ Key pair generated successfully!\n");
    println!("Private Key (hex):");
    println!("  {}", hex::encode(&private_key));
    println!();
    println!("Public Key (hex):");
    println!("  {}", hex::encode(&public_key));
    println!();
    println!("⚠️  WARNING: Keep your private key secure! Never share it.");
}













