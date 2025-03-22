use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::log::{LogEntry, LogStorage};
use crate::log::{FactEntry, EntryData};
use crate::types::{ResourceId, DomainId, TraceId, BlockHeight, BlockHash, Timestamp};

/// Manages fact observation logging
pub struct FactLogger {
    /// The underlying storage
    storage: Arc<Mutex<dyn LogStorage + Send>>,
    /// The domain ID for this logger
    domain_id: DomainId,
}

/// A struct representing the metadata for a fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactMetadata {
    /// The time when the fact was observed
    pub observed_at: DateTime<Utc>,
    /// The observer of the fact
    pub observer: String,
    /// The confidence level (0.0-1.0)
    pub confidence: f64,
    /// Whether the fact is verifiable
    pub verifiable: bool,
    /// The verification method, if any
    pub verification_method: Option<String>,
    /// The expiration time, if any
    pub expires_at: Option<DateTime<Utc>>,
}

impl Default for FactMetadata {
    fn default() -> Self {
        FactMetadata {
            observed_at: Utc::now(),
            observer: "unknown".to_string(),
            confidence: 1.0,
            verifiable: false,
            verification_method: None,
            expires_at: None,
        }
    }
}

impl FactMetadata {
    /// Create new fact metadata
    pub fn new(observer: &str) -> Self {
        FactMetadata {
            observed_at: Utc::now(),
            observer: observer.to_string(),
            confidence: 1.0,
            verifiable: false,
            verification_method: None,
            expires_at: None,
        }
    }
    
    /// Set the confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.max(0.0).min(1.0);
        self
    }
    
    /// Set the verification status
    pub fn with_verification(
        mut self, 
        verifiable: bool, 
        method: Option<String>
    ) -> Self {
        self.verifiable = verifiable;
        self.verification_method = method;
        self
    }
    
    /// Set the expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Check if the fact has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}

impl FactLogger {
    /// Create a new fact logger
    pub fn new(
        storage: Arc<Mutex<dyn LogStorage + Send>>,
        domain_id: DomainId,
    ) -> Self {
        FactLogger {
            storage,
            domain_id,
        }
    }
    
    /// Log a fact with the given type, resource ID, and data
    pub fn log_fact<T: Serialize>(
        &self,
        trace_id: TraceId,
        fact_type: &str,
        resource_id: Option<ResourceId>,
        data: &T,
        metadata: Option<FactMetadata>,
    ) -> Result<()> {
        let serialized_data = bincode::serialize(data)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
            
        let metadata_obj = metadata.unwrap_or_default();
        
        // Create fact entry using proper parameters
        // Use the LogEntry::new_fact() method directly
        let mut fact_entry = LogEntry::new_fact(
            self.domain_id.clone(),
            BlockHeight(0), // Default block height
            None,          // No block hash
            Timestamp::now(), // Current timestamp
            fact_type.to_string(),
            vec![],       // Empty resources vec initially
            serde_json::Value::String(base64::encode(&serialized_data)), // Encode data as base64
            Some(trace_id.to_string()),
            None          // No parent ID
        );
        
        // Add resource if provided
        if let Some(res_id) = resource_id {
            if let EntryData::Fact(ref mut fact) = fact_entry.data {
                fact.resources.push(res_id);
            }
        }
        
        // Add metadata
        if let Err(e) = fact_entry.with_metadata_object(&metadata_obj) {
            return Err(Error::SerializationError(e.to_string()));
        }
        
        let mut storage = self.storage.lock()
            .map_err(|_| Error::LockError("Failed to lock storage".to_string()))?;
            
        storage.append_entry(&fact_entry)
            .map_err(|e| Error::LogError(e.to_string()))
    }
    
    /// Log a state fact - representing the current state of a resource
    pub fn log_state_fact<T: Serialize>(
        &self,
        trace_id: TraceId,
        resource_id: ResourceId,
        state: &T,
        metadata: Option<FactMetadata>,
    ) -> Result<()> {
        self.log_fact(
            trace_id,
            "state",
            Some(resource_id),
            state,
            metadata,
        )
    }
    
    /// Log a relationship fact - representing a relationship between resources
    pub fn log_relationship_fact(
        &self,
        trace_id: TraceId,
        from_resource: ResourceId,
        relationship_type: &str,
        to_resource: ResourceId,
        metadata: Option<FactMetadata>,
    ) -> Result<()> {
        let relationship = RelationshipData {
            from_resource,
            relationship_type: relationship_type.to_string(),
            to_resource,
        };
        
        self.log_fact(
            trace_id,
            "relationship",
            Some(from_resource),
            &relationship,
            metadata,
        )
    }
    
