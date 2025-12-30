use goldilocks_crypto::schnorr::verify_signature;
use hex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════");
    println!("Verifying captured signature from sign_and_export");
    println!("═══════════════════════════════════════════════════════════\n");

    // Values captured from SIG_DEBUG output
    let hash_hex = "9b3c49db81f28aa8ed1a85269f6be71ad49475b4ee63fb8c6bcd1880bf5962b5baddbc6c93ada474";
    let pubkey_hex = "99f3473027655c41eebb21afd06b516b438b42ad70c27ac8208cdb56b60be7d5c9ddfb05e3cf9518";
    let sig_hex = "7240b672b8821da0f99a49e8e353a8ac6517fc33e13aeb8a089eb90086ff83e5b7e6d8f12ce11e02fa403a51a16ff334d246a4327a52d571b0bd5c0862fb1df9779319dd9a9bcca6295294c7fcfb8b56";

    // Convert to bytes
    let hash_bytes = hex::decode(hash_hex)?;
    let pubkey_bytes = hex::decode(pubkey_hex)?;
    let sig_bytes = hex::decode(sig_hex)?;

    println!("Hash:      {}", hash_hex);
    println!("PubKey:    {}", pubkey_hex);
    println!("Signature: {}", sig_hex);
    println!("Signature Length: {} bytes\n", sig_bytes.len());

    // Verify
    match verify_signature(&sig_bytes, hash_bytes.as_slice(), pubkey_bytes.as_slice()) {
        Ok(is_valid) => {
            if is_valid {
                println!("✅ SIGNATURE IS VALID!");
                println!("   Client signing is cryptographically correct.");
                println!("   If server rejects with 21120, it's a field mismatch issue.");
            } else {
                println!("❌ SIGNATURE IS INVALID!");
                println!("   This indicates a problem with our signing algorithm.");
            }
        }
        Err(e) => {
            println!("❌ VERIFICATION ERROR: {}", e);
        }
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("Transaction Details (from SIG_DEBUG elements)");
    println!("═══════════════════════════════════════════════════════════");
    println!("Chain ID:        304 (mainnet)");
    println!("TX Type:         14 (CREATE_ORDER)");
    println!("Nonce:           1217");
    println!("ExpiredAt:       1767030573539");
    println!("AccountIndex:    361816");
    println!("ApiKeyIndex:     6");
    println!("MarketIndex:     0");
    println!("ClientOrderIdx:  99999");
    println!("BaseAmount:      1000");
    println!("Price:           350000");
    println!("IsAsk:           0");
    println!("Type:            1 (MarketOrder)");
    println!("TimeInForce:     0");
    println!("ReduceOnly:      0");
    println!("TriggerPrice:    0");
    println!("OrderExpiry:     0\n");

    Ok(())
}
