// Timeout pattern for concurrency
//
// This module provides timeout patterns for futures, ensuring that they
// complete within a specified time limit.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use futures::future::FutureExt;
use tokio::time::sleep;

use crate::error::{Error, Result};
use crate::effect::{EffectContext, random::{RandomEffectFactory, RandomType}};

/// Run a future with a timeout
///
/// If the future completes within the timeout, its result is returned.
/// Otherwise, an error is returned.
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    match tokio::time::timeout(duration, future).await {
        Ok(result) => Ok(result),
        Err(_) => Err(Error::Timeout(format!("Operation timed out after {:?}", duration))),
    }
}

/// Run a fallible future with a timeout
///
/// If the future completes successfully within the timeout, its result is returned.
/// If the future fails, its error is returned.
/// If the future times out, a timeout error is returned.
pub async fn timeout_result<F, T, E>(duration: Duration, future: F) -> Result<T>
where
    F: Future<Output = std::result::Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    match tokio::time::timeout(duration, future).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err.into()),
        Err(_) => Err(Error::Timeout(format!("Operation timed out after {:?}", duration))),
    }
}

/// A future that can time out
///
/// This struct wraps a future and adds a timeout. If the wrapped future
/// doesn't complete within the timeout, it returns a timeout error.
pub struct WithTimeout<F, T> {
    /// The wrapped future
    future: F,
    /// The timeout duration
    timeout: Duration,
    /// Whether the future has completed
    _phantom: std::marker::PhantomData<T>,
}

impl<F, T> WithTimeout<F, T>
where
    F: Future<Output = T>,
{
    /// Create a new timeout-wrapped future
    pub fn new(future: F, timeout: Duration) -> Self {
        WithTimeout {
            future,
            timeout,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, T> Future for WithTimeout<F, T>
where
    F: Future<Output = T> + Unpin,
{
    type Output = Result<T>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        
        // Create a timer future for the timeout
        let timer = sleep(this.timeout);
        let mut timer = Box::pin(timer);
        
        // Use tokio's select! macro to poll both futures
        futures::select_biased! {
            result = this.future.boxed() => {
                std::task::Poll::Ready(Ok(result))
            },
            _ = timer => {
                std::task::Poll::Ready(Err(Error::Timeout(format!("Operation timed out after {:?}", this.timeout))))
            }
        }
    }
}

/// Add a timeout to a future
///
/// This function wraps a future with a timeout. If the wrapped future
/// doesn't complete within the timeout, it returns a timeout error.
pub fn with_timeout<F, T>(future: F, timeout: Duration) -> WithTimeout<F, T>
where
    F: Future<Output = T>,
{
    WithTimeout::new(future, timeout)
}

/// Run a function with a timeout and retry a specified number of times
///
/// This function runs a given function with a timeout, and if it fails
/// or times out, retries it up to the specified number of times.
pub async fn timeout_with_retry<F, Fut, T, E>(
    func: F,
    timeout: Duration,
    max_retries: usize,
    retry_delay: Duration,
) -> Result<T>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = std::result::Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    let mut last_error = None;
    
    for retry in 0..=max_retries {
        // Add some jitter to the retry delay to avoid thundering herd
        let jittered_delay = if retry > 0 {
            // calculate_retry_delay uses RandomEffect
            calculate_retry_delay(retry as u32, retry_delay).await
        } else {
            Duration::from_secs(0)
        };
        
        // Wait for the jittered delay
        if retry > 0 {
            tokio::time::sleep(jittered_delay).await;
        }
        
        // Run the function with a timeout
        match timeout_result(timeout, func()).await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);
                continue;
            }
        }
    }
    
    // If we get here, all retries failed
    Err(last_error.unwrap_or_else(|| Error::OperationFailed("All retries failed".to_string())))
}

