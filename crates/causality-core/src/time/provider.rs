// Time Provider interface
//
// This module defines abstractions for providing time from different sources,
// allowing for interchangeable time sources (real-time, simulated, etc).

use std::sync::Arc;
use std::fmt::Debug;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Duration;

use super::{Timestamp, TimeMap};
use super::duration::TimeDelta;
use super::types::{DomainId, DomainPosition};
use super::map::TimeMapSnapshot;
use super::timestamp::now;

/// Error type for time provider operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to acquire lock")]
    LockError,
    
    #[error("Domain not found: {0}")]
    DomainNotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Time provider interface for abstracting different sources of time
#[async_trait]
pub trait TimeProvider: Send + Sync + Debug {
    /// Get the current timestamp
    async fn now(&self) -> Result<Timestamp>;
    
    /// Sleep for the specified duration
    async fn sleep(&self, duration: TimeDelta) -> Result<()>;
    
    /// Create a deadline after the specified duration
    async fn deadline(&self, duration: TimeDelta) -> Result<Timestamp> {
        let now = self.now().await?;
        Ok(now + duration)
    }
    
    /// Get the timestamp for a specific domain
    async fn domain_timestamp(&self, domain_id: &str) -> Result<Option<Timestamp>>;
    
    /// Get the current time map
    async fn time_map(&self) -> Result<Arc<TimeMap>>;
    
    /// Update a domain position in the time map
    async fn update_domain_position(&self, domain_id: &str, timestamp: u64) -> Result<Option<DomainPosition>>;
    
    /// Get a snapshot of the current time state
    async fn snapshot(&self) -> Result<TimeMapSnapshot>;
}

/// Factory for creating time providers
pub struct TimeProviderFactory;

impl TimeProviderFactory {
    /// Create a real-time provider that uses the system clock
    pub fn create_real_time_provider() -> Arc<dyn TimeProvider> {
        Arc::new(RealTimeProvider::new())
    }
    
    /// Create a simulated time provider for testing
    pub fn create_simulation_provider(
        initial_time: Option<Timestamp>,
        time_scale: Option<f64>
    ) -> Arc<dyn TimeProvider> {
        Arc::new(SimulationTimeProvider::new(initial_time, time_scale))
    }
    
    /// Create an in-memory provider for testing and development
    pub fn create_in_memory() -> Arc<dyn TimeProvider> {
        // Delegate to simulation provider with default settings
        Self::create_simulation_provider(None, None)
    }
}

/// Real-time provider that uses the system clock
#[derive(Debug, Clone)]
pub struct RealTimeProvider {
    time_map: Arc<std::sync::Mutex<TimeMap>>,
}

impl RealTimeProvider {
    /// Create a new real-time provider
    pub fn new() -> Self {
        Self {
            time_map: Arc::new(std::sync::Mutex::new(TimeMap::new())),
        }
    }
}

#[async_trait]
impl TimeProvider for RealTimeProvider {
    async fn now(&self) -> Result<Timestamp> {
        Ok(super::timestamp::now())
    }
    
    async fn sleep(&self, duration: TimeDelta) -> Result<()> {
        tokio::time::sleep(tokio::time::Duration::from_nanos(duration.as_nanos() as u64)).await;
        Ok(())
    }
    
    async fn domain_timestamp(&self, domain_id: &str) -> Result<Option<Timestamp>> {
        let guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        if let Some(position) = (*guard).get_position(domain_id) {
            Ok(Some(Timestamp::from_nanos(position.get_timestamp())))
        } else {
            Ok(None)
        }
    }
    
    async fn time_map(&self) -> Result<Arc<TimeMap>> {
        let guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        Ok(Arc::new((*guard).clone()))
    }
    
    async fn update_domain_position(&self, domain_id: &str, timestamp: u64) -> Result<Option<DomainPosition>> {
        let mut guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        Ok((*guard).update_position(domain_id, timestamp))
    }
    
    async fn snapshot(&self) -> Result<TimeMapSnapshot> {
        let guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        Ok((*guard).snapshot())
    }
}

/// Simulation time provider for testing
#[derive(Debug, Clone)]
pub struct SimulationTimeProvider {
    current_time: Arc<std::sync::Mutex<Timestamp>>,
    time_scale: f64,
    time_map: Arc<std::sync::Mutex<TimeMap>>,
}

