// Fork pattern for concurrency
//
// This module provides the fork pattern, which allows running multiple
// futures concurrently and collecting their results.

use std::future::Future;
use std::pin::Pin;

use crate::error::{Error, Result};

/// Run multiple futures concurrently, collecting their results
///
/// This function takes a collection of futures and runs them all concurrently,
/// returning a vector of their results. It's similar to `futures::future::join_all`.
pub async fn fork<F, T>(futures: Vec<F>) -> Vec<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    // Create a vector of handles
    let mut handles = Vec::with_capacity(futures.len());
    
    // Spawn each future as a tokio task
    for future in futures {
        let handle = tokio::spawn(future);
        handles.push(handle);
    }
    
    // Wait for all tasks to complete and collect results
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        // Await the join handle, which gives a Result<T, JoinError>
        // We ignore JoinError (panic in the task) and just use the default value
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }
    
    results
}

/// Run multiple fallible futures concurrently, collecting their results
///
/// This is similar to `fork`, but for futures that return `Result<T, E>`.
/// If any future fails, the error is returned, otherwise a vector of results is returned.
pub async fn fork_join<F, T, E>(futures: Vec<F>) -> Result<Vec<T>>
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
    
    // Create a vector of handles
    let mut handles = Vec::with_capacity(futures.len());
    
    // Spawn each future as a tokio task
    for future in futures {
        let handle = tokio::spawn(future);
        handles.push(handle);
    }
    
    // Wait for all tasks to complete and collect results
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        // Await the join handle, which gives a Result<Result<T, Error>, JoinError>
        match handle.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(err)) => return Err(err),
            Err(err) => return Err(Error::OperationFailed(format!("Task failed: {}", err))),
        }
    }
    
    Ok(results)
}

/// Run multiple futures concurrently and return all results, both successes and failures
///
/// This is similar to `fork_join`, but it doesn't short-circuit on errors.
/// Instead, it returns a vector of Results, both successes and failures.
pub async fn fork_try_join<F, T, E>(futures: Vec<F>) -> Vec<std::result::Result<T, Error>>
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
    
    // Fork the futures
    let results = fork(futures).await;
    
    // Return all results
    results
}

/// Run multiple futures concurrently and call a callback for each result
///
/// This is similar to `fork`, but it calls a callback for each result
/// as soon as it's available, rather than waiting for all to complete.
pub async fn fork_each<F, T, C, R>(futures: Vec<F>, mut callback: C) -> Vec<R>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
    C: FnMut(T) -> R + Send + 'static,
    R: Send + 'static,
{
    // Create a channel for sending results
    let (tx, mut rx) = tokio::sync::mpsc::channel(futures.len());
    
    // Spawn each future as a tokio task
    for (i, future) in futures.into_iter().enumerate() {
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = future.await;
            let _ = tx.send((i, result)).await;
        });
    }
    
    // Drop the original sender to ensure the channel closes when all tasks are done
    drop(tx);
    
    // Collect results in order of completion
    let mut results = Vec::new();
    while let Some((i, result)) = rx.recv().await {
        // Call the callback with the result
        let callback_result = callback(result);
        
        // Insert the result at the correct position
        if i >= results.len() {
            results.resize_with(i + 1, || None);
        }
        results[i] = Some(callback_result);
    }
    
    // Remove any None values (from tasks that didn't complete)
    results.into_iter().filter_map(|r| r).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_fork() {
        // Create futures that return their inputs
        let futures = vec![
            async { 1 },
            async { 2 },
            async { 3 },
        ];
        
        // Fork the futures
        let results = fork(futures).await;
        
        // Check the results
        assert_eq!(results, vec![1, 2, 3]);
    }
    
    #[tokio::test]
    async fn test_fork_join() {
        // Create futures that return their inputs as Results
        let futures = vec![
            async { Result::<_, Error>::Ok(1) },
            async { Result::<_, Error>::Ok(2) },
            async { Result::<_, Error>::Ok(3) },
        ];
        
        // Fork the futures
        let results = fork_join(futures).await.unwrap();
        
        // Check the results
        assert_eq!(results, vec![1, 2, 3]);
    }
    
    #[tokio::test]
    async fn test_fork_join_error() {
        // Create futures that return their inputs as Results, with one error
        let futures = vec![
            async { Result::<_, Error>::Ok(1) },
            async { Result::<_, Error>::Err(Error::OperationFailed("test error".to_string())) },
            async { Result::<_, Error>::Ok(3) },
        ];
        
        // Fork the futures
        let result = fork_join(futures).await;
        
        // Check the result
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_fork_try_join() {
        // Create futures that return their inputs as Results, with one error
        let futures = vec![
            async { Result::<_, Error>::Ok(1) },
            async { Result::<_, Error>::Err(Error::OperationFailed("test error".to_string())) },
            async { Result::<_, Error>::Ok(3) },
        ];
        
        // Fork the futures
        let results = fork_try_join(futures).await;
        
        // Check the results
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }
    
    #[tokio::test]
    async fn test_fork_each() {
        // Create futures that return their inputs after some delay
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
        
        // Track the order of completion
        let mut order = Vec::new();
        
        // Fork the futures
        let results = fork_each(futures, |result| {
            order.push(result);
            result
        }).await;
        
        // Check the results
        assert_eq!(results, vec![1, 2, 3]);
        
        // Check the order of completion (based on the sleep times)
        assert_eq!(order, vec![2, 3, 1]);
    }
} 