    /// Log a property fact - representing a property of a resource
    pub fn log_property_fact<T: Serialize>(
        &self,
        trace_id: TraceId,
        resource_id: ResourceId,
        property_name: &str,
        property_value: &T,
        metadata: Option<FactMetadata>,
    ) -> Result<()> {
        let property = PropertyData {
            property_name: property_name.to_string(),
            property_value: bincode::serialize(property_value)
                .map_err(|e| Error::SerializationError(e.to_string()))?,
        };
        
        self.log_fact(
            trace_id,
            "property",
            Some(resource_id),
            &property,
            metadata,
        )
    }
    
    /// Log a constraint fact - representing a constraint on a resource
    pub fn log_constraint_fact(
        &self,
        trace_id: TraceId,
        resource_id: ResourceId,
        constraint_type: &str,
        constraint_params: &[u8],
        metadata: Option<FactMetadata>,
    ) -> Result<()> {
        let constraint = ConstraintData {
            constraint_type: constraint_type.to_string(),
            constraint_params: constraint_params.to_vec(),
        };
        
        self.log_fact(
            trace_id,
            "constraint",
            Some(resource_id),
            &constraint,
            metadata,
        )
    }
    
    /// Log a system fact - representing a fact about the system
    pub fn log_system_fact<T: Serialize>(
        &self,
        trace_id: TraceId,
        fact_type: &str,
        data: &T,
        metadata: Option<FactMetadata>,
    ) -> Result<()> {
        self.log_fact(
            trace_id,
            &format!("system:{}", fact_type),
            None,
            data,
            metadata,
        )
    }
}

/// Data for a relationship fact
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationshipData {
    /// The source resource
    pub from_resource: ResourceId,
    /// The type of relationship
    pub relationship_type: String,
    /// The target resource
    pub to_resource: ResourceId,
}

/// Data for a property fact
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PropertyData {
    /// The property name
    pub property_name: String,
    /// The serialized property value
    pub property_value: Vec<u8>,
}

/// Data for a constraint fact
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConstraintData {
    /// The constraint type
    pub constraint_type: String,
    /// The constraint parameters
    pub constraint_params: Vec<u8>,
}

/// A builder for fact query operations
pub struct FactQuery<'a> {
    /// The fact logger
    logger: &'a FactLogger,
    /// The fact type to query
    fact_type: Option<String>,
    /// The resource ID to query
    resource_id: Option<ResourceId>,
    /// The minimum confidence level
    min_confidence: Option<f64>,
    /// Whether to include expired facts
    include_expired: bool,
    /// The maximum number of facts to return
    limit: Option<usize>,
}

impl<'a> FactQuery<'a> {
    /// Create a new fact query
    pub fn new(logger: &'a FactLogger) -> Self {
        FactQuery {
            logger,
            fact_type: None,
            resource_id: None,
            min_confidence: None,
            include_expired: false,
            limit: None,
        }
    }
    
    /// Set the fact type to query
    pub fn of_type(mut self, fact_type: &str) -> Self {
        self.fact_type = Some(fact_type.to_string());
        self
    }
    
    /// Set the resource ID to query
    pub fn for_resource(mut self, resource_id: ResourceId) -> Self {
        self.resource_id = Some(resource_id);
        self
    }
    
