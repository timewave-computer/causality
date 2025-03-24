// Barrier pattern for concurrency
//
// This module provides a barrier pattern, which blocks until a condition
// is met or resources are available.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::future::FutureExt;
use tokio::select;
use tokio::time::sleep;

use crate::error::{Error, Result};
use crate::crypto::hash::ContentId;
use crate::concurrency::primitives::{ResourceManager, SharedResourceManager};

/// A barrier that waits for a condition to be true
///
/// This structure represents a barrier that blocks until a condition is met.
/// Optionally, it can have a timeout.
pub struct Barrier<F>
where
    F: Fn() -> bool,
{
    /// The condition to check
    condition: F,
    /// The resources required for the barrier to pass
    resources: Vec<ContentId>,
    /// The resource manager to use for acquiring resources
    resource_manager: Option<SharedResourceManager>,
    /// The timeout for the barrier
    timeout: Option<Duration>,
    /// The polling interval
    poll_interval: Duration,
}

impl<F> Barrier<F>
where
    F: Fn() -> bool,
{
    /// Create a new barrier with the given condition
    pub fn new(condition: F) -> Self {
        Barrier {
            condition,
            resources: Vec::new(),
            resource_manager: None,
            timeout: None,
            poll_interval: Duration::from_millis(10),
        }
    }
    
    /// Add a required resource to the barrier
    pub fn require_resource(mut self, resource: ContentId) -> Self {
        self.resources.push(resource);
        self
    }
    
    /// Add multiple required resources to the barrier
    pub fn require_resources(mut self, resources: Vec<ContentId>) -> Self {
        self.resources.extend(resources);
        self
    }
    
    /// Set the resource manager to use for acquiring resources
    pub fn with_resource_manager(mut self, resource_manager: SharedResourceManager) -> Self {
        self.resource_manager = Some(resource_manager);
        self
    }
    
    /// Set a timeout for the barrier
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Set the polling interval for the barrier
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
    
    /// Wait for the barrier to pass
    ///
    /// This method blocks until the condition is met or the timeout is reached.
    /// If resources are specified, they will be acquired before checking the condition.
    pub async fn wait(self) -> Result<bool> {
        // Create the actual future that waits for the barrier
        self.wait_inner().await
    }
    
    // Internal implementation of the wait function
    async fn wait_inner(self) -> Result<bool> {
        // If there's a timeout, we'll race against it
        let timeout_future = match self.timeout {
            Some(timeout) => sleep(timeout).boxed(),
            None => futures::future::pending().boxed(),
        };
        
        // The main waiting future
        let waiting_future = async {
            loop {
                // Check if all resources are available
                let resources_available = match &self.resource_manager {
                    Some(manager) => {
                        let mut all_available = true;
                        for resource in &self.resources {
                            if !manager.is_resource_available(resource.clone())? {
                                all_available = false;
                                break;
                            }
                        }
                        all_available
                    },
                    None => true, // No resources needed
                };
                
                // If resources are available, check the condition
                if resources_available && (self.condition)() {
                    return Ok(true);
                }
                
                // Wait for a bit before checking again
                sleep(self.poll_interval).await;
            }
        };
        
        // Race between the waiting future and the timeout
        select! {
            result = waiting_future => result,
            _ = timeout_future => Ok(false),
        }
    }
}

/// Create a new barrier with the given condition
pub fn barrier<F: Fn() -> bool>(condition: F) -> Barrier<F> {
    Barrier::new(condition)
}

/// Create a new barrier with the given resources and condition
pub fn resource_barrier<F: Fn() -> bool>(
    resources: Vec<ContentId>,
    condition: F,
    resource_manager: SharedResourceManager,
) -> Barrier<F> {
    Barrier::new(condition)
        .require_resources(resources)
        .with_resource_manager(resource_manager)
}

/// Create a new barrier with a timeout
pub fn timeout_barrier<F: Fn() -> bool>(
    timeout: Duration,
    condition: F,
) -> Barrier<F> {
    Barrier::new(condition).with_timeout(timeout)
}

