// Observation Module
//
// This module provides observation capabilities for the causality core.

pub mod extraction;
pub mod proxy;
pub mod reconstruction;
pub mod provider;
pub mod indexer;

pub use extraction::{
    ExtractedFact, ExtractionRule, FactExtractor, RuleEngine, 
    BasicExtractor, BlockData, ExtractionError
};

pub use proxy::{
    ProxyConfig, ProxyEvent, ProxyEventHandler, ChainStatus, 
    ProxyError, ObservationProxy, LoggingEventHandler
};

pub use reconstruction::{
    ReconstructionConfig, ReconstructionStatus, LogReconstructor,
    BasicReconstructor, ReconstructorFactory, ReconstructionError
};

pub use provider::{
    ObservationProvider, ObservationProviderConfig, ProviderConfig,
    ProviderAuth, DataProvider, ProviderFactory, ProviderData,
    ProviderStatus, ProviderError
};

pub use indexer::{
    IndexerConfig, IndexerStatus, ChainIndexer, BasicIndexer,
    IndexerFactory, IndexerCreator, BasicIndexerCreator, IndexerError
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_structure() {
        // Test that the module structure is as expected
    }
} 