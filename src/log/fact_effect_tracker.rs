// Fact-Effect Tracker for Causality
//
// This module provides a tracker for causal relationships between
// facts and effects in the Causality system.

use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};
use crate::log::entry::{LogEntry, EntryType, EntryData};
use crate::log::fact_snapshot::{FactId, FactSnapshot, FactDependency, FactDependencyType};
use crate::log::storage::LogStorage;
use crate::types::{ResourceId, DomainId, Timestamp, TraceId};

/// Represents a causal relationship between a fact and an effect
#[derive(Debug, Clone)]
pub struct FactEffectRelation {
    /// The fact ID
    pub fact_id: FactId,
    /// The effect ID (log entry ID)
    pub effect_id: String,
    /// The timestamp of the effect
    pub effect_timestamp: Timestamp,
    /// The type of dependency
    pub dependency_type: FactDependencyType,
    /// The resources affected by the relationship
    pub resources: HashSet<ResourceId>,
    /// The domains involved in the relationship
    pub domains: HashSet<DomainId>,
    /// The trace ID (if any)
    pub trace_id: Option<TraceId>,
}

/// Tracker for fact-effect causal relationships
pub struct FactEffectTracker {
    /// Map of fact IDs to the effects that depend on them
    fact_to_effects: RwLock<HashMap<FactId, HashSet<String>>>,
    /// Map of effect IDs to the facts they depend on
    effect_to_facts: RwLock<HashMap<String, HashSet<FactId>>>,
    /// Map of resource IDs to the fact-effect relations that involve them
    resource_relations: RwLock<HashMap<ResourceId, HashSet<(FactId, String)>>>,
    /// Map of domain IDs to the fact-effect relations that involve them
    domain_relations: RwLock<HashMap<DomainId, HashSet<(FactId, String)>>>,
    /// Map of trace IDs to the fact-effect relations that involve them
    trace_relations: RwLock<HashMap<TraceId, HashSet<(FactId, String)>>>,
    /// Detailed relation information
    relations: RwLock<HashMap<(FactId, String), FactEffectRelation>>,
    /// Time-indexed facts
    time_indexed_facts: RwLock<BTreeMap<Timestamp, HashSet<FactId>>>,
    /// Time-indexed effects
    time_indexed_effects: RwLock<BTreeMap<Timestamp, HashSet<String>>>,
}

impl FactEffectTracker {
    /// Create a new fact-effect tracker
    pub fn new() -> Self {
        FactEffectTracker {
            fact_to_effects: RwLock::new(HashMap::new()),
            effect_to_facts: RwLock::new(HashMap::new()),
            resource_relations: RwLock::new(HashMap::new()),
            domain_relations: RwLock::new(HashMap::new()),
            trace_relations: RwLock::new(HashMap::new()),
            relations: RwLock::new(HashMap::new()),
            time_indexed_facts: RwLock::new(BTreeMap::new()),
            time_indexed_effects: RwLock::new(BTreeMap::new()),
        }
    }
    
