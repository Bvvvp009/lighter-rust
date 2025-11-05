use api_client::LighterClient;
use signer::KeyManager;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "═".repeat(80));
    println!("🔑 SETUP API KEY EXAMPLE (System Setup)");
    println!("{}", "═".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL")?;
    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("📋 Configuration:");
    println!("  Base URL: {}", base_url);
    println!("  Account Index: {}", account_index);
    println!("  Current API Key Index: {}", api_key_index);
    println!();

    // Generate a new API key pair
    println!("🔑 Generating new API key pair...");
    let new_key_manager = KeyManager::generate();
    let new_private_key = new_key_manager.private_key_bytes();
    let new_public_key = new_key_manager.public_key_bytes();

    println!("✅ New API key generated!");
    println!("  Private Key (hex): {}", hex::encode(&new_private_key));
    println!("  Public Key (hex): {}", hex::encode(&new_public_key));
    println!();

    // Determine the new API key index (typically current + 1)
    let new_api_key_index = api_key_index + 1;
    println!("📝 Setting up new API key at index {}...", new_api_key_index);
    println!();

    // Note: change_api_key requires the new API key's private key to be used
    // for signing the transaction. For a complete setup, you would:
    // 1. Use your ETH private key to sign an L1 message
    // 2. Use the new API key to sign the change_pub_key transaction
    // 3. Submit both signatures
    //
    // This example shows the structure, but you need to handle L1 signature separately
    println!("⚠️  NOTE: change_api_key requires:");
    println!("  1. L1 signature (signed with ETH private key)");
    println!("  2. L2 signature (signed with new API private key)");
    println!("  3. The transaction must be signed with the NEW API key, not the current one");
    println!();
    println!("  For complete setup, register the public key on the exchange.");
    println!("  You'll need to sign an L1 message with your Ethereum wallet.");
    println!();

    // Example: Show how to create the client with new key
    println!("📝 Example: Using new API key for future transactions:");
    println!("  Update your .env file:");
    println!("    API_PRIVATE_KEY={}", hex::encode(&new_private_key));
    println!("    API_KEY_INDEX={}", new_api_key_index);
    println!();

    // If you have the new key manager ready, you can change the API key
    // This is a placeholder - you'll need to implement the full flow with L1 signature
    println!("💡 To complete the setup:");
    println!("  1. Generate L1 signature using your ETH private key");
    println!("  2. Use the new API key manager to sign the change_pub_key transaction");
    println!("  3. Submit the transaction with both signatures");

    Ok(())
}
