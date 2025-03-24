// Common utilities for domain adapters
use std::fmt;
use std::collections::HashMap;
use serde_json::Value;

use crate::tel::types::{Effect, DomainId, AssetId, Address, Amount, ResourceId};
use crate::tel::error::{TelError, TelResult};
use super::traits::CompilerContext;

/// Error encountered during effect validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Type of validation error
    pub error_type: ValidationErrorType,
    /// Description of the error
    pub message: String,
    /// Path to the part of the effect that failed validation
    pub path: Option<String>,
    /// Additional context
    pub context: HashMap<String, Value>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{} at {}: {}", self.error_type, path, self.message)
        } else {
            write!(f, "{}: {}", self.error_type, self.message)
        }
    }
}

/// Type of validation error
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    /// Invalid domain
    InvalidDomain,
    /// Invalid asset
    InvalidAsset,
    /// Invalid amount
    InvalidAmount,
    /// Invalid address
    InvalidAddress,
    /// Invalid resource
    InvalidResource,
    /// Unsupported effect
    UnsupportedEffect,
    /// Missing field
    MissingField,
    /// Invalid format
    InvalidFormat,
    /// Invalid combination
    InvalidCombination,
    /// Authorization error
    AuthorizationError,
    /// Generic error
    Other,
}

impl fmt::Display for ValidationErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationErrorType::InvalidDomain => write!(f, "Invalid domain"),
            ValidationErrorType::InvalidAsset => write!(f, "Invalid asset"),
            ValidationErrorType::InvalidAmount => write!(f, "Invalid amount"),
            ValidationErrorType::InvalidAddress => write!(f, "Invalid address"),
            ValidationErrorType::InvalidResource => write!(f, "Invalid resource"),
            ValidationErrorType::UnsupportedEffect => write!(f, "Unsupported effect"),
            ValidationErrorType::MissingField => write!(f, "Missing field"),
            ValidationErrorType::InvalidFormat => write!(f, "Invalid format"),
            ValidationErrorType::InvalidCombination => write!(f, "Invalid combination"),
            ValidationErrorType::AuthorizationError => write!(f, "Authorization error"),
            ValidationErrorType::Other => write!(f, "Other validation error"),
        }
    }
}

/// Result of validating an effect
pub type ValidationResult = Result<(), ValidationError>;

/// Helper to convert ValidationError to TelError
impl From<ValidationError> for TelError {
    fn from(err: ValidationError) -> Self {
        TelError::ValidationError(err.to_string())
    }
}

/// Common validation functions for domain adapters
#[derive(Debug)]
pub struct CommonValidators {
    /// Mapping of supported domains to their configurations
    domain_configs: HashMap<DomainId, DomainConfig>,
}

/// Configuration for a domain
#[derive(Debug, Clone)]
pub struct DomainConfig {
    /// Domain identifier
    pub domain_id: DomainId,
    /// Supported assets
    pub supported_assets: HashMap<AssetId, AssetConfig>,
    /// Address format configuration
    pub address_format: AddressFormat,
    /// Maximum transaction size in bytes
    pub max_tx_size: usize,
    /// Maximum amount for transactions
    pub max_amount: Option<Amount>,
    /// Additional domain-specific parameters
    pub parameters: HashMap<String, Value>,
}

/// Configuration for an asset
#[derive(Debug, Clone)]
pub struct AssetConfig {
    /// Asset identifier
    pub asset_id: AssetId,
    /// Asset name
    pub name: String,
    /// Asset symbol
    pub symbol: String,
    /// Number of decimals
    pub decimals: u8,
    /// Contract address (if applicable)
    pub contract_address: Option<Address>,
    /// Minimum transferable amount
    pub min_amount: Option<Amount>,
    /// Maximum transferable amount
    pub max_amount: Option<Amount>,
}

