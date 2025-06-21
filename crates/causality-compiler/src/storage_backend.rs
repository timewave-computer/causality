// ------------ STORAGE BACKEND INTEGRATION ------------
// Purpose: Real database integration for Almanac storage backends

use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Real Almanac storage integration when feature is enabled
#[cfg(feature = "almanac")]
use indexer_storage::{Storage, StorageConfig, PostgresStorage, RocksDbStorage};

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageBackendConfig {
    /// Backend type selection
    pub backend_type: StorageBackendType,
    /// PostgreSQL configuration (if using PostgreSQL)
    pub postgres_config: Option<PostgresConfig>,
    /// RocksDB configuration (if using RocksDB)
    pub rocksdb_config: Option<RocksDbConfig>,
    /// Connection pool settings
    pub pool_config: PoolConfig,
    /// Migration settings
    pub migration_config: MigrationConfig,
}

/// Storage backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackendType {
    /// PostgreSQL database
    PostgreSQL,
    /// RocksDB embedded database
    RocksDB,
    /// In-memory storage (for testing)
    InMemory,
}

/// PostgreSQL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// SSL mode
    pub ssl_mode: String,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

/// RocksDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocksDbConfig {
    /// Database path
    pub path: String,
    /// Maximum number of open files
    pub max_open_files: i32,
    /// Write buffer size
    pub write_buffer_size: usize,
    /// Block cache size
    pub block_cache_size: usize,
    /// Compression type
    pub compression_type: CompressionType,
}

/// RocksDB compression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Snappy,
    Zlib,
    Bz2,
    Lz4,
    Lz4hc,
    Zstd,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Whether to run migrations automatically
    pub auto_migrate: bool,
    /// Migration directory path
    pub migration_path: String,
    /// Whether to create database if it doesn't exist
    pub create_database: bool,
}

impl Default for StorageBackendConfig {
    fn default() -> Self {
        Self {
            backend_type: StorageBackendType::RocksDB,
            postgres_config: None,
            rocksdb_config: Some(RocksDbConfig::default()),
            pool_config: PoolConfig::default(),
            migration_config: MigrationConfig::default(),
        }
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "causality".to_string(),
            username: "causality".to_string(),
            password: "causality".to_string(),
            ssl_mode: "prefer".to_string(),
            connection_timeout: 30,
        }
    }
}

impl Default for RocksDbConfig {
    fn default() -> Self {
        Self {
            path: "./data/causality.db".to_string(),
            max_open_files: 1000,
            write_buffer_size: 64 * 1024 * 1024, // 64MB
            block_cache_size: 256 * 1024 * 1024, // 256MB
            compression_type: CompressionType::Lz4,
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            idle_timeout: 600,
        }
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            auto_migrate: true,
            migration_path: "./migrations".to_string(),
            create_database: true,
        }
    }
}

/// Storage backend manager
pub struct StorageBackendManager {
    config: StorageBackendConfig,
    #[cfg(feature = "almanac")]
    storage: Option<Arc<dyn Storage + Send + Sync>>,
    #[cfg(not(feature = "almanac"))]
    storage: Option<Arc<MockStorage>>,
}

impl StorageBackendManager {
    /// Create a new storage backend manager
    pub fn new(config: StorageBackendConfig) -> Self {
        Self {
            config,
            storage: None,
        }
    }

    /// Initialize the storage backend
    pub async fn initialize(&mut self) -> Result<()> {
        match self.config.backend_type {
            StorageBackendType::PostgreSQL => {
                self.initialize_postgres().await?;
            }
            StorageBackendType::RocksDB => {
                self.initialize_rocksdb().await?;
            }
            StorageBackendType::InMemory => {
                self.initialize_in_memory().await?;
            }
        }
        Ok(())
    }

    /// Initialize PostgreSQL backend
    #[cfg(feature = "almanac")]
    async fn initialize_postgres(&mut self) -> Result<()> {
        let postgres_config = self.config.postgres_config.as_ref()
            .ok_or_else(|| anyhow!("PostgreSQL config required for PostgreSQL backend"))?;

        let storage_config = StorageConfig::postgres(
            &postgres_config.host,
            postgres_config.port,
            &postgres_config.database,
            &postgres_config.username,
            &postgres_config.password,
        );

        let storage = PostgresStorage::new(storage_config).await?;
        
        // Run migrations if configured
        if self.config.migration_config.auto_migrate {
            storage.migrate().await?;
        }

        self.storage = Some(Arc::new(storage));
        Ok(())
    }

    #[cfg(not(feature = "almanac"))]
    async fn initialize_postgres(&mut self) -> Result<()> {
        // Mock implementation for development
        self.storage = Some(Arc::new(MockStorage::new()));
        Ok(())
    }

    /// Initialize RocksDB backend
    #[cfg(feature = "almanac")]
    async fn initialize_rocksdb(&mut self) -> Result<()> {
        let rocksdb_config = self.config.rocksdb_config.as_ref()
            .ok_or_else(|| anyhow!("RocksDB config required for RocksDB backend"))?;

        let storage_config = StorageConfig::rocksdb(&rocksdb_config.path);
        let storage = RocksDbStorage::new(storage_config).await?;

        self.storage = Some(Arc::new(storage));
        Ok(())
    }

    #[cfg(not(feature = "almanac"))]
    async fn initialize_rocksdb(&mut self) -> Result<()> {
        // Mock implementation for development
        self.storage = Some(Arc::new(MockStorage::new()));
        Ok(())
    }

