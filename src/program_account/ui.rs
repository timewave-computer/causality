use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::address::Address;
use crate::resource::{
    RegisterId, Register, ResourceCapability, Right, CapabilityId,
    ResourceAPI, ResourceApiResult,
};
use crate::program_account::{
    ProgramAccount, AssetProgramAccount, UtilityProgramAccount, DomainBridgeProgramAccount,
    ProgramAccountResource, AvailableEffect, EffectParameter,
    AccountType, TransactionRecord, TransactionStatus,
};

/// A serializable view of a program account for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAccountView {
    /// Unique identifier for the account
    pub id: String,
    /// Address of the account owner
    pub owner: String,
    /// Human-readable name
    pub name: String,
    /// Type of account (Asset, Utility, DomainBridge, etc.)
    pub account_type: String,
    /// Domains this account can interact with
    pub domains: Vec<String>,
    /// Asset balances for this account
    pub balances: Vec<BalanceView>,
    /// Resources available to this account
    pub resources: Vec<ResourceView>,
    /// Capabilities granted to this account
    pub capabilities: Vec<CapabilityView>,
    /// Effects that can be invoked
    pub effects: Vec<EffectView>,
    /// Recent transactions
    pub transactions: Vec<TransactionView>,
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}

/// A serializable view of a resource for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceView {
    /// Unique identifier for the resource
    pub id: String,
    /// Type of resource
    pub resource_type: String,
    /// Domain the resource belongs to, if any
    pub domain: Option<String>,
    /// Preview of the resource content
    pub preview: Option<String>,
    /// Resource state (e.g., "active", "locked")
    pub state: String,
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}

/// A serializable view of an asset balance for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceView {
    /// Unique identifier for the asset
    pub asset_id: String,
    /// Human-readable name of the asset
    pub asset_name: String,
    /// Amount held
    pub amount: String,
    /// Asset type (token, NFT, etc.)
    pub asset_type: String,
    /// Symbol for the asset (e.g., "ETH", "USD")
    pub symbol: Option<String>,
    /// Number of decimal places for display
    pub decimals: Option<u8>,
    /// URL to asset icon, if available
    pub icon_url: Option<String>,
}

/// A serializable view of a capability for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityView {
    /// Unique identifier for the capability
    pub id: String,
    /// Action this capability grants (e.g., "read", "write", "transfer")
    pub action: String,
    /// Target resource this capability applies to
    pub target: Option<String>,
    /// Restrictions on using this capability
    pub restrictions: Option<HashMap<String, String>>,
    /// When this capability expires, if applicable
    pub expires_at: Option<u64>,
}

/// A serializable view of an available effect for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectView {
    /// Unique identifier for the effect
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what the effect does
    pub description: String,
    /// Domain this effect operates on, if applicable
    pub domain: Option<String>,
    /// Parameters required to invoke this effect
    pub parameters: Vec<ParameterView>,
    /// Whether this effect is currently available
    pub available: bool,
    /// Any requirements to use this effect (e.g., minimum balance)
    pub requirements: Vec<String>,
}

/// A serializable view of an effect parameter for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterView {
    /// Parameter name
    pub name: String,
    /// Parameter type (string, number, address, etc.)
    pub param_type: String,
    /// Description of the parameter
    pub description: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value, if any
    pub default_value: Option<String>,
    /// Parameter constraints (min, max, pattern, etc.)
    pub constraints: Option<HashMap<String, String>>,
}

/// A serializable view of a transaction for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionView {
    /// Unique identifier for the transaction
    pub id: String,
    /// Type of transaction
    pub transaction_type: String,
    /// Unix timestamp
    pub timestamp: u64,
    /// Status (pending, confirmed, failed, etc.)
    pub status: String,
    /// Resources involved in this transaction
    pub resources: Vec<String>,
    /// Effects invoked in this transaction
    pub effects: Vec<String>,
    /// Domains involved
    pub domains: Vec<String>,
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
    /// Hash of the transaction, if available
    pub hash: Option<String>,
    /// Block number, if confirmed
    pub block_number: Option<u64>,
    /// Gas/fee information
    pub fee: Option<FeeView>,
}

