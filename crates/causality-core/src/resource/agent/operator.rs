// operator.rs - Operator agent implementation
//
// This file implements the specialized OperatorAgent type, representing system administrators
// responsible for maintaining infrastructure and critical operations.

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::ResourceError;
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::serialization::{Serializable, DeserializationError};
use crate::effect::Effect;

use super::types::{AgentId, AgentType, AgentState, AgentRelationship, AgentError};
use super::agent::{Agent, AgentImpl};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Operator-specific error types
#[derive(Error, Debug)]
pub enum OperatorAgentError {
    /// Base agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    /// System operation error
    #[error("System operation error: {0}")]
    SystemOperationError(String),
    
    /// Maintenance error
    #[error("Maintenance error: {0}")]
    MaintenanceError(String),
    
    /// Other error
    #[error("Operator error: {0}")]
    Other(String),
}

/// Operator role within the system
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorRole {
    /// System administrator with full access
    SystemAdmin,
    
    /// Security administrator focused on security controls
    SecurityAdmin,
    
    /// Data administrator focused on data management
    DataAdmin,
    
    /// Network administrator focused on networking
    NetworkAdmin,
    
    /// Backup administrator focused on backups and recovery
    BackupAdmin,
    
    /// Monitoring administrator focused on system monitoring
    MonitoringAdmin,
    
    /// Custom role with specific description
    Custom(String),
}

/// System operation performed by an operator
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemOperation {
    /// Operation ID
    pub operation_id: String,
    
    /// Operation type
    pub operation_type: SystemOperationType,
    
    /// Target resource ID
    pub target_resource_id: Option<ResourceId>,
    
    /// Operation parameters
    pub parameters: HashMap<String, String>,
    
    /// Operation timestamp
    pub timestamp: u64,
    
    /// Operation result
    pub result: Option<SystemOperationResult>,
}

/// Type of system operation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemOperationType {
    /// Restart a service or system
    Restart,
    
    /// Update a component
    Update,
    
    /// Configure a component
    Configure,
    
    /// Backup a component
    Backup,
    
    /// Restore from backup
    Restore,
    
    /// Monitor system health
    Monitor,
    
    /// Deploy a new component
    Deploy,
    
    /// Upgrade a component
    Upgrade,
    
    /// Maintenance action
    Maintenance,
    
    /// Emergency action
    Emergency,
    
    /// Custom operation
    Custom(String),
}

/// Result of a system operation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemOperationResult {
    /// Status code
    pub status: SystemOperationStatus,
    
    /// Result message
    pub message: String,
    
    /// Result timestamp
    pub timestamp: u64,
    
    /// Result data
    pub data: HashMap<String, String>,
}

/// Status of a system operation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemOperationStatus {
    /// Operation succeeded
    Success,
    
    /// Operation failed
    Failure,
    
    /// Operation is in progress
    InProgress,
    
    /// Operation was canceled
    Canceled,
    
    /// Operation timed out
    Timeout,
}

/// Maintenance window for scheduled operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    /// Window ID
    pub window_id: String,
    
    /// Window title
    pub title: String,
    
    /// Window description
    pub description: String,
    
    /// Start timestamp
    pub start_time: u64,
    
    /// End timestamp
    pub end_time: u64,
    
    /// Affected resources
    pub affected_resources: Vec<ResourceId>,
    
    /// Operations scheduled during this window
    pub scheduled_operations: Vec<String>,
    
    /// Whether the window is active
    pub active: bool,
}

/// Operator agent that specializes the base agent for operator-specific functionality
#[derive(Clone, Debug)]
pub struct OperatorAgent {
    /// Base agent implementation
    base: AgentImpl,
    
    /// Operator role
    role: OperatorRole,
    
    /// System operations performed by this operator
    operations: Vec<SystemOperation>,
    
    /// Maintenance windows scheduled by this operator
    maintenance_windows: Vec<MaintenanceWindow>,
    
    /// Active operations
    active_operations: HashSet<String>,
    
    /// Emergency access flag
    emergency_access: bool,
    
    /// Certification level (0-5)
    certification_level: u8,
}

impl OperatorAgent {
    /// Create a new operator agent
    pub fn new(
        base_agent: AgentImpl,
        role: OperatorRole,
        certification_level: u8,
    ) -> Result<Self, OperatorAgentError> {
        // Verify that the base agent has the correct type
        if base_agent.agent_type() != &AgentType::Operator {
            return Err(OperatorAgentError::Other(
                "Base agent must have Operator agent type".to_string()
            ));
        }
        
        // Ensure certification level is between 0 and 5
        let cert_level = certification_level.min(5);
        
        Ok(Self {
            base: base_agent,
            role,
            operations: Vec::new(),
            maintenance_windows: Vec::new(),
            active_operations: HashSet::new(),
            emergency_access: false,
            certification_level: cert_level,
        })
    }
    
