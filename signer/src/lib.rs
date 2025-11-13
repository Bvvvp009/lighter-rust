use goldilocks_crypto::{schnorr::{sign_with_nonce}, ScalarField, Goldilocks};
use thiserror::Error;
use hex;

#[derive(Error, Debug)]
pub enum SignerError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] goldilocks_crypto::CryptoError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, SignerError>;

pub struct KeyManager {
    private_key: ScalarField,
}

impl KeyManager {
    pub fn new(private_key_bytes: &[u8]) -> Result<Self> {
        if private_key_bytes.len() != 40 {
            return Err(SignerError::Crypto(goldilocks_crypto::CryptoError::InvalidPrivateKeyLength(private_key_bytes.len())));
        }
        // Use all 40 bytes for 5-limb scalar
        let private_key = ScalarField::from_bytes_le(private_key_bytes)
            .map_err(|_| SignerError::Crypto(goldilocks_crypto::CryptoError::InvalidPrivateKeyLength(private_key_bytes.len())))?;
        Ok(Self { private_key })
    }
    
        pub fn from_hex(hex_str: &str) -> Result<Self> {
        let hex_str = if hex_str.starts_with("0x") {
            &hex_str[2..]
        } else {
            hex_str
        };
        
        let bytes = hex::decode(hex_str)?;
        Self::new(&bytes)
    }

    /// Generate a new random key pair
    pub fn generate() -> Self {
        let random_scalar = ScalarField::sample_crypto();
        Self { private_key: random_scalar }
    }

    /// Get the public key as bytes (40 bytes)
    pub fn public_key_bytes(&self) -> [u8; 40] {
        use goldilocks_crypto::schnorr::Point;
        // Public key = generator * private_key, encoded as Fp5Element
        let generator = Point::generator();
        let public_point = generator.mul(&self.private_key);
        let public_fp5 = public_point.encode();
        public_fp5.to_bytes_le()
    }

    /// Get the private key as bytes (40 bytes)
    pub fn private_key_bytes(&self) -> [u8; 40] {
        self.private_key.to_bytes_le()
    }

    pub fn sign(&self, message: &[u8; 40]) -> Result<[u8; 80]> {
        let nonce_scalar = ScalarField::sample_crypto();
        let nonce_bytes = nonce_scalar.to_bytes_le();
        self.sign_with_fixed_nonce(message, &nonce_bytes)
    }
    
    fn sign_with_fixed_nonce(&self, message: &[u8; 40], nonce_bytes: &[u8]) -> Result<[u8; 80]> {
        let pk_bytes = self.private_key.to_bytes_le();
        let signature = sign_with_nonce(&pk_bytes, message, nonce_bytes)?;
        
        let mut result = [0u8; 80];
        result.copy_from_slice(&signature);
        Ok(result)
    }
    
    pub fn create_auth_token(
        &self,
        deadline: i64,
        account_index: i64,
        api_key_index: u8,
    ) -> Result<String> {
        // Match Go: ConstructAuthToken format "deadline:account_index:api_key_index"
        let auth_data = format!("{}:{}:{}", deadline, account_index, api_key_index);
        
        // Convert message bytes to Goldilocks elements
        let auth_bytes = auth_data.as_bytes();
        
        // CRITICAL: Pad each 8-byte chunk individually
        // Calculate missing bytes: (8 - len(in) % 8) % 8, then pad the last chunk with zeros
        let missing = (8 - auth_bytes.len() % 8) % 8;
        
        let mut elements = Vec::new();
        
        // Process in chunks of 8 bytes (one Goldilocks element per 8 bytes)
        let mut i = 0;
        while i < auth_bytes.len() {
            let next_start = (i + 8).min(auth_bytes.len());
            let chunk = &auth_bytes[i..next_start];
            
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            
            // Pad only the last chunk if needed
            if chunk.len() < 8 && missing > 0 {
                bytes[chunk.len()..].fill(0);
            }
            
            // Read as little-endian u64, then convert to Goldilocks
            let val = u64::from_le_bytes(bytes);
            elements.push(Goldilocks::from_canonical_u64(val));
            
            i = next_start;
        }
        
        // Hash the elements using Poseidon2 (matching Go's HashToQuinticExtension)
        use poseidon_hash::hash_to_quintic_extension;
        let hash_fp5 = hash_to_quintic_extension(&elements);
        
        // Convert Fp5Element to 40-byte array for signing
        let message_bytes = hash_fp5.to_bytes_le();
        
        // Sign the hash
        let signature = self.sign(&message_bytes)?;
        let signature_hex = hex::encode(&signature);
        
        Ok(format!("{}:{}", auth_data, signature_hex))
    }
}
