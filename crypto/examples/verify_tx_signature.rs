use goldilocks_crypto::{sign_hashed_message, schnorr::verify_signature, ScalarField};
use hex;

fn main() {
    // Inputs from debug run
    let priv_hex = "616fdf2e72ef775c8585c371d60a2b528c05e8fb370853a48131a5db116102979284634536b56654";
    let msg_hex = "4467eda35526b1a850c67592d798d1615ac896948985b500ceca4df3dc687a42feb9e12c3b9ae573";

    let priv_bytes = hex::decode(priv_hex).expect("invalid priv hex");
    let mut msg_bytes = [0u8; 40];
    msg_bytes.copy_from_slice(&hex::decode(msg_hex).expect("invalid msg hex"));

    // Deterministic nonce for reproducibility
    let nonce = ScalarField::from_u64(1).to_bytes_le();

    let sig = sign_hashed_message(&priv_bytes, &msg_bytes, &nonce).expect("sign failed");

    let mut pub_bytes = [0u8; 40];
    let priv_scalar = ScalarField::from_bytes_le(&priv_bytes).unwrap();
    let pub_fp5 = goldilocks_crypto::schnorr::Point::generator().mul(&priv_scalar).encode();
    pub_bytes.copy_from_slice(&pub_fp5.to_bytes_le());

    println!("Message: {}", msg_hex);
    println!("Public key: {}", hex::encode(pub_bytes));
    println!("Signature: {}", hex::encode(&sig));

    let ok = verify_signature(&sig, &msg_bytes, &pub_bytes).expect("verify error");
    println!("Verify result: {}", ok);
}
