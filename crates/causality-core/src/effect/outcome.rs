// Effect Outcome
//
// This module provides types for effect execution outcomes, including success,
// failure, and other result states.

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use serde::{Serialize, Deserialize};

use super::types::EffectId;
use causality_types::ContentId;
use crate::resource::types::ResourceId;

/// Status of an effect execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectStatus {
    /// Effect executed successfully
    Success,
    /// Effect execution failed
    Failure,
    /// Effect execution is pending or in progress
    Pending,
    /// Effect execution was cancelled
    Cancelled,
    /// Effect is waiting for input
    Waiting,
    /// Effect execution timed out
    Timeout,
}

impl Display for EffectStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EffectStatus::Success => write!(f, "success"),
            EffectStatus::Failure => write!(f, "failure"),
            EffectStatus::Pending => write!(f, "pending"),
            EffectStatus::Cancelled => write!(f, "cancelled"),
            EffectStatus::Waiting => write!(f, "waiting"),
            EffectStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Structured result data for effect outcomes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResultData {
    /// No data (void)
    None,
    
    /// Boolean result
    Boolean(bool),
    
    /// Numeric result
    Number(f64),
    
    /// String result
    String(String),
    
    /// Resource ID result
    Resource(ResourceId),
    
    /// Multiple resource IDs
    Resources(Vec<ResourceId>),
    
    /// Key-value map result
    Map(HashMap<String, String>),
    
    /// Binary data result
    Binary(Vec<u8>),
    
    /// JSON serialized data
    Json(String),
    
    /// Content-addressed reference
    ContentRef(String),
}

impl ResultData {
    /// Check if result is empty/none
    pub fn is_none(&self) -> bool {
        matches!(self, ResultData::None)
    }
    
