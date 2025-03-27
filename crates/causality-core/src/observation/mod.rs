// Observation module
//
// This module provides functionality for observations, which are trusted observers
// of external systems (like blockchains) used to extract facts and reconstruct logs.

// Re-export main types from submodules
pub use self::extraction::{
    ExtractedFact, ExtractionRule, FilterCondition, FactExtractor, 
    RuleEngine, BasicExtractor
};
pub use self::proxy::{
    ProxyConfig, ChainStatus, ProxyEvent, ProxyEventHandler, 
    LoggingEventHandler, ObservationProxy
};
pub use self::reconstruction::{
    ReconstructionConfig, ReconstructionStatus, LogReconstructor, 
    ReconstructorFactory, BasicReconstructor
};
pub use self::provider::{
    ProviderConfig, ProvidedData, ProviderFactory, ProviderCreator, 
    DataProvider, ProviderStatus, HttpProvider, HttpProviderConfig,
    HttpProviderCreator
};

// Export submodules
pub mod extraction;
pub mod proxy;
pub mod reconstruction;
pub mod provider;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_structure() {
        // Just a simple test to verify the module structure
        // This will fail to compile if any of the re-exports are invalid
        assert!(true);
    }
} 