/// Address format configuration
#[derive(Debug, Clone, PartialEq)]
pub enum AddressFormat {
    /// Ethereum-style addresses (20 bytes)
    Ethereum,
    /// Solana-style addresses (32 bytes)
    Solana,
    /// Bitcoin-style addresses
    Bitcoin,
    /// Generic format with custom validation
    Custom {
        /// Minimum length
        min_length: usize,
        /// Maximum length
        max_length: usize,
        /// Required prefix
        prefix: Option<Vec<u8>>,
    },
}

impl CommonValidators {
    /// Create a new CommonValidators instance
    pub fn new() -> Self {
        Self {
            domain_configs: HashMap::new(),
        }
    }
    
    /// Register a domain configuration
    pub fn register_domain(&mut self, config: DomainConfig) {
        self.domain_configs.insert(config.domain_id.clone(), config);
    }
    
    /// Check if a domain is supported
    pub fn is_domain_supported(&self, domain_id: &DomainId) -> bool {
        self.domain_configs.contains_key(domain_id)
    }
    
    /// Get a domain configuration
    pub fn get_domain_config(&self, domain_id: &DomainId) -> Option<&DomainConfig> {
        self.domain_configs.get(domain_id)
    }
    
    /// Validate a domain
    pub fn validate_domain(&self, domain_id: &DomainId) -> ValidationResult {
        if self.is_domain_supported(domain_id) {
            Ok(())
        } else {
            Err(ValidationError {
                error_type: ValidationErrorType::InvalidDomain,
                message: format!("Domain '{}' is not supported", domain_id),
                path: None,
                context: HashMap::new(),
            })
        }
    }
    
    /// Validate an asset for a specific domain
    pub fn validate_asset(&self, domain_id: &DomainId, asset_id: &AssetId) -> ValidationResult {
        let domain_config = self.get_domain_config(domain_id).ok_or_else(|| {
            ValidationError {
                error_type: ValidationErrorType::InvalidDomain,
                message: format!("Domain '{}' is not supported", domain_id),
                path: None,
                context: HashMap::new(),
            }
        })?;
        
        if domain_config.supported_assets.contains_key(asset_id) {
            Ok(())
        } else {
            Err(ValidationError {
                error_type: ValidationErrorType::InvalidAsset,
                message: format!("Asset '{}' is not supported for domain '{}'", asset_id, domain_id),
                path: None,
                context: HashMap::new(),
            })
        }
    }
    
