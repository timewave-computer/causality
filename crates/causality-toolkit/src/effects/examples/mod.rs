//! Example effect implementations demonstrating the AlgebraicEffect trait

pub mod transfer;
pub use transfer::{
    TokenTransfer,
    TransferReceipt,
    TransferError,
}; 