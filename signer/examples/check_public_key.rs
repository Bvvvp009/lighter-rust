//! Check the public key derived from the private key

use signer::KeyManager;
use std::env;

fn load_dotenv() {
    if let Ok(current_dir) = env::current_dir() {
        let env_files = [
            current_dir.join(".env"),
            current_dir.join("..").join(".env"),
            current_dir.join("..").join("..").join(".env"),
        ];
        for env_file in env_files.iter() {
            if env_file.exists() {
                if let Ok(content) = std::fs::read_to_string(env_file) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        if let Some((key, value)) = line.split_once('=') {
                            let key = key.trim();
                            let value = value.trim().trim_matches('"').trim_matches('\'');
                            if env::var(key).is_err() {
                                env::set_var(key, value);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    
    let api_private_key = env::var("API_PRIVATE_KEY")
        .map_err(|_| "API_PRIVATE_KEY environment variable is required")?;
    
    let key_manager = KeyManager::from_hex(&api_private_key)?;
    
    let public_key = key_manager.public_key_bytes();
    let private_key = key_manager.private_key_bytes();
    
    println!("=== Key Information ===\n");
    println!("Private Key: {}", hex::encode(&private_key));
    println!("Public Key:  {}\n", hex::encode(&public_key));
    
    println!("This public key should match what's registered on the Lighter server");
    println!("for API_KEY_INDEX: {}", env::var("API_KEY_INDEX").unwrap_or_else(|_| "not set".to_string()));
    
    Ok(())
}
