// Domain map implementation
// Original file: src/domain/map/map.rs

// Time Map Implementation
//
// This module implements the time map representation for tracking
// the observed state of domains over time.

use std::collections::{HashMap, BTreeMap};
use std::fmt::Debug;
use std::sync::{Arc, RwLock, Mutex};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

use causality_types::{Error, Result};
use causality_types::{BlockHash, BlockHeight, Timestamp};
use crate::domain::DomainId;
use super::{TimeRange, TimeWindow};

/// A time map entry for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMapEntry {
    /// Domain identifier
    pub domain_id: DomainId,
    /// Block height
    pub height: BlockHeight,
    /// Block hash
    pub hash: BlockHash,
    /// Timestamp
    pub timestamp: Timestamp,
    /// When this entry was observed
    pub observed_at: DateTime<Utc>,
    /// Confidence in this entry (0.0-1.0)
    pub confidence: f64,
    /// Whether this entry is verified
    pub verified: bool,
    /// Source of this entry (e.g., "rpc", "peers", "cache")
    pub source: String,
    /// Additional metadata about this entry
    pub metadata: HashMap<String, String>,
}

impl TimeMapEntry {
    /// Create a new time map entry
    pub fn new(
        domain_id: DomainId,
        height: BlockHeight,
        hash: BlockHash,
        timestamp: Timestamp,
    ) -> Self {
        TimeMapEntry {
            domain_id,
            height,
            hash,
            timestamp,
            observed_at: Utc::now(),
            confidence: 1.0,
            verified: false,
            source: "default".to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.max(0.0).min(1.0);
        self
    }
    
    /// Set the verification status
    pub fn with_verification(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
    
    /// Set the source of this entry
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }
    
    /// Add metadata to this entry
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if this entry was observed recently
    pub fn is_recent(&self, threshold_minutes: i64) -> bool {
        let now = Utc::now();
        let threshold = Duration::minutes(threshold_minutes);
        now.signed_duration_since(self.observed_at) <= threshold
    }
    
    /// Check if this entry is verifiable (has block hash)
    pub fn is_verifiable(&self) -> bool {
        !self.hash.is_empty()
    }
}

/// Time map representing the observed state of all domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMap {
    /// Map from domain ID to time map entry
    pub entries: HashMap<DomainId, TimeMapEntry>,
    /// When this time map was created
    pub created_at: DateTime<Utc>,
    /// Version of this time map (for tracking updates)
    pub version: u64,
    /// Additional metadata about this time map
    pub metadata: HashMap<String, String>,
}

impl TimeMap {
    /// Create a new empty time map
    pub fn new() -> Self {
        TimeMap {
            entries: HashMap::new(),
            created_at: Utc::now(),
            version: 1,
            metadata: HashMap::new(),
        }
    }
    
    /// Get the height of a domain
    pub fn get_height(&self, domain_id: &DomainId) -> Option<BlockHeight> {
        self.entries.get(domain_id).map(|e| e.height)
    }
    
    /// Get the hash of a domain
    pub fn get_hash(&self, domain_id: &DomainId) -> Option<&BlockHash> {
        self.entries.get(domain_id).map(|e| &e.hash)
    }
    
    /// Get the timestamp of a domain
    pub fn get_timestamp(&self, domain_id: &DomainId) -> Option<Timestamp> {
        self.entries.get(domain_id).map(|e| e.timestamp)
    }
    
    /// Update or insert a domain entry
    pub fn update_domain(&mut self, entry: TimeMapEntry) {
        self.entries.insert(entry.domain_id.clone(), entry);
        self.version += 1;
    }
    
    /// Remove a domain entry
    pub fn remove_domain(&mut self, domain_id: &DomainId) -> Option<TimeMapEntry> {
        let entry = self.entries.remove(domain_id);
        if entry.is_some() {
            self.version += 1;
        }
        entry
    }
    
    /// Check if the time map contains a domain
    pub fn contains_domain(&self, domain_id: &DomainId) -> bool {
        self.entries.contains_key(domain_id)
    }
    
    /// Get all domains in the time map
    pub fn domains(&self) -> Vec<&DomainId> {
        self.entries.keys().collect()
    }
    
    /// Get the number of domains in the time map
    pub fn domain_count(&self) -> usize {
        self.entries.len()
    }
    
    /// Merge another time map into this one
    ///
    /// For each domain in the other time map, the entry is only
    /// updated if it has a higher height or more recent timestamp.
    pub fn merge(&mut self, other: &TimeMap) {
        for (domain_id, entry) in &other.entries {
            match self.entries.get(domain_id) {
                Some(existing) => {
                    if entry.height > existing.height || entry.timestamp > existing.timestamp {
                        self.entries.insert(domain_id.clone(), entry.clone());
                    }
                }
                None => {
                    self.entries.insert(domain_id.clone(), entry.clone());
                }
            }
        }
        self.version += 1;
    }
    
