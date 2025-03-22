// Test implementations for basic effects
//
// This module provides test implementations of the three-layer effect architecture
// for testing purposes.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::json;

use crate::effect::{
    AlgebraicEffect, EffectContext, EffectResult, EffectError, EffectOutcome,
    TransferEffect, StorageEffect, StorageVisibility, TelHints
};
use crate::resource::{ResourceId, ResourceCapability, Right};
use crate::address::Address;
use crate::domain::DomainId;

/// A basic transfer effect for testing
pub struct TestTransferEffect {
    /// Name of the effect
    name: String,
    
    /// Description of the effect
    description: String,
    
    /// Source address
    source: Address,
    
    /// Destination address
    destination: Address,
    
    /// Resource to transfer
    resource_id: ResourceId,
    
    /// Amount to transfer
    amount: u128,
    
    /// Primary domain
    domain_id: DomainId,
}

impl TestTransferEffect {
    /// Create a new test transfer effect
    pub fn new(
        source: Address,
        destination: Address,
        resource_id: ResourceId,
        amount: u128,
        domain_id: DomainId,
    ) -> Self {
        Self {
            name: "TestTransferEffect".to_string(),
            description: "Transfers resources from one address to another".to_string(),
            source,
            destination,
            resource_id,
            amount,
            domain_id,
        }
    }
}

#[async_trait]
impl AlgebraicEffect for TestTransferEffect {
    fn effect_type(&self) -> &'static str {
        "transfer"
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn resource_ids(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    fn primary_domain(&self) -> Option<DomainId> {
        Some(self.domain_id.clone())
    }
    
    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("source".to_string(), json!(self.source.to_string()));
        params.insert("destination".to_string(), json!(self.destination.to_string()));
        params.insert("resource_id".to_string(), json!(self.resource_id.to_string()));
        params.insert("amount".to_string(), json!(self.amount));
        params
    }
    
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)> {
        vec![
            (self.resource_id.clone(), Right::Read),
            (self.resource_id.clone(), Right::Write),
        ]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would actually transfer resources
        // For testing, we just return a mock outcome
        
        let mut data = HashMap::new();
        data.insert("transferred".to_string(), self.amount.to_string());
        data.insert("resource".to_string(), self.resource_id.clone());
        data.insert("from".to_string(), self.source.to_string());
        data.insert("to".to_string(), self.destination.to_string());
        
        Ok(EffectOutcome {
            id: crate::effect::EffectId::new_unique(),
            execution_id: Some(context.execution_id),
            success: true,
            data,
            error: None,
            resource_changes: vec![],
            metadata: HashMap::new(),
        })
    }
    
    async fn validate(&self, context: &EffectContext) -> EffectResult<()> {
        // Validate that source and destination are different
        if self.source == self.destination {
            return Err(EffectError::InvalidParameter(
                "Source and destination cannot be the same".to_string()
            ));
        }
        
        // Validate that amount is positive
        if self.amount == 0 {
            return Err(EffectError::InvalidParameter(
                "Amount must be greater than zero".to_string()
            ));
        }
        
        // In a real implementation, we would check balances, etc.
        
        Ok(())
    }
    
    fn constraint_traits(&self) -> Vec<&'static str> {
        vec!["TransferEffect"]
    }
    
    fn satisfies_constraint(&self, constraint: &str) -> bool {
        matches!(constraint, "TransferEffect" | "AlgebraicEffect")
    }
    
    fn tel_hints(&self) -> Option<TelHints> {
        Some(TelHints {
            domain_type: "generic".to_string(),
            function_pattern: "transfer_resource".to_string(),
            parameter_mappings: {
                let mut mappings = HashMap::new();
                mappings.insert("source".to_string(), "sender".to_string());
                mappings.insert("destination".to_string(), "recipient".to_string());
                mappings.insert("amount".to_string(), "amount".to_string());
                mappings
            },
            required_imports: vec!["std::transfer".to_string()],
            metadata: HashMap::new(),
        })
    }

    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        // Return an empty vector as this is a test implementation
        Vec::new()
    }

    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        // Return None as this is a test implementation
        None
    }

    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // No validation needed for test implementation
        Ok(())
    }
}

