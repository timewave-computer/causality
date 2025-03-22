// Invocation Context Module
//
// This module defines the invocation context and related types for tracking
// execution state during effect invocations.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::types::{ResourceId, DomainId, TraceId};
use crate::domain::map::map::TimeMap;
use crate::error::Result;
use crate::log::fact_types::FactType;

/// Invocation state tracking the current execution progress
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvocationState {
    /// Invocation has been created but not started
    Created,
    /// Invocation is currently running
    Running,
    /// Invocation has completed successfully
    Completed,
    /// Invocation has failed
    Failed(String),
    /// Invocation has been canceled
    Canceled,
    /// Invocation is waiting for a resource
    Waiting(ResourceId),
    /// Invocation is waiting for an external fact
    WaitingForFact(String),
}

/// Track resource acquisition during invocation
#[derive(Debug, Clone)]
pub struct ResourceAcquisition {
    /// The resource ID
    pub resource_id: ResourceId,
    /// When the resource was acquired
    pub acquired_at: DateTime<Utc>,
    /// Whether this is a read-only acquisition
    pub read_only: bool,
    /// Reason for acquiring this resource
    pub reason: String,
}

/// Track an external fact that was observed during invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactObservation {
    /// Domain where the fact was observed
    pub domain_id: DomainId,
    /// Unique identifier for the fact
    pub fact_id: String,
    /// The observed fact
    pub fact: FactType,
    /// When the fact was observed
    pub observed_at: DateTime<Utc>,
    /// Whether this fact has been verified
    pub verified: bool,
}

/// Invocation context for tracking execution state
#[derive(Debug, Clone)]
pub struct InvocationContext {
    /// Unique identifier for this invocation
    pub invocation_id: String,
    /// Trace ID for tracking related invocations
    pub trace_id: TraceId,
    /// Parent invocation ID if this is a child invocation
    pub parent_id: Option<String>,
    /// Current state of the invocation
    pub state: InvocationState,
    /// Time map snapshot at the start of invocation
    pub time_map: TimeMap,
    /// When the invocation was created
    pub created_at: DateTime<Utc>,
    /// When the invocation started execution
    pub started_at: Option<DateTime<Utc>>,
    /// When the invocation completed execution
    pub completed_at: Option<DateTime<Utc>>,
    /// Resources acquired during this invocation
    pub acquired_resources: Vec<ResourceAcquisition>,
    /// External facts observed during this invocation
    pub observed_facts: Vec<FactObservation>,
    /// Execution metadata (arbitrary key-value pairs)
    pub metadata: HashMap<String, String>,
    /// Child invocations spawned by this invocation
    pub children: HashSet<String>,
}

impl InvocationContext {
    /// Create a new invocation context with a given ID and trace ID
    pub fn new(invocation_id: String, trace_id: TraceId, time_map: TimeMap) -> Self {
        InvocationContext {
            invocation_id,
            trace_id,
            parent_id: None,
            state: InvocationState::Created,
            time_map,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            acquired_resources: Vec::new(),
            observed_facts: Vec::new(),
            metadata: HashMap::new(),
            children: HashSet::new(),
        }
    }
    
    /// Create a child invocation context from this context
    pub fn create_child(&self, invocation_id: String) -> Self {
        InvocationContext {
            invocation_id,
            trace_id: self.trace_id.clone(),
            parent_id: Some(self.invocation_id.clone()),
            state: InvocationState::Created,
            time_map: self.time_map.clone(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            acquired_resources: Vec::new(),
            observed_facts: self.observed_facts.clone(),
            metadata: HashMap::new(),
            children: HashSet::new(),
        }
    }
    
    /// Mark the invocation as started
    pub fn start(&mut self) -> Result<()> {
        if self.state != InvocationState::Created {
            return Err(crate::error::Error::InvocationStateError(
                format!("Cannot start invocation in state: {:?}", self.state)
            ));
        }
        
        self.state = InvocationState::Running;
        self.started_at = Some(Utc::now());
        
        Ok(())
    }
    
    /// Mark the invocation as completed
    pub fn complete(&mut self) -> Result<()> {
        if self.state != InvocationState::Running {
            return Err(crate::error::Error::InvocationStateError(
                format!("Cannot complete invocation in state: {:?}", self.state)
            ));
        }
        
        self.state = InvocationState::Completed;
        self.completed_at = Some(Utc::now());
        
        Ok(())
    }
    
    /// Mark the invocation as failed with a reason
    pub fn fail(&mut self, reason: &str) -> Result<()> {
        self.state = InvocationState::Failed(reason.to_string());
        self.completed_at = Some(Utc::now());
        
        Ok(())
    }
    
    /// Mark the invocation as waiting for a resource
    pub fn wait_for_resource(&mut self, resource_id: ResourceId) -> Result<()> {
        if self.state != InvocationState::Running {
            return Err(crate::error::Error::InvocationStateError(
                format!("Cannot wait for resource in state: {:?}", self.state)
            ));
        }
        
        self.state = InvocationState::Waiting(resource_id);
        
        Ok(())
    }
    
    /// Mark the invocation as waiting for a fact
    pub fn wait_for_fact(&mut self, fact_id: &str) -> Result<()> {
        if self.state != InvocationState::Running {
            return Err(crate::error::Error::InvocationStateError(
                format!("Cannot wait for fact in state: {:?}", self.state)
            ));
        }
        
        self.state = InvocationState::WaitingForFact(fact_id.to_string());
        
        Ok(())
    }
    
    /// Resume the invocation from a waiting state
    pub fn resume(&mut self) -> Result<()> {
        match self.state {
            InvocationState::Waiting(_) | InvocationState::WaitingForFact(_) => {
                self.state = InvocationState::Running;
                Ok(())
            },
            _ => Err(crate::error::Error::InvocationStateError(
                format!("Cannot resume invocation in state: {:?}", self.state)
            )),
        }
    }
    
    /// Track a resource acquisition
    pub fn acquire_resource(&mut self, 
        resource_id: ResourceId, 
        read_only: bool,
        reason: &str
    ) -> Result<()> {
        let acquisition = ResourceAcquisition {
            resource_id,
            acquired_at: Utc::now(),
            read_only,
            reason: reason.to_string(),
        };
        
        self.acquired_resources.push(acquisition);
        
        Ok(())
    }
    
    /// Track an observed fact
    pub fn observe_fact(&mut self, 
        domain_id: DomainId, 
        fact_id: &str,
        fact: FactType,
        verified: bool
    ) -> Result<()> {
        let observation = FactObservation {
            domain_id,
            fact_id: fact_id.to_string(),
            fact,
            observed_at: Utc::now(),
            verified,
        };
        
        self.observed_facts.push(observation);
        
        Ok(())
    }
    
    /// Add a child invocation
    pub fn add_child(&mut self, invocation_id: &str) -> Result<()> {
        self.children.insert(invocation_id.to_string());
        
        Ok(())
    }
    
    /// Add metadata to the invocation
    pub fn add_metadata(&mut self, key: &str, value: &str) -> Result<()> {
        self.metadata.insert(key.to_string(), value.to_string());
        
        Ok(())
    }
    
    /// Get invocation duration in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => {
                let duration = end.timestamp_millis() - start.timestamp_millis();
                Some(duration as u64)
            },
            _ => None,
        }
    }
    
    /// Check if the invocation is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self.state, 
            InvocationState::Completed | 
            InvocationState::Failed(_) | 
            InvocationState::Canceled
        )
    }
    
    /// Check if the invocation is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, 
            InvocationState::Running | 
            InvocationState::Waiting(_) | 
            InvocationState::WaitingForFact(_)
        )
    }
}