    /// Create a subset of this time map with only the specified domains
    pub fn subset(&self, domain_ids: &[DomainId]) -> TimeMap {
        let mut result = TimeMap::new();
        
        for domain_id in domain_ids {
            if let Some(entry) = self.entries.get(domain_id) {
                result.entries.insert(domain_id.clone(), entry.clone());
            }
        }
        
        result.created_at = self.created_at;
        result.version = self.version;
        result
    }
    
    /// Filter the time map by a predicate
    ///
    /// This creates a new time map containing only entries
    /// that satisfy the predicate.
    pub fn filter<F>(&self, predicate: F) -> TimeMap
    where
        F: Fn(&TimeMapEntry) -> bool,
    {
        let mut result = TimeMap::new();
        
        for (domain_id, entry) in &self.entries {
            if predicate(entry) {
                result.entries.insert(domain_id.clone(), entry.clone());
            }
        }
        
        result.created_at = self.created_at;
        result.version = self.version;
        result
    }
    
    /// Get only verified entries
    pub fn verified_only(&self) -> TimeMap {
        self.filter(|entry| entry.verified)
    }
    
    /// Get only recent entries
    pub fn recent_only(&self, threshold_minutes: i64) -> TimeMap {
        self.filter(|entry| entry.is_recent(threshold_minutes))
    }
    
    /// Get entries sorted by timestamp
    pub fn entries_by_timestamp(&self) -> Vec<&TimeMapEntry> {
        let mut entries: Vec<&TimeMapEntry> = self.entries.values().collect();
        entries.sort_by_key(|e| e.timestamp);
        entries
    }
    
    /// Get the earliest timestamp in the time map
    pub fn earliest_timestamp(&self) -> Option<Timestamp> {
        self.entries.values()
            .map(|e| e.timestamp)
            .min()
    }
    
    /// Get the latest timestamp in the time map
    pub fn latest_timestamp(&self) -> Option<Timestamp> {
        self.entries.values()
            .map(|e| e.timestamp)
            .max()
    }
    
    /// Add a tag to this time map
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.metadata.insert("tag".to_string(), tag.to_string());
        self
    }
    
    /// Add metadata to this time map
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Query time windows across domains
    /// 
    /// This method returns all time windows that overlap with the given time range
    pub fn query_windows(&self, range: &TimeRange) -> Vec<TimeWindow> {
        let mut windows = vec![];
        
        for (domain_id, entry) in &self.entries {
            // Convert entries to time windows covering the range if possible
            if let Some(window) = self.entry_to_window(domain_id, entry, range) {
                windows.push(window);
            }
        }
        
        windows
    }
    
    /// Convert a time map entry to a time window if it overlaps with the given range
    fn entry_to_window(&self, domain_id: &DomainId, entry: &TimeMapEntry, range: &TimeRange) -> Option<TimeWindow> {
        // Determine if the entry's timestamp falls within the range
        if !range.contains(entry.timestamp) {
            return None;
        }
        
        // Create a window covering just the point in time where we have data
        let point_range = TimeRange::new(entry.timestamp, entry.timestamp);
        
        Some(TimeWindow::new(
            domain_id.clone(),
            point_range,
            entry.height,
            entry.hash.clone(),
        ))
    }
    
    /// Query entries within a specific time range
    pub fn query_by_time(&self, range: &TimeRange) -> Vec<&TimeMapEntry> {
        self.entries
            .values()
            .filter(|entry| range.contains(entry.timestamp))
            .collect()
    }
    
    /// Query the latest entry for each domain before a certain timestamp
    pub fn query_before(&self, timestamp: Timestamp) -> HashMap<DomainId, &TimeMapEntry> {
        let mut result = HashMap::new();
        
        for (domain_id, entry) in &self.entries {
            if entry.timestamp <= timestamp {
                // If we already have an entry for this domain, only keep the most recent one
                match result.get(domain_id) {
                    Some(existing) if existing.timestamp < entry.timestamp => {
                        result.insert(domain_id.clone(), entry);
                    },
                    None => {
                        result.insert(domain_id.clone(), entry);
                    },
                    _ => {}
                }
            }
        }
        
        result
    }
    
    /// Query the earliest entry for each domain after a certain timestamp
    pub fn query_after(&self, timestamp: Timestamp) -> HashMap<DomainId, &TimeMapEntry> {
        let mut result = HashMap::new();
        
        for (domain_id, entry) in &self.entries {
            if entry.timestamp >= timestamp {
                // If we already have an entry for this domain, only keep the earliest one
                match result.get(domain_id) {
                    Some(existing) if existing.timestamp > entry.timestamp => {
                        result.insert(domain_id.clone(), entry);
                    },
                    None => {
                        result.insert(domain_id.clone(), entry);
                    },
                    _ => {}
                }
            }
        }
        
        result
    }
    
    /// Find entries from different domains that are approximately synchronized
    /// within the given time tolerance (in seconds)
    pub fn find_synchronized(&self, tolerance: u64) -> Vec<Vec<&TimeMapEntry>> {
        let mut entries: Vec<&TimeMapEntry> = self.entries.values().collect();
        
        // Sort by timestamp
        entries.sort_by_key(|e| e.timestamp);
        
        // Group entries that are within tolerance of each other
        let mut groups = vec![];
        let mut current_group = vec![];
        
        // Use a sliding window approach to group entries
        if let Some(first) = entries.first() {
            current_group.push(*first);
            
            for entry in entries.iter().skip(1) {
                let last_ts = current_group.last().unwrap().timestamp;
                
                if entry.timestamp.0 <= last_ts.0 + tolerance {
                    // This entry is within tolerance of the last one
                    current_group.push(*entry);
                } else {
                    // This entry is too far from the previous group
                    if !current_group.is_empty() {
                        groups.push(current_group);
                        current_group = vec![*entry];
                    }
                }
            }
            
            // Add the last group if it's not empty
            if !current_group.is_empty() {
                groups.push(current_group);
            }
        }
        
        // Only return groups with entries from different domains
        groups.into_iter()
            .filter(|group| {
                let domains: std::collections::HashSet<_> = group.iter()
                    .map(|e| &e.domain_id)
                    .collect();
                domains.len() > 1
            })
            .collect()
    }
}

