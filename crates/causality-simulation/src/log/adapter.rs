// Purpose: Provides adapters between the simulation log structures and the engine log structures.

use anyhow::Result;
use std::sync::Arc;

use crate::log::Log;

#[cfg(feature = "engine")]
pub mod engine_adapter {
    use causality_types::{ContentId, DomainId, TraceId};
    use causality_core::effect::EffectType;
    use std::collections::HashMap;
    use serde_json::Value;
    use causality_core::log::LogStorage;
    use chrono::Utc;
    use std::sync::Mutex;
    use async_trait::async_trait;

    /// Converts a domain from the simulation model to a domain_id for the engine
    pub fn convert_domain_to_domain_id(domain: &DomainId) -> DomainId {
        domain.clone()
    }

    /// Converts a fact entry from the engine model to the simulation model
    pub fn convert_engine_fact_to_simulation(fact: &causality_engine::log::types::FactEntry) -> crate::log::Fact {
        crate::log::Fact {
            id: fact.fact_type.clone(),
            domain_id: fact.domain.clone(),
            resources: fact.resources.clone(),
            agent_id: "unknown".to_string(), // May need to be extracted from context
            intent_id: None,
            timestamp: fact.observed_at,
        }
    }

    /// Converts a fact entry from the simulation model to the engine model
    pub fn convert_simulation_fact_to_engine(fact: &crate::log::Fact) -> causality_engine::log::types::FactEntry {
        causality_engine::log::types::FactEntry {
            domain: fact.domain_id.clone(),
            block_height: 0,
            block_hash: None,
            observed_at: fact.timestamp,
            fact_type: fact.id.clone(),
            resources: fact.resources.clone(),
            data: causality_engine::log::types::BorshJsonValue(serde_json::Value::Null),
            verified: false,
        }
    }