impl SimulationTimeProvider {
    /// Create a new simulation time provider
    pub fn new(initial_time: Option<Timestamp>, time_scale: Option<f64>) -> Self {
        Self {
            current_time: Arc::new(std::sync::Mutex::new(
                initial_time.unwrap_or_else(|| Timestamp::from_nanos(0))
            )),
            time_scale: time_scale.unwrap_or(1.0),
            time_map: Arc::new(std::sync::Mutex::new(TimeMap::new())),
        }
    }
    
    /// Advance the simulated time by the specified duration
    pub fn advance(&self, duration: TimeDelta) -> Result<Timestamp> {
        let mut current = self.current_time.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current time"))?;
        
        *current = *current + duration;
        Ok(*current)
    }
    
    /// Set the simulated time to a specific timestamp
    pub fn set_time(&self, timestamp: Timestamp) -> Result<()> {
        let mut current = self.current_time.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current time"))?;
        
        *current = timestamp;
        Ok(())
    }
    
    /// Set the time scale factor (1.0 = real time, 2.0 = double speed, etc.)
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale;
    }
}

#[async_trait]
impl TimeProvider for SimulationTimeProvider {
    async fn now(&self) -> Result<Timestamp> {
        let current = self.current_time.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current time"))?;
        
        Ok(*current)
    }
    
    async fn sleep(&self, duration: TimeDelta) -> Result<()> {
        // Scale the sleep duration by the time scale
        let scaled_duration = TimeDelta::from_nanos(
            ((duration.as_nanos() as f64) / self.time_scale) as i64
        );
        
        // In simulation mode, we might just advance the clock instead of actually sleeping
        // But we'll also sleep a tiny bit to allow other tasks to run
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        
        // Advance the simulation time
        let mut current = self.current_time.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current time"))?;
        
        *current = *current + duration;
        
        Ok(())
    }
    
    async fn domain_timestamp(&self, domain_id: &str) -> Result<Option<Timestamp>> {
        let guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        if let Some(position) = (*guard).get_position(domain_id) {
            Ok(Some(Timestamp::from_nanos(position.get_timestamp())))
        } else {
            Ok(None)
        }
    }
    
    async fn time_map(&self) -> Result<Arc<TimeMap>> {
        let guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        Ok(Arc::new((*guard).clone()))
    }
    
    async fn update_domain_position(&self, domain_id: &str, timestamp: u64) -> Result<Option<DomainPosition>> {
        let mut guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        Ok((*guard).update_position(domain_id, timestamp))
    }
    
    async fn snapshot(&self) -> Result<TimeMapSnapshot> {
        let guard = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        // In simulation mode, we use the current simulation time for the snapshot
        let current = self.current_time.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock current time"))?;
        
        let mut snapshot = (*guard).snapshot();
        snapshot.timestamp = current.as_nanos() as u64;
        
        Ok(snapshot)
    }
}

/// Base time provider implementation
#[derive(Debug)]
pub struct BaseTimeProvider {
    time_map: Arc<Mutex<TimeMap>>,
}

impl BaseTimeProvider {
    /// Create a new base time provider
    pub fn new() -> Self {
        Self {
            time_map: Arc::new(Mutex::new(TimeMap::new())),
        }
    }

    /// Get the current position for a domain
    pub fn get_position(&self, domain_id: &str) -> Result<Option<DomainPosition>, Error> {
        let guard = self.time_map.lock().map_err(|_| Error::LockError)?;
        
        Ok((*guard).get_position(domain_id))
    }

    /// Update the position for a domain
    pub fn update_position(&self, domain_id: &str, timestamp: u64) -> Result<Option<DomainPosition>, Error> {
        let mut guard = self.time_map.lock().map_err(|_| Error::LockError)?;
        
        Ok((*guard).update_position(domain_id, timestamp))
    }

    /// Get a snapshot of the current time state
    pub fn snapshot(&self) -> Result<TimeMapSnapshot, Error> {
        let guard = self.time_map.lock().map_err(|_| Error::LockError)?;
        
        Ok((*guard).snapshot())
    }
}

impl Default for BaseTimeProvider {
    fn default() -> Self {
        Self::new()
    }
} 