/// History of time maps
#[derive(Debug, Clone)]
pub struct TimeMapHistory {
    /// Historical time maps, indexed by version
    history: BTreeMap<u64, TimeMap>,
    /// Maximum number of versions to keep
    max_versions: usize,
}

impl TimeMapHistory {
    /// Create a new time map history
    pub fn new(max_versions: usize) -> Self {
        TimeMapHistory {
            history: BTreeMap::new(),
            max_versions: max_versions.max(1),
        }
    }
    
    /// Add a time map to the history
    pub fn add(&mut self, time_map: TimeMap) {
        // Add the new time map
        self.history.insert(time_map.version, time_map);
        
        // Prune old versions if we're over the limit
        if self.history.len() > self.max_versions {
            let keys: Vec<u64> = self.history.keys().copied().collect();
            let to_remove = keys.len() - self.max_versions;
            
            for &key in keys.iter().take(to_remove) {
                self.history.remove(&key);
            }
        }
    }
    
    /// Get a specific version of the time map
    pub fn get_version(&self, version: u64) -> Option<&TimeMap> {
        self.history.get(&version)
    }
    
    /// Get the latest version of the time map
    pub fn latest(&self) -> Option<&TimeMap> {
        self.history.values().last()
    }
    
    /// Get the time map at or before a specific timestamp
    pub fn at_timestamp(&self, timestamp: DateTime<Utc>) -> Option<&TimeMap> {
        // Find the most recent time map before the timestamp
        self.history.values()
            .filter(|tm| tm.created_at <= timestamp)
            .max_by_key(|tm| tm.created_at)
    }
    
    /// Get the number of versions in the history
    pub fn version_count(&self) -> usize {
        self.history.len()
    }
    
    /// Get all version numbers in the history
    pub fn versions(&self) -> Vec<u64> {
        self.history.keys().copied().collect()
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
    }
}

/// Thread-safe shared time map
#[derive(Clone)]
pub struct SharedTimeMap {
    inner: Arc<RwLock<TimeMap>>,
    history: Arc<Mutex<TimeMapHistory>>,
}

impl SharedTimeMap {
    /// Create a new shared time map
    pub fn new() -> Self {
        SharedTimeMap {
            inner: Arc::new(RwLock::new(TimeMap::new())),
            history: Arc::new(Mutex::new(TimeMapHistory::new(100))),
        }
    }
    
    /// Get a copy of the time map
    pub fn get(&self) -> Result<TimeMap> {
        let map = self.inner.read().map_err(|_| 
            Error::InternalError("Failed to read time map".to_string()))?;
        Ok(map.clone())
    }
    
