// Crypto-specific error types
// These errors are specifically for the causality-crypto crate

use thiserror::Error;
use crate::{CausalityError, ErrorCode, ErrorDomain};

/// Crypto-specific error codes
pub mod codes {
    use crate::ErrorCode;
    
    // Crypto error codes start with 3000
    pub const HASH_ERROR: ErrorCode = ErrorCode(3001);
    pub const SIGNATURE_ERROR: ErrorCode = ErrorCode(3002);
    pub const VERIFICATION_ERROR: ErrorCode = ErrorCode(3003);
    pub const KEY_ERROR: ErrorCode = ErrorCode(3004);
    pub const RANDOM_ERROR: ErrorCode = ErrorCode(3005);
    pub const ENCODING_ERROR: ErrorCode = ErrorCode(3006);
    pub const CONTENT_ID_ERROR: ErrorCode = ErrorCode(3007);
}

/// Crypto-specific error types
#[derive(Error, Debug, Clone)]
pub enum CryptoError {
    /// Hash computation error
    #[error("Hash error: {0}")]
    HashError(String),
    
    /// Signature creation error
    #[error("Signature error: {0}")]
    SignatureError(String),
    
    /// Verification failure
    #[error("Verification error: {0}")]
    VerificationError(String),
    
    /// Key management error
    #[error("Key error: {0}")]
    KeyError(String),
    
    /// Random number generation error
    #[error("Random error: {0}")]
    RandomError(String),
    
    /// Encoding/decoding error
    #[error("Encoding error: {0}")]
    EncodingError(String),
    
    /// ContentId error
    #[error("ContentId error: {0}")]
    ContentIdError(String),
}

impl CausalityError for CryptoError {
    fn code(&self) -> ErrorCode {
        use codes::*;
        match self {
            CryptoError::HashError(_) => HASH_ERROR,
            CryptoError::SignatureError(_) => SIGNATURE_ERROR,
            CryptoError::VerificationError(_) => VERIFICATION_ERROR,
            CryptoError::KeyError(_) => KEY_ERROR,
            CryptoError::RandomError(_) => RANDOM_ERROR,
            CryptoError::EncodingError(_) => ENCODING_ERROR,
            CryptoError::ContentIdError(_) => CONTENT_ID_ERROR,
        }
    }
    
    fn domain(&self) -> ErrorDomain {
        ErrorDomain::Crypto
    }
}

/// Convenient Result type for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Convert from crypto error to boxed error
impl From<CryptoError> for Box<dyn CausalityError> {
    fn from(err: CryptoError) -> Self {
        Box::new(err)
    }
}

// Helper methods for creating crypto errors
impl CryptoError {
    /// Create a new hash error
    pub fn hash_error(message: impl Into<String>) -> Self {
        CryptoError::HashError(message.into())
    }
    
    /// Create a new signature error
    pub fn signature_error(message: impl Into<String>) -> Self {
        CryptoError::SignatureError(message.into())
    }
    
    /// Create a new verification error
    pub fn verification_error(message: impl Into<String>) -> Self {
        CryptoError::VerificationError(message.into())
    }
    
    /// Create a new ContentId error
    pub fn contentid_error(message: impl Into<String>) -> Self {
        CryptoError::ContentIdError(message.into())
    }
} 