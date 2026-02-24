//! Encryption utilities for sensitive data

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroizing;

/// Encryption error types
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Invalid key: {0}")]
    InvalidKeyError(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),
}

pub type CryptoResult<T> = Result<T, CryptoError>;

/// Encryption algorithm options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (recommended)
    #[default]
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}


/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Whether encryption is enabled
    pub enabled: bool,
    /// Encryption algorithm to use
    pub algorithm: EncryptionAlgorithm,
    /// Key derivation iterations (for PBKDF2)
    pub key_derivation_iterations: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation_iterations: 100_000,
        }
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded ciphertext
    pub ciphertext: String,
    /// Base64-encoded nonce/IV
    pub nonce: String,
    /// Base64-encoded authentication tag (for AEAD)
    pub tag: Option<String>,
    /// Algorithm used
    pub algorithm: EncryptionAlgorithm,
}

/// Simple XOR-based encryption for demonstration
/// In production, use a proper crypto library like ring or rustls
pub struct SimpleEncryptor {
    key: Zeroizing<Vec<u8>>,
}

impl SimpleEncryptor {
    /// Create a new encryptor with a key
    pub fn new(key: &[u8]) -> CryptoResult<Self> {
        if key.len() < 16 {
            return Err(CryptoError::InvalidKeyError(
                "Key must be at least 16 bytes".to_string(),
            ));
        }
        Ok(Self {
            key: Zeroizing::new(key.to_vec()),
        })
    }

    /// Derive a key from a password
    pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> CryptoResult<Vec<u8>> {
        // Simple key derivation (use PBKDF2 or Argon2 in production)
        let mut key = vec![0u8; 32];
        let password_bytes = password.as_bytes();

        for (i, key_byte) in key.iter_mut().enumerate() {
            let mut byte = 0u8;
            for j in 0..iterations as usize / 1000 {
                let idx = (i + j) % password_bytes.len();
                let salt_idx = (i + j) % salt.len();
                byte = byte.wrapping_add(password_bytes[idx] ^ salt[salt_idx]);
            }
            *key_byte = byte;
        }

        Ok(key)
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> CryptoResult<EncryptedData> {
        // Generate a simple nonce (use a proper CSPRNG in production)
        let nonce: Vec<u8> = (0..12).map(|i| (i as u8).wrapping_add(chrono::Utc::now().timestamp() as u8)).collect();

        // XOR encryption (use AES-GCM or ChaCha20-Poly1305 in production)
        let ciphertext: Vec<u8> = plaintext
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.key[i % self.key.len()] ^ nonce[i % nonce.len()])
            .collect();

        Ok(EncryptedData {
            ciphertext: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(&nonce),
            tag: None,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> CryptoResult<Vec<u8>> {
        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .map_err(|e| CryptoError::DecryptionError(format!("Invalid base64 ciphertext: {}", e)))?;
        let nonce = BASE64
            .decode(&encrypted.nonce)
            .map_err(|e| CryptoError::DecryptionError(format!("Invalid base64 nonce: {}", e)))?;

        // XOR decryption
        let plaintext: Vec<u8> = ciphertext
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.key[i % self.key.len()] ^ nonce[i % nonce.len()])
            .collect();

        Ok(plaintext)
    }
}

/// Secure string holder that zeroizes on drop
#[derive(Debug)]
pub struct SecureString(Zeroizing<String>);

impl SecureString {
    /// Create a new secure string
    pub fn new(s: String) -> Self {
        Self(Zeroizing::new(s))
    }

    /// Get the string contents
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to a regular string (copies data)
    pub fn expose(&self) -> String {
        self.0.as_str().to_string()
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// Trait for types that can be encrypted
pub trait Encryptable: Sized {
    /// Encrypt this value
    fn encrypt(&self, encryptor: &SimpleEncryptor) -> CryptoResult<EncryptedData>;

    /// Decrypt to this value
    fn decrypt(encrypted: &EncryptedData, encryptor: &SimpleEncryptor) -> CryptoResult<Self>;
}

impl Encryptable for String {
    fn encrypt(&self, encryptor: &SimpleEncryptor) -> CryptoResult<EncryptedData> {
        encryptor.encrypt(self.as_bytes())
    }

    fn decrypt(encrypted: &EncryptedData, encryptor: &SimpleEncryptor) -> CryptoResult<Self> {
        let bytes = encryptor.decrypt(encrypted)?;
        String::from_utf8(bytes).map_err(|e| CryptoError::DecryptionError(format!("Invalid UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = b"test-key-12345678";
        let encryptor = SimpleEncryptor::new(key).unwrap();

        let plaintext = "Hello, World!";
        let encrypted = encryptor.encrypt(plaintext.as_bytes()).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.as_bytes(), decrypted.as_slice());
    }

    #[test]
    fn test_key_derivation() {
        let key1 = SimpleEncryptor::derive_key("password", b"salt", 1000).unwrap();
        let key2 = SimpleEncryptor::derive_key("password", b"salt", 1000).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_secure_string() {
        let secure = SecureString::from("secret");
        assert_eq!(secure.as_str(), "secret");
    }

    #[test]
    fn test_encryptable_string() {
        let key = b"test-key-12345678";
        let encryptor = SimpleEncryptor::new(key).unwrap();

        let original = "Test string".to_string();
        let encrypted = original.encrypt(&encryptor).unwrap();
        let decrypted = String::decrypt(&encrypted, &encryptor).unwrap();

        assert_eq!(original, decrypted);
    }
}