    /// Validate an amount for a specific asset and domain
    pub fn validate_amount(
        &self, 
        domain_id: &DomainId, 
        asset_id: &AssetId, 
        amount: Amount
    ) -> ValidationResult {
        let domain_config = self.get_domain_config(domain_id).ok_or_else(|| {
            ValidationError {
                error_type: ValidationErrorType::InvalidDomain,
                message: format!("Domain '{}' is not supported", domain_id),
                path: None,
                context: HashMap::new(),
            }
        })?;
        
        let asset_config = domain_config.supported_assets.get(asset_id).ok_or_else(|| {
            ValidationError {
                error_type: ValidationErrorType::InvalidAsset,
                message: format!("Asset '{}' is not supported for domain '{}'", asset_id, domain_id),
                path: None,
                context: HashMap::new(),
            }
        })?;
        
        // Check minimum amount
        if let Some(min_amount) = asset_config.min_amount {
            if amount < min_amount {
                return Err(ValidationError {
                    error_type: ValidationErrorType::InvalidAmount,
                    message: format!(
                        "Amount {} is less than minimum allowed ({}) for asset '{}'", 
                        amount, min_amount, asset_id
                    ),
                    path: None,
                    context: HashMap::new(),
                });
            }
        }
        
        // Check maximum amount
        if let Some(max_amount) = asset_config.max_amount {
            if amount > max_amount {
                return Err(ValidationError {
                    error_type: ValidationErrorType::InvalidAmount,
                    message: format!(
                        "Amount {} exceeds maximum allowed ({}) for asset '{}'", 
                        amount, max_amount, asset_id
                    ),
                    path: None,
                    context: HashMap::new(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate an address for a specific domain
    pub fn validate_address(&self, domain_id: &DomainId, address: &Address) -> ValidationResult {
        let domain_config = self.get_domain_config(domain_id).ok_or_else(|| {
            ValidationError {
                error_type: ValidationErrorType::InvalidDomain,
                message: format!("Domain '{}' is not supported", domain_id),
                path: None,
                context: HashMap::new(),
            }
        })?;
        
        match &domain_config.address_format {
            AddressFormat::Ethereum => {
                if address.len() != 20 {
                    return Err(ValidationError {
                        error_type: ValidationErrorType::InvalidAddress,
                        message: format!(
                            "Ethereum address must be 20 bytes, got {} bytes", 
                            address.len()
                        ),
                        path: None,
                        context: HashMap::new(),
                    });
                }
            },
            AddressFormat::Solana => {
                if address.len() != 32 {
                    return Err(ValidationError {
                        error_type: ValidationErrorType::InvalidAddress,
                        message: format!(
                            "Solana address must be 32 bytes, got {} bytes", 
                            address.len()
                        ),
                        path: None,
                        context: HashMap::new(),
                    });
                }
            },
            AddressFormat::Bitcoin => {
                // Simple check for now, would need more specific validation
                if address.len() < 26 || address.len() > 35 {
                    return Err(ValidationError {
                        error_type: ValidationErrorType::InvalidAddress,
                        message: format!(
                            "Bitcoin address should be 26-35 bytes, got {} bytes", 
                            address.len()
                        ),
                        path: None,
                        context: HashMap::new(),
                    });
                }
            },
            AddressFormat::Custom { min_length, max_length, prefix } => {
                if address.len() < *min_length || address.len() > *max_length {
                    return Err(ValidationError {
                        error_type: ValidationErrorType::InvalidAddress,
                        message: format!(
                            "Address length must be between {} and {}, got {} bytes", 
                            min_length, max_length, address.len()
                        ),
                        path: None,
                        context: HashMap::new(),
                    });
                }
                
                if let Some(required_prefix) = prefix {
                    if address.len() < required_prefix.len() || 
                       &address[0..required_prefix.len()] != required_prefix.as_slice() {
                        return Err(ValidationError {
                            error_type: ValidationErrorType::InvalidAddress,
                            message: format!("Address does not have required prefix"),
                            path: None,
                            context: HashMap::new(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate a deposit effect
    pub fn validate_deposit(&self, effect: &Effect) -> ValidationResult {
        if let Effect::Deposit { domain, asset, amount } = effect {
            self.validate_domain(domain)?;
            self.validate_asset(domain, asset)?;
            self.validate_amount(domain, asset, *amount)?;
            Ok(())
        } else {
            Err(ValidationError {
                error_type: ValidationErrorType::UnsupportedEffect,
                message: "Expected Deposit effect".to_string(),
                path: None,
                context: HashMap::new(),
            })
        }
    }
    
    /// Validate a withdraw effect
    pub fn validate_withdraw(&self, effect: &Effect) -> ValidationResult {
        if let Effect::Withdraw { domain, asset, amount, address } = effect {
            self.validate_domain(domain)?;
            self.validate_asset(domain, asset)?;
            self.validate_amount(domain, asset, *amount)?;
            self.validate_address(domain, address)?;
            Ok(())
        } else {
            Err(ValidationError {
                error_type: ValidationErrorType::UnsupportedEffect,
                message: "Expected Withdraw effect".to_string(),
                path: None,
                context: HashMap::new(),
            })
        }
    }
    
    /// Validate a transfer effect
    pub fn validate_transfer(&self, effect: &Effect, domain_id: &DomainId) -> ValidationResult {
        if let Effect::Transfer { from, to, asset, amount } = effect {
            self.validate_asset(domain_id, asset)?;
            self.validate_amount(domain_id, asset, *amount)?;
            self.validate_address(domain_id, from)?;
            self.validate_address(domain_id, to)?;
            Ok(())
        } else {
            Err(ValidationError {
                error_type: ValidationErrorType::UnsupportedEffect,
                message: "Expected Transfer effect".to_string(),
                path: None,
                context: HashMap::new(),
            })
        }
    }
} 
