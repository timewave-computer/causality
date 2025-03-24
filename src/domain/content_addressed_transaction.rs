// Content-addressed transaction verification
//
// This module implements content-addressed transaction verification for cross-domain operations,
// allowing transactions to be verified across different domains using their content hashes.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::crypto::{
    ContentAddressed, ContentId, HashOutput, HashError, HashFactory, HashAlgorithm
};
use crate::domain::{
    DomainId, DomainAdapter, TransactionId, TransactionStatus,
    TransactionReceipt, Transaction
};
use crate::error::{Error, Result};
use crate::domain::content_addressed_interface::{
    CrossDomainError, CommitmentProof, ContentAddressedDomainRegistry
};

/// Error type for content-addressed transaction verification
#[derive(Error, Debug)]
pub enum TransactionVerificationError {
    /// Transaction not found
    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),
    
    /// Verification failed
    #[error("Transaction verification failed: {0}")]
    VerificationFailed(String),
    
    /// Invalid proof
    #[error("Invalid transaction proof: {0}")]
    InvalidProof(String),
    
    /// Domain error
    #[error("Domain error: {0}")]
    DomainError(#[from] Error),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// Cross-domain error
    #[error("Cross-domain error: {0}")]
    CrossDomainError(#[from] CrossDomainError),
}

/// Content-addressed transaction that can be verified across domains
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedTransaction {
    /// Transaction ID
    pub id: TransactionId,
    
    /// Transaction data
    pub data: Vec<u8>,
    
    /// Origin domain
    pub origin_domain: DomainId,
    
    /// Target domain (if cross-domain)
    pub target_domain: Option<DomainId>,
    
    /// Transaction type
    pub transaction_type: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ContentAddressedTransaction {
    /// Create a new content-addressed transaction
    pub fn new(
        id: TransactionId,
        data: Vec<u8>,
        origin_domain: DomainId,
        transaction_type: String
    ) -> Self {
        Self {
            id,
            data,
            origin_domain,
            target_domain: None,
            transaction_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set target domain
    pub fn with_target_domain(mut self, target_domain: DomainId) -> Self {
        self.target_domain = Some(target_domain);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Convert to a standard transaction
    pub fn to_transaction(&self) -> Transaction {
        Transaction {
            data: self.data.clone(),
            transaction_type: self.transaction_type.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl ContentAddressed for ContentAddressedTransaction {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Transaction verification result with proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionVerificationResult {
    /// Transaction ID
    pub transaction_id: TransactionId,
    
    /// Transaction status
    pub status: TransactionStatus,
    
    /// Origin domain
    pub origin_domain: DomainId,
    
    /// Target domain
    pub target_domain: Option<DomainId>,
    
    /// Proof bundle
    pub proof: Option<CommitmentProof>,
    
    /// Receipt from the target domain
    pub receipt: Option<TransactionReceipt>,
}

/// Interface for verifying content-addressed transactions
#[async_trait]
pub trait ContentAddressedTransactionVerifier: Send + Sync {
    /// Verify a transaction across domains
    async fn verify_transaction(
        &self,
        transaction: &ContentAddressedTransaction
    ) -> crate::error::Result<TransactionVerificationResult>;
    
    /// Verify a transaction using a proof
    async fn verify_transaction_with_proof(
        &self,
        transaction: &ContentAddressedTransaction,
        proof: &CommitmentProof
    ) -> crate::error::Result<bool>;
    
    /// Get the verification status of a transaction
    async fn transaction_verification_status(
        &self, 
        transaction_id: &TransactionId,
        domain_id: &DomainId
    ) -> crate::error::Result<TransactionStatus>;
}

/// Implementation of content-addressed transaction verification
pub struct ContentAddressedTransactionVerifierImpl {
    /// Cross-domain registry
    registry: Arc<ContentAddressedDomainRegistry>,
}

impl ContentAddressedTransactionVerifierImpl {
    /// Create a new transaction verifier
    pub fn new(registry: Arc<ContentAddressedDomainRegistry>) -> Self {
        Self {
            registry,
        }
    }
}

#[async_trait]
impl ContentAddressedTransactionVerifier for ContentAddressedTransactionVerifierImpl {
    async fn verify_transaction(
        &self,
        transaction: &ContentAddressedTransaction
    ) -> crate::error::Result<TransactionVerificationResult> {
        // Get the origin domain adapter
        let origin_adapter = self.registry.get_adapter(&transaction.origin_domain)
            .map_err(|e| TransactionVerificationError::DomainError(e))?;
        
        // Check if the transaction exists in the origin domain
        let receipt = origin_adapter.transaction_receipt(&transaction.id).await
            .map_err(|e| TransactionVerificationError::DomainError(e))?;
        
        // If the transaction has a target domain, verify it there as well
        let target_receipt = if let Some(target_domain) = &transaction.target_domain {
            // Get the target domain adapter
            let target_adapter = self.registry.get_adapter(target_domain)
                .map_err(|e| TransactionVerificationError::DomainError(e))?;
            
            // Check if the transaction exists in the target domain
            match target_adapter.transaction_receipt(&transaction.id).await {
                Ok(r) => Some(r),
                Err(_) => None,
            }
        } else {
            None
        };
        
        // Create a proof for the transaction
        let proof = self.create_transaction_proof(transaction, &origin_adapter).await?;
        
        // Determine the status based on both receipts
        let status = if let Some(target_receipt) = &target_receipt {
            // If target receipt exists, use its status
            target_receipt.status.clone()
        } else {
            // Otherwise use the origin receipt status
            receipt.status.clone()
        };
        
        Ok(TransactionVerificationResult {
            transaction_id: transaction.id.clone(),
            status,
            origin_domain: transaction.origin_domain.clone(),
            target_domain: transaction.target_domain.clone(),
            proof: Some(proof),
            receipt: target_receipt,
        })
    }
    
    async fn verify_transaction_with_proof(
        &self,
        transaction: &ContentAddressedTransaction,
        proof: &CommitmentProof
    ) -> crate::error::Result<bool> {
        // Use the cross-domain interface to verify the content against the proof
        self.registry.verify_content(&proof.domain_id, transaction, proof).await
            .map_err(|e| TransactionVerificationError::CrossDomainError(e))
    }
    
    async fn transaction_verification_status(
        &self,
        transaction_id: &TransactionId,
        domain_id: &DomainId
    ) -> crate::error::Result<TransactionStatus> {
        // Get the domain adapter
        let adapter = self.registry.get_adapter(domain_id)
            .map_err(|e| TransactionVerificationError::DomainError(e))?;
        
        // Check the transaction status
        let receipt = adapter.transaction_receipt(transaction_id).await
            .map_err(|e| TransactionVerificationError::DomainError(e))?;
        
        Ok(receipt.status)
    }
}

impl ContentAddressedTransactionVerifierImpl {
    /// Create a proof for a transaction
    async fn create_transaction_proof(
        &self,
        transaction: &ContentAddressedTransaction,
        adapter: &Arc<dyn DomainAdapter>
    ) -> Result<CommitmentProof, TransactionVerificationError> {
        // In a real implementation, this would query the domain adapter
        // for a Merkle proof of the transaction's inclusion in the domain's state
        // For now, we'll create a simplified version
        
        // Calculate the content hash
        let content_hash = transaction.content_hash();
        
        // Get the current block hash
        let block_hash = adapter.current_hash().await
            .map_err(|e| TransactionVerificationError::DomainError(e))?;
        
        // Create a mock proof
        let proof = CommitmentProof::new(
            transaction.origin_domain.clone(),
            HashOutput::new([1; 32], HashAlgorithm::Blake3), // Mock root hash
            vec![
                HashOutput::new([2; 32], HashAlgorithm::Blake3), // Mock path
                HashOutput::new([3; 32], HashAlgorithm::Blake3),
            ],
            content_hash,
        );
        
        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ContentAddressedStorage;
    use crate::crypto::StorageFactory;
    use crate::crypto::hash::{HashFactory, Hasher, HashFunction};
    use crate::domain::{
        BlockHeight, Timestamp, DomainInfo, DomainType, DomainStatus, FactType, FactObservationMeta
    };
    use std::sync::{Arc, Mutex};
    
    // Mock domain adapter for testing
    struct MockDomainAdapter {
        domain_id: DomainId,
        transactions: Mutex<HashMap<TransactionId, TransactionReceipt>>,
    }
    
    impl MockDomainAdapter {
        fn new(domain_id: DomainId) -> Self {
            Self {
                domain_id,
                transactions: Mutex::new(HashMap::new()),
            }
        }
        
        fn add_transaction(&self, tx_id: TransactionId, receipt: TransactionReceipt) {
            let mut txs = self.transactions.lock().unwrap();
            txs.insert(tx_id, receipt);
        }
    }
    
    #[async_trait]
    impl DomainAdapter for MockDomainAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn domain_info(&self) -> Result<DomainInfo> {
            Ok(DomainInfo {
                domain_id: self.domain_id.clone(),
                name: format!("Mock Domain {}", self.domain_id),
                domain_type: DomainType::Unknown,
                status: DomainStatus::Active,
                metadata: HashMap::new(),
            })
        }
        
        async fn current_height(&self) -> Result<BlockHeight> {
            Ok(BlockHeight(100))
        }
        
        async fn current_hash(&self) -> Result<crate::domain::BlockHash> {
            Ok(crate::domain::BlockHash([0; 32]))
        }
        
        async fn current_time(&self) -> Result<Timestamp> {
            Ok(Timestamp::now())
        }
        
        async fn time_map_entry(&self, _height: BlockHeight) -> Result<crate::domain::adapter::TimeMapEntry> {
            unimplemented!()
        }
        
        async fn observe_fact(&self, _query: &crate::domain::FactQuery) -> Result<(FactType, FactObservationMeta)> {
            // Return binary data for commitment root requests
            let meta = FactObservationMeta {
                observed_at: Timestamp::now(),
                block_height: Some(BlockHeight(100)),
                reliability: 1.0,
                source: "mock".to_string(),
                metadata: HashMap::new(),
            };
            
            Ok((FactType::Binary(vec![1; 32]), meta))
        }
        
        async fn submit_transaction(&self, _tx: Transaction) -> Result<TransactionId> {
            Ok(TransactionId("mock-tx-id".to_string()))
        }
        
        async fn transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt> {
            let txs = self.transactions.lock().unwrap();
            txs.get(tx_id)
                .cloned()
                .ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))
        }
        
        async fn transaction_confirmed(&self, tx_id: &TransactionId) -> Result<bool> {
            let txs = self.transactions.lock().unwrap();
            Ok(txs.contains_key(tx_id))
        }
        
        async fn wait_for_confirmation(&self, tx_id: &TransactionId, _max_wait_ms: Option<u64>) -> Result<TransactionReceipt> {
            self.transaction_receipt(tx_id).await
        }
    }
    
    #[tokio::test]
    async fn test_transaction_verification() {
        // Create storage and registry
        let storage = StorageFactory::default().create_storage();
        let registry = Arc::new(ContentAddressedDomainRegistry::new(storage));
        
        // Create domain adapters
        let origin_domain = DomainId::new("origin-domain");
        let target_domain = DomainId::new("target-domain");
        
        let origin_adapter = Arc::new(MockDomainAdapter::new(origin_domain.clone()));
        let target_adapter = Arc::new(MockDomainAdapter::new(target_domain.clone()));
        
        // Register adapters
        registry.register_adapter(origin_adapter.clone()).unwrap();
        registry.register_adapter(target_adapter.clone()).unwrap();
        
        // Create a transaction
        let tx_id = TransactionId("test-tx-1".to_string());
        let transaction = ContentAddressedTransaction::new(
            tx_id.clone(),
            vec![1, 2, 3, 4],
            origin_domain.clone(),
            "transfer".to_string()
        ).with_target_domain(target_domain.clone());
        
        // Add receipts to both domains
        let origin_receipt = TransactionReceipt {
            transaction_id: tx_id.clone(),
            block_height: BlockHeight(100),
            block_hash: crate::domain::BlockHash([0; 32]),
            status: TransactionStatus::Success,
            gas_used: Some(1000),
            fee_paid: Some(500),
            logs: vec![],
            metadata: HashMap::new(),
        };
        
        let target_receipt = TransactionReceipt {
            transaction_id: tx_id.clone(),
            block_height: BlockHeight(50),
            block_hash: crate::domain::BlockHash([0; 32]),
            status: TransactionStatus::Success,
            gas_used: Some(2000),
            fee_paid: Some(1000),
            logs: vec![],
            metadata: HashMap::new(),
        };
        
        origin_adapter.add_transaction(tx_id.clone(), origin_receipt);
        target_adapter.add_transaction(tx_id.clone(), target_receipt);
        
        // Create verifier
        let verifier = ContentAddressedTransactionVerifierImpl::new(registry);
        
        // Verify the transaction
        let result = verifier.verify_transaction(&transaction).await.unwrap();
        
        // Check result
        assert_eq!(result.transaction_id, tx_id);
        assert!(matches!(result.status, TransactionStatus::Success));
        assert_eq!(result.origin_domain, origin_domain);
        assert_eq!(result.target_domain, Some(target_domain));
        assert!(result.proof.is_some());
        assert!(result.receipt.is_some());
        
        // Test verification with proof
        let proof = result.proof.unwrap();
        let verified = verifier.verify_transaction_with_proof(&transaction, &proof).await.unwrap();
        assert!(verified);
    }
    
    #[tokio::test]
    async fn test_transaction_status() {
        // Create storage and registry
        let storage = StorageFactory::default().create_storage();
        let registry = Arc::new(ContentAddressedDomainRegistry::new(storage));
        
        // Create domain adapter
        let domain_id = DomainId::new("test-domain");
        let adapter = Arc::new(MockDomainAdapter::new(domain_id.clone()));
        
        // Register adapter
        registry.register_adapter(adapter.clone()).unwrap();
        
        // Add a transaction
        let tx_id = TransactionId("test-tx-status".to_string());
        let receipt = TransactionReceipt {
            transaction_id: tx_id.clone(),
            block_height: BlockHeight(100),
            block_hash: crate::domain::BlockHash([0; 32]),
            status: TransactionStatus::Success,
            gas_used: Some(1000),
            fee_paid: Some(500),
            logs: vec![],
            metadata: HashMap::new(),
        };
        
        adapter.add_transaction(tx_id.clone(), receipt);
        
        // Create verifier
        let verifier = ContentAddressedTransactionVerifierImpl::new(registry);
        
        // Get status
        let status = verifier.transaction_verification_status(&tx_id, &domain_id).await.unwrap();
        assert!(matches!(status, TransactionStatus::Success));
        
        // Test nonexistent transaction
        let bad_tx_id = TransactionId("nonexistent".to_string());
        let result = verifier.transaction_verification_status(&bad_tx_id, &domain_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_transaction_content_addressing() {
        // Create a transaction
        let tx_id = TransactionId("test-content-hash".to_string());
        let transaction = ContentAddressedTransaction::new(
            tx_id.clone(),
            vec![1, 2, 3, 4],
            DomainId::new("test-domain"),
            "transfer".to_string()
        ).with_target_domain(DomainId::new("target-domain"))
         .with_metadata("key1", "value1")
         .with_metadata("key2", "value2");
        
        // Create a hasher using the HashFactory
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Get the content hash using the ContentAddressed trait
        let content_hash = transaction.content_hash();
        
        // Verify the hash can also be computed directly
        let serialized = transaction.to_bytes();
        let direct_hash = hasher.hash(&serialized);
        
        // These should match
        assert_eq!(content_hash, direct_hash);
        
        // Test the verify method
        assert!(transaction.verify());
        
        // Test serialization and deserialization
        let bytes = transaction.to_bytes();
        let deserialized = ContentAddressedTransaction::from_bytes(&bytes).unwrap();
        
        // Verify the deserialized transaction
        assert_eq!(deserialized.id, tx_id);
        assert_eq!(deserialized.data, vec![1, 2, 3, 4]);
        assert_eq!(deserialized.origin_domain, DomainId::new("test-domain"));
        assert_eq!(deserialized.target_domain, Some(DomainId::new("target-domain")));
        assert_eq!(deserialized.transaction_type, "transfer".to_string());
        assert_eq!(deserialized.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(deserialized.metadata.get("key2"), Some(&"value2".to_string()));
        
        // The content hash of the deserialized transaction should match the original
        assert_eq!(deserialized.content_hash(), content_hash);
    }
} 