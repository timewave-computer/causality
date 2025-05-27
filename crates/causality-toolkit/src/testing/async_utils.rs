//! Async testing utilities

use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;

/// Run an async test with a default timeout
pub async fn run_async_test<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    timeout_test(future, Duration::from_secs(30)).await
}

/// Run an async test with a custom timeout
pub async fn timeout_test<F, T>(future: F, duration: Duration) -> T
where
    F: Future<Output = T>,
{
    timeout(duration, future)
        .await
        .expect("Test timed out")
}

/// Run a blocking test in an async context
pub fn run_blocking_test<F, T>(test_fn: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async move {
        tokio::task::spawn_blocking(test_fn)
            .await
            .expect("Blocking test panicked")
    })
}

/// Helper to run async code in sync tests
pub fn block_on<F: Future>(future: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(future)),
        Err(_) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(future)
        }
    }
}

/// Create a test runtime for async tests
pub fn create_test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create test runtime")
}

/// Async test macro helper
#[macro_export]
macro_rules! async_test {
    ($test_fn:expr) => {
        $crate::testing::async_utils::run_async_test($test_fn).await
    };
    ($test_fn:expr, $timeout:expr) => {
        $crate::testing::async_utils::timeout_test($test_fn, $timeout).await
    };
}

/// Wait for a condition to become true with timeout
pub async fn wait_for_condition<F>(mut condition: F, timeout_duration: Duration) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout_duration {
        if condition() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    false
}

/// Wait for an async condition to become true with timeout
pub async fn wait_for_async_condition<F, Fut>(mut condition: F, timeout_duration: Duration) -> bool
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout_duration {
        if condition().await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    false
}

/// Retry an async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;
    
    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_retries - 1 {
                    return Err(e);
                }
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
    
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_run_async_test() {
        let result = run_async_test(async { 42 }).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_timeout_test() {
        let result = timeout_test(async { "success" }, Duration::from_secs(1)).await;
        assert_eq!(result, "success");
    }

    #[test]
    fn test_run_blocking_test() {
        let result = run_blocking_test(|| {
            std::thread::sleep(Duration::from_millis(10));
            "blocking_result"
        });
        assert_eq!(result, "blocking_result");
    }

    #[test]
    fn test_block_on() {
        let result = block_on(async { "async_result" });
        assert_eq!(result, "async_result");
    }

    #[tokio::test]
    async fn test_wait_for_condition() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        // Start a task that increments the counter after a delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            *counter_clone.lock().unwrap() = 5;
        });
        
        let result = wait_for_condition(
            || *counter.lock().unwrap() == 5,
            Duration::from_secs(1),
        ).await;
        
        assert!(result);
    }

    #[tokio::test]
    async fn test_wait_for_async_condition() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        // Start a task that increments the counter after a delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            *counter_clone.lock().unwrap() = 10;
        });
        
        let result = wait_for_async_condition(
            || {
                let counter = counter.clone();
                async move { *counter.lock().unwrap() == 10 }
            },
            Duration::from_secs(1),
        ).await;
        
        assert!(result);
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    if *count < 3 {
                        Err("not ready")
                    } else {
                        Ok("success")
                    }
                }
            },
            5,
            Duration::from_millis(10),
        ).await;
        
        assert_eq!(result, Ok("success"));
        assert_eq!(*counter.lock().unwrap(), 3);
    }
} 