#[async_trait]
impl TransferEffect for TestTransferEffect {
    fn source(&self) -> &Address {
        &self.source
    }
    
    fn destination(&self) -> &Address {
        &self.destination
    }
    
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    
    fn quantity(&self) -> Option<u128> {
        Some(self.amount)
    }
    
    async fn validate_transfer(&self, context: &EffectContext) -> EffectResult<()> {
        self.validate(context).await
    }
}

/// A basic storage effect for testing
pub struct TestStorageEffect {
    /// Name of the effect
    name: String,
    
    /// Description of the effect
    description: String,
    
    /// Resource to store
    resource_id: ResourceId,
    
    /// Storage domain
    domain_id: DomainId,
    
    /// Fields to store
    fields: HashSet<String>,
    
    /// Visibility of stored data
    visibility: StorageVisibility,
    
    /// Data to store
    data: HashMap<String, serde_json::Value>,
}

impl TestStorageEffect {
    /// Create a new test storage effect
    pub fn new(
        resource_id: ResourceId,
        domain_id: DomainId,
        fields: HashSet<String>,
        visibility: StorageVisibility,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            name: "TestStorageEffect".to_string(),
            description: "Stores resource data on-chain".to_string(),
            resource_id,
            domain_id,
            fields,
            visibility,
            data,
        }
    }
}

#[async_trait]
impl AlgebraicEffect for TestStorageEffect {
    fn effect_type(&self) -> &'static str {
        "store"
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn resource_ids(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    fn primary_domain(&self) -> Option<DomainId> {
        Some(self.domain_id.clone())
    }
    
    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("resource_id".to_string(), json!(self.resource_id.to_string()));
        params.insert("fields".to_string(), json!(self.fields));
        params.insert("visibility".to_string(), json!(format!("{:?}", self.visibility)));
        params.insert("data".to_string(), json!(self.data));
        params
    }
    
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)> {
        vec![
            (self.resource_id.clone(), Right::Write),
        ]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would actually store data
        // For testing, we just return a mock outcome
        
        let mut data = HashMap::new();
        data.insert("stored_resource".to_string(), self.resource_id.clone());
        data.insert("domain".to_string(), self.domain_id.clone());
        data.insert("field_count".to_string(), self.fields.len().to_string());
        data.insert("visibility".to_string(), format!("{:?}", self.visibility));
        
        Ok(EffectOutcome {
            id: crate::effect::EffectId::new_unique(),
            execution_id: Some(context.execution_id),
            success: true,
            data,
            error: None,
            resource_changes: vec![],
            metadata: HashMap::new(),
        })
    }
    
    async fn validate(&self, context: &EffectContext) -> EffectResult<()> {
        // Validate that we have fields to store
        if self.fields.is_empty() {
            return Err(EffectError::InvalidParameter(
                "No fields specified for storage".to_string()
            ));
        }
        
        // Validate that we have data for all fields
        for field in &self.fields {
            if !self.data.contains_key(field) {
                return Err(EffectError::InvalidParameter(
                    format!("Missing data for field: {}", field)
                ));
            }
        }
        
        Ok(())
    }
    
    fn constraint_traits(&self) -> Vec<&'static str> {
        vec!["StorageEffect"]
    }
    
    fn satisfies_constraint(&self, constraint: &str) -> bool {
        matches!(constraint, "StorageEffect" | "AlgebraicEffect")
    }
    
    fn tel_hints(&self) -> Option<TelHints> {
        Some(TelHints {
            domain_type: "generic".to_string(),
            function_pattern: "store_resource_data".to_string(),
            parameter_mappings: {
                let mut mappings = HashMap::new();
                mappings.insert("fields".to_string(), "fields".to_string());
                mappings.insert("data".to_string(), "data".to_string());
                mappings.insert("visibility".to_string(), "visibility".to_string());
                mappings
            },
            required_imports: vec!["std::storage".to_string()],
            metadata: HashMap::new(),
        })
    }

    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        // Return an empty vector as this is a test implementation
        Vec::new()
    }

    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        // Return None as this is a test implementation
        None
    }

    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // No validation needed for test implementation
        Ok(())
    }
}

