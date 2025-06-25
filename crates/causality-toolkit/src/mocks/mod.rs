//! Mock generation system for automatic effect handler creation

pub mod strategy;
pub mod blockchain;

pub use strategy::{
    MockStrategy,
    ChainConfig,
    StrategyConfig,
    StrategyError,
};

pub use generator::{
    MockHandler,
    MockGenerator,
};

pub use blockchain::{
    BlockchainSimulationMock,
    ChainParams,
    MockChainState,
    PendingTransaction,
    TransactionRecord,
    TransactionStatus,
    CongestionState,
    ContractState,
    ContractType,
    ContractStateValue,
    ForkChoiceParams,
    NetworkTopology,
}; 