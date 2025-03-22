// Race pattern for concurrency
//
// This module provides the race pattern, which runs multiple futures concurrently
// and returns the result of the first one to complete.

use std::future::Future;
use std::pin::Pin;

use tokio::sync::oneshot;

use crate::error::{Error, Result};

/// Run multiple futures concurrently and return the result of the first one to complete
///
/// If all futures fail, returns the last error received.
pub async fn race<F, T>(futures: Vec<F>) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    // Special case for empty futures
    if futures.is_empty() {
        panic!("Cannot race empty futures");
    }
    
    // Special case for a single future
    if futures.len() == 1 {
        return futures.into_iter().next().unwrap().await;
    }
    
    // Create a channel for the first result
    let (tx, rx) = oneshot::channel();
    
    // Spawn tasks for each future
    for future in futures {
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = future.await;
            // It's OK if the receiver is dropped - that just means another future won the race
            let _ = tx.send(result);
        });
    }
    
    // Drop the original sender to avoid a memory leak if no future completes
    drop(tx);
    
    // Wait for the first result
    match rx.await {
        Ok(result) => result,
        Err(_) => panic!("All racing futures were dropped without sending a result"),
    }
}

/// Run multiple fallible futures concurrently and return the result of the first one to succeed
///
/// If all futures fail, returns the last error received.
pub async fn race_ok<F, T, E>(futures: Vec<F>) -> Result<T>
where
    F: Future<Output = std::result::Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    // Convert the result type to our Error type
    let futures = futures
        .into_iter()
        .map(|future| async move {
            match future.await {
                Ok(value) => Ok(value),
                Err(err) => Err(err.into()),
            }
        })
        .collect::<Vec<_>>();
    
    // Special case for empty futures
    if futures.is_empty() {
        return Err(Error::OperationFailed("Cannot race empty futures".to_string()));
    }
    
    // Special case for a single future
    if futures.len() == 1 {
        return futures.into_iter().next().unwrap().await;
    }
    
    // Create a channel for the first success result
    let (success_tx, success_rx) = oneshot::channel();
    
    // Create a channel for the last error result
    let (error_tx, error_rx) = oneshot::channel();
    
    // Keep track of how many futures are still running
    let remaining = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(futures.len()));
    
    // Spawn tasks for each future
    for future in futures {
        let success_tx = success_tx.clone();
        let error_tx = error_tx.clone();
        let remaining = remaining.clone();
        
        tokio::spawn(async move {
            let result = future.await;
            
            match result {
                Ok(value) => {
                    // Send the successful result, but it's OK if the receiver is dropped
                    let _ = success_tx.send(value);
                }
                Err(err) => {
                    // Decrement the remaining count
                    let prev = remaining.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    
                    // If this is the last future to complete and it failed, send the error
                    if prev == 1 {
                        let _ = error_tx.send(err);
                    }
                }
            }
        });
    }
    
    // Drop the original senders to avoid memory leaks
    drop(success_tx);
    drop(error_tx);
    
    // Create a combined future that waits for either a success or the last error
    tokio::select! {
        Ok(result) = success_rx => Ok(result),
        Ok(err) = error_rx => Err(err),
        else => Err(Error::OperationFailed("All racing futures were dropped without sending a result".to_string())),
    }
}

/// Run multiple fallible futures concurrently and return the result of the first one to complete
///
/// This is similar to `race`, but for futures that return `Result<T, E>`.
/// Unlike `race_ok`, this returns the result of the first future to complete,
/// whether it's a success or failure.
pub async fn race_result<F, T, E>(futures: Vec<F>) -> std::result::Result<T, Error>
where
    F: Future<Output = std::result::Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    // Convert the result type to our Error type
    let futures = futures
        .into_iter()
        .map(|future| async move {
            match future.await {
                Ok(value) => Ok(value),
                Err(err) => Err(err.into()),
            }
        })
        .collect::<Vec<_>>();
    
    // Race the futures
    race(futures).await
}

/// Run multiple futures concurrently until one of them returns a value that satisfies a predicate
///
/// This is similar to `race`, but instead of returning the result of the first future to complete,
/// it returns the result of the first future whose result satisfies a predicate.
pub async fn race_until<F, T, P>(futures: Vec<F>, predicate: P) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Clone + Send + 'static,
    P: Fn(&T) -> bool + Send + Sync + 'static,
{
    // Special case for empty futures
    if futures.is_empty() {
        panic!("Cannot race empty futures");
    }
    
    // Special case for a single future
    if futures.len() == 1 {
        let result = futures.into_iter().next().unwrap().await;
        if predicate(&result) {
            return result;
        } else {
            panic!("The only future didn't satisfy the predicate");
        }
    }
    
    // Create a channel for the first result that satisfies the predicate
    let (tx, rx) = oneshot::channel();
    
    // Create a predicate Arc for sharing
    let predicate = std::sync::Arc::new(predicate);
    
    // Spawn tasks for each future
    for future in futures {
        let tx = tx.clone();
        let predicate = predicate.clone();
        
        tokio::spawn(async move {
            let result = future.await;
            
            if predicate(&result) {
                // It's OK if the receiver is dropped - that means another future already won the race
                let _ = tx.send(result);
            }
        });
    }
    
    // Drop the original sender to avoid a memory leak if no future completes
    drop(tx);
    
    // Wait for a result that satisfies the predicate
    match rx.await {
        Ok(result) => result,
        Err(_) => panic!("All racing futures were dropped without sending a result that satisfied the predicate"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_race() {
        // Create futures that return after different delays
        let futures = vec![
            async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                1
            },
            async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                2
            },
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                3
            },
        ];
        
        // Race the futures
        let result = race(futures).await;
        
        // The second future should win
        assert_eq!(result, 2);
    }
    
    #[tokio::test]
    async fn test_race_ok() {
        // Create futures that return after different delays
        let futures = vec![
            async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Result::<_, Error>::Ok(1)
            },
            async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Result::<_, Error>::Err(Error::OperationFailed("test error".to_string()))
            },
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Result::<_, Error>::Ok(3)
            },
        ];
        
        // Race the futures
        let result = race_ok(futures).await;
        
        // The third future should win, since the second returns an error
        assert_eq!(result.unwrap(), 3);
    }
    
    #[tokio::test]
    async fn test_race_ok_all_errors() {
        // Create futures that all return errors
        let futures = vec![
            async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Result::<i32, _>::Err(Error::OperationFailed("error 1".to_string()))
            },
            async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Result::<i32, _>::Err(Error::OperationFailed("error 2".to_string()))
            },
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Result::<i32, _>::Err(Error::OperationFailed("error 3".to_string()))
            },
        ];
        
        // Race the futures
        let result = race_ok(futures).await;
        
        // All futures should fail, so we should get an error
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_race_result() {
        // Create futures that return after different delays
        let futures = vec![
            async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Result::<_, Error>::Ok(1)
            },
            async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Result::<_, Error>::Err(Error::OperationFailed("test error".to_string()))
            },
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Result::<_, Error>::Ok(3)
            },
        ];
        
        // Race the futures
        let result = race_result(futures).await;
        
        // The second future should win, but it returns an error
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_race_until() {
        // Create futures that return after different delays
        let futures = vec![
            async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                1
            },
            async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                2
            },
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                3
            },
        ];
        
        // Race the futures until one returns an odd number
        let result = race_until(futures, |&n| n % 2 != 0).await;
        
        // The third future should win, since it's the first to return an odd number
        assert_eq!(result, 3);
    }
} 