    /// Update the time map and track if it changed
    pub fn update<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut TimeMap) -> (T, bool),
    {
        let mut map = self.inner.write().map_err(|_| 
            Error::InternalError("Failed to write time map".to_string()))?;
        
        let (result, changed) = f(&mut map);
        
        // If the time map changed, update the history
        if changed {
            // Add to history
            if let Ok(mut history) = self.history.lock() {
                history.add(map.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Update a domain entry
    pub fn update_domain(&self, entry: TimeMapEntry) -> Result<()> {
        self.update(|map| {
            let domain_id = entry.domain_id.clone();
            map.update_domain(entry);
            ((), true)
        })
    }
    
    /// Update a domain entry asynchronously
    pub async fn update_entry(&self, entry: TimeMapEntry) -> Result<()> {
        self.update_domain(entry)
    }
    
    /// Get a specific version of the time map
    pub fn get_version(&self, version: u64) -> Result<Option<TimeMap>> {
        let history = self.history.lock().map_err(|_| 
            Error::InternalError("Failed to lock time map history".to_string()))?;
        Ok(history.get_version(version).cloned())
    }
    
    /// Get the current version of the time map
    pub fn current_version(&self) -> Result<u64> {
        let map = self.get()?;
        Ok(map.version)
    }
    
    /// Get all entries in the time map
    pub async fn get_all_entries(&self) -> Result<HashMap<DomainId, TimeMapEntry>> {
        let map = self.get()?;
        Ok(map.entries)
    }
    
    /// Merge another time map into this one
    pub fn merge(&self, other: &TimeMap) -> Result<()> {
        self.update(|map| {
            map.merge(other);
            ((), true)
        })
    }
}

/// Notifier for time map changes
pub struct TimeMapNotifier {
    /// The shared time map to monitor
    time_map: SharedTimeMap,
    /// Subscribers to time map changes
    subscribers: Mutex<Vec<Box<dyn Fn(&TimeMap) + Send + Sync>>>,
}

impl TimeMapNotifier {
    /// Create a new time map notifier
    pub fn new(time_map: SharedTimeMap) -> Self {
        TimeMapNotifier {
            time_map,
            subscribers: Mutex::new(Vec::new()),
        }
    }
    
    /// Subscribe to time map changes
    pub fn subscribe<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(&TimeMap) + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.lock().map_err(|_| 
            Error::InternalError("Failed to lock subscribers".to_string()))?;
        
        subscribers.push(Box::new(callback));
        
        Ok(())
    }
    
    /// Notify all subscribers of a time map change
    pub fn notify(&self) -> Result<()> {
        let map = self.time_map.get()?;
        
        let subscribers = self.subscribers.lock().map_err(|_| 
            Error::InternalError("Failed to lock subscribers".to_string()))?;
        
        for subscriber in subscribers.iter() {
            subscriber(&map);
        }
        
        Ok(())
    }
    
    /// Update the time map and notify subscribers if it changed
    pub fn update<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut TimeMap) -> (T, bool),
    {
        let result = self.time_map.update(f)?;
        let _ = self.notify();
        Ok(result)
    }
}

/// Time-based index for efficient timestamp lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeIndex {
    /// Entries indexed by timestamp bucket
    buckets: HashMap<u64, Vec<DomainId>>,
    /// Bucket size in seconds
    bucket_size: u64,
}

impl TimeIndex {
    /// Create a new time index with the given bucket size
    pub fn new(bucket_size: u64) -> Self {
        TimeIndex {
            buckets: HashMap::new(),
            bucket_size: bucket_size.max(1), // Ensure at least 1-second buckets
        }
    }
    
    /// Add a domain entry to the index
    pub fn add(&mut self, domain_id: &DomainId, timestamp: Timestamp) {
        let bucket = timestamp.0 / self.bucket_size;
        
        self.buckets
            .entry(bucket)
            .or_insert_with(Vec::new)
            .push(domain_id.clone());
    }
    
    /// Remove a domain entry from the index
    pub fn remove(&mut self, domain_id: &DomainId, timestamp: Timestamp) {
        let bucket = timestamp.0 / self.bucket_size;
        
        if let Some(domains) = self.buckets.get_mut(&bucket) {
            domains.retain(|d| d != domain_id);
            
            // Clean up empty buckets
            if domains.is_empty() {
                self.buckets.remove(&bucket);
            }
        }
    }
    
    /// Find domains with entries in the given time range
    pub fn find_in_range(&self, range: &TimeRange) -> Vec<DomainId> {
        let start_bucket = range.start.0 / self.bucket_size;
        let end_bucket = range.end.0 / self.bucket_size;
        
        let mut result = Vec::new();
        for bucket in start_bucket..=end_bucket {
            if let Some(domains) = self.buckets.get(&bucket) {
                for domain in domains {
                    if !result.contains(domain) {
                        result.push(domain.clone());
                    }
                }
            }
        }
        
        result
    }
    
    /// Find the nearest bucket to the given timestamp
    pub fn find_nearest(&self, timestamp: Timestamp, max_distance: u64) -> Option<Vec<DomainId>> {
        let target_bucket = timestamp.0 / self.bucket_size;
        let max_bucket_dist = (max_distance / self.bucket_size) + 1;
        
        // First check if the target bucket exists
        if let Some(domains) = self.buckets.get(&target_bucket) {
            return Some(domains.clone());
        }
        
        // Otherwise find the nearest bucket
        for dist in 1..=max_bucket_dist {
            // Check lower bucket
            if target_bucket >= dist {
                if let Some(domains) = self.buckets.get(&(target_bucket - dist)) {
                    return Some(domains.clone());
                }
            }
            
            // Check higher bucket
            if let Some(domains) = self.buckets.get(&(target_bucket + dist)) {
                return Some(domains.clone());
            }
        }
        
        None
    }
}

impl TimeMap {
    /// Add time indexing to a TimeMap
    pub fn with_index(mut self, bucket_size: u64) -> TimeMapWithIndex {
        let mut index = TimeIndex::new(bucket_size);
        
        // Populate the index with existing entries
        for (domain_id, entry) in &self.entries {
            index.add(domain_id, entry.timestamp);
        }
        
        TimeMapWithIndex {
            time_map: self,
            index,
        }
    }
}

/// A time map with an index for efficient timestamp lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMapWithIndex {
    /// The underlying time map
    time_map: TimeMap,
    /// The time index
    index: TimeIndex,
}