/// Wait for all resources to be available
///
/// This is a convenience function that creates a barrier that waits
/// for a set of resources to be available.
pub async fn wait_for_resources(
    resources: Vec<ContentId>,
    resource_manager: SharedResourceManager,
    timeout: Option<Duration>,
) -> Result<bool> {
    let barrier = Barrier::new(|| true)
        .require_resources(resources)
        .with_resource_manager(resource_manager);
    
    // Add timeout if specified
    let barrier = match timeout {
        Some(timeout) => barrier.with_timeout(timeout),
        None => barrier,
    };
    
    barrier.wait().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_barrier_basic() -> Result<()> {
        // Create a condition that is initially false, then becomes true
        let condition_met = Arc::new(AtomicBool::new(false));
        let condition_clone = condition_met.clone();
        
        // Spawn a task that sets the condition to true after a delay
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            condition_clone.store(true, Ordering::SeqCst);
        });
        
        // Create a barrier that waits for the condition
        let barrier = Barrier::new(move || condition_met.load(Ordering::SeqCst));
        
        // Wait for the barrier
        let passed = barrier.wait().await?;
        
        // The barrier should pass
        assert!(passed);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_barrier_timeout() -> Result<()> {
        // Create a condition that is always false
        let barrier = Barrier::new(|| false)
            .with_timeout(Duration::from_millis(50));
        
        // Wait for the barrier with a timeout
        let start = Instant::now();
        let passed = barrier.wait().await?;
        let elapsed = start.elapsed();
        
        // The barrier should time out
        assert!(!passed);
        assert!(elapsed >= Duration::from_millis(50));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_resource_barrier() -> Result<()> {
        // Create a resource manager
        let resource_manager = Arc::new(ResourceManager::new());
        
        // Register a resource
        let resource_id = ContentId::new("test");
        resource_manager.register_resource(resource_id.clone(), "initial value")?;
        
        // Acquire the resource
        let guard = resource_manager.acquire_resource::<String>(resource_id.clone(), "owner").await?;
        
        // Create a condition that always returns true
        let condition_met = Arc::new(AtomicBool::new(true));
        
        // Create a barrier that waits for the resource to be available
        let barrier = Barrier::new(move || condition_met.load(Ordering::SeqCst))
            .require_resource(resource_id.clone())
            .with_resource_manager(resource_manager.clone())
            .with_timeout(Duration::from_millis(100));
        
        // Spawn a task that releases the resource after a delay
        let resource_id_clone = resource_id.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            drop(guard); // Release the resource
        });
        
        // Wait for the barrier
        let passed = barrier.wait().await?;
        
        // The barrier should pass once the resource is available
        assert!(passed);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_wait_for_resources() -> Result<()> {
        // Create a resource manager
        let resource_manager = Arc::new(ResourceManager::new());
        
        // Register resources
        let resource1 = ContentId::new("resource1");
        let resource2 = ContentId::new("resource2");
        resource_manager.register_resource(resource1.clone(), "value1")?;
        resource_manager.register_resource(resource2.clone(), "value2")?;
        
        // Acquire the resources
        let guard1 = resource_manager.acquire_resource::<String>(resource1.clone(), "owner1").await?;
        let guard2 = resource_manager.acquire_resource::<String>(resource2.clone(), "owner2").await?;
        
        // Spawn a task that releases the resources after delays
        tokio::spawn(async move {
            sleep(Duration::from_millis(30)).await;
            drop(guard1); // Release resource1
            sleep(Duration::from_millis(30)).await;
            drop(guard2); // Release resource2
        });
        
        // Wait for the resources
        let passed = wait_for_resources(
            vec![resource1.clone(), resource2.clone()],
            resource_manager.clone(),
            Some(Duration::from_millis(100)),
        ).await?;
        
        // The wait should succeed
        assert!(passed);
        
        // Make sure both resources are available
        assert!(resource_manager.is_resource_available(resource1)?);
        assert!(resource_manager.is_resource_available(resource2)?);
        
        Ok(())
    }
} 