    /// Track a fact-effect relationship from a log entry
    pub fn track_entry(&self, entry: &LogEntry) -> Result<()> {
        match entry.entry_type {
            EntryType::Fact => {
                // Register the fact
                if let EntryData::Fact(fact) = &entry.data {
                    let fact_id = FactId(fact.fact_id.clone());
                    
                    // Register in time index
                    let mut time_index = self.time_indexed_facts.write().map_err(|_| {
                        Error::LockError("Failed to acquire write lock on time_indexed_facts".to_string())
                    })?;
                    
                    time_index
                        .entry(entry.timestamp)
                        .or_insert_with(HashSet::new)
                        .insert(fact_id);
                }
            },
            EntryType::Effect => {
                // Register the effect and its fact dependencies
                if let EntryData::Effect(effect) = &entry.data {
                    // Find dependencies in effect metadata or attached fact snapshots
                    let dependencies = self.extract_dependencies(entry)?;
                    
                    if !dependencies.is_empty() {
                        // Register effect-to-fact relationships
                        let mut effect_facts = self.effect_to_facts.write().map_err(|_| {
                            Error::LockError("Failed to acquire write lock on effect_to_facts".to_string())
                        })?;
                        
                        let fact_ids: HashSet<FactId> = dependencies
                            .iter()
                            .map(|dep| dep.fact_id.clone())
                            .collect();
                        
                        effect_facts.insert(entry.id.clone(), fact_ids.clone());
                        
                        // Register fact-to-effect relationships
                        let mut fact_effects = self.fact_to_effects.write().map_err(|_| {
                            Error::LockError("Failed to acquire write lock on fact_to_effects".to_string())
                        })?;
                        
                        for dep in &dependencies {
                            fact_effects
                                .entry(dep.fact_id.clone())
                                .or_insert_with(HashSet::new)
                                .insert(entry.id.clone());
                        }
                        
                        // Register detailed relations
                        self.register_relations(entry, &dependencies)?;
                        
                        // Register in time index
                        let mut time_index = self.time_indexed_effects.write().map_err(|_| {
                            Error::LockError("Failed to acquire write lock on time_indexed_effects".to_string())
                        })?;
                        
                        time_index
                            .entry(entry.timestamp)
                            .or_insert_with(HashSet::new)
                            .insert(entry.id.clone());
                    }
                }
            },
            _ => {
                // We don't track other entry types
            }
        }
        
        Ok(())
    }
    