    /// Converts an effect entry from the engine model to the simulation model
    pub fn convert_engine_effect_to_simulation(effect: &causality_engine::log::types::EffectEntry) -> crate::log::Effect {
        crate::log::Effect {
            id: effect.effect_type.to_string(),
            domain_id: if !effect.domains.is_empty() { effect.domains[0].clone() } else { DomainId::default() },
            resources: effect.resources.clone(),
            agent_id: "unknown".to_string(), // May need to be extracted from context
            fact_id: "unknown".to_string(),
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Converts an effect entry from the simulation model to the engine model
    pub fn convert_simulation_effect_to_engine(effect: &crate::log::Effect) -> causality_engine::log::types::EffectEntry {
        causality_engine::log::types::EffectEntry {
            effect_type: causality_engine::log::types::SerializableEffectType(effect.id.clone()),
            resources: effect.resources.clone(),
            domains: vec![effect.domain_id.clone()],
            code_hash: None,
            parameters: HashMap::new(),
            result: None,
            success: true,
            error: None,
        }
    }

    /// Converts a log entry from the engine model to the simulation model
    pub fn convert_engine_log_entry_to_simulation(entry: &causality_engine::log::types::LogEntry) -> crate::log::LogEntry {
        match &entry.data {
            causality_engine::log::types::EntryData::Fact(fact_entry) => {
                let fact = crate::log::Fact {
                    id: fact_entry.fact_type.clone(),
                    domain_id: fact_entry.domain.clone(),
                    resources: fact_entry.resources.clone(),
                    agent_id: "unknown".to_string(), // May need to be extracted from context
                    intent_id: None,
                    timestamp: fact_entry.observed_at,
                };
                
                crate::log::LogEntry::Fact {
                    fact,
                    entry_id: entry.id.clone(),
                    timestamp: entry.timestamp.to_millis() as i64,
                }
            }
            causality_engine::log::types::EntryData::Effect(effect_entry) => {
                let effect = crate::log::Effect {
                    id: effect_entry.effect_type.to_string(),
                    domain_id: if !effect_entry.domains.is_empty() { effect_entry.domains[0].clone() } else { DomainId::default() },
                    resources: effect_entry.resources.clone(),
                    agent_id: "unknown".to_string(), // May need to be extracted from context
                    fact_id: "unknown".to_string(), // May need to be linked differently
                    timestamp: entry.timestamp.to_millis() as i64,
                };
                
                crate::log::LogEntry::Effect {
                    effect,
                    entry_id: entry.id.clone(),
                    timestamp: entry.timestamp.to_millis() as i64,
                }
            }
            causality_engine::log::types::EntryData::Custom(intent_type, intent_data) if intent_type == "Intent" => {
                // Try to parse the intent data
                let intent = match serde_json::from_value::<causality_core::fact::intent::Intent>(intent_data.0.clone()) {
                    Ok(i) => i,
                    Err(_) => causality_core::fact::intent::Intent {
                        id: entry.id.clone(),
                        domain_id: TraceId::default(),
                        timestamp: entry.timestamp.to_millis() as i64,
                        agent_id: "unknown".to_string(),
                        intent_type: "unknown".to_string(),
                        parameters: Value::Null,
                        context: Default::default(),
                    },
                };
                
                crate::log::LogEntry::Intent {
                    intent,
                    entry_id: entry.id.clone(),
                    timestamp: entry.timestamp.to_millis() as i64,
                }
            }
            _ => {
                // Default to an Intent for other types
                let intent = causality_core::fact::intent::Intent {
                    id: entry.id.clone(),
                    domain_id: TraceId::default(),
                    timestamp: entry.timestamp.to_millis() as i64,
                    agent_id: "unknown".to_string(),
                    intent_type: "unknown".to_string(),
                    parameters: Value::Null,
                    context: Default::default(),
                };
                
                crate::log::LogEntry::Intent {
                    intent,
                    entry_id: entry.id.clone(),
                    timestamp: entry.timestamp.to_millis() as i64,
                }
            }
        }
    }

    /// Converts a log event from the simulation model to the engine log entry
    pub fn convert_simulation_log_event_to_engine(event: &crate::log::LogEvent) -> Result<causality_engine::log::types::LogEntry, anyhow::Error> {
        let trace_id = event.trace_id.clone().unwrap_or_else(|| ContentId::generate().to_string());
        let trace_id_parsed = TraceId::from_str(&trace_id)
            .map_err(|_| anyhow::anyhow!("Invalid trace ID format"))?;
        
        match &event.log_entry {
            crate::log::LogEntry::Fact { fact, entry_id, timestamp } => {
                let entry_data = causality_engine::log::types::EntryData::Fact(
                    causality_engine::log::types::FactEntry {
                        domain: fact.domain_id.clone(),
                        block_height: 0,
                        block_hash: None,
                        observed_at: *timestamp,
                        fact_type: fact.id.clone(),
                        resources: fact.resources.clone(),
                        data: causality_engine::log::types::BorshJsonValue(serde_json::Value::Null),
                        verified: false,
                    }
                );
                
                Ok(causality_engine::log::types::LogEntry {
                    id: entry_id.clone(),
                    timestamp: (*timestamp as u64).into(),
                    entry_type: causality_engine::log::types::EntryType::Fact,
                    data: entry_data,
                    trace_id: Some(trace_id_parsed),
                    parent_id: event.parent_id.clone(),
                    metadata: HashMap::new(),
                })
            },
            crate::log::LogEntry::Effect { effect, entry_id, timestamp } => {
                let entry_data = causality_engine::log::types::EntryData::Effect(
                    causality_engine::log::types::EffectEntry {
                        effect_type: causality_engine::log::types::SerializableEffectType(effect.id.clone()),
                        resources: effect.resources.clone(),
                        domains: vec![effect.domain_id.clone()],
                        code_hash: None,
                        parameters: HashMap::new(),
                        result: None,
                        success: true,
                        error: None,
                    }
                );
                
                Ok(causality_engine::log::types::LogEntry {
                    id: entry_id.clone(),
                    timestamp: (*timestamp as u64).into(),
                    entry_type: causality_engine::log::types::EntryType::Effect,
                    data: entry_data,
                    trace_id: Some(trace_id_parsed),
                    parent_id: event.parent_id.clone(),
                    metadata: HashMap::new(),
                })
            },
            crate::log::LogEntry::Intent { intent, entry_id, timestamp } => {
                // Create a Custom entry for Intent
                let intent_json = serde_json::to_value(intent).unwrap_or(serde_json::Value::Null);
                let entry_data = causality_engine::log::types::EntryData::Custom(
                    "Intent".to_string(),
                    causality_engine::log::types::BorshJsonValue(intent_json)
                );
                
                Ok(causality_engine::log::types::LogEntry {
                    id: entry_id.clone(),
                    timestamp: (*timestamp as u64).into(),
                    entry_type: causality_engine::log::types::EntryType::Custom("Intent".to_string()),
                    data: entry_data,
                    trace_id: Some(trace_id_parsed),
                    parent_id: event.parent_id.clone(),
                    metadata: HashMap::new(),
                })
            },
        }
    }

    /// Adapter to use the core engine logging system
    pub struct CoreLogAdapter {
        log_storage: Arc<dyn LogStorage>,
    }

    impl CoreLogAdapter {
        pub fn new(log_storage: Arc<dyn LogStorage>) -> Self {
            Self { log_storage }
        }
    }

    impl Log for CoreLogAdapter {
        fn log_event(&self, event_type: &str, data: &str) -> Result<()> {
            let timestamp = Utc::now().timestamp_millis() as u64;
            
            let entry = causality_core::log::LogEntry {
                timestamp,
                event_type: event_type.to_string(),
                data: data.to_string(),
                context: Default::default(),
            };
            
            self.log_storage.append_entry(entry)?;
            Ok(())
        }

        fn get_entries(&self, limit: usize) -> Result<Vec<String>> {
            let entries = self.log_storage.get_entries(limit)?;
            
            Ok(entries
                .into_iter()
                .map(|entry| format!(
                    "{} - {}: {}", 
                    entry.timestamp, 
                    entry.event_type, 
                    entry.data
                ))
                .collect())
        }
    }

    /// Adapter for the full engine log implementation
    pub struct EngineLogAdapter {
        engine_log: Arc<dyn causality_engine::log::Log>,
    }

    impl EngineLogAdapter {
        pub fn new(engine_log: Arc<dyn causality_engine::log::Log>) -> Self {
            Self { engine_log }
        }
    }

    #[async_trait]
    impl crate::log::AsyncLog for EngineLogAdapter {
        async fn log_event(&self, event: crate::log::LogEvent) -> Result<()> {
            let engine_entry = convert_simulation_log_event_to_engine(&event)?;
            self.engine_log.add_entry(engine_entry).await
        }

        async fn get_facts(&self, domain_id: &str, since: Option<i64>) -> Result<Vec<crate::log::Fact>> {
            // Use the domain_id parameter to filter entries
            let domain = DomainId::from_str(domain_id)
                .map_err(|_| anyhow::anyhow!("Invalid domain ID"))?;
            
            // Query entries by type and domain
            let entries = self.engine_log.query_entries(
                &domain, 
                causality_engine::log::types::EntryType::Fact,
                since.map(|ts| ts as u64)
            ).await?;
            
            let facts = entries.into_iter()
                .filter_map(|entry| {
                    if let causality_engine::log::types::EntryData::Fact(fact_entry) = &entry.data {
                        Some(crate::log::Fact {
                            id: fact_entry.fact_type.clone(),
                            domain_id: fact_entry.domain.clone(),
                            resources: fact_entry.resources.clone(),
                            agent_id: "unknown".to_string(), // Need to extract from context
                            intent_id: None,
                            timestamp: fact_entry.observed_at,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            
            Ok(facts)
        }

        async fn get_effects(&self, domain_id: &str, since: Option<i64>) -> Result<Vec<crate::log::Effect>> {
            // Use the domain_id parameter to filter entries
            let domain = DomainId::from_str(domain_id)
                .map_err(|_| anyhow::anyhow!("Invalid domain ID"))?;
            
            // Query effects and convert them
            let entries = self.engine_log.query_entries(
                &domain, 
                causality_engine::log::types::EntryType::Effect,
                since.map(|ts| ts as u64)
            ).await?;
            
            let effects = entries.into_iter()
                .filter_map(|entry| {
                    if let causality_engine::log::types::EntryData::Effect(effect_entry) = &entry.data {
                        if effect_entry.domains.is_empty() {
                            return None;
                        }
                        
                        Some(crate::log::Effect {
                            id: effect_entry.effect_type.to_string(),
                            domain_id: effect_entry.domains[0].clone(),
                            resources: effect_entry.resources.clone(),
                            agent_id: "unknown".to_string(),
                            fact_id: "unknown".to_string(), // Need to link differently
                            timestamp: entry.timestamp.to_millis() as i64,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            
            Ok(effects)
        }

        async fn get_intents(&self, domain_id: &str, since: Option<i64>) -> Result<Vec<causality_core::fact::intent::Intent>> {
            // Use the domain_id parameter to filter entries
            let domain = DomainId::from_str(domain_id)
                .map_err(|_| anyhow::anyhow!("Invalid domain ID"))?;
            
            // Currently, engine doesn't directly support intents in the same way
            // We'll query custom entries with the "Intent" type
            let entries = self.engine_log.query_entries(
                &domain, 
                causality_engine::log::types::EntryType::Custom("Intent".to_string()),
                since.map(|ts| ts as u64)
            ).await?;
            
            let intents = entries.into_iter()
                .filter_map(|entry| {
                    if let causality_engine::log::types::EntryData::Custom(intent_type, intent_data) = &entry.data {
                        if intent_type == "Intent" {
                            serde_json::from_value::<causality_core::fact::intent::Intent>(intent_data.0.clone()).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            
            Ok(intents)
        }
    }
}

// Standalone log implementation
#[cfg(feature = "standalone")]
pub struct InMemoryLog {
    entries: std::sync::Mutex<Vec<(chrono::DateTime<chrono::Utc>, String, String)>>,
}

#[cfg(feature = "standalone")]
impl InMemoryLog {
    pub fn new() -> Self {
        Self {
            entries: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[cfg(feature = "standalone")]
impl Log for InMemoryLog {
    fn log_event(&self, event_type: &str, data: &str) -> Result<()> {
        let mut entries = self.entries.lock().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        entries.push((chrono::Utc::now(), event_type.to_string(), data.to_string()));
        Ok(())
    }

    fn get_entries(&self, limit: usize) -> Result<Vec<String>> {
        let entries = self.entries.lock().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        Ok(entries
            .iter()
            .rev()
            .take(limit)
            .map(|(timestamp, event_type, data)| {
                format!("{} - {}: {}", timestamp.to_rfc3339(), event_type, data)
            })
            .collect())
    }
} 