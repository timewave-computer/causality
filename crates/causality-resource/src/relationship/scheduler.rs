// Relationship-aware scheduler
// Original file: src/resource/relationship/scheduler.rs

// Cross-Domain Relationship Synchronization Scheduler
//
// This module implements a scheduler for cross-domain relationship synchronization,
// providing mechanisms to automate synchronization tasks based on configured strategies.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};

use causality_types::{Error, Result};
use causality_types::DomainId;
use super::cross_domain::{CrossDomainRelationship, CrossDomainRelationshipManager, CrossDomainMetadata};
use super::sync::{CrossDomainSyncManager, SyncStrategy, SyncResult, SyncStatus};

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Whether the scheduler is enabled
    pub enabled: bool,
    
    /// Maximum concurrent synchronization tasks
    pub max_concurrent_tasks: usize,
    
    /// Default interval for checking periodic relationships
    pub periodic_check_interval: Duration,
    
    /// Default timeout for synchronization tasks
    pub sync_timeout: Duration,
    
    /// Whether to retry failed synchronizations
    pub retry_failed: bool,
    
    /// Maximum number of retry attempts
    pub max_retry_attempts: usize,
    
    /// Backoff strategy for retries
    pub retry_backoff: RetryBackoff,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_tasks: 10,
            periodic_check_interval: Duration::from_secs(60),
            sync_timeout: Duration::from_secs(300),
            retry_failed: true,
            max_retry_attempts: 3,
            retry_backoff: RetryBackoff::Exponential {
                initial: Duration::from_secs(5),
                max: Duration::from_secs(3600),
                multiplier: 2.0,
            },
        }
    }
}

/// Retry backoff strategy
#[derive(Debug, Clone)]
pub enum RetryBackoff {
    /// Fixed interval between retries
    Fixed(Duration),
    
    /// Linear backoff, increases by a fixed amount each time
    Linear {
        initial: Duration,
        increment: Duration,
        max: Duration,
    },
    
    /// Exponential backoff, multiplies by a factor each time
    Exponential {
        initial: Duration,
        multiplier: f64,
        max: Duration,
    },
}

impl RetryBackoff {
    /// Calculate the backoff duration for a retry attempt
    pub fn calculate(&self, attempt: usize) -> Duration {
        match self {
            RetryBackoff::Fixed(duration) => *duration,
            RetryBackoff::Linear { initial, increment, max } => {
                let calculated = *initial + (*increment * attempt as u32);
                std::cmp::min(calculated, *max)
            },
            RetryBackoff::Exponential { initial, multiplier, max } => {
                let multiplier_factor = multiplier.powi(attempt as i32);
                let seconds = initial.as_secs_f64() * multiplier_factor;
                let calculated = Duration::from_secs_f64(seconds);
                std::cmp::min(calculated, *max)
            },
        }
    }
}

/// Status of the scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerStatus {
    /// Scheduler is running
    Running,
    
    /// Scheduler is stopped
    Stopped,
    
    /// Scheduler is paused
    Paused,
}

/// Information about a scheduled task
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// Relationship ID
    pub relationship_id: String,
    
    /// Source domain
    pub source_domain: DomainId,
    
    /// Target domain
    pub target_domain: DomainId,
    
    /// When the task was scheduled
    pub scheduled_at: Instant,
    
    /// When the task should be executed
    pub execute_at: Instant,
    
    /// Number of retry attempts
    pub retry_attempt: usize,
    
    /// Task priority (lower number = higher priority)
    pub priority: usize,
}

/// Statistics about scheduler operation
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    /// Number of successful synchronizations
    pub successful_syncs: usize,
    
    /// Number of failed synchronizations
    pub failed_syncs: usize,
    
    /// Number of retry attempts
    pub retry_attempts: usize,
    
    /// Total number of relationships managed
    pub total_relationships: usize,
    
    /// Number of active tasks
    pub active_tasks: usize,
    
    /// Number of pending tasks
    pub pending_tasks: usize,
    
    /// Average synchronization time
    pub avg_sync_time_ms: f64,
    
    /// Timestamp of the last scheduler run
    pub last_run_timestamp: Option<u64>,
}

/// Scheduler for cross-domain relationship synchronization
pub struct CrossDomainSyncScheduler {
    /// Configuration
    config: RwLock<SchedulerConfig>,
    