#[async_trait]
impl StorageEffect for TestStorageEffect {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    
    fn storage_domain(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn fields(&self) -> &HashSet<String> {
        &self.fields
    }
    
    fn visibility(&self) -> StorageVisibility {
        self.visibility
    }
    
    async fn validate_storage(&self, context: &EffectContext) -> EffectResult<()> {
        self.validate(context).await
    }
}

/// A basic query effect for testing
pub struct TestQueryEffect {
    /// Name of the effect
    name: String,
    
    /// Description of the effect
    description: String,
    
    /// Resource to query
    resource_id: ResourceId,
    
    /// Query domain
    domain_id: DomainId,
    
    /// Query parameters
    query_params: HashMap<String, serde_json::Value>,
}

impl TestQueryEffect {
    /// Create a new test query effect
    pub fn new(
        resource_id: ResourceId,
        domain_id: DomainId,
        query_params: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            name: "TestQueryEffect".to_string(),
            description: "Queries resource data".to_string(),
            resource_id,
            domain_id,
            query_params,
        }
    }
}

#[async_trait]
impl AlgebraicEffect for TestQueryEffect {
    fn effect_type(&self) -> &'static str {
        "query"
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn resource_ids(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    fn primary_domain(&self) -> Option<DomainId> {
        Some(self.domain_id.clone())
    }
    
    fn parameters(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("resource_id".to_string(), json!(self.resource_id.to_string()));
        params.insert("query".to_string(), json!(self.query_params));
        params
    }
    
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)> {
        vec![
            (self.resource_id.clone(), Right::Read),
        ]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would actually query data
        // For testing, we just return a mock outcome
        
        let mut data = HashMap::new();
        data.insert("queried_resource".to_string(), self.resource_id.clone());
        data.insert("domain".to_string(), self.domain_id.clone());
        data.insert("param_count".to_string(), self.query_params.len().to_string());
        
        // Add mock result data
        data.insert("result_count".to_string(), "42".to_string());
        data.insert("timestamp".to_string(), chrono::Utc::now().to_string());
        
        Ok(EffectOutcome {
            id: crate::effect::EffectId::new_unique(),
            execution_id: Some(context.execution_id),
            success: true,
            data,
            error: None,
            resource_changes: vec![],
            metadata: HashMap::new(),
        })
    }
    
    async fn validate(&self, context: &EffectContext) -> EffectResult<()> {
        // Validate that we have query parameters
        if self.query_params.is_empty() {
            return Err(EffectError::InvalidParameter(
                "No query parameters specified".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn constraint_traits(&self) -> Vec<&'static str> {
        vec!["QueryEffect"]
    }
    
    fn satisfies_constraint(&self, constraint: &str) -> bool {
        matches!(constraint, "QueryEffect" | "AlgebraicEffect")
    }
    
    fn tel_hints(&self) -> Option<TelHints> {
        Some(TelHints {
            domain_type: "generic".to_string(),
            function_pattern: "query_resource".to_string(),
            parameter_mappings: {
                let mut mappings = HashMap::new();
                mappings.insert("query".to_string(), "query_params".to_string());
                mappings
            },
            required_imports: vec!["std::query".to_string()],
            metadata: HashMap::new(),
        })
    }

    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        // Return an empty vector as this is a test implementation
        Vec::new()
    }

    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        // Return None as this is a test implementation
        None
    }

    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // No validation needed for test implementation
        Ok(())
    }
}