    /// Get the operator role
    pub fn role(&self) -> &OperatorRole {
        &self.role
    }
    
    /// Set the operator role
    pub async fn set_role(&mut self, role: OperatorRole) -> Result<(), OperatorAgentError> {
        self.role = role;
        
        // Update the content hash
        self.base.set_metadata("role_updated", &chrono::Utc::now().to_rfc3339()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Get the certification level
    pub fn certification_level(&self) -> u8 {
        self.certification_level
    }
    
    /// Set the certification level
    pub async fn set_certification_level(&mut self, level: u8) -> Result<(), OperatorAgentError> {
        // Ensure level is between 0 and 5
        self.certification_level = level.min(5);
        
        // Update the content hash
        self.base.set_metadata("certification_updated", &self.certification_level.to_string()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Check if the operator has emergency access
    pub fn has_emergency_access(&self) -> bool {
        self.emergency_access
    }
    
    /// Enable emergency access
    pub async fn enable_emergency_access(&mut self) -> Result<(), OperatorAgentError> {
        self.emergency_access = true;
        
        // Update the content hash
        self.base.set_metadata("emergency_access", "true").await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Disable emergency access
    pub async fn disable_emergency_access(&mut self) -> Result<(), OperatorAgentError> {
        self.emergency_access = false;
        
        // Update the content hash
        self.base.set_metadata("emergency_access", "false").await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Perform a system operation
    pub async fn perform_operation(
        &mut self,
        operation_type: SystemOperationType,
        target_resource_id: Option<ResourceId>,
        parameters: HashMap<String, String>,
    ) -> Result<String, OperatorAgentError> {
        // Check if the operator has the necessary certification level for this operation type
        let required_level = match operation_type {
            SystemOperationType::Restart => 2,
            SystemOperationType::Update => 3,
            SystemOperationType::Configure => 2,
            SystemOperationType::Backup => 1,
            SystemOperationType::Restore => 3,
            SystemOperationType::Monitor => 1,
            SystemOperationType::Deploy => 4,
            SystemOperationType::Upgrade => 4,
            SystemOperationType::Maintenance => 2,
            SystemOperationType::Emergency => 5,
            SystemOperationType::Custom(_) => 3,
        };
        
        // Verify certification level, unless emergency access is enabled
        if !self.emergency_access && self.certification_level < required_level {
            return Err(OperatorAgentError::AuthorizationError(
                format!("Insufficient certification level for this operation. Required: {}, Current: {}",
                    required_level, self.certification_level)
            ));
        }
        
        // Generate an operation ID
        let operation_id = format!("operation-{}", uuid::Uuid::new_v4());
        
        // Create a new operation
        let operation = SystemOperation {
            operation_id: operation_id.clone(),
            operation_type,
            target_resource_id,
            parameters,
            timestamp: chrono::Utc::now().timestamp() as u64,
            result: None,
        };
        
        // Add the operation
        self.operations.push(operation);
        self.active_operations.insert(operation_id.clone());
        
        // Update the content hash
        self.base.set_metadata("operation_started", &chrono::Utc::now().to_rfc3339()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(operation_id)
    }
    
    /// Complete a system operation
    pub async fn complete_operation(
        &mut self,
        operation_id: &str,
        status: SystemOperationStatus,
        message: String,
        data: HashMap<String, String>,
    ) -> Result<(), OperatorAgentError> {
        // Find the operation
        let operation = self.operations.iter_mut()
            .find(|o| o.operation_id == operation_id)
            .ok_or_else(|| OperatorAgentError::SystemOperationError(
                format!("Operation {} not found", operation_id)
            ))?;
        
        // Create the result
        let result = SystemOperationResult {
            status,
            message,
            timestamp: chrono::Utc::now().timestamp() as u64,
            data,
        };
        
        // Update the operation
        operation.result = Some(result);
        
        // Remove from active operations if completed
        if status != SystemOperationStatus::InProgress {
            self.active_operations.remove(operation_id);
        }
        
        // Update the content hash
        self.base.set_metadata("operation_completed", &chrono::Utc::now().to_rfc3339()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Get a system operation by ID
    pub fn get_operation(&self, operation_id: &str) -> Option<&SystemOperation> {
        self.operations.iter().find(|o| o.operation_id == operation_id)
    }
    
    /// Get all operations
    pub fn operations(&self) -> &[SystemOperation] {
        &self.operations
    }
    
    /// Get active operations
    pub fn active_operations(&self) -> Vec<&SystemOperation> {
        self.operations.iter()
            .filter(|o| self.active_operations.contains(&o.operation_id))
            .collect()
    }
    
    /// Create a maintenance window
    pub async fn create_maintenance_window(
        &mut self,
        title: String,
        description: String,
        start_time: u64,
        end_time: u64,
        affected_resources: Vec<ResourceId>,
    ) -> Result<String, OperatorAgentError> {
        // Validate time range
        if end_time <= start_time {
            return Err(OperatorAgentError::MaintenanceError(
                "End time must be after start time".to_string()
            ));
        }
        
        // Generate a window ID
        let window_id = format!("window-{}", uuid::Uuid::new_v4());
        
        // Create a new maintenance window
        let window = MaintenanceWindow {
            window_id: window_id.clone(),
            title,
            description,
            start_time,
            end_time,
            affected_resources,
            scheduled_operations: Vec::new(),
            active: false,
        };
        
        // Add the window
        self.maintenance_windows.push(window);
        
        // Update the content hash
        self.base.set_metadata("window_created", &chrono::Utc::now().to_rfc3339()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(window_id)
    }
    
    /// Activate a maintenance window
    pub async fn activate_maintenance_window(&mut self, window_id: &str) -> Result<(), OperatorAgentError> {
        // Find the window
        let window = self.maintenance_windows.iter_mut()
            .find(|w| w.window_id == window_id)
            .ok_or_else(|| OperatorAgentError::MaintenanceError(
                format!("Maintenance window {} not found", window_id)
            ))?;
        
        // Check if the window can be activated
        let now = chrono::Utc::now().timestamp() as u64;
        if now > window.end_time {
            return Err(OperatorAgentError::MaintenanceError(
                "Cannot activate a maintenance window that has already ended".to_string()
            ));
        }
        
        // Activate the window
        window.active = true;
        
        // Update the content hash
        self.base.set_metadata("window_activated", &chrono::Utc::now().to_rfc3339()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Deactivate a maintenance window
    pub async fn deactivate_maintenance_window(&mut self, window_id: &str) -> Result<(), OperatorAgentError> {
        // Find the window
        let window = self.maintenance_windows.iter_mut()
            .find(|w| w.window_id == window_id)
            .ok_or_else(|| OperatorAgentError::MaintenanceError(
                format!("Maintenance window {} not found", window_id)
            ))?;
        
        // Deactivate the window
        window.active = false;
        
        // Update the content hash
        self.base.set_metadata("window_deactivated", &chrono::Utc::now().to_rfc3339()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Schedule an operation within a maintenance window
    pub async fn schedule_operation(
        &mut self,
        window_id: &str,
        operation_id: &str,
    ) -> Result<(), OperatorAgentError> {
        // Find the window
        let window = self.maintenance_windows.iter_mut()
            .find(|w| w.window_id == window_id)
            .ok_or_else(|| OperatorAgentError::MaintenanceError(
                format!("Maintenance window {} not found", window_id)
            ))?;
        
        // Check if the operation exists
        if !self.operations.iter().any(|o| o.operation_id == operation_id) {
            return Err(OperatorAgentError::SystemOperationError(
                format!("Operation {} not found", operation_id)
            ));
        }
        
        // Add the operation to the window
        window.scheduled_operations.push(operation_id.to_string());
        
        // Update the content hash
        self.base.set_metadata("operation_scheduled", &chrono::Utc::now().to_rfc3339()).await
            .map_err(OperatorAgentError::AgentError)?;
        
        Ok(())
    }
    
    /// Get a maintenance window by ID
    pub fn get_maintenance_window(&self, window_id: &str) -> Option<&MaintenanceWindow> {
        self.maintenance_windows.iter().find(|w| w.window_id == window_id)
    }
    
    /// Get all maintenance windows
    pub fn maintenance_windows(&self) -> &[MaintenanceWindow] {
        &self.maintenance_windows
    }
    
    /// Get active maintenance windows
    pub fn active_maintenance_windows(&self) -> Vec<&MaintenanceWindow> {
        let now = chrono::Utc::now().timestamp() as u64;
        self.maintenance_windows.iter()
            .filter(|w| w.active && w.start_time <= now && now <= w.end_time)
            .collect()
    }
    
    /// Check if a maintenance window is active for a resource
    pub fn is_maintenance_active_for_resource(&self, resource_id: &ResourceId) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        self.maintenance_windows.iter().any(|w| {
            w.active && 
            w.start_time <= now && 
            now <= w.end_time && 
            w.affected_resources.iter().any(|r| r == resource_id)
        })
    }
}

// Delegate the Agent trait methods to the base agent
#[async_trait]
impl Agent for OperatorAgent {
    fn agent_id(&self) -> &AgentId {
        self.base.agent_id()
    }
    
    fn agent_type(&self) -> &AgentType {
        self.base.agent_type()
    }
    
    fn state(&self) -> &AgentState {
        self.base.state()
    }
    
    async fn set_state(&mut self, state: AgentState) -> Result<(), AgentError> {
        self.base.set_state(state).await
    }
    
    async fn add_capability(&mut self, capability: Capability) -> Result<(), AgentError> {
        self.base.add_capability(capability).await
    }
    
    async fn remove_capability(&mut self, capability_id: &str) -> Result<(), AgentError> {
        self.base.remove_capability(capability_id).await
    }
    
    fn has_capability(&self, capability_id: &str) -> bool {
        self.base.has_capability(capability_id)
    }
    
    fn capabilities(&self) -> Vec<Capability> {
        self.base.capabilities()
    }
    
    async fn add_relationship(&mut self, relationship: AgentRelationship) -> Result<(), AgentError> {
        self.base.add_relationship(relationship).await
    }
    
    async fn remove_relationship(&mut self, target_id: &ResourceId) -> Result<(), AgentError> {
        self.base.remove_relationship(target_id).await
    }
    
    fn relationships(&self) -> Vec<AgentRelationship> {
        self.base.relationships()
    }
    
    fn get_relationship(&self, target_id: &ResourceId) -> Option<&AgentRelationship> {
        self.base.get_relationship(target_id)
    }
    
    fn clone_agent(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}

// Implement the Resource trait through delegation to the base agent
impl crate::resource::Resource for OperatorAgent {
    fn id(&self) -> &ResourceId {
        self.base.id()
    }
    
    fn resource_type(&self) -> ResourceType {
        self.base.resource_type()
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        self.base.metadata()
    }
    
    fn metadata_mut(&mut self) -> &mut HashMap<String, String> {
        self.base.metadata_mut()
    }
    
    fn clone_resource(&self) -> Box<dyn crate::resource::Resource> {
        Box::new(self.clone())
    }
}

/// Builder for creating operator agents
pub struct OperatorAgentBuilder {
    /// Base agent builder
    base_builder: super::agent::AgentBuilder,
    
    /// Operator role
    role: Option<OperatorRole>,
    
    /// Certification level
    certification_level: u8,
    
    /// Emergency access
    emergency_access: bool,
}

impl OperatorAgentBuilder {
    /// Create a new operator agent builder
    pub fn new() -> Self {
        Self {
            base_builder: super::agent::AgentBuilder::new().agent_type(AgentType::Operator),
            role: None,
            certification_level: 0,
            emergency_access: false,
        }
    }
    
    /// Set the agent state
    pub fn state(mut self, state: AgentState) -> Self {
        self.base_builder = self.base_builder.state(state);
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.base_builder = self.base_builder.with_capability(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.base_builder = self.base_builder.with_capabilities(capabilities);
        self
    }
    
    /// Add a relationship
    pub fn with_relationship(mut self, relationship: AgentRelationship) -> Self {
        self.base_builder = self.base_builder.with_relationship(relationship);
        self
    }
    
    /// Add multiple relationships
    pub fn with_relationships(mut self, relationships: Vec<AgentRelationship>) -> Self {
        self.base_builder = self.base_builder.with_relationships(relationships);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.base_builder = self.base_builder.with_metadata(key, value);
        self
    }
    
    /// Set the operator role
    pub fn with_role(mut self, role: OperatorRole) -> Self {
        self.role = Some(role);
        self
    }
    
    /// Set the certification level
    pub fn with_certification_level(mut self, level: u8) -> Self {
        self.certification_level = level.min(5);
        self
    }
    
    /// Enable emergency access
    pub fn with_emergency_access(mut self, enabled: bool) -> Self {
        self.emergency_access = enabled;
        self
    }
    
    /// Build the operator agent
    pub fn build(self) -> Result<OperatorAgent, OperatorAgentError> {
        // Build the base agent
        let base_agent = self.base_builder.build()
            .map_err(OperatorAgentError::AgentError)?;
        
        // Create the operator agent
        let role = self.role.ok_or_else(|| {
            OperatorAgentError::Other("Operator role is required".to_string())
        })?;
        
        let mut operator = OperatorAgent::new(
            base_agent,
            role,
            self.certification_level,
        )?;
        
        // Set emergency access if needed
        if self.emergency_access {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    operator.enable_emergency_access().await
                })
            })?;
        }
        
        Ok(operator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_operator_creation() {
        // Create an operator agent
        let operator = OperatorAgentBuilder::new()
            .state(AgentState::Active)
            .with_role(OperatorRole::SystemAdmin)
            .with_certification_level(3)
            .build()
            .unwrap();
        
        // Check the agent type
        assert_eq!(operator.agent_type(), &AgentType::Operator);
        
        // Check the role
        assert!(matches!(operator.role(), OperatorRole::SystemAdmin));
        
        // Check the certification level
        assert_eq!(operator.certification_level(), 3);
    }
    
    #[tokio::test]
    async fn test_system_operations() {
        // Create an operator agent
        let mut operator = OperatorAgentBuilder::new()
            .state(AgentState::Active)
            .with_role(OperatorRole::SystemAdmin)
            .with_certification_level(5)
            .build()
            .unwrap();
        
        // Create a system operation
        let mut params = HashMap::new();
        params.insert("service".to_string(), "database".to_string());
        
        let operation_id = operator.perform_operation(
            SystemOperationType::Restart,
            None,
            params,
        ).await.unwrap();
        
        // Check active operations
        assert_eq!(operator.active_operations().len(), 1);
        
        // Complete the operation
        let mut result_data = HashMap::new();
        result_data.insert("restart_time".to_string(), "5s".to_string());
        
        operator.complete_operation(
            &operation_id,
            SystemOperationStatus::Success,
            "Service restarted successfully".to_string(),
            result_data,
        ).await.unwrap();
        
        // Check active operations again
        assert_eq!(operator.active_operations().len(), 0);
        
        // Check the operation result
        let operation = operator.get_operation(&operation_id).unwrap();
        assert!(matches!(operation.result.as_ref().unwrap().status, SystemOperationStatus::Success));
    }
    
    #[tokio::test]
    async fn test_maintenance_windows() {
        // Create an operator agent
        let mut operator = OperatorAgentBuilder::new()
            .state(AgentState::Active)
            .with_role(OperatorRole::SystemAdmin)
            .with_certification_level(3)
            .build()
            .unwrap();
        
        // Create a maintenance window
        let now = chrono::Utc::now().timestamp() as u64;
        let start_time = now - 3600; // 1 hour ago
        let end_time = now + 3600; // 1 hour from now
        
        let resource_id = ResourceId::new(ContentHash::default());
        
        let window_id = operator.create_maintenance_window(
            "System Upgrade".to_string(),
            "Upgrading the database system".to_string(),
            start_time,
            end_time,
            vec![resource_id.clone()],
        ).await.unwrap();
        
        // Activate the window
        operator.activate_maintenance_window(&window_id).await.unwrap();
        
        // Check active windows
        assert_eq!(operator.active_maintenance_windows().len(), 1);
        
        // Check if maintenance is active for the resource
        assert!(operator.is_maintenance_active_for_resource(&resource_id));
        
        // Create an operation
        let mut params = HashMap::new();
        params.insert("service".to_string(), "database".to_string());
        
        let operation_id = operator.perform_operation(
            SystemOperationType::Upgrade,
            Some(resource_id.clone()),
            params,
        ).await.unwrap();
        
        // Schedule the operation in the window
        operator.schedule_operation(&window_id, &operation_id).await.unwrap();
        
        // Check that the operation is scheduled in the window
        let window = operator.get_maintenance_window(&window_id).unwrap();
        assert_eq!(window.scheduled_operations.len(), 1);
        assert_eq!(&window.scheduled_operations[0], &operation_id);
    }
    
    #[tokio::test]
    async fn test_emergency_access() {
        // Create an operator agent
        let mut operator = OperatorAgentBuilder::new()
            .state(AgentState::Active)
            .with_role(OperatorRole::SystemAdmin)
            .with_certification_level(1) // Too low for emergency operations
            .build()
            .unwrap();
        
        // Try to perform an emergency operation (should fail)
        let mut params = HashMap::new();
        params.insert("reason".to_string(), "critical outage".to_string());
        
        let result = operator.perform_operation(
            SystemOperationType::Emergency,
            None,
            params.clone(),
        ).await;
        
        // Should fail due to insufficient certification level
        assert!(result.is_err());
        
        // Enable emergency access
        operator.enable_emergency_access().await.unwrap();
        
        // Now try the operation again
        let operation_id = operator.perform_operation(
            SystemOperationType::Emergency,
            None,
            params,
        ).await.unwrap();
        
        // Should succeed
        assert!(!operation_id.is_empty());
    }
} 