// Program account system
// Original file: src/program_account/mod.rs

// Program Account Module
//
// This module defines the user-facing program account layer that serves
// as the touchpoint for interacting with the system.

// Define the sub-modules
pub mod base_account;
pub mod registry;
pub mod asset_account;
pub mod utility_account;
pub mod authorization;
pub mod ui;
pub mod effect_adapter;

// Re-export types from sub-modules
pub use base_account::BaseAccount;
pub use registry::{
    StandardProgramAccountRegistry,
    AccountType,
    AccountWrapper,
};
pub use asset_account::{AssetAccount, AssetType, AssetCollection};
pub use utility_account::{UtilityAccount, StoredData};
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

pub use effect_adapter::{
    ProgramAccountEffectAdapter,
    ProgramAccountEffectAdapterImpl,
    EffectInfo,
    EffectParameterType,
}; 