    /// Initialize in-memory backend
    async fn initialize_in_memory(&mut self) -> Result<()> {
        #[cfg(feature = "almanac")]
        {
            let storage_config = StorageConfig::in_memory();
            let storage = indexer_storage::InMemoryStorage::new(storage_config).await?;
            self.storage = Some(Arc::new(storage));
        }
        
        #[cfg(not(feature = "almanac"))]
        {
            self.storage = Some(Arc::new(MockStorage::new()));
        }
        
        Ok(())
    }

    /// Get the storage instance
    #[cfg(feature = "almanac")]
    pub fn storage(&self) -> Option<Arc<dyn Storage + Send + Sync>> {
        self.storage.clone()
    }

    #[cfg(not(feature = "almanac"))]
    pub fn storage(&self) -> Option<Arc<MockStorage>> {
        self.storage.clone()
    }

    /// Test the storage connection
    pub async fn test_connection(&self) -> Result<bool> {
        if let Some(storage) = &self.storage {
            #[cfg(feature = "almanac")]
            {
                storage.health_check().await.map_err(|e| anyhow!("Storage health check failed: {}", e))
            }
            
            #[cfg(not(feature = "almanac"))]
            {
                storage.health_check().await
            }
        } else {
            Err(anyhow!("Storage not initialized"))
        }
    }

    /// Get storage statistics
    pub async fn get_statistics(&self) -> Result<StorageStatistics> {
        if let Some(storage) = &self.storage {
            #[cfg(feature = "almanac")]
            {
                let stats = storage.get_statistics().await?;
                Ok(StorageStatistics {
                    total_events: stats.total_events,
                    total_accounts: stats.total_accounts,
                    total_contracts: stats.total_contracts,
                    storage_size_bytes: stats.storage_size_bytes,
                    last_indexed_block: stats.last_indexed_block,
                })
            }
            
            #[cfg(not(feature = "almanac"))]
            {
                Ok(StorageStatistics::mock())
            }
        } else {
            Err(anyhow!("Storage not initialized"))
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    pub total_events: u64,
    pub total_accounts: u64,
    pub total_contracts: u64,
    pub storage_size_bytes: u64,
    pub last_indexed_block: u64,
}

impl StorageStatistics {
    #[cfg(not(feature = "almanac"))]
    pub fn mock() -> Self {
        Self {
            total_events: 1000,
            total_accounts: 50,
            total_contracts: 25,
            storage_size_bytes: 1024 * 1024, // 1MB
            last_indexed_block: 12345,
        }
    }
}

/// Mock storage for development (when almanac feature is not enabled)
#[cfg(not(feature = "almanac"))]
pub struct MockStorage {
    data: BTreeMap<String, String>,
}

#[cfg(not(feature = "almanac"))]
impl MockStorage {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    pub async fn store(&mut self, key: String, value: String) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.data.get(key).cloned())
    }
}

/// Database migration utilities
pub struct MigrationManager {
    config: MigrationConfig,
}

impl MigrationManager {
    pub fn new(config: MigrationConfig) -> Self {
        Self { config }
    }

    /// Run database migrations
    pub async fn run_migrations(&self, storage_type: &StorageBackendType) -> Result<()> {
        match storage_type {
            StorageBackendType::PostgreSQL => {
                self.run_postgres_migrations().await
            }
            StorageBackendType::RocksDB => {
                self.run_rocksdb_migrations().await
            }
            StorageBackendType::InMemory => {
                // No migrations needed for in-memory storage
                Ok(())
            }
        }
    }

    async fn run_postgres_migrations(&self) -> Result<()> {
        #[cfg(feature = "almanac")]
        {
            // Use Almanac's migration system
            let migration_runner = indexer_storage::migrations::MigrationRunner::new(&self.config.migration_path);
            migration_runner.run().await?;
        }
        
        #[cfg(not(feature = "almanac"))]
        {
            // Mock migration for development
            log::info!("Mock PostgreSQL migrations completed");
        }
        
        Ok(())
    }

    async fn run_rocksdb_migrations(&self) -> Result<()> {
        #[cfg(feature = "almanac")]
        {
            // RocksDB migrations are typically schema-less, but we can version the data format
            let migration_runner = indexer_storage::migrations::RocksDbMigrationRunner::new(&self.config.migration_path);
            migration_runner.run().await?;
        }
        
        #[cfg(not(feature = "almanac"))]
        {
            // Mock migration for development
            log::info!("Mock RocksDB migrations completed");
        }
        
        Ok(())
    }
}

/// Storage backend factory
pub struct StorageBackendFactory;

impl StorageBackendFactory {
    /// Create a storage backend from configuration
    pub async fn create(config: StorageBackendConfig) -> Result<StorageBackendManager> {
        let mut manager = StorageBackendManager::new(config);
        manager.initialize().await?;
        Ok(manager)
    }

    /// Create a PostgreSQL storage backend
    pub async fn create_postgres(postgres_config: PostgresConfig) -> Result<StorageBackendManager> {
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::PostgreSQL,
            postgres_config: Some(postgres_config),
            rocksdb_config: None,
            pool_config: PoolConfig::default(),
            migration_config: MigrationConfig::default(),
        };
        Self::create(config).await
    }

    /// Create a RocksDB storage backend
    pub async fn create_rocksdb(rocksdb_config: RocksDbConfig) -> Result<StorageBackendManager> {
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::RocksDB,
            postgres_config: None,
            rocksdb_config: Some(rocksdb_config),
            pool_config: PoolConfig::default(),
            migration_config: MigrationConfig::default(),
        };
        Self::create(config).await
    }

    /// Create an in-memory storage backend (for testing)
    pub async fn create_in_memory() -> Result<StorageBackendManager> {
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: PoolConfig::default(),
            migration_config: MigrationConfig::default(),
        };
        Self::create(config).await
    }
} 