/// A serializable view of transaction fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeView {
    /// Amount of fee
    pub amount: String,
    /// Currency of fee
    pub currency: String,
    /// Gas price if applicable
    pub gas_price: Option<String>,
    /// Gas used if applicable
    pub gas_used: Option<u64>,
}

/// A trait for transforming internal models to UI views
pub trait ViewTransformer<T, V> {
    /// Transforms an internal model to a UI view
    fn to_view(&self, model: &T) -> V;
    
    /// Transforms multiple internal models to UI views
    fn to_views(&self, models: &[T]) -> Vec<V> {
        models.iter().map(|m| self.to_view(m)).collect()
    }
}

/// Implementation of ViewTransformer for ProgramAccount to ProgramAccountView
pub struct ProgramAccountViewTransformer;

impl ViewTransformer<dyn ProgramAccount, ProgramAccountView> for ProgramAccountViewTransformer {
    fn to_view(&self, account: &dyn ProgramAccount) -> ProgramAccountView {
        ProgramAccountView {
            id: account.id().to_string(),
            owner: account.owner().to_string(),
            name: account.name().to_owned(),
            account_type: account.account_type().to_string(),
            domains: account.supported_domains()
                .iter()
                .map(|d| d.to_string())
                .collect(),
            balances: self.extract_balances(account),
            resources: self.extract_resources(account),
            capabilities: self.extract_capabilities(account),
            effects: self.extract_effects(account),
            transactions: self.extract_transactions(account),
            metadata: account.metadata().clone(),
        }
    }
}

impl ProgramAccountViewTransformer {
    /// Creates a new transformer
    pub fn new() -> Self {
        Self {}
    }
    