/// Calculate the delay for a retry, with exponential backoff and jitter
pub async fn calculate_retry_delay(retry_attempt: u32, base_delay: Duration) -> Duration {
    // Calculate exponential backoff
    let exp_backoff = base_delay.mul_f64(2.0f64.powi(retry_attempt as i32));
    
    // Apply jitter (-10% to +10%)
    let context = EffectContext::default();
    let random_effect = RandomEffectFactory::create_effect(RandomType::Standard);
    
    // Get a random float
    let random_float = random_effect.gen_f64(&context)
        .await
        .unwrap_or(0.5);
    
    let jitter = (random_float * 0.2 - 0.1) * exp_backoff.as_secs_f64();
    let with_jitter = exp_backoff.as_secs_f64() + jitter;
    
    // Cap at 30 seconds
    let capped = with_jitter.min(30.0);
    Duration::from_secs_f64(capped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_timeout_success() {
        let start = Instant::now();
        
        // Run a future that completes quickly
        let result = timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(10)).await;
            42
        }).await;
        
        let elapsed = start.elapsed();
        
        // The future should complete successfully
        assert_eq!(result.unwrap(), 42);
        assert!(elapsed < Duration::from_millis(100));
    }
    
    #[tokio::test]
    async fn test_timeout_failure() {
        let start = Instant::now();
        
        // Run a future that takes too long
        let result = timeout(Duration::from_millis(50), async {
            sleep(Duration::from_millis(100)).await;
            42
        }).await;
        
        let elapsed = start.elapsed();
        
        // The future should timeout
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                Error::Timeout(_) => {}, // Expected
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
        
        // The elapsed time should be close to the timeout
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100));
    }
    
    #[tokio::test]
    async fn test_with_timeout() {
        // Create a future with a timeout
        let future = with_timeout(async {
            sleep(Duration::from_millis(10)).await;
            42
        }, Duration::from_millis(100));
        
        // The future should complete successfully
        let result = future.await;
        assert_eq!(result.unwrap(), 42);
        
        // Create a future that times out
        let future = with_timeout(async {
            sleep(Duration::from_millis(100)).await;
            42
        }, Duration::from_millis(50));
        
        // The future should timeout
        let result = future.await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_timeout_result() {
        // Run a fallible future that succeeds
        let result = timeout_result(Duration::from_millis(100), async {
            sleep(Duration::from_millis(10)).await;
            Ok::<_, Error>(42)
        }).await;
        
        // The future should complete successfully
        assert_eq!(result.unwrap(), 42);
        
        // Run a fallible future that fails
        let result = timeout_result(Duration::from_millis(100), async {
            sleep(Duration::from_millis(10)).await;
            Err(Error::OperationFailed("test error".to_string()))
        }).await;
        
        // The future should fail with the expected error
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                Error::OperationFailed(_) => {}, // Expected
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
        
        // Run a fallible future that times out
        let result = timeout_result(Duration::from_millis(50), async {
            sleep(Duration::from_millis(100)).await;
            Ok::<_, Error>(42)
        }).await;
        
        // The future should timeout
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                Error::Timeout(_) => {}, // Expected
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }
    
    #[tokio::test]
    async fn test_timeout_with_retry() {
        // Count the number of attempts
        let attempts = Arc::new(AtomicUsize::new(0));
        
        // Create a function that succeeds on the third attempt
        let func = {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    
                    if attempt < 3 {
                        // Fail for the first two attempts
                        Err(Error::OperationFailed(format!("Attempt {} failed", attempt)))
                    } else {
                        // Succeed on the third attempt
                        Ok(42)
                    }
                }
            }
        };
        
        // Run the function with retry
        let result = timeout_with_retry(
            func,
            Duration::from_millis(100),
            3, // Max retries
            Duration::from_millis(10), // Retry delay
        ).await;
        
        // The function should succeed on the third attempt
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
    
    #[tokio::test]
    async fn test_timeout_with_retry_all_fail() {
        // Count the number of attempts
        let attempts = Arc::new(AtomicUsize::new(0));
        
        // Create a function that always fails
        let func = {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    Err(Error::OperationFailed(format!("Attempt {} failed", attempt)))
                }
            }
        };
        
        // Run the function with retry
        let result = timeout_with_retry(
            func,
            Duration::from_millis(100),
            2, // Max retries
            Duration::from_millis(10), // Retry delay
        ).await;
        
        // The function should fail all attempts
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }
} 