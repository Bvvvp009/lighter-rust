//! Batch signature verification for improved throughput

use crate::{CryptoError, Result};

/// Verify multiple Schnorr signatures sequentially.
///
/// Verifies n signatures one by one, providing a convenient API for batch operations.
pub fn batch_verify(
    signatures: &[Vec<u8>],
    messages: &[[u8; 40]],
    public_keys: &[[u8; 40]],
) -> Result<bool> {
    let n = signatures.len();
    
    if n == 0 {
        return Ok(true);
    }
    
    if messages.len() != n || public_keys.len() != n {
        return Err(CryptoError::InvalidSignature);
    }
    
    for i in 0..n {
        let is_valid = crate::verify_signature(&signatures[i], &messages[i], &public_keys[i])?;
        if !is_valid {
            return Ok(false);
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point, ScalarField};
    use crate::schnorr::sign_with_nonce;
    
    #[test]
    fn test_batch_verify_valid() {
        let mut sigs = Vec::new();
        let mut msgs = Vec::new();
        let mut pks = Vec::new();
        
        // Use deterministic keys and nonces to avoid randomness issues
        for i in 1..=5 {
            let mut sk_bytes = [0u8; 40];
            sk_bytes[0] = i as u8;
            
            let mut nonce_bytes = [0u8; 40];
            nonce_bytes[0] = (i * 17) as u8; // Deterministic nonce
            
            let sk = ScalarField::from_bytes_le(&sk_bytes).unwrap();
            let pk = Point::generator().mul(&sk).encode().to_bytes_le();
            
            // Create deterministic messages
            let mut msg = [0u8; 40];
            msg[0] = i as u8;
            msg[1] = (i * 2) as u8;
            msg[2] = (i * 3) as u8;
            
            let sig = sign_with_nonce(&sk_bytes, &msg, &nonce_bytes).unwrap();
            
            sigs.push(sig);
            msgs.push(msg);
            pks.push(pk);
        }
        
        assert!(batch_verify(&sigs, &msgs, &pks).unwrap());
    }
}