// Add test module with unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::QueryEffect;
    
    #[tokio::test]
    async fn test_transfer_effect() {
        let source = Address::new("source");
        let destination = Address::new("destination");
        let resource_id = "resource123".to_string();
        let amount = 100;
        let domain_id = DomainId::new("test-domain");
        
        let effect = TestTransferEffect::new(
            source.clone(),
            destination.clone(),
            resource_id.clone(),
            amount,
            domain_id.clone(),
        );
        
        // Test AlgebraicEffect implementation
        assert_eq!(effect.effect_type(), "transfer");
        assert_eq!(effect.name(), "TestTransferEffect");
        assert_eq!(effect.resource_ids(), vec![resource_id.clone()]);
        assert_eq!(effect.primary_domain(), Some(domain_id.clone()));
        
        // Test TransferEffect implementation
        assert_eq!(effect.source(), &source);
        assert_eq!(effect.destination(), &destination);
        assert_eq!(effect.resource_id(), &resource_id);
        assert_eq!(effect.quantity(), Some(amount));
        
        // Test constraint traits
        assert!(effect.satisfies_constraint("TransferEffect"));
        assert!(effect.satisfies_constraint("AlgebraicEffect"));
        assert!(!effect.satisfies_constraint("StorageEffect"));
        
        // Test TEL hints
        let hints = effect.tel_hints().unwrap();
        assert_eq!(hints.function_pattern, "transfer_resource");
        assert!(hints.parameter_mappings.contains_key("source"));
    }
    
    #[tokio::test]
    async fn test_storage_effect() {
        let resource_id = "resource123".to_string();
        let domain_id = DomainId::new("test-domain");
        let mut fields = HashSet::new();
        fields.insert("field1".to_string());
        fields.insert("field2".to_string());
        let visibility = StorageVisibility::Public;
        
        let mut data = HashMap::new();
        data.insert("field1".to_string(), json!("value1"));
        data.insert("field2".to_string(), json!(42));
        
        let effect = TestStorageEffect::new(
            resource_id.clone(),
            domain_id.clone(),
            fields.clone(),
            visibility,
            data.clone(),
        );
        
        // Test AlgebraicEffect implementation
        assert_eq!(effect.effect_type(), "store");
        assert_eq!(effect.name(), "TestStorageEffect");
        assert_eq!(effect.resource_ids(), vec![resource_id.clone()]);
        assert_eq!(effect.primary_domain(), Some(domain_id.clone()));
        
        // Test StorageEffect implementation
        assert_eq!(effect.resource_id(), &resource_id);
        assert_eq!(effect.storage_domain(), &domain_id);
        assert_eq!(effect.fields(), &fields);
        assert_eq!(effect.visibility(), visibility);
        
        // Test constraint traits
        assert!(effect.satisfies_constraint("StorageEffect"));
        assert!(effect.satisfies_constraint("AlgebraicEffect"));
        assert!(!effect.satisfies_constraint("TransferEffect"));
        
        // Test validation
        let context = EffectContext::default();
        assert!(effect.validate(&context).await.is_ok());
        
        // Test validation with missing data
        let mut fields2 = fields.clone();
        fields2.insert("field3".to_string());
        let effect2 = TestStorageEffect::new(
            resource_id.clone(),
            domain_id.clone(),
            fields2,
            visibility,
            data.clone(),
        );
        assert!(effect2.validate(&context).await.is_err());
    }
}

#[async_trait]
impl crate::effect::QueryEffect for TestQueryEffect {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    
    fn query_domain(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn query_parameters(&self) -> &HashMap<String, serde_json::Value> {
        &self.query_params
    }
    
    async fn validate_query(&self, context: &EffectContext) -> EffectResult<()> {
        self.validate(context).await
    }
} 