    /// Extracts balances from an account
    fn extract_balances(&self, account: &dyn ProgramAccount) -> Vec<BalanceView> {
        match account.account_type() {
            AccountType::Asset => {
                if let Some(asset_account) = account.as_asset_account() {
                    let balances = asset_account.balances();
                    balances.iter().map(|(asset_id, balance)| {
                        let asset_info = asset_account.asset_info(asset_id);
                        BalanceView {
                            asset_id: asset_id.to_string(),
                            asset_name: asset_info.map(|i| i.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
                            amount: balance.to_string(),
                            asset_type: asset_account.asset_type(asset_id).to_string(),
                            symbol: asset_info.and_then(|i| i.symbol.clone()),
                            decimals: asset_info.map(|i| i.decimals),
                            icon_url: asset_info.and_then(|i| i.icon_url.clone()),
                        }
                    }).collect()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
    
    /// Extracts resources from an account
    fn extract_resources(&self, account: &dyn ProgramAccount) -> Vec<ResourceView> {
        account.resources().iter().map(|resource| {
            ResourceView {
                id: resource.id.to_string(),
                resource_type: resource.resource_type.clone(),
                domain: resource.domain.as_ref().map(|d| d.to_string()),
                preview: resource.preview.clone(),
                state: resource.state.clone(),
                metadata: resource.metadata.clone(),
            }
        }).collect()
    }
    
    /// Extracts capabilities from an account
    fn extract_capabilities(&self, account: &dyn ProgramAccount) -> Vec<CapabilityView> {
        account.capabilities().iter().map(|capability| {
            let restrictions = capability.restrictions.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            CapabilityView {
                id: capability.id.to_string(),
                action: capability.action.clone(),
                target: capability.target.as_ref().map(|t| t.to_string()),
                restrictions: Some(restrictions),
                expires_at: capability.expires_at,
            }
        }).collect()
    }
    
    /// Extracts available effects from an account
    fn extract_effects(&self, account: &dyn ProgramAccount) -> Vec<EffectView> {
        account.available_effects().iter().map(|effect| {
            let parameters = effect.parameters.iter().map(|param| {
                let constraints = param.constraints.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                
                ParameterView {
                    name: param.name.clone(),
                    param_type: param.param_type.clone(),
                    description: param.description.clone(),
                    required: param.required,
                    default_value: param.default_value.clone(),
                    constraints: if param.constraints.is_empty() { None } else { Some(constraints) },
                }
            }).collect();
            
            EffectView {
                id: effect.id.to_string(),
                name: effect.name.clone(),
                description: effect.description.clone(),
                domain: effect.domain.as_ref().map(|d| d.to_string()),
                parameters,
                available: effect.available,
                requirements: effect.requirements.clone(),
            }
        }).collect()
    }
    
    /// Extracts transaction history from an account
    fn extract_transactions(&self, account: &dyn ProgramAccount) -> Vec<TransactionView> {
        account.transaction_history().iter().map(|tx| {
            TransactionView {
                id: tx.id.to_string(),
                transaction_type: tx.transaction_type.clone(),
                timestamp: tx.timestamp,
                status: match tx.status {
                    TransactionStatus::Pending => "pending".to_string(),
                    TransactionStatus::Confirmed => "confirmed".to_string(),
                    TransactionStatus::Failed(_) => "failed".to_string(),
                },
                resources: tx.resources.iter().map(|r| r.to_string()).collect(),
                effects: tx.effects.iter().map(|e| e.to_string()).collect(),
                domains: tx.domains.iter().map(|d| d.to_string()).collect(),
                metadata: tx.metadata.clone(),
                hash: tx.hash.clone(),
                block_number: tx.block_number,
                fee: tx.fee.as_ref().map(|f| FeeView {
                    amount: f.amount.to_string(),
                    currency: f.currency.clone(),
                    gas_price: f.gas_price.as_ref().map(|gp| gp.to_string()),
                    gas_used: f.gas_used,
                }),
            }
        }).collect()
    }
}

/// JSON serialization module for UI views
pub mod serialization {
    use super::*;
    use serde_json::{self, Value};
    use std::io;
    
    /// Error type for serialization operations
    #[derive(Debug, thiserror::Error)]
    pub enum SerializationError {
        #[error("JSON serialization error: {0}")]
        Json(#[from] serde_json::Error),
        
        #[error("IO error: {0}")]
        Io(#[from] io::Error),
        
        #[error("Invalid data format: {0}")]
        InvalidFormat(String),
    }
    
    /// Result type for serialization operations
    pub type SerializationResult<T> = Result<T, SerializationError>;
    
    /// Serializes a UI view to JSON string
    pub fn to_json<T: Serialize>(view: &T) -> SerializationResult<String> {
        serde_json::to_string(view).map_err(SerializationError::Json)
    }
    
    /// Serializes a UI view to pretty-printed JSON string
    pub fn to_pretty_json<T: Serialize>(view: &T) -> SerializationResult<String> {
        serde_json::to_string_pretty(view).map_err(SerializationError::Json)
    }
    
    /// Deserializes a JSON string to a UI view
    pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> SerializationResult<T> {
        serde_json::from_str(json).map_err(SerializationError::Json)
    }
    
    /// Serializes a UI view to JSON value
    pub fn to_json_value<T: Serialize>(view: &T) -> SerializationResult<Value> {
        serde_json::to_value(view).map_err(SerializationError::Json)
    }
    
    /// Writes a UI view to a file as JSON
    pub fn write_to_file<T: Serialize>(view: &T, path: &str) -> SerializationResult<()> {
        let json = to_pretty_json(view)?;
        std::fs::write(path, json).map_err(SerializationError::Io)
    }
    
    /// Reads a UI view from a JSON file
    pub fn read_from_file<T: for<'de> Deserialize<'de>>(path: &str) -> SerializationResult<T> {
        let json = std::fs::read_to_string(path).map_err(SerializationError::Io)?;
        from_json(&json)
    }
} 