/// Context propagation module for invocation contexts
pub mod propagation;

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_context() -> InvocationContext {
        let time_map = TimeMap::new();
        let trace_id = TraceId::new();
        
        InvocationContext::new(
            "test_invocation".to_string(),
            trace_id,
            time_map,
        )
    }
    
    #[test]
    fn test_invocation_lifecycle() -> Result<()> {
        let mut context = create_test_context();
        
        // Check initial state
        assert_eq!(context.state, InvocationState::Created);
        assert!(context.started_at.is_none());
        assert!(context.completed_at.is_none());
        
        // Start the invocation
        context.start()?;
        assert_eq!(context.state, InvocationState::Running);
        assert!(context.started_at.is_some());
        assert!(context.completed_at.is_none());
        
        // Track resource acquisition
        let resource_id = ResourceId::new("test_resource");
        context.acquire_resource(resource_id, true, "Testing")?;
        assert_eq!(context.acquired_resources.len(), 1);
        
        // Track observed fact
        let domain_id = DomainId::new("test_domain");
        context.observe_fact(domain_id, "test_fact", FactType::new("test_fact_type"), true)?;
        assert_eq!(context.observed_facts.len(), 1);
        
        // Add metadata
        context.add_metadata("test_key", "test_value")?;
        assert_eq!(context.metadata.get("test_key"), Some(&"test_value".to_string()));
        
        // Complete the invocation
        context.complete()?;
        assert_eq!(context.state, InvocationState::Completed);
        assert!(context.completed_at.is_some());
        assert!(context.duration_ms().is_some());
        assert!(context.is_final());
        assert!(!context.is_active());
        
        Ok(())
    }
    
    #[test]
    fn test_child_invocation() -> Result<()> {
        let mut parent = create_test_context();
        
        // Create a child invocation
        let child = parent.create_child("child_invocation".to_string());
        
        // Check child properties
        assert_eq!(child.parent_id, Some(parent.invocation_id.clone()));
        assert_eq!(child.trace_id, parent.trace_id);
        assert_eq!(child.state, InvocationState::Created);
        
        // Add the child to the parent
        parent.add_child(&child.invocation_id)?;
        assert!(parent.children.contains(&child.invocation_id));
        
        Ok(())
    }
    
    #[test]
    fn test_wait_and_resume() -> Result<()> {
        let mut context = create_test_context();
        
        // Start the invocation
        context.start()?;
        
        // Wait for a resource
        let resource_id = ResourceId::new("test_resource");
        context.wait_for_resource(resource_id)?;
        
        match &context.state {
            InvocationState::Waiting(waiting_resource_id) => {
                assert_eq!(*waiting_resource_id, resource_id);
            },
            _ => panic!("Expected Waiting state"),
        }
        
        // Resume the invocation
        context.resume()?;
        assert_eq!(context.state, InvocationState::Running);
        
        // Wait for a fact
        context.wait_for_fact("test_fact")?;
        
        match &context.state {
            InvocationState::WaitingForFact(fact_id) => {
                assert_eq!(fact_id, "test_fact");
            },
            _ => panic!("Expected WaitingForFact state"),
        }
        
        // Resume the invocation
        context.resume()?;
        assert_eq!(context.state, InvocationState::Running);
        
        Ok(())
    }
    
    #[test]
    fn test_failure() -> Result<()> {
        let mut context = create_test_context();
        
        // Start the invocation
        context.start()?;
        
        // Fail the invocation
        context.fail("Test failure")?;
        
        match &context.state {
            InvocationState::Failed(reason) => {
                assert_eq!(reason, "Test failure");
            },
            _ => panic!("Expected Failed state"),
        }
        
        assert!(context.completed_at.is_some());
        assert!(context.is_final());
        
        Ok(())
    }
} 