//! Algebraic effects for automatic mock and test generation

pub mod core;
pub mod schema;
pub mod error;
pub mod defi;

pub use core::{
    AlgebraicEffect,
    EffectCategory,
    FailureMode,
    EffectResult,
    EffectError,
    EffectLibrary,
};

pub use schema::{
    EffectSchema,
    EffectMetadata,
    ParameterDef,
    TypeDef,
    SchemaError,
};

pub use error::{
    MockError,
    TestError,
    AutoTestError,
    MockResult,
    TestResult,
    AutoTestResult,
};

pub use defi::{
    TokenTransfer,
    TransferReceipt,
    TransferError,
    LiquiditySwap,
    SwapReceipt,
    SwapError,
    DexProtocol,
    SwapType,
    PoolInfo,
}; 