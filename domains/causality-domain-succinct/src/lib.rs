// Succinct domain adapter module
// Original file: src/domain_adapters/succinct/mod.rs

// Succinct ZK-VM adapter module
//
// This module provides integration with the Succinct ZK-VM platform.

mod adapter;
mod types;
mod bridge;

// Re-export public items
pub use adapter::SuccinctAdapter;
pub use types::{PublicInputs, ProofData, ProgramId, ProofOptions};
pub use bridge::{SuccinctVmBridge, create_succinct_vm_bridge};

/// Get the default Succinct adapter implementation
pub fn default_adapter() -> causality_types::Result<SuccinctAdapter> {
    SuccinctAdapter::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Just a simple test to ensure the module exports are correct
        assert!(std::any::TypeId::of::<SuccinctAdapter>() != std::any::TypeId::of::<()>());
        assert!(std::any::TypeId::of::<PublicInputs>() != std::any::TypeId::of::<()>());
        assert!(std::any::TypeId::of::<SuccinctVmBridge>() != std::any::TypeId::of::<()>());
    }
} 