    /// Relationship manager
    relationship_manager: Arc<CrossDomainRelationshipManager>,
    
    /// Synchronization manager
    sync_manager: Arc<CrossDomainSyncManager>,
    
    /// Task queue
    task_queue: Arc<RwLock<Vec<ScheduledTask>>>,
    
    /// Currently running tasks
    running_tasks: Arc<RwLock<HashSet<String>>>,
    
    /// Task results
    task_results: Arc<RwLock<HashMap<String, (SyncResult, Instant)>>>,
    
    /// Scheduler statistics
    stats: Arc<RwLock<SchedulerStats>>,
    
    /// Scheduler status
    status: Arc<RwLock<SchedulerStatus>>,
    
    /// Scheduler control mutex
    control_mutex: Arc<Mutex<()>>,
}

impl CrossDomainSyncScheduler {
    /// Create a new scheduler
    pub fn new(
        relationship_manager: Arc<CrossDomainRelationshipManager>,
        sync_manager: Arc<CrossDomainSyncManager>,
    ) -> Self {
        Self {
            config: RwLock::new(SchedulerConfig::default()),
            relationship_manager,
            sync_manager,
            task_queue: Arc::new(RwLock::new(Vec::new())),
            running_tasks: Arc::new(RwLock::new(HashSet::new())),
            task_results: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(SchedulerStats::default())),
            status: Arc::new(RwLock::new(SchedulerStatus::Stopped)),
            control_mutex: Arc::new(Mutex::new(())),
        }
    }
    
    /// Start the scheduler
    pub fn start(&self) -> Result<()> {
        let _lock = self.control_mutex.lock().map_err(|_| {
            Error::Internal("Failed to acquire scheduler control lock".to_string())
        })?;
        
        let mut status = self.status.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock for scheduler status".to_string())
        })?;
        
        if *status == SchedulerStatus::Running {
            return Ok(());
        }
        
        *status = SchedulerStatus::Running;
        
        // Start the scheduler thread
        let scheduler_clone = self.clone();
        thread::spawn(move || {
            scheduler_clone.run_scheduler_loop();
        });
        
        info!("Cross-domain relationship scheduler started");
        Ok(())
    }
    
    /// Stop the scheduler
    pub fn stop(&self) -> Result<()> {
        let _lock = self.control_mutex.lock().map_err(|_| {
            Error::Internal("Failed to acquire scheduler control lock".to_string())
        })?;
        
        let mut status = self.status.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock for scheduler status".to_string())
        })?;
        
        if *status == SchedulerStatus::Stopped {
            return Ok(());
        }
        
        *status = SchedulerStatus::Stopped;
        info!("Cross-domain relationship scheduler stopped");
        Ok(())
    }
    
    /// Pause the scheduler
    pub fn pause(&self) -> Result<()> {
        let _lock = self.control_mutex.lock().map_err(|_| {
            Error::Internal("Failed to acquire scheduler control lock".to_string())
        })?;
        
        let mut status = self.status.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock for scheduler status".to_string())
        })?;
        
        if *status == SchedulerStatus::Paused {
            return Ok(());
        }
        
        if *status == SchedulerStatus::Running {
            *status = SchedulerStatus::Paused;
            info!("Cross-domain relationship scheduler paused");
        }
        
        Ok(())
    }
    
    /// Resume the scheduler
    pub fn resume(&self) -> Result<()> {
        let _lock = self.control_mutex.lock().map_err(|_| {
            Error::Internal("Failed to acquire scheduler control lock".to_string())
        })?;
        
        let mut status = self.status.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock for scheduler status".to_string())
        })?;
        
        if *status == SchedulerStatus::Running {
            return Ok(());
        }
        
        if *status == SchedulerStatus::Paused {
            *status = SchedulerStatus::Running;
            info!("Cross-domain relationship scheduler resumed");
        }
        
        Ok(())
    }
    
    /// Update scheduler configuration
    pub fn update_config(&self, config: SchedulerConfig) -> Result<()> {
        let mut current_config = self.config.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock for scheduler config".to_string())
        })?;
        
        *current_config = config;
        info!("Scheduler configuration updated");
        Ok(())
    }
    
    /// Get scheduler configuration
    pub fn get_config(&self) -> Result<SchedulerConfig> {
        let config = self.config.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock for scheduler config".to_string())
        })?;
        
        Ok(config.clone())
    }
    
    /// Get scheduler status
    pub fn get_status(&self) -> Result<SchedulerStatus> {
        let status = self.status.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock for scheduler status".to_string())
        })?;
        
        Ok(*status)
    }
    
    /// Get scheduler statistics
    pub fn get_stats(&self) -> Result<SchedulerStats> {
        let stats = self.stats.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock for scheduler stats".to_string())
        })?;
        
        Ok(stats.clone())
    }
    
    /// Schedule immediate synchronization for a relationship
    pub fn schedule_sync_now(&self, relationship_id: &str) -> Result<()> {
        // Get the relationship
        let relationship = self.relationship_manager.get_relationship(relationship_id)?;
        
        // Create a high-priority task for immediate execution
        let task = ScheduledTask {
            relationship_id: relationship_id.to_string(),
            source_domain: relationship.source_domain.clone(),
            target_domain: relationship.target_domain.clone(),
            scheduled_at: Instant::now(),
            execute_at: Instant::now(),
            retry_attempt: 0,
            priority: 0, // Highest priority
        };
        
        // Add to the queue
        let mut queue = self.task_queue.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock for task queue".to_string())
        })?;
        
        queue.push(task);
        
        // Sort the queue by execute_at time and priority
        queue.sort_by(|a, b| {
            a.execute_at
                .cmp(&b.execute_at)
                .then_with(|| a.priority.cmp(&b.priority))
        });
        
        Ok(())
    }
    
    /// Main scheduler loop
    fn run_scheduler_loop(&self) {
        let config = match self.get_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to get scheduler config: {}", e);
                return;
            }
        };
        
        let check_interval = Duration::from_millis(100);
        
        loop {
            // Check if scheduler is still running
            let status = match self.get_status() {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to get scheduler status: {}", e);
                    break;
                }
            };
            
            if status == SchedulerStatus::Stopped {
                break;
            }
            
            if status == SchedulerStatus::Paused {
                thread::sleep(check_interval);
                continue;
            }
            
            // Update scheduler stats
            self.update_current_stats();
            
            // Check for periodic relationships that need synchronization
            if let Err(e) = self.check_periodic_relationships() {
                error!("Failed to check periodic relationships: {}", e);
            }
            
            // Process task queue
            if let Err(e) = self.process_task_queue() {
                error!("Failed to process task queue: {}", e);
            }
            
            // Sleep for a short interval
            thread::sleep(check_interval);
        }
    }
    
    /// Check for periodic relationships that need synchronization
    fn check_periodic_relationships(&self) -> Result<()> {
        // Get all relationships
        let relationships = self.relationship_manager.get_all_relationships()?;
        
        // Update total relationships count
        {
            let mut stats = self.stats.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock for scheduler stats".to_string())
            })?;
            stats.total_relationships = relationships.len();
        }
        
        for relationship in relationships {
            // Skip if the relationship is already in the queue or running
            let relationship_id = &relationship.id;
            
            // Check if relationship should be synchronized
            if self.sync_manager.should_sync(&relationship) {
                // Create a task for this relationship
                let task = ScheduledTask {
                    relationship_id: relationship_id.clone(),
                    source_domain: relationship.source_domain.clone(),
                    target_domain: relationship.target_domain.clone(),
                    scheduled_at: Instant::now(),
                    execute_at: Instant::now(),
                    retry_attempt: 0,
                    priority: 10, // Normal priority
                };
                
                // Add to the queue
                let mut queue = self.task_queue.write().map_err(|_| {
                    Error::Internal("Failed to acquire write lock for task queue".to_string())
                })?;
                
                // Check if the relationship is already in the queue
                let already_in_queue = queue.iter().any(|t| t.relationship_id == *relationship_id);
                
                if !already_in_queue {
                    queue.push(task);
                    
                    // Sort the queue by execute_at time and priority
                    queue.sort_by(|a, b| {
                        a.execute_at
                            .cmp(&b.execute_at)
                            .then_with(|| a.priority.cmp(&b.priority))
                    });
                    
                    debug!("Added relationship {} to sync queue", relationship_id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Process the task queue
    fn process_task_queue(&self) -> Result<()> {
        let config = self.get_config()?;
        
        // Get current time
        let now = Instant::now();
        
        // Get tasks that are ready to execute
        let mut tasks_to_execute = Vec::new();
        
        {
            let queue = self.task_queue.read().map_err(|_| {
                Error::Internal("Failed to acquire read lock for task queue".to_string())
            })?;
            
            let running = self.running_tasks.read().map_err(|_| {
                Error::Internal("Failed to acquire read lock for running tasks".to_string())
            })?;
            
            for task in queue.iter() {
                // Skip if the task is not ready yet
                if task.execute_at > now {
                    continue;
                }
                
                // Skip if the relationship is already running
                if running.contains(&task.relationship_id) {
                    continue;
                }
                
                // Add to execution list if we haven't reached max concurrent tasks
                if running.len() < config.max_concurrent_tasks {
                    tasks_to_execute.push(task.clone());
                    
                    // Stop if we've reached max concurrent tasks
                    if tasks_to_execute.len() + running.len() >= config.max_concurrent_tasks {
                        break;
                    }
                }
            }
        }
        
        // Execute each task
        for task in tasks_to_execute {
            // Mark as running
            {
                let mut running = self.running_tasks.write().map_err(|_| {
                    Error::Internal("Failed to acquire write lock for running tasks".to_string())
                })?;
                running.insert(task.relationship_id.clone());
            }
            
            // Remove from queue
            {
                let mut queue = self.task_queue.write().map_err(|_| {
                    Error::Internal("Failed to acquire write lock for task queue".to_string())
                })?;
                queue.retain(|t| t.relationship_id != task.relationship_id);
            }
            
            // Execute in a new thread
            let scheduler_clone = self.clone();
            let task_clone = task.clone();
            thread::spawn(move || {
                scheduler_clone.execute_task(task_clone);
            });
        }
        
        Ok(())
    }
    
    /// Execute a synchronization task
    fn execute_task(&self, task: ScheduledTask) {
        debug!(
            "Executing sync task for relationship {} (attempt {})",
            task.relationship_id, task.retry_attempt + 1
        );
        
        let start_time = Instant::now();
        let config = match self.get_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to get scheduler config: {}", e);
                self.handle_task_completion(&task, SyncResult::failure(&format!(
                    "Failed to get scheduler config: {}", e
                )), start_time);
                return;
            }
        };
        
        // Get the relationship
        let relationship = match self.relationship_manager.get_relationship(&task.relationship_id) {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Failed to get relationship {}: {}",
                    task.relationship_id, e
                );
                self.handle_task_completion(&task, SyncResult::failure(&format!(
                    "Failed to get relationship: {}", e
                )), start_time);
                return;
            }
        };
        
        // Perform synchronization
        let result = match self.sync_manager.sync_relationship(&relationship) {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Failed to synchronize relationship {}: {}",
                    task.relationship_id, e
                );
                SyncResult::failure(&format!("Synchronization error: {}", e))
            }
        };
        
        self.handle_task_completion(&task, result, start_time);
    }
    
    /// Handle task completion
    fn handle_task_completion(&self, task: &ScheduledTask, result: SyncResult, start_time: Instant) {
        let elapsed = start_time.elapsed();
        
        // Log the result
        match result.status {
            SyncStatus::Success => {
                info!(
                    "Successfully synchronized relationship {} in {:?}",
                    task.relationship_id, elapsed
                );
                
                // Update statistics
                {
                    let mut stats = self.stats.write().unwrap_or_else(|_| {
                        error!("Failed to acquire write lock for scheduler stats");
                        panic!("Lock poisoned");
                    });
                    stats.successful_syncs += 1;
                    
                    // Update average sync time
                    let elapsed_ms = elapsed.as_millis() as f64;
                    if stats.avg_sync_time_ms == 0.0 {
                        stats.avg_sync_time_ms = elapsed_ms;
                    } else {
                        stats.avg_sync_time_ms = (stats.avg_sync_time_ms + elapsed_ms) / 2.0;
                    }
                }
            },
            SyncStatus::Failed => {
                warn!(
                    "Failed to synchronize relationship {}: {}",
                    task.relationship_id,
                    result.error.as_deref().unwrap_or("Unknown error")
                );
                
                // Update statistics
                {
                    let mut stats = self.stats.write().unwrap_or_else(|_| {
                        error!("Failed to acquire write lock for scheduler stats");
                        panic!("Lock poisoned");
                    });
                    stats.failed_syncs += 1;
                }
                
                // Check if we should retry
                let config = match self.get_config() {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        error!("Failed to get scheduler config: {}", e);
                        return;
                    }
                };
                
                if config.retry_failed && task.retry_attempt < config.max_retry_attempts {
                    let next_attempt = task.retry_attempt + 1;
                    let backoff = config.retry_backoff.calculate(next_attempt);
                    
                    debug!(
                        "Scheduling retry #{} for relationship {} in {:?}",
                        next_attempt, task.relationship_id, backoff
                    );
                    
                    // Create a new task for retry
                    let retry_task = ScheduledTask {
                        relationship_id: task.relationship_id.clone(),
                        source_domain: task.source_domain.clone(),
                        target_domain: task.target_domain.clone(),
                        scheduled_at: Instant::now(),
                        execute_at: Instant::now() + backoff,
                        retry_attempt: next_attempt,
                        priority: task.priority,
                    };
                    
                    // Add to the queue
                    if let Ok(mut queue) = self.task_queue.write() {
                        queue.push(retry_task);
                        
                        // Sort the queue by execute_at time and priority
                        queue.sort_by(|a, b| {
                            a.execute_at
                                .cmp(&b.execute_at)
                                .then_with(|| a.priority.cmp(&b.priority))
                        });
                    } else {
                        error!("Failed to acquire write lock for task queue");
                    }
                    
                    // Update retry statistics
                    if let Ok(mut stats) = self.stats.write() {
                        stats.retry_attempts += 1;
                    }
                }
            },
            _ => {
                // Other statuses are not expected here
                warn!(
                    "Unexpected sync status {:?} for relationship {}",
                    result.status, task.relationship_id
                );
            }
        }
        
        // Store the result
        if let Ok(mut results) = self.task_results.write() {
            results.insert(task.relationship_id.clone(), (result, Instant::now()));
        } else {
            error!("Failed to acquire write lock for task results");
        }
        
        // Remove from running tasks
        if let Ok(mut running) = self.running_tasks.write() {
            running.remove(&task.relationship_id);
        } else {
            error!("Failed to acquire write lock for running tasks");
        }
    }
    
    /// Update current statistics
    fn update_current_stats(&self) {
        if let Ok(mut stats) = self.stats.write() {
            // Update active and pending task counts
            if let Ok(running) = self.running_tasks.read() {
                stats.active_tasks = running.len();
            }
            
            if let Ok(queue) = self.task_queue.read() {
                stats.pending_tasks = queue.len();
            }
            
            // Update timestamp
            stats.last_run_timestamp = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
        }
    }
}

