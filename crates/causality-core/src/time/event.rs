// Event and Timer definitions

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use crate::resource::ResourceId;
use crate::effect::types::EffectId;
use crate::utils::content_addressing::hash_string;
use causality_types::crypto_primitives::ContentId;
use blake3;
use causality_types;

// Placeholder for TimeEvent and TimeEventKind if needed
pub enum TimeEventKind {
    TimerExpired,
    TimeAdvanced,
    // Add other kinds
}

pub struct TimeEvent {
    pub kind: TimeEventKind,
    pub timestamp: DateTime<Utc>,
    // Add other relevant fields
}

/// Represents a scheduled timer
#[derive(Debug, Clone, Serialize, Deserialize)] // Added Serialize/Deserialize
pub struct Timer {
    pub id: String,
    pub resource_id: ResourceId,
    pub scheduled_at: DateTime<Utc>,
    pub duration: Duration, // Use chrono::Duration
    pub recurring: bool,
    pub callback_effect: Option<EffectId>, // ID of the effect to trigger
}

impl Timer {
    /// Create a new timer for a domain
    pub fn new(domain_id: &str) -> Self {
        // Generate a content-addressed timer ID
        let seed = format!("timer_{}", domain_id);
        let content_id = ContentId::from_bytes(seed.as_bytes());
        let timer_id = format!("timer:{}", content_id);
        
        // Use a simple string hash approach rather than ContentHash
        let domain_bytes = domain_id.as_bytes();
        let hash_result = blake3::hash(domain_bytes);
        let hash_bytes = hash_result.as_bytes().to_vec();
        let domain_hash = causality_types::crypto_primitives::ContentHash::new("blake3", hash_bytes);
        let resource_id = ResourceId::new(domain_hash);
        
        Self {
            id: timer_id,
            resource_id,
            scheduled_at: Utc::now(),
            duration: Duration::seconds(0),
            recurring: false,
            callback_effect: None,
        }
    }
} 