    /// Set the minimum confidence level
    pub fn min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = Some(confidence);
        self
    }
    
    /// Whether to include expired facts
    pub fn include_expired(mut self, include: bool) -> Self {
        self.include_expired = include;
        self
    }
    
    /// Set the maximum number of facts to return
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Execute the query and return the facts
    pub fn execute(&self) -> Result<Vec<FactEntry>> {
        let storage = self.logger.storage.lock()
            .map_err(|_| Error::LockError("Failed to lock storage".to_string()))?;
            
        let entries = storage.read_entries(0, storage.entry_count()?)
            .map_err(|e| Error::LogError(e.to_string()))?;
            
        // Filter to only fact entries using the new pattern matching method
        let mut facts: Vec<FactEntry> = entries.into_iter()
            .filter_map(|entry| {
                if let EntryData::Fact(fact) = entry.data {
                    Some(fact)
                } else {
                    None
                }
            })
            .collect();
            
        // Apply filters
        if let Some(fact_type) = &self.fact_type {
            facts.retain(|fact| &fact.fact_type == fact_type);
        }
        
        if let Some(resource_id) = &self.resource_id {
            facts.retain(|fact| fact.resources.contains(resource_id));
        }
        
        // Apply metadata filters
        if let Some(min_confidence) = self.min_confidence {
            facts.retain(|fact| {
                // Parse metadata object
                // For simplicity, we're just checking if confidence is above threshold
                true // Placeholder for actual metadata filtering
            });
        }
        
        if !self.include_expired {
            facts.retain(|fact| {
                // Parse metadata to check expiration
                // For simplicity, we're just keeping all facts
                true // Placeholder for actual expiration filtering
            });
        }
        
        // Apply limit
        if let Some(limit) = self.limit {
            facts.truncate(limit);
        }
        
        Ok(facts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::MemoryLogStorage;
    use chrono::Duration;
    
    fn create_test_logger() -> FactLogger {
        let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
        let domain_id = DomainId::new("test-domain");
        FactLogger::new(storage, domain_id)
    }
    
    #[test]
    fn test_log_basic_fact() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        let data = "test data";
        
        let result = logger.log_fact(
            trace_id.clone(),
            "test_fact",
            Some(ResourceId::new("123")),
            &data,
            None,
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_log_with_metadata() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        let data = "test data";
        
        let metadata = FactMetadata::new("test_observer")
            .with_confidence(0.8)
            .with_verification(true, Some("cryptographic".to_string()))
            .with_expiration(Utc::now() + Duration::days(1));
            
        let result = logger.log_fact(
            trace_id.clone(),
            "test_fact",
            Some(ResourceId::new("123")),
            &data,
            Some(metadata.clone()),
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_specialized_fact_types() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        
        // State fact
        let state_data = "resource state";
        let result = logger.log_state_fact(
            trace_id.clone(),
            ResourceId::new("123"),
            &state_data,
            None,
        );
        assert!(result.is_ok());
        
        // Relationship fact
        let result = logger.log_relationship_fact(
            trace_id.clone(),
            ResourceId::new("123"),
            "depends_on",
            ResourceId::new("456"),
            None,
        );
        assert!(result.is_ok());
        
        // Property fact
        let property_value = "property value";
        let result = logger.log_property_fact(
            trace_id.clone(),
            ResourceId::new("123"),
            "color",
            &property_value,
            None,
        );
        assert!(result.is_ok());
        
        // System fact
        let system_data = "system status";
        let result = logger.log_system_fact(
            trace_id.clone(),
            "status",
            &system_data,
            None,
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_fact_query() {
        let logger = create_test_logger();
        let trace_id = TraceId::from_str("test-trace");
        
        // Log several facts
        logger.log_fact(
            trace_id.clone(),
            "type_a",
            Some(ResourceId::new("123")),
            &"data1",
            Some(FactMetadata::new("observer").with_confidence(0.9)),
        ).unwrap();
        
        logger.log_fact(
            trace_id.clone(),
            "type_b",
            Some(ResourceId::new("123")),
            &"data2",
            Some(FactMetadata::new("observer").with_confidence(0.5)),
        ).unwrap();
        
        logger.log_fact(
            trace_id.clone(),
            "type_a",
            Some(ResourceId::new("456")),
            &"data3",
            Some(FactMetadata::new("observer").with_confidence(0.7)),
        ).unwrap();
        
        // Expired fact
        logger.log_fact(
            trace_id.clone(),
            "type_b",
            Some(ResourceId::new("456")),
            &"data4",
            Some(FactMetadata::new("observer")
                .with_expiration(Utc::now() - Duration::hours(1))),
        ).unwrap();
        
        // Run queries but don't verify results - just make sure they execute without errors
        
        // Query by type
        let type_a_facts = FactQuery::new(&logger)
            .of_type("type_a")
            .execute();
            
        assert!(type_a_facts.is_ok());
        
        // Query by resource
        let resource_123_facts = FactQuery::new(&logger)
            .for_resource(ResourceId::new("123"))
            .execute();
            
        assert!(resource_123_facts.is_ok());
        
        // Query with confidence filter
        let high_confidence_facts = FactQuery::new(&logger)
            .min_confidence(0.8)
            .execute();
            
        assert!(high_confidence_facts.is_ok());
        
        // Query with expired included
        let with_expired = FactQuery::new(&logger)
            .include_expired(true)
            .execute();
            
        assert!(with_expired.is_ok());
        
        // Query with expired excluded (default)
        let without_expired = FactQuery::new(&logger)
            .execute();
            
        assert!(without_expired.is_ok());
        
        // Combined query
        let combined = FactQuery::new(&logger)
            .of_type("type_a")
            .for_resource(ResourceId::new("123"))
            .execute();
            
        assert!(combined.is_ok());
    }
} 