impl Clone for CrossDomainSyncScheduler {
    fn clone(&self) -> Self {
        Self {
            config: RwLock::new(self.config.read().unwrap_or_else(|_| panic!("Lock poisoned")).clone()),
            relationship_manager: Arc::clone(&self.relationship_manager),
            sync_manager: Arc::clone(&self.sync_manager),
            task_queue: Arc::clone(&self.task_queue),
            running_tasks: Arc::clone(&self.running_tasks),
            task_results: Arc::clone(&self.task_results),
            stats: Arc::clone(&self.stats),
            status: Arc::clone(&self.status),
            control_mutex: Arc::clone(&self.control_mutex),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_backoff() {
        // Test fixed backoff
        let fixed = RetryBackoff::Fixed(Duration::from_secs(5));
        assert_eq!(fixed.calculate(1), Duration::from_secs(5));
        assert_eq!(fixed.calculate(3), Duration::from_secs(5));
        
        // Test linear backoff
        let linear = RetryBackoff::Linear {
            initial: Duration::from_secs(5),
            increment: Duration::from_secs(10),
            max: Duration::from_secs(60),
        };
        assert_eq!(linear.calculate(0), Duration::from_secs(5));
        assert_eq!(linear.calculate(1), Duration::from_secs(15));
        assert_eq!(linear.calculate(2), Duration::from_secs(25));
        assert_eq!(linear.calculate(10), Duration::from_secs(60)); // Max capped
        
        // Test exponential backoff
        let exponential = RetryBackoff::Exponential {
            initial: Duration::from_secs(5),
            multiplier: 2.0,
            max: Duration::from_secs(60),
        };
        assert_eq!(exponential.calculate(0), Duration::from_secs(5));
        assert_eq!(exponential.calculate(1), Duration::from_secs(10));
        assert_eq!(exponential.calculate(2), Duration::from_secs(20));
        assert_eq!(exponential.calculate(4), Duration::from_secs(60)); // Max capped
    }
} 