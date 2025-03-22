// Domain Registry for Causality
//
// This module provides a registry for managing Domain adapters.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::domain::{DomainAdapter, DomainId};
use crate::domain::{DomainType, DomainStatus, DomainInfo};
use crate::error::{Error, Result};

/// Registry for Domain adapters
#[derive(Debug)]
pub struct DomainRegistry {
    domains: RwLock<HashMap<DomainId, (Arc<dyn DomainAdapter>, DomainInfo)>>,
}

impl DomainRegistry {
    /// Create a new Domain registry
    pub fn new() -> Self {
        DomainRegistry {
            domains: RwLock::new(HashMap::new()),
        }
    }

    /// Register a Domain adapter
    pub fn register(
        &self,
        adapter: Arc<dyn DomainAdapter>,
        info: DomainInfo,
    ) -> Result<()> {
        let mut domains = self.domains.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on Domain registry".to_string())
        })?;

        domains.insert(info.id.clone(), (adapter, info));
        Ok(())
    }

    /// Get a Domain adapter by ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Result<Arc<dyn DomainAdapter>> {
        let domains = self.domains.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on Domain registry".to_string())
        })?;

        match domains.get(domain_id) {
            Some((adapter, _)) => Ok(adapter.clone()),
            None => Err(Error::InvalidArgument(format!(
                "Domain not found: {}",
                domain_id
            ))),
        }
    }

    /// Get Domain info by ID
    pub fn get_info(&self, domain_id: &DomainId) -> Result<DomainInfo> {
        let domains = self.domains.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on Domain registry".to_string())
        })?;

        match domains.get(domain_id) {
            Some((_, info)) => Ok(info.clone()),
            None => Err(Error::InvalidArgument(format!(
                "Domain not found: {}",
                domain_id
            ))),
        }
    }

    /// List all registered Domains
    pub fn list_domains(&self) -> Result<Vec<DomainId>> {
        let domains = self.domains.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on Domain registry".to_string())
        })?;

        Ok(domains.keys().cloned().collect())
    }

    /// Get a domain adapter by ID, returning None if not found
    pub fn get_domain(&self, domain_id: &DomainId) -> Option<Arc<dyn DomainAdapter>> {
        match self.domains.read() {
            Ok(domains) => domains.get(domain_id).map(|(adapter, _)| adapter.clone()),
            Err(_) => None,
        }
    }

    /// Register a Domain adapter directly
    pub fn register_domain(
        &self,
        adapter: Arc<dyn DomainAdapter>,
    ) -> Result<()> {
        // Get domain info from the adapter
        let domain_id = adapter.domain_id().clone();
        let info = tokio::runtime::Handle::current().block_on(async {
            adapter.domain_info().await
        })?;
        
        self.register(adapter, info)
    }
} 