    /// Convert to boolean if possible
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ResultData::Boolean(b) => Some(*b),
            ResultData::Number(n) => Some(*n != 0.0),
            ResultData::String(s) => s.parse().ok(),
            ResultData::Resources(r) => Some(!r.is_empty()),
            ResultData::Map(m) => Some(!m.is_empty()),
            ResultData::Binary(b) => Some(!b.is_empty()),
            _ => None,
        }
    }
    
    /// Convert to number if possible
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ResultData::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            ResultData::Number(n) => Some(*n),
            ResultData::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// Convert to string if possible
    pub fn as_string(&self) -> Option<String> {
        match self {
            ResultData::Boolean(b) => Some(b.to_string()),
            ResultData::Number(n) => Some(n.to_string()),
            ResultData::String(s) => Some(s.clone()),
            ResultData::Resource(r) => Some(r.to_string()),
            ResultData::Json(j) => Some(j.clone()),
            ResultData::ContentRef(c) => Some(c.clone()),
            _ => None,
        }
    }
    
    /// Convert to resource ID if possible
    pub fn as_resource(&self) -> Option<ResourceId> {
        match self {
            ResultData::Resource(r) => Some(r.clone()),
            ResultData::Resources(r) => Some(r[0].clone()),
            ResultData::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// Convert to multiple resource IDs if possible
    pub fn as_resources(&self) -> Option<Vec<ResourceId>> {
        match self {
            ResultData::Resource(r) => Some(vec![r.clone()]),
            ResultData::Resources(r) => Some(r.clone()),
            _ => None,
        }
    }
    
    /// Convert to a HashMap if possible
    pub fn as_map(&self) -> Option<&HashMap<String, String>> {
        match self {
            ResultData::Map(m) => Some(m),
            _ => None,
        }
    }
    
    /// Get content reference if available
    pub fn as_content_ref(&self) -> Option<&str> {
        match self {
            ResultData::ContentRef(c) => Some(c),
            _ => None,
        }
    }
    
    /// Convert from a string data map
    pub fn from_map(map: HashMap<String, String>) -> Self {
        ResultData::Map(map)
    }
}

/// Enhanced effect execution outcome with content addressing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectOutcome {
    /// Optional effect ID if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_id: Option<EffectId>,
    
    /// Status of the effect execution
    pub status: EffectStatus,
    
    /// Data produced by the effect (for backward compatibility)
    pub data: HashMap<String, String>,
    
    /// Structured result data (enhanced)
    #[serde(skip_serializing_if = "ResultData::is_none")]
    pub result: ResultData,
    
    /// Error message if execution failed
    pub error_message: Option<String>,
    
    /// Content IDs of resources affected by this effect
    pub affected_resources: Vec<ContentId>,
    
    /// Child effect outcomes for composite effects
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub child_outcomes: Vec<EffectOutcome>,
    
    /// Content hash for this outcome (computed when needed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

impl EffectOutcome {
    /// Create a successful outcome
    pub fn success(data: HashMap<String, String>) -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Success,
            data,
            result: ResultData::None,
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Create a successful outcome with data map
    pub fn success_with_data(data: HashMap<String, String>) -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Success,
            data: data.clone(),
            result: ResultData::Map(data),
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Create a successful outcome with structured result
    pub fn success_with_result(result: ResultData) -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Success,
            data: HashMap::new(),
            result,
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Create a failure outcome
    pub fn failure(error_message: String) -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Failure,
            data: HashMap::new(),
            result: ResultData::None,
            error_message: Some(error_message),
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Create a pending outcome
    pub fn pending() -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Pending,
            data: HashMap::new(),
            result: ResultData::None,
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Create a cancelled outcome
    pub fn cancelled(reason: Option<String>) -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Cancelled,
            data: HashMap::new(),
            result: ResultData::None,
            error_message: reason,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Create a waiting outcome
    pub fn waiting(message: String) -> Self {
        let mut data = HashMap::new();
        data.insert("waiting_message".to_string(), message.clone());
        
        Self {
            effect_id: None,
            status: EffectStatus::Waiting,
            data,
            result: ResultData::String(message),
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Create a timeout outcome
    pub fn timeout(message: String) -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Timeout,
            data: HashMap::new(),
            result: ResultData::None,
            error_message: Some(message),
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }
    
    /// Set the effect ID
    pub fn with_effect_id(mut self, id: EffectId) -> Self {
        self.effect_id = Some(id);
        self
    }
    
    /// Check if the outcome was successful
    pub fn is_success(&self) -> bool {
        self.status == EffectStatus::Success
    }
    
    /// Check if the outcome was a failure
    pub fn is_failure(&self) -> bool {
        self.status == EffectStatus::Failure
    }
    
    /// Check if the outcome is pending
    pub fn is_pending(&self) -> bool {
        self.status == EffectStatus::Pending
    }
    
    /// Check if the outcome is waiting
    pub fn is_waiting(&self) -> bool {
        self.status == EffectStatus::Waiting
    }
    
    /// Check if the outcome was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.status == EffectStatus::Cancelled
    }
    
    /// Check if the outcome timed out
    pub fn is_timeout(&self) -> bool {
        self.status == EffectStatus::Timeout
    }
    
    /// Add an affected resource
    pub fn with_affected_resource(mut self, resource_id: ContentId) -> Self {
        self.affected_resources.push(resource_id);
        self
    }
    
    /// Add multiple affected resources
    pub fn with_affected_resources(mut self, resource_ids: Vec<ContentId>) -> Self {
        self.affected_resources.extend(resource_ids);
        self
    }
    
    /// Add data to the outcome (updates both data and result)
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let k = key.into();
        let v = value.into();
        self.data.insert(k.clone(), v.clone());
        
        // If result is a map, update it too
        if let ResultData::Map(ref mut map) = self.result {
            map.insert(k, v);
        } else if self.result.is_none() {
            // If no result yet, create a map
            let mut map = HashMap::new();
            map.insert(k, v);
            self.result = ResultData::Map(map);
        }
        
        self
    }
    
    /// Add multiple data entries (updates both data and result)
    pub fn with_data_map(mut self, data: HashMap<String, String>) -> Self {
        self.data.extend(data.clone());
        
        // If result is a map, update it too
        if let ResultData::Map(ref mut map) = self.result {
            map.extend(data);
        } else if self.result.is_none() {
            // If no result yet, create a map
            self.result = ResultData::Map(data);
        }
        
        self
    }
    
    /// Set the structured result directly
    pub fn with_result(mut self, result: ResultData) -> Self {
        // Also update the data map for backward compatibility
        match &result {
            ResultData::Map(m) => {
                self.data.extend(m.clone());
            },
            ResultData::String(s) => {
                self.data.insert("value".to_string(), s.clone());
            },
            ResultData::Boolean(b) => {
                self.data.insert("value".to_string(), b.to_string());
            },
            ResultData::Number(n) => {
                self.data.insert("value".to_string(), n.to_string());
            },
            ResultData::Resource(r) => {
                self.data.insert("resource_id".to_string(), r.to_string());
            },
            _ => {}
        }
        
        self.result = result;
        self
    }
    
    /// Add a child outcome
    pub fn with_child_outcome(mut self, outcome: EffectOutcome) -> Self {
        self.child_outcomes.push(outcome);
        self
    }
    
    /// Add multiple child outcomes
    pub fn with_child_outcomes(mut self, outcomes: Vec<EffectOutcome>) -> Self {
        self.child_outcomes.extend(outcomes);
        self
    }
    
    /// Compute or get the content hash of this outcome
    pub fn content_hash(&mut self) -> String {
        if let Some(ref hash) = self.content_hash {
            return hash.clone();
        }
        
        // Compute a deterministic hash of the outcome's content
        // In a real implementation, this would use a proper hashing algorithm
        let hash = format!("{:?}:{:?}:{:?}", self.effect_id, self.status, self.result);
        self.content_hash = Some(hash.clone());
        hash
    }
    
    /// Create a summary of this outcome
    pub fn summary(&self) -> String {
        let effect_id = self.effect_id.as_ref().map_or("unknown".to_string(), |id| id.to_string());
        
        let result_desc = match &self.result {
            ResultData::None => "None".to_string(),
            ResultData::Boolean(b) => format!("Boolean({})", b),
            ResultData::Number(n) => format!("Number({})", n),
            ResultData::String(s) => {
                if s.len() > 30 {
                    format!("String({:.30}...)", s)
                } else {
                    format!("String({})", s)
                }
            },
            ResultData::Resource(r) => format!("Resource({})", r),
            ResultData::Resources(r) => format!("Resources(count={})", r.len()),
            ResultData::Map(m) => format!("Map(count={})", m.len()),
            ResultData::Binary(b) => format!("Binary(size={})", b.len()),
            ResultData::Json(j) => format!("Json(len={})", j.len()),
            ResultData::ContentRef(c) => format!("ContentRef({})", c),
        };
        
        let error = self.error_message.as_ref().map_or("None".to_string(), |e| {
            if e.len() > 50 {
                format!("{:.50}...", e)
            } else {
                e.clone()
            }
        });
        
        format!(
            "Effect[{}] {} - Result: {}, Error: {}, Resources: {}, Children: {}",
            effect_id,
            self.status,
            result_desc,
            error,
            self.affected_resources.len(),
            self.child_outcomes.len()
        )
    }

    /// Create a success outcome with data
    pub fn success_from_result(data: Box<ResultData>) -> Self {
        EffectOutcome::success_with_result(*data)
    }
    
    /// Create an error outcome
    pub fn error(error: Box<crate::effect::EffectError>) -> Self {
        EffectOutcome::failure(error.to_string())
    }

    /// Get the data from this outcome
    pub fn get_data(&self) -> Option<&HashMap<String, String>> {
        Some(&self.data)
    }
    
    /// Get the error message from this outcome
    pub fn get_error(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    /// Create a failure outcome with data
    pub fn failure_with_data(error: impl ToString, data: HashMap<String, String>) -> Self {
        let mut data_map = data;
        data_map.insert("error".to_string(), error.to_string());
        data_map.insert("status".to_string(), "failure".to_string());
        
        let result_data = ResultData::Map(data_map);
        let data_map = if let ResultData::Map(ref m) = result_data {
            m.clone()
        } else {
            HashMap::new() // Should never reach here
        };
        
        Self {
            effect_id: None,
            status: EffectStatus::Failure,
            data: data_map,
            result: result_data,
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        }
    }

    /// Create an error outcome with an error message and data
    pub fn error_with_data(error: impl ToString, data: HashMap<String, String>) -> Self {
        Self::failure_with_data(error, data)
    }

    /// Create a failure outcome from an error
    pub fn from_error<E: std::fmt::Display>(error: E, additional_data: Option<HashMap<String, String>>) -> Self {
        use crate::effect::utils::error_to_map;
        
        let mut data = error_to_map(error);
        
        // Merge additional data if provided
        if let Some(extra_data) = additional_data {
            for (key, value) in extra_data {
                data.insert(key, value);
            }
        }
        
        Self::failure_with_data("Operation failed", data)
    }
}

/// Builder for effect outcomes
#[derive(Debug)]
pub struct EffectOutcomeBuilder {
    effect_id: Option<EffectId>,
    status: EffectStatus,
    data: HashMap<String, String>,
    result: ResultData,
    error_message: Option<String>,
    affected_resources: Vec<ContentId>,
    child_outcomes: Vec<EffectOutcome>,
}

impl EffectOutcomeBuilder {
    /// Create a new outcome builder
    pub fn new() -> Self {
        Self {
            effect_id: None,
            status: EffectStatus::Success,
            data: HashMap::new(),
            result: ResultData::None,
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
        }
    }
    
    /// Set the effect ID
    pub fn effect_id(mut self, id: EffectId) -> Self {
        self.effect_id = Some(id);
        self
    }
    
    /// Set the status
    pub fn status(mut self, status: EffectStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Set as successful
    pub fn success(mut self) -> Self {
        self.status = EffectStatus::Success;
        self.error_message = None;
        self
    }
    
    /// Set as failed
    pub fn failure(mut self, message: impl Into<String>) -> Self {
        self.status = EffectStatus::Failure;
        self.error_message = Some(message.into());
        self
    }
    
    /// Set as pending
    pub fn pending(mut self) -> Self {
        self.status = EffectStatus::Pending;
        self.error_message = None;
        self
    }
    
    /// Set as waiting
    pub fn waiting(mut self, message: impl Into<String>) -> Self {
        self.status = EffectStatus::Waiting;
        let msg = message.into();
        self.data.insert("waiting_message".to_string(), msg.clone());
        self.result = ResultData::String(msg);
        self.error_message = None;
        self
    }
    
    /// Set as cancelled
    pub fn cancelled(mut self, reason: impl Into<String>) -> Self {
        self.status = EffectStatus::Cancelled;
        self.error_message = Some(reason.into());
        self
    }
    
    /// Set as timed out
    pub fn timeout(mut self, reason: impl Into<String>) -> Self {
        self.status = EffectStatus::Timeout;
        self.error_message = Some(reason.into());
        self
    }
    
    /// Add data
    pub fn data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let k = key.into();
        let v = value.into();
        self.data.insert(k.clone(), v.clone());
        
        // If result is a map, update it too
        if let ResultData::Map(ref mut map) = self.result {
            map.insert(k, v);
        } else if self.result.is_none() {
            // If no result yet, create a map
            let mut map = HashMap::new();
            map.insert(k, v);
            self.result = ResultData::Map(map);
        }
        
        self
    }
    
    /// Add multiple data entries
    pub fn data_map(mut self, data: HashMap<String, String>) -> Self {
        self.data.extend(data.clone());
        
        // If result is a map, update it too
        if let ResultData::Map(ref mut map) = self.result {
            map.extend(data);
        } else if self.result.is_none() {
            // If no result yet, create a map
            self.result = ResultData::Map(data);
        }
        
        self
    }
    
    /// Set the result
    pub fn result(mut self, result: ResultData) -> Self {
        self.result = result;
        self
    }
    
    /// Set a boolean result
    pub fn boolean_result(mut self, value: bool) -> Self {
        self.result = ResultData::Boolean(value);
        self.data.insert("value".to_string(), value.to_string());
        self
    }
    
    /// Set a numeric result
    pub fn number_result(mut self, value: f64) -> Self {
        self.result = ResultData::Number(value);
        self.data.insert("value".to_string(), value.to_string());
        self
    }
    
    /// Set a string result
    pub fn string_result(mut self, value: impl Into<String>) -> Self {
        let s = value.into();
        self.result = ResultData::String(s.clone());
        self.data.insert("value".to_string(), s);
        self
    }
    
    /// Set a resource ID result
    pub fn resource_result(mut self, resource_id: ResourceId) -> Self {
        self.result = ResultData::Resource(resource_id.clone());
        self.data.insert("resource_id".to_string(), resource_id.to_string());
        self
    }
    
    /// Set multiple resource IDs as result
    pub fn resources_result(mut self, resource_ids: Vec<ResourceId>) -> Self {
        self.result = ResultData::Resources(resource_ids);
        self
    }
    
    /// Add an affected resource
    pub fn affected_resource(mut self, resource_id: ContentId) -> Self {
        self.affected_resources.push(resource_id);
        self
    }
    
    /// Add multiple affected resources
    pub fn affected_resources(mut self, resource_ids: Vec<ContentId>) -> Self {
        self.affected_resources.extend(resource_ids);
        self
    }
    
    /// Add a child outcome
    pub fn child_outcome(mut self, outcome: EffectOutcome) -> Self {
        self.child_outcomes.push(outcome);
        self
    }
    
    /// Add multiple child outcomes
    pub fn child_outcomes(mut self, outcomes: Vec<EffectOutcome>) -> Self {
        self.child_outcomes.extend(outcomes);
        self
    }
    
    /// Build the effect outcome
    pub fn build(self) -> EffectOutcome {
        EffectOutcome {
            effect_id: self.effect_id,
            status: self.status,
            data: self.data,
            result: self.result,
            error_message: self.error_message,
            affected_resources: self.affected_resources,
            child_outcomes: self.child_outcomes,
            content_hash: None,
        }
    }
}

impl Default for EffectOutcomeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type for effect operations
pub type EffectResult<T> = Result<T, crate::effect::EffectError>; 