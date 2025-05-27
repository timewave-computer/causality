//! Mock PostgreSQL Storage Implementation
//!
//! This module provides a mock implementation of a PostgreSQL-backed storage system
//! for Causality. Rather than using an actual PostgreSQL database, it uses the memory
//! database implementation from causality-db.
//!
//! The real PostgreSQL implementation will be integrated from the Almanac project later.

use causality_db::{Database, DbFactory};
use causality_error::{BoxError, StorageError};
use std::sync::Arc;

/// Mock ResourceId type for the storage implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

impl ResourceId {
    /// Create a new ResourceId from a string
    pub fn from_string(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            return Err("ResourceId cannot be empty".to_string());
        }
        Ok(Self(s.to_string()))
    }
    
    /// Convert the ResourceId to a string
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Mock PostgreSQL connection string
#[derive(Debug, Clone)]
pub struct PostgresConnectionString(pub String);

impl PostgresConnectionString {
    /// Create a new PostgreSQL connection string
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
    
    /// Get the underlying connection string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Mock PostgreSQL client that internally uses memory storage
pub struct PostgresClient {
    /// Connection string (for logging/debugging only)
    connection_string: PostgresConnectionString,
    /// In-memory database used for storage
    db: Arc<Box<dyn Database>>,
}

impl PostgresClient {
    /// Create a new PostgreSQL client with the given connection string
    pub async fn connect(connection_string: PostgresConnectionString) -> Result<Self, BoxError> {
        tracing::info!("Mock PostgreSQL client connecting to: {}", connection_string.as_str());
        
        // Use memory database under the hood
        let db = DbFactory::create_memory_db()
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to create memory DB: {}", e));
                Box::new(storage_err) as BoxError
            })?;
        
        Ok(Self {
            connection_string,
            db: Arc::new(db),
        })
    }
    
    /// Get a resource by ID
    pub async fn get_resource(&self, id: &ResourceId) -> Result<Option<Vec<u8>>, BoxError> {
        let key = id.to_string().into_bytes();
        self.db.get(&key)
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to get resource: {}", e));
                Box::new(storage_err) as BoxError
            })
    }
    
    /// Store a resource
    pub async fn store_resource(&self, id: &ResourceId, data: &[u8]) -> Result<(), BoxError> {
        let key = id.to_string().into_bytes();
        self.db.put(&key, data)
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to store resource: {}", e));
                Box::new(storage_err) as BoxError
            })
    }
    
    /// Delete a resource
    pub async fn delete_resource(&self, id: &ResourceId) -> Result<(), BoxError> {
        let key = id.to_string().into_bytes();
        self.db.delete(&key)
            .map_err(|e| {
                let storage_err = StorageError::DatabaseError(format!("Failed to delete resource: {}", e));
                Box::new(storage_err) as BoxError
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_postgres_client() {
        // Create a mock Postgres client
        let client = PostgresClient::connect(
            PostgresConnectionString::new("postgres://mock:mock@localhost/mock")
        ).await.unwrap();
        
        // Create a test resource ID
        let resource_id = ResourceId::from_string("test:resource:1").unwrap();
        let data = b"test data".to_vec();
        
        // Store and retrieve data
        client.store_resource(&resource_id, &data).await.unwrap();
        let retrieved = client.get_resource(&resource_id).await.unwrap();
        
        assert_eq!(retrieved, Some(data));
        
        // Delete data
        client.delete_resource(&resource_id).await.unwrap();
        let retrieved = client.get_resource(&resource_id).await.unwrap();
        
        assert_eq!(retrieved, None);
    }
} 