impl TimeMapWithIndex {
    /// Create a new indexed time map
    pub fn new(bucket_size: u64) -> Self {
        TimeMapWithIndex {
            time_map: TimeMap::new(),
            index: TimeIndex::new(bucket_size),
        }
    }
    
    /// Get the underlying time map
    pub fn time_map(&self) -> &TimeMap {
        &self.time_map
    }
    
    /// Update a domain entry
    pub fn update_domain(&mut self, entry: TimeMapEntry) -> &mut Self {
        let domain_id = entry.domain_id.clone();
        let timestamp = entry.timestamp;
        
        // Remove old entry from index if it exists
        if let Some(old_entry) = self.time_map.entries.get(&domain_id) {
            self.index.remove(&domain_id, old_entry.timestamp);
        }
        
        // Update the time map
        self.time_map.update_domain(entry);
        
        // Add new entry to index
        self.index.add(&domain_id, timestamp);
        
        self
    }
    
    /// Query domains that have entries in the given time range
    pub fn query_domains_in_range(&self, range: &TimeRange) -> Vec<DomainId> {
        self.index.find_in_range(range)
    }
    
    /// Efficiently query entries within a specific time range
    pub fn query_by_time(&self, range: &TimeRange) -> Vec<&TimeMapEntry> {
        let domains = self.index.find_in_range(range);
        
        domains.iter()
            .filter_map(|domain_id| self.time_map.entries.get(domain_id))
            .filter(|entry| range.contains(entry.timestamp))
            .collect()
    }
    
    /// Find the nearest entries to the given timestamp within max_distance seconds
    pub fn find_nearest(&self, timestamp: Timestamp, max_distance: u64) -> Vec<&TimeMapEntry> {
        if let Some(domains) = self.index.find_nearest(timestamp, max_distance) {
            domains.iter()
                .filter_map(|domain_id| self.time_map.entries.get(domain_id))
                .filter(|entry| {
                    let diff = if entry.timestamp.0 > timestamp.0 {
                        entry.timestamp.0 - timestamp.0
                    } else {
                        timestamp.0 - entry.timestamp.0
                    };
                    
                    diff <= max_distance
                })
                .collect()
        } else {
            vec![]
        }
    }
}

/// A causally linked time point that can be used to establish causality between events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalTimePoint {
    /// The domain ID
    pub domain_id: DomainId,
    /// The time point
    pub time_point: TimeMapEntry,
    /// Causally preceding time points
    pub preceding_points: HashMap<DomainId, TimeMapEntry>,
    /// When this causal point was created
    pub created_at: DateTime<Utc>,
}

impl CausalTimePoint {
    /// Create a new causal time point
    pub fn new(domain_id: DomainId, time_point: TimeMapEntry) -> Self {
        CausalTimePoint {
            domain_id,
            time_point,
            preceding_points: HashMap::new(),
            created_at: Utc::now(),
        }
    }
    
    /// Add a causally preceding time point
    pub fn with_preceding(mut self, entry: TimeMapEntry) -> Self {
        self.preceding_points.insert(entry.domain_id.clone(), entry);
        self
    }
    
    /// Add multiple causally preceding time points
    pub fn with_preceding_map(mut self, entries: HashMap<DomainId, TimeMapEntry>) -> Self {
        self.preceding_points.extend(entries);
        self
    }
    
    /// Check if this causal point causally precedes another
    pub fn causally_precedes(&self, other: &CausalTimePoint) -> bool {
        // If this point is for the same domain as the other, check timestamps
        if self.domain_id == other.domain_id {
            return self.time_point.timestamp <= other.time_point.timestamp;
        }
        
        // Check if this point is among the preceding points of the other
        if let Some(preceding) = other.preceding_points.get(&self.domain_id) {
            return self.time_point.timestamp <= preceding.timestamp;
        }
        
        // Otherwise, there's no established causal relationship
        false
    }
    
    /// Check if this point and another are concurrent (neither causally precedes the other)
    pub fn concurrent_with(&self, other: &CausalTimePoint) -> bool {
        !self.causally_precedes(other) && !other.causally_precedes(self)
    }
}

/// A causal history tracking system for time points across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalHistory {
    /// All causal time points, indexed by domain ID and timestamp
    points: HashMap<DomainId, Vec<CausalTimePoint>>,
    /// The most recent time map derived from the history
    latest_map: TimeMap,
}

impl CausalHistory {
    /// Create a new causal history
    pub fn new() -> Self {
        CausalHistory {
            points: HashMap::new(),
            latest_map: TimeMap::new(),
        }
    }
    
    /// Add a causal time point to the history
    pub fn add_point(&mut self, point: CausalTimePoint) {
        // Update the latest map
        self.latest_map.update_domain(point.time_point.clone());
        
        // Add preceding points to the map as well
        for (_, entry) in &point.preceding_points {
            self.latest_map.update_domain(entry.clone());
        }
        
        // Add to the history
        self.points
            .entry(point.domain_id.clone())
            .or_insert_with(Vec::new)
            .push(point);
    }
    
