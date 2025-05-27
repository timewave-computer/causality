// Resource state management
//
// This file defines the state management interfaces and types for resources.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use super::types::{ResourceId, ResourceType, ResourceTag, ResourceState};

// Define ResourceResult here since interface.rs is gone
pub type ResourceResult<T> = Result<T, String>;

/// Resource state data
///
/// Contains all state information for a resource, including its
/// attributes, metadata, and current lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStateData {
    /// Resource identifier
    pub id: ResourceId,
    
    /// Resource type
    pub resource_type: ResourceType,
    
    /// Current lifecycle state
    pub state: ResourceState,
    
    /// Resource attributes