    /// Extract dependencies from an entry
    fn extract_dependencies(&self, entry: &LogEntry) -> Result<Vec<FactDependency>> {
        let mut dependencies = Vec::new();
        
        if let EntryData::Effect(effect) = &entry.data {
            // Look for fact snapshot in the metadata
            if let Some(snapshot_json) = entry.metadata.get("fact_snapshot") {
                if let Ok(snapshot) = serde_json::from_str::<FactSnapshot>(snapshot_json) {
                    // Add all facts from the snapshot
                    for fact_id in &snapshot.observed_facts {
                        let domains: Vec<DomainId> = snapshot.domains.iter().cloned().collect();
                        let domain = domains.first().cloned().unwrap_or_else(|| DomainId::new(0));
                        
                        dependencies.push(FactDependency::new(
                            fact_id.clone(),
                            domain,
                            FactDependencyType::Required
                        ));
                    }
                    
                    // Add all register observation facts
                    for (_, observation) in &snapshot.register_observations {
                        dependencies.push(FactDependency::new(
                            observation.fact_id.clone(),
                            observation.domain_id.clone(),
                            FactDependencyType::Required
                        ));
                    }
                }
            }
            
            // Look for explicit fact dependencies in the metadata
            if let Some(deps_json) = entry.metadata.get("fact_dependencies") {
                if let Ok(deps) = serde_json::from_str::<Vec<FactDependency>>(deps_json) {
                    dependencies.extend(deps);
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Register detailed relations
    fn register_relations(&self, entry: &LogEntry, dependencies: &[FactDependency]) -> Result<()> {
        if let EntryData::Effect(effect) = &entry.data {
            let mut relations = self.relations.write().map_err(|_| {
                Error::LockError("Failed to acquire write lock on relations".to_string())
            })?;
            
            let mut resource_rels = self.resource_relations.write().map_err(|_| {
                Error::LockError("Failed to acquire write lock on resource_relations".to_string())
            })?;
            
            let mut domain_rels = self.domain_relations.write().map_err(|_| {
                Error::LockError("Failed to acquire write lock on domain_relations".to_string())
            })?;
            
            let mut trace_rels = self.trace_relations.write().map_err(|_| {
                Error::LockError("Failed to acquire write lock on trace_relations".to_string())
            })?;
            
            // Extract resources and domains
            let resources = effect.resources.clone().unwrap_or_default()
                .into_iter()
                .collect::<HashSet<_>>();
                
            let domains = effect.domains.clone().unwrap_or_default()
                .into_iter()
                .collect::<HashSet<_>>();
            
            // Create relations for each dependency
            for dep in dependencies {
                let relation = FactEffectRelation {
                    fact_id: dep.fact_id.clone(),
                    effect_id: entry.id.clone(),
                    effect_timestamp: entry.timestamp,
                    dependency_type: dep.dependency_type.clone(),
                    resources: resources.clone(),
                    domains: domains.clone(),
                    trace_id: entry.trace_id.clone(),
                };
                
                // Store the relation
                relations.insert((dep.fact_id.clone(), entry.id.clone()), relation);
                
                // Update resource relations
                for resource in &resources {
                    resource_rels
                        .entry(resource.clone())
                        .or_insert_with(HashSet::new)
                        .insert((dep.fact_id.clone(), entry.id.clone()));
                }
                
                // Update domain relations
                for domain in &domains {
                    domain_rels
                        .entry(domain.clone())
                        .or_insert_with(HashSet::new)
                        .insert((dep.fact_id.clone(), entry.id.clone()));
                }
                
                // Update trace relations
                if let Some(trace_id) = &entry.trace_id {
                    trace_rels
                        .entry(trace_id.clone())
                        .or_insert_with(HashSet::new)
                        .insert((dep.fact_id.clone(), entry.id.clone()));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get all effects that depend on a fact
    pub fn get_dependent_effects(&self, fact_id: &FactId) -> Result<HashSet<String>> {
        let fact_effects = self.fact_to_effects.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on fact_to_effects".to_string())
        })?;
        
        Ok(fact_effects.get(fact_id).cloned().unwrap_or_default())
    }
    
    /// Get all facts that an effect depends on
    pub fn get_effect_dependencies(&self, effect_id: &str) -> Result<HashSet<FactId>> {
        let effect_facts = self.effect_to_facts.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on effect_to_facts".to_string())
        })?;
        
        Ok(effect_facts.get(effect_id).cloned().unwrap_or_default())
    }
    
    /// Get facts observed within a time range
    pub fn get_facts_in_time_range(
        &self,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> Result<HashSet<FactId>> {
        let time_index = self.time_indexed_facts.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on time_indexed_facts".to_string())
        })?;
        
        let mut result = HashSet::new();
        
        for (_, facts) in time_index.range(start_time..=end_time) {
            result.extend(facts.iter().cloned());
        }
        
        Ok(result)
    }
    
    /// Get effects executed within a time range
    pub fn get_effects_in_time_range(
        &self,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> Result<HashSet<String>> {
        let time_index = self.time_indexed_effects.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on time_indexed_effects".to_string())
        })?;
        
        let mut result = HashSet::new();
        
        for (_, effects) in time_index.range(start_time..=end_time) {
            result.extend(effects.iter().cloned());
        }
        
        Ok(result)
    }
    
    /// Get all relations for a resource
    pub fn get_resource_relations(&self, resource_id: &ResourceId) -> Result<Vec<FactEffectRelation>> {
        let resource_rels = self.resource_relations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on resource_relations".to_string())
        })?;
        
        let relations = self.relations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on relations".to_string())
        })?;
        
        let mut result = Vec::new();
        
        if let Some(rel_keys) = resource_rels.get(resource_id) {
            for key in rel_keys {
                if let Some(relation) = relations.get(key) {
                    result.push(relation.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get all relations for a domain
    pub fn get_domain_relations(&self, domain_id: &DomainId) -> Result<Vec<FactEffectRelation>> {
        let domain_rels = self.domain_relations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on domain_relations".to_string())
        })?;
        
        let relations = self.relations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on relations".to_string())
        })?;
        
        let mut result = Vec::new();
        
        if let Some(rel_keys) = domain_rels.get(domain_id) {
            for key in rel_keys {
                if let Some(relation) = relations.get(key) {
                    result.push(relation.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get all relations for a trace
    pub fn get_trace_relations(&self, trace_id: &TraceId) -> Result<Vec<FactEffectRelation>> {
        let trace_rels = self.trace_relations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on trace_relations".to_string())
        })?;
        
        let relations = self.relations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on relations".to_string())
        })?;
        
        let mut result = Vec::new();
        
        if let Some(rel_keys) = trace_rels.get(trace_id) {
            for key in rel_keys {
                if let Some(relation) = relations.get(key) {
                    result.push(relation.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Create a fact snapshot for a set of resources and domains
    pub fn create_snapshot(
        &self,
        resources: &[ResourceId],
        domains: &[DomainId],
        observer: &str
    ) -> Result<FactSnapshot> {
        let mut snapshot = FactSnapshot::new(observer);
        
        // Add facts for resources
        for resource in resources {
            let relations = self.get_resource_relations(resource)?;
            
            for relation in relations {
                let domain = relation.domains.iter().next()
                    .cloned()
                    .unwrap_or_else(|| DomainId::new(0));
                    
                if !snapshot.contains_fact(&relation.fact_id) {
                    snapshot.add_fact(relation.fact_id.clone(), domain);
                }
            }
        }
        
        // Add facts for domains
        for domain in domains {
            let relations = self.get_domain_relations(domain)?;
            
            for relation in relations {
                if !snapshot.contains_fact(&relation.fact_id) {
                    snapshot.add_fact(relation.fact_id.clone(), domain.clone());
                }
            }
        }
        
        Ok(snapshot)
    }
    
    /// Build a tracker from log storage
    pub fn build_from_storage(storage: &dyn LogStorage) -> Result<Self> {
        let tracker = Self::new();
        
        // Read all entries from storage
        let entry_count = storage.entry_count()?;
        let batch_size = 100;
        let mut offset = 0;
        
        while offset < entry_count {
            let entries = storage.read(offset, batch_size)?;
            
            if entries.is_empty() {
                break;
            }
            
            for entry in &entries {
                tracker.track_entry(entry)?;
            }
            
            offset += entries.len();
        }
        
        Ok(tracker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::entry::{FactEntry, EffectEntry};
    use crate::log::storage::memory_storage::MemoryLogStorage;
    use std::sync::Arc;
    
    fn create_fact_entry(id: &str, fact_id: &str, domain_id: DomainId) -> LogEntry {
        LogEntry {
            id: id.to_string(),
            timestamp: Timestamp::now(),
            entry_type: EntryType::Fact,
            data: EntryData::Fact(FactEntry {
                fact_id: fact_id.to_string(),
                fact_type: "test".to_string(),
                domain_id: domain_id.clone(),
                height: 10,
                hash: "hash123".to_string(),
                timestamp: Timestamp::now(),
                resources: Some(vec!["resource1".to_string()]),
                domains: Some(vec![domain_id.to_string()]),
                data: serde_json::json!({}),
            }),
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        }
    }
    
    fn create_effect_entry(
        id: &str,
        resource_id: &str,
        domain_id: DomainId,
        fact_dependencies: Vec<FactDependency>
    ) -> LogEntry {
        let mut metadata = HashMap::new();
        
        if !fact_dependencies.is_empty() {
            metadata.insert(
                "fact_dependencies".to_string(),
                serde_json::to_string(&fact_dependencies).unwrap(),
            );
        }
        
        LogEntry {
            id: id.to_string(),
            timestamp: Timestamp::now(),
            entry_type: EntryType::Effect,
            data: EntryData::Effect(EffectEntry {
                effect_type: "test".to_string(),
                resources: Some(vec![resource_id.to_string()]),
                domains: Some(vec![domain_id.to_string()]),
                code_hash: None,
                parameters: serde_json::json!({}),
                result: None,
                success: true,
                error: None,
            }),
            trace_id: Some("trace1".to_string()),
            parent_id: None,
            metadata,
            entry_hash: None,
        }
    }
    
    #[test]
    fn test_tracking_relationships() -> Result<()> {
        let tracker = FactEffectTracker::new();
        
        // Create a fact
        let domain_id = DomainId::new(1);
        let fact_entry = create_fact_entry("entry1", "fact1", domain_id.clone());
        let fact_id = FactId("fact1".to_string());
        
        // Track the fact
        tracker.track_entry(&fact_entry)?;
        
        // Create an effect that depends on the fact
        let dependencies = vec![
            FactDependency::new(
                fact_id.clone(),
                domain_id.clone(),
                FactDependencyType::Required,
            ),
        ];
        
        let effect_entry = create_effect_entry(
            "effect1",
            "resource1",
            domain_id.clone(),
            dependencies,
        );
        
        // Track the effect
        tracker.track_entry(&effect_entry)?;
        
        // Verify relationships
        let dependent_effects = tracker.get_dependent_effects(&fact_id)?;
        assert!(dependent_effects.contains("effect1"));
        
        let effect_dependencies = tracker.get_effect_dependencies("effect1")?;
        assert!(effect_dependencies.contains(&fact_id));
        
        // Check resource relations
        let resource_relations = tracker.get_resource_relations(&"resource1".to_string())?;
        assert_eq!(resource_relations.len(), 1);
        assert_eq!(resource_relations[0].fact_id, fact_id);
        assert_eq!(resource_relations[0].effect_id, "effect1");
        
        // Check domain relations
        let domain_relations = tracker.get_domain_relations(&domain_id)?;
        assert_eq!(domain_relations.len(), 1);
        
        // Check trace relations
        let trace_relations = tracker.get_trace_relations(&"trace1".to_string())?;
        assert_eq!(trace_relations.len(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_build_from_storage() -> Result<()> {
        // Create a storage
        let storage = MemoryLogStorage::new();
        
        // Create a fact
        let domain_id = DomainId::new(1);
        let fact_entry = create_fact_entry("entry1", "fact1", domain_id.clone());
        let fact_id = FactId("fact1".to_string());
        
        // Add fact to storage
        storage.append(fact_entry)?;
        
        // Create an effect that depends on the fact
        let dependencies = vec![
            FactDependency::new(
                fact_id.clone(),
                domain_id.clone(),
                FactDependencyType::Required,
            ),
        ];
        
        let effect_entry = create_effect_entry(
            "effect1",
            "resource1",
            domain_id.clone(),
            dependencies,
        );
        
        // Add effect to storage
        storage.append(effect_entry)?;
        
        // Build tracker from storage
        let tracker = FactEffectTracker::build_from_storage(&storage)?;
        
        // Verify relationships were loaded
        let dependent_effects = tracker.get_dependent_effects(&fact_id)?;
        assert!(dependent_effects.contains("effect1"));
        
        let effect_dependencies = tracker.get_effect_dependencies("effect1")?;
        assert!(effect_dependencies.contains(&fact_id));
        
        Ok(())
    }
    
    #[test]
    fn test_create_snapshot() -> Result<()> {
        let tracker = FactEffectTracker::new();
        
        // Create facts for different resources and domains
        let domain1 = DomainId::new(1);
        let domain2 = DomainId::new(2);
        
        let fact1 = create_fact_entry("entry1", "fact1", domain1.clone());
        let fact2 = create_fact_entry("entry2", "fact2", domain2.clone());
        
        // Track the facts
        tracker.track_entry(&fact1)?;
        tracker.track_entry(&fact2)?;
        
        // Create effects with dependencies
        let dependencies1 = vec![
            FactDependency::new(
                FactId("fact1".to_string()),
                domain1.clone(),
                FactDependencyType::Required,
            ),
        ];
        
        let dependencies2 = vec![
            FactDependency::new(
                FactId("fact2".to_string()),
                domain2.clone(),
                FactDependencyType::Required,
            ),
        ];
        
        let effect1 = create_effect_entry(
            "effect1",
            "resource1",
            domain1.clone(),
            dependencies1,
        );
        
        let effect2 = create_effect_entry(
            "effect2",
            "resource2",
            domain2.clone(),
            dependencies2,
        );
        
        // Track the effects
        tracker.track_entry(&effect1)?;
        tracker.track_entry(&effect2)?;
        
        // Create a snapshot for resource1 and domain1
        let snapshot = tracker.create_snapshot(
            &[String::from("resource1")],
            &[domain1.clone()],
            "test_observer",
        )?;
        
        // Verify snapshot
        assert_eq!(snapshot.observer, "test_observer");
        assert!(snapshot.contains_fact(&FactId("fact1".to_string())));
        assert!(!snapshot.contains_fact(&FactId("fact2".to_string())));
        
        // Create a snapshot for both resources and domains
        let snapshot2 = tracker.create_snapshot(
            &[String::from("resource1"), String::from("resource2")],
            &[domain1.clone(), domain2.clone()],
            "test_observer",
        )?;
        
        // Verify snapshot
        assert!(snapshot2.contains_fact(&FactId("fact1".to_string())));
        assert!(snapshot2.contains_fact(&FactId("fact2".to_string())));
        
        Ok(())
    }
} 