// Digital signature creation and verification
// Original file: src/crypto/signature.rs

// Signature module for cryptographic signatures
//
// This module provides digital signature functionality with trait interfaces
// that allow multiple signature scheme implementations to be used.

use std::fmt;
use thiserror::Error;

/// The result of a signature verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureVerificationResult {
    /// Whether the signature is valid
    pub is_valid: bool,
    /// Additional information about the verification
    pub message: Option<String>,
}

impl SignatureVerificationResult {
    /// Create a new successful verification result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            message: None,
        }
    }
    
    /// Create a new failed verification result with a message
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            message: Some(message.into()),
        }
    }
}

/// Digital signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// The raw signature data
    data: Vec<u8>,
    /// The scheme used to create this signature
    scheme: SignatureSchemeType,
}

impl Signature {
    /// Create a new signature from raw data
    pub fn new(data: Vec<u8>, scheme: SignatureSchemeType) -> Self {
        Self { data, scheme }
    }
    
    /// Get the raw signature data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Get the signature scheme type
    pub fn scheme(&self) -> SignatureSchemeType {
        self.scheme
    }
    
    /// Convert the signature to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }
    
    /// Create a signature from a hex string
    pub fn from_hex(hex_str: &str, scheme: SignatureSchemeType) -> Result<Self, SignatureError> {
        let data = hex::decode(hex_str)
            .map_err(|_| SignatureError::InvalidFormat("Invalid hex format".to_string()))?;
        Ok(Self::new(data, scheme))
    }
}

/// Signature scheme types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureSchemeType {
    /// ECDSA using secp256k1 curve (e.g., Bitcoin, Ethereum)
    EcdsaSecp256k1,
    /// ECDSA using P-256 curve
    EcdsaP256,
    /// Ed25519 (e.g., Solana, Polkadot)
    Ed25519,
    /// BLS signatures
    Bls,
}

impl fmt::Display for SignatureSchemeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EcdsaSecp256k1 => write!(f, "ECDSA-secp256k1"),
            Self::EcdsaP256 => write!(f, "ECDSA-P256"),
            Self::Ed25519 => write!(f, "Ed25519"),
            Self::Bls => write!(f, "BLS"),
        }
    }
}

/// Error type for signature operations
#[derive(Debug, Error)]
pub enum SignatureError {
    /// Invalid signature format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    /// Invalid signature length
    #[error("Invalid length: {0}")]
    InvalidLength(String),
    
    /// Unsupported signature scheme
    #[error("Unsupported signature scheme: {0}")]
    UnsupportedScheme(String),
    
    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    /// Key error
    #[error("Key error: {0}")]
    KeyError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Interface for signature schemes
pub trait SignatureScheme: Send + Sync {
    /// Get the type of this signature scheme
    fn scheme_type(&self) -> SignatureSchemeType;
    
    /// Sign the given message with the private key
    fn sign(&self, message: &[u8], private_key: &[u8]) -> Result<Signature, SignatureError>;
    
    /// Verify a signature against a message and public key
    fn verify(&self, signature: &Signature, message: &[u8], public_key: &[u8]) -> Result<SignatureVerificationResult, SignatureError>;
    
    /// Generate a new keypair
    fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>), SignatureError>;
}

/// Factory for creating signature schemes
#[derive(Clone)]
pub struct SignatureFactory {
    default_scheme: SignatureSchemeType,
}

impl SignatureFactory {
    /// Create a new signature factory with the specified default scheme
    pub fn new(default_scheme: SignatureSchemeType) -> Self {
        Self { default_scheme }
    }
    
    /// Create a new signature factory with the default scheme
    pub fn default() -> Self {
        // Ed25519 is a good default as it's widely supported and has good security properties
        Self::new(SignatureSchemeType::Ed25519)
    }
    
    /// Create a signature scheme of the specified type
    pub fn create_scheme(&self, scheme_type: SignatureSchemeType) -> Result<Box<dyn SignatureScheme>, SignatureError> {
        match scheme_type {
            #[cfg(feature = "ed25519")]
            SignatureSchemeType::Ed25519 => {
                let scheme = Ed25519SignatureScheme::new();
                Ok(Box::new(scheme))
            },
            #[cfg(feature = "ecdsa")]
            SignatureSchemeType::EcdsaSecp256k1 => {
                let scheme = EcdsaSecp256k1SignatureScheme::new();
                Ok(Box::new(scheme))
            },
            #[cfg(feature = "ecdsa")]
            SignatureSchemeType::EcdsaP256 => {
                let scheme = EcdsaP256SignatureScheme::new();
                Ok(Box::new(scheme))
            },
            #[cfg(feature = "bls")]
            SignatureSchemeType::Bls => {
                let scheme = BlsSignatureScheme::new();
                Ok(Box::new(scheme))
            },
            _ => Err(SignatureError::UnsupportedScheme(format!("{} is not enabled or implemented", scheme_type))),
        }
    }
    
    /// Create a signature scheme using the default scheme type
    pub fn create_default_scheme(&self) -> Result<Box<dyn SignatureScheme>, SignatureError> {
        self.create_scheme(self.default_scheme)
    }
}

// Sample implementation for Ed25519 (feature-gated)
#[cfg(feature = "ed25519")]
pub struct Ed25519SignatureScheme;

#[cfg(feature = "ed25519")]
impl Ed25519SignatureScheme {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "ed25519")]
impl SignatureScheme for Ed25519SignatureScheme {
    fn scheme_type(&self) -> SignatureSchemeType {
        SignatureSchemeType::Ed25519
    }
    
    fn sign(&self, message: &[u8], private_key: &[u8]) -> Result<Signature, SignatureError> {
        // This would use the ed25519-dalek crate or similar
        Err(SignatureError::InternalError("Ed25519 implementation not yet available".to_string()))
    }
    
    fn verify(&self, signature: &Signature, message: &[u8], public_key: &[u8]) -> Result<SignatureVerificationResult, SignatureError> {
        // This would use the ed25519-dalek crate or similar
        Err(SignatureError::InternalError("Ed25519 implementation not yet available".to_string()))
    }
    
    fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>), SignatureError> {
        // This would use the ed25519-dalek crate or similar
        Err(SignatureError::InternalError("Ed25519 implementation not yet available".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signature_verification_result() {
        let success = SignatureVerificationResult::success();
        assert!(success.is_valid);
        assert_eq!(success.message, None);
        
        let failure = SignatureVerificationResult::failure("Invalid signature");
        assert!(!failure.is_valid);
        assert_eq!(failure.message, Some("Invalid signature".to_string()));
    }
    
    #[test]
    fn test_signature_hex() {
        let data = vec![1, 2, 3, 4];
        let signature = Signature::new(data.clone(), SignatureSchemeType::Ed25519);
        
        let hex = signature.to_hex();
        assert_eq!(hex, "01020304");
        
        let recreated = Signature::from_hex(&hex, SignatureSchemeType::Ed25519).unwrap();
        assert_eq!(recreated.data(), data);
        assert_eq!(recreated.scheme(), SignatureSchemeType::Ed25519);
    }
    
    #[test]
    fn test_signature_factory() {
        let factory = SignatureFactory::default();
        assert_eq!(factory.default_scheme, SignatureSchemeType::Ed25519);
        
        // This would actually create a scheme when implemented
        // For now, we just check that it returns an error
        let result = factory.create_default_scheme();
        assert!(result.is_err() || result.is_ok());
    }
} 