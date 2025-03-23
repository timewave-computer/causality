use std::collections::HashMap;
use uuid::Uuid;

/// Represents the outcome of executing an effect
#[derive(Debug, Clone)]
pub struct EffectOutcome {
    /// Whether the effect execution was successful
    pub success: bool,
    
    /// Effect identifier
    pub effect_id: String,
    
    /// Error message (if any)
    pub error: Option<String>,
    
    /// Additional result data
    pub data: HashMap<String, String>,
    
    /// Execution context ID
    pub execution_id: Option<Uuid>,
    
    /// Resource changes
    pub resource_changes: Vec<String>,
}

impl EffectOutcome {
    /// Create a new empty effect outcome
    pub fn new() -> Self {
        Self {
            success: false,
            effect_id: String::new(),
            error: None,
            data: HashMap::new(),
            execution_id: None,
            resource_changes: Vec::new(),
        }
    }
    
    /// Create a successful effect outcome
    pub fn success(effect_id: impl Into<String>) -> Self {
        Self {
            success: true,
            effect_id: effect_id.into(),
            error: None,
            data: HashMap::new(),
            execution_id: None,
            resource_changes: Vec::new(),
        }
    }
    
    /// Create a failed effect outcome
    pub fn failure(effect_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            effect_id: effect_id.into(),
            error: Some(error.into()),
            data: HashMap::new(),
            execution_id: None,
            resource_changes: Vec::new(),
        }
    }
    
    /// Set the success status
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }
    
    /// Set the effect ID
    pub fn with_effect_id(mut self, effect_id: impl Into<String>) -> Self {
        self.effect_id = effect_id.into();
        self
    }
    
    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
    
    /// Add a data entry
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
    
    /// Add a resource change
    pub fn with_resource_change(mut self, change: impl Into<String>) -> Self {
        self.resource_changes.push(change.into());
        self
    }
    
    /// Set the execution context ID
    pub fn with_execution_id(mut self, execution_id: Uuid) -> Self {
        self.execution_id = Some(execution_id);
        self
    }
    
    /// Get a data value by key
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    
    /// Check if the outcome has an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

impl Default for EffectOutcome {
    fn default() -> Self {
        Self::new()
    }
} 