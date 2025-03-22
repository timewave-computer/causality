// Program Account Module
//
// This module defines the user-facing program account layer that serves
// as the touchpoint for interacting with the system.

// Define the sub-modules
pub mod base_account;
pub mod registry;
pub mod asset_account;
pub mod utility_account;
pub mod domain_bridge_account;
pub mod authorization;
pub mod ui;
pub mod ui_model;
pub mod effect_adapter;

// Re-export types from the main module
pub use crate::program_account::{
    ProgramAccount,
    ProgramAccountRegistry,
    AssetProgramAccount,
    UtilityProgramAccount,
    DomainBridgeProgramAccount,
    ProgramAccountResource,
    ProgramAccountCapability,
    AvailableEffect,
    EffectParameter,
    EffectResult,
    EffectStatus,
    TransactionRecord,
    TransactionStatus,
    CrossDomainTransfer,
    TransferStatus,
};

// Re-export types from sub-modules
pub use base_account::BaseAccount;
pub use registry::{
    StandardProgramAccountRegistry,
    AccountType,
    AccountWrapper,
};
pub use asset_account::{AssetAccount, AssetType, AssetCollection};
pub use utility_account::{UtilityAccount, StoredData};
pub use domain_bridge_account::{DomainBridgeAccount, CrossDomainTransfer, TransferStatus};
pub use authorization::{
    AuthorizationManager,
    AuthorizationContext,
    AuthorizationResult,
    AuthorizationLevel,
    ProgramAccountAuthorization,
    Role,
    DelegateAuthorization,
    SignatureVerificationResult,
};

// Re-export UI types
pub use ui::{
    ProgramAccountView, ResourceView, BalanceView, CapabilityView, 
    EffectView, TransactionView, FeeView, ParameterView,
    ViewTransformer, ProgramAccountViewTransformer,
};
pub use ui::serialization::{
    SerializationError, SerializationResult,
    to_json, to_pretty_json, from_json, to_json_value,
    write_to_file, read_from_file,
};

pub use self::effect_adapter::{
    ProgramAccountEffectAdapter,
    ProgramAccountEffectAdapterImpl,
    EffectInfo,
    EffectParameter,
    EffectParameterType,
}; 