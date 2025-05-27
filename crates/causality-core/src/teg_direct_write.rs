// Direct TEG write system for SMT-backed TEG data
//
// This module provides direct write capabilities for TEG nodes, effects, resources,
// intents, and handlers with domain-aware storage and temporal relationship validation.

use crate::smt::{TegMultiDomainSmt, MemoryBackend};
use causality_types::{
    core::id::AsId,
    Effect,
    resource::Resource,
};
use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};

/// Universal TEG node storage interface using domain-aware SMT
pub struct TegDirectWriter {
    smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>,
}

/// Result of a TEG write operation
#[derive(Debug, Clone)]
pub struct TegWriteResult {
    pub operation_id: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub smt_key: Option<String>,
}

impl TegDirectWriter {
    /// Create a new TEG direct writer
    pub fn new(smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>) -> Self {
        Self { smt }
    }

    /// Store a TEG effect with domain awareness and temporal validation
    pub fn store_effect(&self, effect: &Effect) -> Result<TegWriteResult> {
        let mut smt = self.smt.lock().map_err(|_| anyhow!("Failed to lock SMT"))?;
        
        // Store the effect using SMT
        let smt_key = smt.store_teg_effect(effect)
            .map_err(|e| anyhow!("Failed to store effect: {}", e))?;
        
        Ok(TegWriteResult {
            operation_id: format!("store_effect_{}", hex::encode(effect.id.inner())),
            success: true,
            error_message: None,
            smt_key: Some(smt_key),
        })
    }

    /// Store a TEG resource with access constraint validation
    pub fn store_resource(&self, resource: &Resource) -> Result<TegWriteResult> {
        let mut smt = self.smt.lock().map_err(|_| anyhow!("Failed to lock SMT"))?;
        
        // Store the resource using SMT
        let smt_key = smt.store_teg_resource(resource)
            .map_err(|e| anyhow!("Failed to store resource: {}", e))?;
        
        Ok(TegWriteResult {
            operation_id: format!("store_resource_{}", hex::encode(resource.id.inner())),
            success: true,
            error_message: None,
            smt_key: Some(smt_key),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smt::MemoryBackend;

    #[test]
    fn test_teg_direct_writer_creation() {
        let backend = MemoryBackend::new();
        let smt = Arc::new(Mutex::new(TegMultiDomainSmt::new(backend)));
        let _writer = TegDirectWriter::new(smt);
        
        // Just verify we can create the writer
        assert!(true);
    }
} 