    /// Create a causal time point with the current state
    pub fn create_point(&self, domain_id: DomainId, time_point: TimeMapEntry) -> CausalTimePoint {
        // Get the latest time map entry for each domain to establish causality
        let mut preceding_points = HashMap::new();
        
        for (d_id, entries) in &self.points {
            if d_id != &domain_id && !entries.is_empty() {
                // Find the latest entry for this domain
                if let Some(latest) = entries.iter()
                    .max_by_key(|p| p.time_point.timestamp) {
                    preceding_points.insert(d_id.clone(), latest.time_point.clone());
                }
            }
        }
        
        CausalTimePoint::new(domain_id, time_point)
            .with_preceding_map(preceding_points)
    }
    
    /// Get the latest time map
    pub fn latest_map(&self) -> &TimeMap {
        &self.latest_map
    }
    
    /// Find all causal points that causally precede the given timestamp in the specified domain
    pub fn find_preceding(&self, domain_id: &DomainId, timestamp: Timestamp) -> Vec<&CausalTimePoint> {
        let mut result = Vec::new();
        
        for (d_id, points) in &self.points {
            for point in points {
                // If this is the requested domain, check timestamp
                if d_id == domain_id {
                    if point.time_point.timestamp < timestamp {
                        result.push(point);
                    }
                } else {
                    // For other domains, add points that might have influenced the given time
                    if let Some(preceding) = point.preceding_points.get(domain_id) {
                        if preceding.timestamp < timestamp {
                            result.push(point);
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Compute the causal cone (all points that causally precede) for a specific point
    pub fn causal_cone(&self, domain_id: &DomainId, timestamp: Timestamp) -> Vec<&CausalTimePoint> {
        let mut result = Vec::new();
        let mut to_process = self.find_preceding(domain_id, timestamp);
        let mut processed = std::collections::HashSet::new();
        
        while let Some(point) = to_process.pop() {
            // Create a key for this point to avoid processing it multiple times
            let key = format!("{}:{}", point.domain_id, point.time_point.timestamp);
            
            if !processed.contains(&key) {
                processed.insert(key);
                result.push(point);
                
                // Add all points that precede this one
                for (preceding_id, preceding_entry) in &point.preceding_points {
                    to_process.extend(self.find_preceding(preceding_id, preceding_entry.timestamp));
                }
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_domain_id(id: u8) -> DomainId {
        format!("domain_{}", id).into()
    }
    
    fn create_test_entry(domain_id: DomainId, height: u64, timestamp: u64) -> TimeMapEntry {
        TimeMapEntry::new(domain_id, height, format!("hash_{}", height), timestamp)
    }
    
    #[test]
    fn test_time_map_basic_operations() {
        let mut map = TimeMap::new();
        
        // Check that a new map is empty
        assert_eq!(map.domain_count(), 0);
        
        // Add some domains
        let domain1 = create_test_domain_id(1);
        let domain2 = create_test_domain_id(2);
        
        let entry1 = create_test_entry(domain1.clone(), 100, 1000);
        let entry2 = create_test_entry(domain2.clone(), 200, 2000);
        
        map.update_domain(entry1);
        map.update_domain(entry2);
        
        // Check that the domains were added
        assert_eq!(map.domain_count(), 2);
        assert!(map.contains_domain(&domain1));
        assert!(map.contains_domain(&domain2));
        
        // Check that we can get domain info
        assert_eq!(map.get_height(&domain1), Some(100));
        assert_eq!(map.get_height(&domain2), Some(200));
        
        // Check that removing a domain works
        map.remove_domain(&domain1);
        assert_eq!(map.domain_count(), 1);
        assert!(!map.contains_domain(&domain1));
        assert!(map.contains_domain(&domain2));
    }
    
    #[test]
    fn test_time_map_merge() {
        let mut map1 = TimeMap::new();
        let mut map2 = TimeMap::new();
        
        // Add some domains to the first map
        let domain1 = create_test_domain_id(1);
        let domain2 = create_test_domain_id(2);
        
        let entry1 = create_test_entry(domain1.clone(), 100, 1000);
        let entry2 = create_test_entry(domain2.clone(), 200, 2000);
        
        map1.update_domain(entry1);
        map1.update_domain(entry2);
        
        // Add some domains to the second map
        let domain2 = create_test_domain_id(2);
        let domain3 = create_test_domain_id(3);
        
        let entry2_newer = create_test_entry(domain2.clone(), 250, 2500);
        let entry3 = create_test_entry(domain3.clone(), 300, 3000);
        
        map2.update_domain(entry2_newer);
        map2.update_domain(entry3);
        
        // Merge the maps
        map1.merge(&map2);
        
        // Check that the merged map contains all domains
        assert_eq!(map1.domain_count(), 3);
        assert!(map1.contains_domain(&domain1));
        assert!(map1.contains_domain(&domain2));
        assert!(map1.contains_domain(&domain3));
        
        // Check that the newer entry for domain2 was used
        assert_eq!(map1.get_height(&domain2), Some(250));
    }
    
    #[test]
    fn test_time_map_filter() {
        let mut map = TimeMap::new();
        
        // Add some domains
        let domain1 = create_test_domain_id(1);
        let domain2 = create_test_domain_id(2);
        let domain3 = create_test_domain_id(3);
        
        let mut entry1 = create_test_entry(domain1.clone(), 100, 1000);
        let mut entry2 = create_test_entry(domain2.clone(), 200, 2000);
        let mut entry3 = create_test_entry(domain3.clone(), 300, 3000);
        
        // Mark some entries as verified
        entry1 = entry1.with_verification(true);
        entry3 = entry3.with_verification(true);
        
        // Set confidence levels
        entry1 = entry1.with_confidence(0.5);
        entry2 = entry2.with_confidence(0.8);
        entry3 = entry3.with_confidence(1.0);
        
        map.update_domain(entry1);
        map.update_domain(entry2);
        map.update_domain(entry3);
        
        // Filter for verified entries
        let verified_map = map.verified_only();
        assert_eq!(verified_map.domain_count(), 2);
        assert!(verified_map.contains_domain(&domain1));
        assert!(!verified_map.contains_domain(&domain2));
        assert!(verified_map.contains_domain(&domain3));
        
        // Filter for high confidence entries
        let high_confidence_map = map.filter(|entry| entry.confidence >= 0.7);
        assert_eq!(high_confidence_map.domain_count(), 2);
        assert!(!high_confidence_map.contains_domain(&domain1));
        assert!(high_confidence_map.contains_domain(&domain2));
        assert!(high_confidence_map.contains_domain(&domain3));
    }
    
    #[test]
    fn test_time_map_history() {
        let mut history = TimeMapHistory::new(3);
        
        // Create some time maps
        let mut map1 = TimeMap::new();
        map1.version = 1;
        
        let mut map2 = TimeMap::new();
        map2.version = 2;
        
        let mut map3 = TimeMap::new();
        map3.version = 3;
        
        let mut map4 = TimeMap::new();
        map4.version = 4;
        
        // Add them to the history
        history.add(map1.clone());
        history.add(map2.clone());
        history.add(map3.clone());
        
        // Check that we can get all versions
        assert_eq!(history.version_count(), 3);
        assert_eq!(history.get_version(1).unwrap().version, 1);
        assert_eq!(history.get_version(2).unwrap().version, 2);
        assert_eq!(history.get_version(3).unwrap().version, 3);
        
        // Add another version, which should evict the oldest
        history.add(map4.clone());
        
        // Check that we only have the three most recent versions
        assert_eq!(history.version_count(), 3);
        assert!(history.get_version(1).is_none());
        assert_eq!(history.get_version(2).unwrap().version, 2);
        assert_eq!(history.get_version(3).unwrap().version, 3);
        assert_eq!(history.get_version(4).unwrap().version, 4);
        
        // Check that latest returns the most recent version
        assert_eq!(history.latest().unwrap().version, 4);
    }
    
    #[test]
    fn test_shared_time_map() -> Result<()> {
        let shared_map = SharedTimeMap::new();
        
        // Add some domains
        let domain1 = create_test_domain_id(1);
        let domain2 = create_test_domain_id(2);
        
        let entry1 = create_test_entry(domain1.clone(), 100, 1000);
        let entry2 = create_test_entry(domain2.clone(), 200, 2000);
        
        shared_map.update_domain(entry1)?;
        shared_map.update_domain(entry2)?;
        
        // Get a copy of the map and check that it contains both domains
        let map = shared_map.get()?;
        assert_eq!(map.domain_count(), 2);
        assert!(map.contains_domain(&domain1));
        assert!(map.contains_domain(&domain2));
        
        // Check that we can get a specific version
        let v = shared_map.current_version()?;
        let versioned_map = shared_map.get_version(v)?.unwrap();
        assert_eq!(versioned_map.domain_count(), 2);
        
        Ok(())
    }
}

// Map implementation for time-based domain mapping
//
// This module provides the implementation of the time map
// for domain synchronization.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::selection::DomainId;
use crate::map::types::TimePoint;
use crate::map::time_map::{TimeMap, TimeMapEntry, TimeMapHistory, SharedTimeMap};
use crate::error::{Result, system_error};

/// Implementation of the time map
pub struct TimeMapImpl {
    /// Time map entries by domain and time point
    entries: RwLock<HashMap<DomainId, HashMap<TimePoint, TimeMapEntry>>>,
    /// History of changes
    history: RwLock<HashMap<DomainId, HashMap<TimePoint, Vec<TimeMapEntry>>>>,
}

impl TimeMapImpl {
    /// Create a new time map
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            history: RwLock::new(HashMap::new()),
        }
    }
}

impl TimeMap for TimeMapImpl {
    fn add_entry(&self, entry: TimeMapEntry) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| system_error("Failed to acquire write lock on time map"))?;
        
        let domain_entries = entries.entry(entry.domain_id.clone()).or_insert_with(HashMap::new);
        
        // Add to history before replacing
        let mut history = self.history.write().map_err(|_| system_error("Failed to acquire write lock on time map history"))?;
        let domain_history = history.entry(entry.domain_id.clone()).or_insert_with(HashMap::new);
        let entry_history = domain_history.entry(entry.time_point.clone()).or_insert_with(Vec::new);
        
        // Add the current entry to history if it exists
        if let Some(current_entry) = domain_entries.get(&entry.time_point) {
            entry_history.push(current_entry.clone());
        }
        
        // Update the entry
        domain_entries.insert(entry.time_point.clone(), entry);
        
        Ok(())
    }
    
    fn get_entry(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<Option<TimeMapEntry>> {
        let entries = self.entries.read().map_err(|_| system_error("Failed to acquire read lock on time map"))?;
        
        if let Some(domain_entries) = entries.get(domain_id) {
            if let Some(entry) = domain_entries.get(time_point) {
                return Ok(Some(entry.clone()));
            }
        }
        
        Ok(None)
    }
    
    fn get_entries_for_domain(&self, domain_id: &DomainId) -> Result<Vec<TimeMapEntry>> {
        let entries = self.entries.read().map_err(|_| system_error("Failed to acquire read lock on time map"))?;
        
        if let Some(domain_entries) = entries.get(domain_id) {
            Ok(domain_entries.values().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    fn remove_entry(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<bool> {
        let mut entries = self.entries.write().map_err(|_| system_error("Failed to acquire write lock on time map"))?;
        
        if let Some(domain_entries) = entries.get_mut(domain_id) {
            Ok(domain_entries.remove(time_point).is_some())
        } else {
            Ok(false)
        }
    }
    
    fn clear_domain(&self, domain_id: &DomainId) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| system_error("Failed to acquire write lock on time map"))?;
        
        entries.remove(domain_id);
        
        // Also clear history
        let mut history = self.history.write().map_err(|_| system_error("Failed to acquire write lock on time map history"))?;
        history.remove(domain_id);
        
        Ok(())
    }
    
    fn clear_all(&self) -> Result<()> {
        let mut entries = self.entries.write().map_err(|_| system_error("Failed to acquire write lock on time map"))?;
        entries.clear();
        
        let mut history = self.history.write().map_err(|_| system_error("Failed to acquire write lock on time map history"))?;
        history.clear();
        
        Ok(())
    }
    
    fn get_domain_ids(&self) -> Result<Vec<DomainId>> {
        let entries = self.entries.read().map_err(|_| system_error("Failed to acquire read lock on time map"))?;
        
        Ok(entries.keys().cloned().collect())
    }
}

impl TimeMapHistory for TimeMapImpl {
    fn get_entry_history(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<Vec<TimeMapEntry>> {
        let history = self.history.read().map_err(|_| system_error("Failed to acquire read lock on time map history"))?;
        
        if let Some(domain_history) = history.get(domain_id) {
            if let Some(entry_history) = domain_history.get(time_point) {
                return Ok(entry_history.clone());
            }
        }
        
        Ok(Vec::new())
    }
    
    fn clear_history_for_domain(&self, domain_id: &DomainId) -> Result<()> {
        let mut history = self.history.write().map_err(|_| system_error("Failed to acquire write lock on time map history"))?;
        
        history.remove(domain_id);
        
        Ok(())
    }
}

impl Default for TimeMapImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedTimeMap {
    /// Create a new shared time map
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TimeMapImpl::new()),
        }
    }
    
    /// Get the inner time map implementation
    pub fn inner(&self) -> Arc<TimeMapImpl> {
        self.inner.clone()
    }
}

impl TimeMap for SharedTimeMap {
    fn add_entry(&self, entry: TimeMapEntry) -> Result<()> {
        self.inner.add_entry(entry)
    }
    
    fn get_entry(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<Option<TimeMapEntry>> {
        self.inner.get_entry(domain_id, time_point)
    }
    
    fn get_entries_for_domain(&self, domain_id: &DomainId) -> Result<Vec<TimeMapEntry>> {
        self.inner.get_entries_for_domain(domain_id)
    }
    
    fn remove_entry(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<bool> {
        self.inner.remove_entry(domain_id, time_point)
    }
    
    fn clear_domain(&self, domain_id: &DomainId) -> Result<()> {
        self.inner.clear_domain(domain_id)
    }
    
    fn clear_all(&self) -> Result<()> {
        self.inner.clear_all()
    }
    
    fn get_domain_ids(&self) -> Result<Vec<DomainId>> {
        self.inner.get_domain_ids()
    }
}

impl TimeMapHistory for SharedTimeMap {
    fn get_entry_history(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<Vec<TimeMapEntry>> {
        self.inner.get_entry_history(domain_id, time_point)
    }
    
    fn clear_history_for_domain(&self, domain_id: &DomainId) -> Result<()> {
        self.inner.clear_history_for_domain(domain_id)
    }
}

impl Default for SharedTimeMap {
    fn default() -> Self {
        Self::new()
    }
} 