//! Demonstrates `SignerClient` — the lightweight signing-only client.
//!
//! Auth token generation requires only the private key and account metadata;
//! no HTTP connection is needed. Using `SignerClient` instead of `LighterClient`
//! avoids constructing an HTTP connection pool for a purely local operation.
use api_client::SignerClient;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("CREATE AUTH TOKEN (SignerClient)");
    println!("{}", "=".repeat(80));
    println!();

    dotenv::dotenv().ok();

    let account_index: i64 = env::var("ACCOUNT_INDEX")?.parse()?;
    let api_key_index: u8 = env::var("API_KEY_INDEX")?.parse()?;
    let api_key = env::var("API_PRIVATE_KEY")?;

    println!("Configuration:");
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}", api_key_index);
    println!();

    // SignerClient: signing only — no HTTP connection pool created.
    let signer = SignerClient::new(&api_key, account_index, api_key_index)?;

    // 7-hour token (default for interactive sessions)
    let default_expiry_seconds = 7 * 60 * 60;
    let token = signer.create_auth_token(default_expiry_seconds)?;

    println!("Auth token (7 h):");
    println!("{}", token);
    println!();
    println!("Token format: deadline:account_index:api_key_index:signature");
    println!(
        "Expiry: {} seconds ({} hours)",
        default_expiry_seconds,
        default_expiry_seconds / 3600
    );

    // 10-minute token (short-lived, e.g. for WebSocket auth)
    println!();
    let short_expiry_seconds = 10 * 60;
    let short_token = signer.create_auth_token(short_expiry_seconds)?;
    println!("Short-lived token (10 min):");
    println!(
        "  {}",
        short_token.chars().take(60).collect::<String>() + "..."
    );
    println!(
        "  Expiry: {} seconds ({} minutes)",
        short_expiry_seconds,
        short_expiry_seconds / 60
    );

    Ok(())
}
