// Race pattern for first-to-complete execution
// Original file: src/concurrency/patterns/race.rs

// Race pattern for concurrency
//
// This module provides the race pattern, which runs multiple futures concurrently
// and returns the result of the first one to complete.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::oneshot;

use causality_types::{Error, Result};

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
pub async fn race_ok<F, T, E>(futures: Vec<F>) -> Result<T, E>
where
    F: Future<Output = std::result::Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Send + 'static + Default,
{
    if futures.is_empty() {
        panic!("Empty futures vector passed to race_ok");
    }

    // Use an mpsc channel instead of cloning oneshot::Sender
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel(futures.len());
    
    for (idx, future) in futures.into_iter().enumerate() {
        let result_tx = result_tx.clone();
        
        tokio::spawn(async move {
            let result = future.await;
            let _ = result_tx.send((idx, result)).await;
        });
    }
    
    // Drop the original sender
    drop(result_tx);
    
    // Wait for any result
    match result_rx.recv().await {
        Some((_, result)) => result,
        None => Err(E::default()),
    }
}

/// Run multiple fallible futures concurrently and return the result of the first one to complete
///
/// This is similar to `race`, but for futures that return `Result<T, E>`.
/// Unlike `race_ok`, this returns the result of the first future to complete,
/// whether it's a success or failure.
pub async fn race_result<F, T, E>(futures: Vec<F>) -> std::result::Result<T, E>
where
    F: Future<Output = std::result::Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    // Convert the result type to our Error type
    let futures = futures
        .into_iter()
        .map(|future| async move {
            match future.await {
                Ok(value) => Ok(value),
                Err(err) => Err(err),
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
    if futures.is_empty() {
        panic!("Empty futures vector passed to race_until");
    }
    
    // Use an mpsc channel instead of cloning oneshot
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel(futures.len());
    let predicate = Arc::new(predicate);
    
    for (idx, future) in futures.into_iter().enumerate() {
        let result_tx = result_tx.clone();
        let predicate = Arc::clone(&predicate);
        
        tokio::spawn(async move {
            let result = future.await;
            
            if predicate(&result) {
                let _ = result_tx.send((idx, result)).await;
            }
        });
    }
    
    // Drop the original sender
    drop(result_tx);
    
    // Wait for a result that satisfies the predicate
    match result_rx.recv().await {
        Some((_, result)) => result,
        None => panic!("All racing futures were dropped without sending a result that satisfied the predicate"),
    }
}

// Replace implementation with a version that doesn't need to clone oneshot::Sender
pub async fn race_first_ok_error<F, T, E>(futures: Vec<F>) -> (Option<T>, Option<E>)
where
    F: Future<Output = std::result::Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    if futures.is_empty() {
        return (None, None);
    }

    // Use mpsc channels for both success and error results
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel(futures.len());
    
    for (idx, future) in futures.into_iter().enumerate() {
        let result_tx = result_tx.clone();
        
        tokio::spawn(async move {
            match future.await {
                Ok(result) => {
                    let _ = result_tx.send((idx, Ok(result))).await;
                }
                Err(error) => {
                    let _ = result_tx.send((idx, Err(error))).await;
                }
            }
        });
    }
    
    // Drop the original sender
    drop(result_tx);
    
    let mut success = None;
    let mut error = None;
    
    // Process results until we get a success or run out of results
    while let Some((_, result)) = result_rx.recv().await {
        match result {
            Ok(value) => {
                success = Some(value);
                break;
            }
            Err(err) => {
                if error.is_none() {
                    error = Some(err);
                }
            }
        }
    }
    
    (success, error)
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