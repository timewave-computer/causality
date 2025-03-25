<!-- Encryption for storage -->
<!-- Original file: docs/src/storage_encryption.md -->

# Storage Encryption in Causality

## Overview

This document describes the storage encryption architecture in the Causality system. Encryption is a foundational component of the storage subsystem, ensuring data confidentiality, protecting sensitive information, and enabling secure cross-domain operations. The architecture provides layers of encryption with different security properties, integrates with the content-addressed storage system, and supports a variety of encryption algorithms and key management schemes.

## Core Concepts

### Encryption Manager

At the core of the storage encryption architecture is the `StorageEncryptionManager`, which provides unified encryption services:

```rust
pub struct StorageEncryptionManager {
    /// Key management service
    key_manager: Arc<KeyManagementService>,
    
    /// Crypto providers for different algorithms
    crypto_providers: HashMap<EncryptionAlgorithm, Box<dyn CryptoProvider>>,
    
    /// Default encryption algorithm
    default_algorithm: EncryptionAlgorithm,
    
    /// Encryption policy manager
    policy_manager: Arc<EncryptionPolicyManager>,
}

impl StorageEncryptionManager {
    /// Encrypt data
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        context: &EncryptionContext,
    ) -> Result<EncryptedData, EncryptionError> {
        // Get the algorithm to use
        let algorithm = context.algorithm.unwrap_or(self.default_algorithm);
        
        // Get the crypto provider
        let provider = self.crypto_providers.get(&algorithm)
            .ok_or_else(|| EncryptionError::UnsupportedAlgorithm(algorithm))?;
        
        // Check if encryption is allowed by policy
        self.policy_manager.validate_encryption_request(context)?;
        
        // Get or generate data encryption key (DEK)
        let dek = match &context.key_id {
            Some(id) => self.key_manager.get_key(id.clone())?,
            None => self.key_manager.generate_data_encryption_key(algorithm)?,
        };
        
        // Encrypt the data with the provider
        let (ciphertext, metadata) = provider.encrypt(plaintext, &dek, context)?;
        
        // Encrypt the DEK with a KEK if necessary
        let encrypted_key = if context.protect_key {
            Some(self.key_manager.wrap_key(&dek, context.kek_id.clone())?)
        } else {
            None
        };
        
        // Create encrypted data object
        let encrypted_data = EncryptedData {
            ciphertext,
            algorithm,
            key_id: dek.id.clone(),
            encrypted_key,
            metadata,
            created_at: SystemTime::now(),
        };
        
        // Audit the encryption operation
        self.audit_encryption_operation(&encrypted_data, context)?;
        
        Ok(encrypted_data)
    }
    
    /// Decrypt data
    pub fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        context: &DecryptionContext,
    ) -> Result<Vec<u8>, EncryptionError> {
        // Get the crypto provider
        let provider = self.crypto_providers.get(&encrypted_data.algorithm)
            .ok_or_else(|| EncryptionError::UnsupportedAlgorithm(encrypted_data.algorithm))?;
        
        // Check if decryption is allowed by policy
        self.policy_manager.validate_decryption_request(encrypted_data, context)?;
        
        // Get the decryption key
        let key = match &encrypted_data.encrypted_key {
            Some(wrapped_key) => {
                // Unwrap the key using the KEK
                let kek_id = context.kek_id.clone()
                    .ok_or_else(|| EncryptionError::MissingKekId)?;
                    
                self.key_manager.unwrap_key(wrapped_key, kek_id)?
            },
            None => {
                // Get the key directly from key manager
                self.key_manager.get_key(encrypted_data.key_id.clone())?
            }
        };
        
        // Decrypt the data
        let plaintext = provider.decrypt(
            &encrypted_data.ciphertext,
            &key,
            &encrypted_data.metadata,
            context,
        )?;
        
        // Audit the decryption operation
        self.audit_decryption_operation(encrypted_data, context)?;
        
        Ok(plaintext)
    }
}
```

### Encryption Algorithms

The system supports multiple encryption algorithms for different security requirements:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-GCM with 128-bit key
    AesGcm128,
    
    /// AES-GCM with 256-bit key
    AesGcm256,
    
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
    
    /// XChaCha20-Poly1305 (extended nonce)
    XChaCha20Poly1305,
    
    /// Deoxys-II (for memory-hard encryption)
    DeoxysII,
    
    /// Format-preserving encryption (FF1)
    FormatPreservingFF1,
    
    /// Deterministic encryption
    DeterministicAes,
    
    /// Homomorphic encryption (limited operations)
    LimitedHomomorphic,
    
    /// Threshold encryption
    ThresholdEncryption,
}
```

### Crypto Provider Interface

Different encryption algorithms are implemented through the `CryptoProvider` interface:

```rust
pub trait CryptoProvider: Send + Sync {
    /// Get the algorithm implemented by this provider
    fn algorithm(&self) -> EncryptionAlgorithm;
    
    /// Encrypt data
    fn encrypt(
        &self,
        plaintext: &[u8],
        key: &EncryptionKey,
        context: &EncryptionContext,
    ) -> Result<(Vec<u8>, EncryptionMetadata), EncryptionError>;
    
    /// Decrypt data
    fn decrypt(
        &self,
        ciphertext: &[u8],
        key: &EncryptionKey,
        metadata: &EncryptionMetadata,
        context: &DecryptionContext,
    ) -> Result<Vec<u8>, EncryptionError>;
    
    /// Check if this provider supports a specific feature
    fn supports_feature(&self, feature: EncryptionFeature) -> bool;
}
```

### Key Management

Secure key management is provided through a dedicated key management service:

```rust
pub struct KeyManagementService {
    /// Key storage
    key_store: Arc<dyn KeyStore>,
    
    /// Key derivation service
    key_derivation: Arc<KeyDerivationService>,
    
    /// Key rotation service
    key_rotation: Arc<KeyRotationService>,
    
    /// External key management integration
    external_integration: Option<Arc<dyn ExternalKeyIntegration>>,
}

impl KeyManagementService {
    /// Generate a new data encryption key
    pub fn generate_data_encryption_key(
        &self,
        algorithm: EncryptionAlgorithm,
    ) -> Result<EncryptionKey, KeyManagementError> {
        // Determine key size based on algorithm
        let key_size = match algorithm {
            EncryptionAlgorithm::AesGcm128 => 16, // 128 bits
            EncryptionAlgorithm::AesGcm256 => 32, // 256 bits
            EncryptionAlgorithm::ChaCha20Poly1305 => 32, // 256 bits
            EncryptionAlgorithm::XChaCha20Poly1305 => 32, // 256 bits
            EncryptionAlgorithm::DeoxysII => 32, // 256 bits
            EncryptionAlgorithm::FormatPreservingFF1 => 32, // 256 bits
            EncryptionAlgorithm::DeterministicAes => 32, // 256 bits
            EncryptionAlgorithm::LimitedHomomorphic => 64, // 512 bits
            EncryptionAlgorithm::ThresholdEncryption => 32, // 256 bits
        };
        
        // Generate secure random bytes for the key
        let mut key_material = vec![0u8; key_size];
        getrandom::getrandom(&mut key_material)
            .map_err(|e| KeyManagementError::RandomError(e.to_string()))?;
        
        // Create the key with metadata
        let key = EncryptionKey {
            id: KeyId::generate(),
            algorithm,
            material: key_material,
            created_at: SystemTime::now(),
            expires_at: None, // No expiration by default
            usage: KeyUsage::DataEncryption,
            metadata: KeyMetadata::new(),
        };
        
        // Store the key
        self.key_store.store_key(&key)?;
        
        // Schedule key rotation if needed
        if let Some(rotation_period) = self.get_rotation_period(algorithm) {
            self.key_rotation.schedule_rotation(key.id.clone(), rotation_period);
        }
        
        Ok(key)
    }
    
    /// Wrap a key using a key encryption key (KEK)
    pub fn wrap_key(
        &self,
        key: &EncryptionKey,
        kek_id: Option<KeyId>,
    ) -> Result<WrappedKey, KeyManagementError> {
        // Get or generate the KEK
        let kek = match kek_id {
            Some(id) => self.key_store.get_key(&id)?,
            None => {
                // Use default KEK
                self.get_default_kek(key.algorithm)?
            }
        };
        
        // Ensure the KEK is authorized for key wrapping
        if kek.usage != KeyUsage::KeyEncryption && kek.usage != KeyUsage::All {
            return Err(KeyManagementError::UnauthorizedKeyUsage(
                format!("Key {} is not authorized for key wrapping", kek.id)
            ));
        }
        
        // Encrypt the key material with the KEK
        // In a real implementation, we'd use a proper key wrapping algorithm like AES-KW
        // For simplicity, we're using AES-GCM here
        let nonce = {
            let mut n = [0u8; 12];
            getrandom::getrandom(&mut n).map_err(|e| KeyManagementError::RandomError(e.to_string()))?;
            n
        };
        
        let aad = kek.id.to_string().as_bytes();
        
        let wrapped_material = match kek.algorithm {
            EncryptionAlgorithm::AesGcm128 | EncryptionAlgorithm::AesGcm256 => {
                let cipher = aes_gcm::Aes256Gcm::new(
                    aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_slice(&kek.material)
                );
                let nonce = aes_gcm::Nonce::from_slice(&nonce);
                
                cipher.encrypt(nonce, aes_gcm::aead::Payload { msg: &key.material, aad })
                    .map_err(|e| KeyManagementError::EncryptionError(e.to_string()))?
            },
            _ => {
                return Err(KeyManagementError::UnsupportedAlgorithm(
                    format!("Key wrapping with algorithm {:?} is not supported", kek.algorithm)
                ));
            }
        };
        
        // Create wrapped key
        let wrapped_key = WrappedKey {
            wrapped_material,
            key_id: key.id.clone(),
            kek_id: kek.id.clone(),
            algorithm: key.algorithm,
            wrapping_algorithm: kek.algorithm,
            nonce: nonce.to_vec(),
            created_at: SystemTime::now(),
        };
        
        Ok(wrapped_key)
    }
}
```

### Encrypted Data Model

Encrypted data is represented with a comprehensive model:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted data
    pub ciphertext: Vec<u8>,
    
    /// The algorithm used for encryption
    pub algorithm: EncryptionAlgorithm,
    
    /// ID of the key used for encryption
    pub key_id: KeyId,
    
    /// Encrypted key (for envelope encryption)
    pub encrypted_key: Option<WrappedKey>,
    
    /// Encryption metadata (IVs, nonces, etc.)
    pub metadata: EncryptionMetadata,
    
    /// When the data was encrypted
    pub created_at: SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Initialization vector or nonce
    pub iv: Option<Vec<u8>>,
    
    /// Authentication tag
    pub tag: Option<Vec<u8>>,
    
    /// Additional authenticated data
    pub aad: Option<Vec<u8>>,
    
    /// Parameters specific to encryption algorithm
    pub algorithm_parameters: HashMap<String, serde_json::Value>,
}
```

## Encryption Schemes

### Envelope Encryption

Envelope encryption uses a two-tier key hierarchy for improved security:

```rust
pub struct EnvelopeEncryptionProvider {
    /// Data encryption algorithm
    data_algorithm: EncryptionAlgorithm,
    
    /// Key encryption algorithm
    key_algorithm: EncryptionAlgorithm,
    
    /// Key manager
    key_manager: Arc<KeyManagementService>,
}

impl EnvelopeEncryptionProvider {
    /// Encrypt data using envelope encryption
    pub fn envelope_encrypt(
        &self,
        plaintext: &[u8],
        context: &EncryptionContext,
    ) -> Result<EncryptedData, EncryptionError> {
        // 1. Generate a data encryption key (DEK)
        let dek = self.key_manager.generate_data_encryption_key(self.data_algorithm)?;
        
        // 2. Encrypt the data with the DEK
        let crypto_provider = self.get_crypto_provider(self.data_algorithm)?;
        let (ciphertext, metadata) = crypto_provider.encrypt(plaintext, &dek, context)?;
        
        // 3. Get the key encryption key (KEK)
        let kek_id = context.kek_id.clone().unwrap_or_else(|| {
            self.key_manager.get_default_kek_id(self.key_algorithm)
        });
        
        // 4. Encrypt the DEK with the KEK
        let wrapped_key = self.key_manager.wrap_key(&dek, Some(kek_id))?;
        
        // 5. Create the result
        let encrypted_data = EncryptedData {
            ciphertext,
            algorithm: self.data_algorithm,
            key_id: dek.id,
            encrypted_key: Some(wrapped_key),
            metadata,
            created_at: SystemTime::now(),
        };
        
        Ok(encrypted_data)
    }
}
```

### Multi-Layer Encryption

For sensitive data requiring multiple layers of protection:

```rust
pub struct MultiLayerEncryptionProvider {
    /// Ordered list of encryption layers
    layers: Vec<EncryptionLayer>,
}

impl MultiLayerEncryptionProvider {
    /// Encrypt data with multiple layers of encryption
    pub fn multi_layer_encrypt(
        &self,
        plaintext: &[u8],
        context: &EncryptionContext,
    ) -> Result<EncryptedData, EncryptionError> {
        let mut current_data = plaintext.to_vec();
        let mut layer_metadata = Vec::with_capacity(self.layers.len());
        
        // Apply each layer of encryption
        for layer in &self.layers {
            // Get the provider for this layer
            let provider = self.get_crypto_provider(layer.algorithm)?;
            
            // Create a context for this layer
            let layer_context = context.clone()
                .with_key_id(layer.key_id.clone())
                .with_layer(layer.name.clone());
            
            // Encrypt the data
            let (layer_ciphertext, layer_meta) = provider.encrypt(
                &current_data, 
                &layer.key, 
                &layer_context
            )?;
            
            // Update current data for next layer
            current_data = layer_ciphertext;
            
            // Store metadata for this layer
            layer_metadata.push(LayerMetadata {
                layer_name: layer.name.clone(),
                algorithm: layer.algorithm,
                key_id: layer.key.id.clone(),
                metadata: layer_meta,
            });
        }
        
        // Create final encrypted data
        let encrypted_data = EncryptedData {
            ciphertext: current_data,
            algorithm: EncryptionAlgorithm::MultiLayer,
            key_id: KeyId::new("multi-layer"), // Special ID for multi-layer
            encrypted_key: None,
            metadata: EncryptionMetadata {
                iv: None,
                tag: None,
                aad: None,
                algorithm_parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "layers".to_string(), 
                        serde_json::to_value(&layer_metadata).unwrap_or_default()
                    );
                    params
                },
            },
            created_at: SystemTime::now(),
        };
        
        Ok(encrypted_data)
    }
}
```

### Field-Level Encryption

For granular protection of structured data:

```rust
pub struct FieldLevelEncryptionProvider {
    /// Base encryption manager
    encryption_manager: Arc<StorageEncryptionManager>,
    
    /// Schema registry
    schema_registry: Arc<SchemaRegistry>,
}

impl FieldLevelEncryptionProvider {
    /// Encrypt specific fields in a document
    pub fn encrypt_fields<T: Serialize + DeserializeOwned>(
        &self,
        document: &T,
        field_policies: &HashMap<String, FieldEncryptionPolicy>,
        context: &EncryptionContext,
    ) -> Result<EncryptedDocument<T>, EncryptionError> {
        // Convert document to a value tree
        let mut value = serde_json::to_value(document)
            .map_err(|e| EncryptionError::SerializationError(e.to_string()))?;
        
        // Track encrypted fields
        let mut encrypted_fields = HashMap::new();
        
        // Process the value tree and encrypt fields according to policies
        self.encrypt_value_fields(&mut value, field_policies, "", context, &mut encrypted_fields)?;
        
        // Convert back to the original type
        let encrypted_document = serde_json::from_value(value)
            .map_err(|e| EncryptionError::DeserializationError(e.to_string()))?;
        
        Ok(EncryptedDocument {
            document: encrypted_document,
            encrypted_fields,
        })
    }
    
    /// Recursive helper to encrypt fields in a value tree
    fn encrypt_value_fields(
        &self,
        value: &mut serde_json::Value,
        field_policies: &HashMap<String, FieldEncryptionPolicy>,
        path_prefix: &str,
        context: &EncryptionContext,
        encrypted_fields: &mut HashMap<String, EncryptedFieldInfo>,
    ) -> Result<(), EncryptionError> {
        match value {
            serde_json::Value::Object(map) => {
                // Process each field in the object
                let keys: Vec<String> = map.keys().cloned().collect();
                
                for key in keys {
                    let field_path = if path_prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path_prefix, key)
                    };
                    
                    // Check if this field should be encrypted
                    if let Some(policy) = field_policies.get(&field_path) {
                        if let Some(field_value) = map.get(&key) {
                            // Serialize the field value
                            let field_data = serde_json::to_vec(field_value)
                                .map_err(|e| EncryptionError::SerializationError(e.to_string()))?;
                            
                            // Create field-specific context
                            let field_context = context.clone()
                                .with_additional_data(field_path.as_bytes())
                                .with_algorithm(policy.algorithm);
                            
                            // Encrypt the field
                            let encrypted = self.encryption_manager.encrypt(
                                &field_data,
                                &field_context,
                            )?;
                            
                            // Store encryption info
                            encrypted_fields.insert(field_path.clone(), EncryptedFieldInfo {
                                algorithm: encrypted.algorithm,
                                key_id: encrypted.key_id.clone(),
                                created_at: encrypted.created_at,
                            });
                            
                            // Create encrypted field marker
                            let encrypted_marker = serde_json::json!({
                                "__encrypted": true,
                                "alg": format!("{:?}", encrypted.algorithm),
                                "data": base64::encode(&encrypted.ciphertext),
                                "key_id": encrypted.key_id.to_string(),
                            });
                            
                            // Replace field with encrypted marker
                            map.insert(key.clone(), encrypted_marker);
                        }
                    } else if let Some(field_value) = map.get_mut(&key) {
                        // Recurse into nested objects and arrays
                        self.encrypt_value_fields(
                            field_value, 
                            field_policies, 
                            &field_path, 
                            context,
                            encrypted_fields,
                        )?;
                    }
                }
            },
            serde_json::Value::Array(arr) => {
                // Process each element in the array
                for (i, element) in arr.iter_mut().enumerate() {
                    let element_path = format!("{}[{}]", path_prefix, i);
                    self.encrypt_value_fields(
                        element, 
                        field_policies, 
                        &element_path, 
                        context,
                        encrypted_fields,
                    )?;
                }
            },
            _ => {
                // Primitive value, nothing to do at this level
            }
        }
        
        Ok(())
    }
}
```

## Content-Addressed Encryption

Integration with content-addressed storage requires special considerations:

```rust
pub struct ContentAddressedEncryptionProvider {
    /// Base encryption manager
    encryption_manager: Arc<StorageEncryptionManager>,
    
    /// Content-addressed storage
    storage: Arc<dyn ContentAddressedStorage>,
}

impl ContentAddressedEncryptionProvider {
    /// Encrypt a content-addressed object
    pub fn encrypt_content_addressed<T: ContentAddressed>(
        &self,
        object: &T,
        context: &EncryptionContext,
    ) -> Result<EncryptedContentRef<T>, EncryptionError> {
        // Get the content hash of the object
        let content_id = object.content_id();
        
        // Serialize the object
        let object_data = object.to_bytes();
        
        // Encrypt the serialized data
        let encrypted = self.encryption_manager.encrypt(&object_data, context)?;
        
        // Store encryption metadata keyed by content ID
        let metadata_key = format!("encryption_meta:{}", content_id);
        let metadata = EncryptionReferenceMetadata {
            content_id: content_id.clone(),
            algorithm: encrypted.algorithm,
            key_id: encrypted.key_id.clone(),
            created_at: encrypted.created_at,
        };
        
        // Store the metadata in a separate location
        self.store_encryption_metadata(&metadata_key, &metadata)?;
        
        // Store the encrypted data
        let encrypted_content_id = self.storage.store_bytes(&encrypted.ciphertext)?;
        
        // Create the encrypted content reference
        let encrypted_ref = EncryptedContentRef {
            original_content_id: content_id,
            encrypted_content_id,
            metadata,
            _phantom: std::marker::PhantomData,
        };
        
        Ok(encrypted_ref)
    }
    
    /// Decrypt a content-addressed object
    pub fn decrypt_content_addressed<T: ContentAddressed>(
        &self,
        encrypted_ref: &EncryptedContentRef<T>,
        context: &DecryptionContext,
    ) -> Result<T, EncryptionError> {
        // Retrieve the encrypted data
        let encrypted_data = self.storage.get_bytes(&encrypted_ref.encrypted_content_id)?;
        
        // Create EncryptedData structure from components
        let encrypted = EncryptedData {
            ciphertext: encrypted_data,
            algorithm: encrypted_ref.metadata.algorithm,
            key_id: encrypted_ref.metadata.key_id.clone(),
            encrypted_key: None, // No envelope encryption for this example
            metadata: EncryptionMetadata::default(),
            created_at: encrypted_ref.metadata.created_at,
        };
        
        // Decrypt the data
        let plaintext = self.encryption_manager.decrypt(&encrypted, context)?;
        
        // Deserialize the object
        let object = T::from_bytes(&plaintext)
            .map_err(|e| EncryptionError::from(e))?;
        
        // Verify the content ID matches
        let content_id = object.content_id();
        if content_id != encrypted_ref.original_content_id {
            return Err(EncryptionError::ContentIdMismatch);
        }
        
        Ok(object)
    }
}
```

## Content-Preserving Encryption

Special encryption modes that preserve content hashes:

```rust
pub struct ContentPreservingEncryption {
    /// Key material for content-preserving encryption
    key: Vec<u8>,
    
    /// Domain separation tag
    domain_tag: Vec<u8>,
}

impl ContentPreservingEncryption {
    /// Create a new content-preserving encryption instance
    pub fn new(key: &[u8], domain_tag: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
            domain_tag: domain_tag.to_vec(),
        }
    }
    
    /// Encrypt data while preserving its content hash
    pub fn encrypt<T: ContentAddressed>(
        &self,
        object: &T,
    ) -> Result<PreservedContentEncryption<T>, EncryptionError> {
        // Get the content hash
        let content_id = object.content_id();
        
        // Serialize the object
        let plaintext = object.to_bytes();
        
        // Generate a deterministic IV based on content ID
        let iv = self.derive_deterministic_iv(&content_id)?;
        
        // Encrypt the data using chacha20
        let key = chacha20poly1305::Key::from_slice(&self.key);
        let cipher = chacha20poly1305::ChaCha20Poly1305::new(key);
        let nonce = chacha20poly1305::Nonce::from_slice(&iv);
        
        // Create AAD from content ID and domain tag
        let mut aad = self.domain_tag.clone();
        aad.extend_from_slice(content_id.as_bytes());
        
        // Encrypt
        let ciphertext = cipher.encrypt(nonce, chacha20poly1305::aead::Payload {
            msg: &plaintext,
            aad: &aad,
        }).map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        // Create the result
        let result = PreservedContentEncryption {
            content_id,
            ciphertext,
            iv,
            _phantom: std::marker::PhantomData,
        };
        
        Ok(result)
    }
    
    /// Decrypt data with content hash preservation
    pub fn decrypt<T: ContentAddressed>(
        &self,
        encrypted: &PreservedContentEncryption<T>,
    ) -> Result<T, EncryptionError> {
        // Get the key and IV
        let key = chacha20poly1305::Key::from_slice(&self.key);
        let cipher = chacha20poly1305::ChaCha20Poly1305::new(key);
        let nonce = chacha20poly1305::Nonce::from_slice(&encrypted.iv);
        
        // Create AAD from content ID and domain tag
        let mut aad = self.domain_tag.clone();
        aad.extend_from_slice(encrypted.content_id.as_bytes());
        
        // Decrypt
        let plaintext = cipher.decrypt(nonce, chacha20poly1305::aead::Payload {
            msg: &encrypted.ciphertext,
            aad: &aad,
        }).map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        // Deserialize the object
        let object = T::from_bytes(&plaintext)
            .map_err(|e| EncryptionError::from(e))?;
        
        // Verify the content ID matches
        let content_id = object.content_id();
        if content_id != encrypted.content_id {
            return Err(EncryptionError::ContentIdMismatch);
        }
        
        Ok(object)
    }
    
    /// Derive a deterministic IV from a content ID
    fn derive_deterministic_iv(&self, content_id: &ContentId) -> Result<Vec<u8>, EncryptionError> {
        // Use HKDF to derive a deterministic IV
        let salt = &self.domain_tag;
        let info = b"content-preserving-encryption-iv";
        
        let mut okm = [0u8; 12]; // 96-bit nonce for ChaCha20-Poly1305
        let hk = hkdf::Hkdf::<sha2::Sha256>::new(Some(salt), &self.key);
        hk.expand(info, &mut okm)
            .map_err(|e| EncryptionError::KeyDerivationError(e.to_string()))?;
            
        // XOR with first 12 bytes of content ID for uniqueness
        for (i, byte) in content_id.as_bytes().iter().take(12).enumerate() {
            okm[i] ^= *byte;
        }
        
        Ok(okm.to_vec())
    }
}
```

## Key Rotation and Versioning

Key rotation is essential for maintaining encryption security:

```rust
pub struct KeyRotationService {
    /// Key store
    key_store: Arc<dyn KeyStore>,
    
    /// Scheduler for key rotation
    scheduler: Arc<dyn SchedulerService>,
    
    /// Pending rotations
    pending_rotations: RwLock<HashMap<KeyId, SystemTime>>,
    
    /// Rotation policies by algorithm
    rotation_policies: HashMap<EncryptionAlgorithm, RotationPolicy>,
}

impl KeyRotationService {
    /// Schedule a key for rotation
    pub fn schedule_rotation(
        &self,
        key_id: KeyId,
        rotation_delay: Duration,
    ) {
        let rotation_time = SystemTime::now() + rotation_delay;
        
        // Store in pending rotations
        {
            let mut pending = self.pending_rotations.write().unwrap();
            pending.insert(key_id.clone(), rotation_time);
        }
        
        // Schedule the rotation task
        let service = self.clone();
        self.scheduler.schedule_task(
            rotation_time,
            move || {
                let _ = service.rotate_key(&key_id);
            },
        );
    }
    
    /// Rotate a key
    pub fn rotate_key(&self, key_id: &KeyId) -> Result<KeyRotationResult, KeyManagementError> {
        // Get the current key
        let current_key = self.key_store.get_key(key_id)?;
        
        // Generate a new key with the same algorithm
        let new_key = self.generate_replacement_key(&current_key)?;
        
        // Get affected data references
        let affected_data = self.find_affected_data(key_id)?;
        
        // Start a transaction
        let transaction = self.key_store.begin_transaction()?;
        
        // Store the new key
        self.key_store.store_key_in_transaction(&new_key, &transaction)?;
        
        // Update key status
        self.key_store.update_key_status_in_transaction(
            key_id,
            KeyStatus::PendingRotation,
            &transaction,
        )?;
        
        // Create rotation record
        let rotation = KeyRotationRecord {
            old_key_id: key_id.clone(),
            new_key_id: new_key.id.clone(),
            rotation_time: SystemTime::now(),
            affected_data_count: affected_data.len(),
        };
        
        self.key_store.store_rotation_record_in_transaction(&rotation, &transaction)?;
        
        // Commit the transaction
        self.key_store.commit_transaction(transaction)?;
        
        // Schedule reencryption of affected data
        self.schedule_reencryption(affected_data, &current_key, &new_key)?;
        
        // Remove from pending rotations
        {
            let mut pending = self.pending_rotations.write().unwrap();
            pending.remove(key_id);
        }
        
        Ok(KeyRotationResult {
            old_key_id: key_id.clone(),
            new_key_id: new_key.id,
            affected_data_count: affected_data.len(),
        })
    }
    
    /// Find all data encrypted with a specific key
    fn find_affected_data(&self, key_id: &KeyId) -> Result<Vec<DataReference>, KeyManagementError> {
        // Query the key usage index
        self.key_store.find_data_references_by_key(key_id)
    }
    
    /// Schedule reencryption of data
    fn schedule_reencryption(
        &self,
        data_refs: Vec<DataReference>,
        old_key: &EncryptionKey,
        new_key: &EncryptionKey,
    ) -> Result<(), KeyManagementError> {
        // Create a reencryption task
        let task = ReencryptionTask {
            data_references: data_refs,
            old_key_id: old_key.id.clone(),
            new_key_id: new_key.id.clone(),
            created_at: SystemTime::now(),
        };
        
        // Submit the task to the reencryption service
        // In a real implementation, this would be handled by a separate service
        // that processes reencryption tasks in the background
        self.submit_reencryption_task(task)
    }
}
```

## Cross-Domain Encryption

Secure data sharing across domain boundaries:

```rust
pub struct CrossDomainEncryptionManager {
    /// Local encryption manager
    local_manager: Arc<StorageEncryptionManager>,
    
    /// Domain adapters for different target domains
    domain_adapters: HashMap<DomainId, Arc<dyn DomainEncryptionAdapter>>,
    
    /// Policy engine for cross-domain sharing
    policy_engine: Arc<CrossDomainPolicyEngine>,
}

impl CrossDomainEncryptionManager {
    /// Encrypt data for cross-domain sharing
    pub fn encrypt_for_domain(
        &self,
        plaintext: &[u8],
        source_domain: &DomainId,
        target_domain: &DomainId,
        context: &EncryptionContext,
    ) -> Result<CrossDomainEncryptedData, EncryptionError> {
        // Validate policy for cross-domain sharing
        self.policy_engine.validate_cross_domain_encryption(
            source_domain,
            target_domain,
            context,
        )?;
        
        // Get the target domain adapter
        let adapter = self.domain_adapters.get(target_domain)
            .ok_or_else(|| EncryptionError::UnsupportedDomain(target_domain.clone()))?;
        
        // Encrypt for the target domain
        adapter.encrypt_for_domain(plaintext, source_domain, context)
    }
    
    /// Receive encrypted data from another domain
    pub fn receive_from_domain(
        &self,
        encrypted: &CrossDomainEncryptedData,
        source_domain: &DomainId,
        context: &DecryptionContext,
    ) -> Result<Vec<u8>, EncryptionError> {
        // Validate policy for cross-domain decryption
        self.policy_engine.validate_cross_domain_decryption(
            source_domain,
            &encrypted.target_domain,
            context,
        )?;
        
        // Decrypt using local encryption manager
        self.local_manager.decrypt(&encrypted.encrypted_data, context)
    }
}

#[async_trait]
pub trait DomainEncryptionAdapter: Send + Sync {
    /// Get the domain this adapter handles
    fn domain_id(&self) -> &DomainId;
    
    /// Encrypt data for the target domain
    fn encrypt_for_domain(
        &self,
        plaintext: &[u8],
        source_domain: &DomainId,
        context: &EncryptionContext,
    ) -> Result<CrossDomainEncryptedData, EncryptionError>;
    
    /// Get public key for the target domain
    async fn get_domain_public_key(&self) -> Result<PublicKey, EncryptionError>;
    
    /// Verify a cross-domain encryption
    fn verify_cross_domain_encryption(
        &self,
        encrypted: &CrossDomainEncryptedData,
    ) -> Result<bool, EncryptionError>;
}
```

## Integration with Storage System

The encryption system integrates with the storage system:

```rust
pub struct EncryptionAwareStorage {
    /// Base storage provider
    storage: Arc<dyn StorageProvider>,
    
    /// Encryption manager
    encryption_manager: Arc<StorageEncryptionManager>,
    
    /// Encryption policy provider
    policy_provider: Arc<EncryptionPolicyProvider>,
}

impl StorageProvider for EncryptionAwareStorage {
    fn storage_type(&self) -> StorageType {
        self.storage.storage_type()
    }
    
    fn store(&self, key: &StorageKey, value: &[u8], options: &StorageOptions) 
        -> Result<StorageMetadata, StorageError> 
    {
        // Check if encryption is required for this key
        let encryption_policy = self.policy_provider.get_policy_for_key(key)?;
        
        let stored_value = if encryption_policy.encrypt_data {
            // Create encryption context
            let context = EncryptionContext::new()
                .with_algorithm(encryption_policy.algorithm)
                .with_additional_data(key.formatted_key().as_bytes());
            
            // Encrypt the data
            let encrypted = self.encryption_manager.encrypt(value, &context)
                .map_err(|e| StorageError::EncryptionError(e.to_string()))?;
            
            // Serialize the encrypted data
            serde_json::to_vec(&encrypted)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?
        } else {
            // No encryption needed
            value.to_vec()
        };
        
        // Store the data
        let mut metadata = self.storage.store(key, &stored_value, options)?;
        
        // Add encryption metadata
        if encryption_policy.encrypt_data {
            metadata.encrypted = true;
            metadata.encryption_algorithm = Some(encryption_policy.algorithm.to_string());
        }
        
        Ok(metadata)
    }
    
    fn retrieve(&self, key: &StorageKey, options: &RetrieveOptions) 
        -> Result<StorageItem, StorageError> 
    {
        // Retrieve the raw data
        let item = self.storage.retrieve(key, options)?;
        
        // Check if the data is encrypted
        if item.metadata.encrypted {
            // Parse the encrypted data
            let encrypted: EncryptedData = serde_json::from_slice(&item.value)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
            
            // Create decryption context
            let context = DecryptionContext::new()
                .with_additional_data(key.formatted_key().as_bytes());
            
            // Decrypt the data
            let decrypted = self.encryption_manager.decrypt(&encrypted, &context)
                .map_err(|e| StorageError::DecryptionError(e.to_string()))?;
            
            // Return decrypted item
            Ok(StorageItem {
                key: item.key,
                value: decrypted,
                metadata: item.metadata,
                from_cache: item.from_cache,
            })
        } else {
            // Data is not encrypted
            Ok(item)
        }
    }
    
    // Other methods delegate to base storage
    // ...
}
```

## Zero-Knowledge Storage Encryption

Integration with ZK proofs for privacy-preserving storage:

```rust
pub struct ZkEncryptionProvider {
    /// Base encryption manager
    encryption_manager: Arc<StorageEncryptionManager>,
    
    /// ZK prover
    zk_prover: Arc<dyn ZkProver>,
}

impl ZkEncryptionProvider {
    /// Encrypt data with ZK proofs
    pub fn encrypt_with_zk_proof<T: ContentAddressed>(
        &self,
        object: &T,
        circuit: &ZkCircuit,
        public_inputs: &[u8],
        context: &EncryptionContext,
    ) -> Result<ZkEncryptedData<T>, EncryptionError> {
        // Get content hash
        let content_id = object.content_id();
        
        // Serialize the object
        let object_data = object.to_bytes();
        
        // Encrypt the data
        let encrypted = self.encryption_manager.encrypt(&object_data, context)?;
        
        // Generate private inputs for ZK proof (includes encryption key)
        let private_inputs = self.generate_private_inputs(&encrypted, &object_data)?;
        
        // Generate ZK proof
        let proof = self.zk_prover.generate_proof(
            circuit,
            public_inputs,
            &private_inputs,
        )?;
        
        // Create the result
        let zk_encrypted = ZkEncryptedData {
            content_id,
            encrypted_data: encrypted,
            zk_proof: proof,
            public_inputs: public_inputs.to_vec(),
            circuit_id: circuit.id(),
            _phantom: std::marker::PhantomData,
        };
        
        Ok(zk_encrypted)
    }
    
    /// Verify and decrypt data with ZK proof
    pub fn verify_and_decrypt<T: ContentAddressed>(
        &self,
        encrypted: &ZkEncryptedData<T>,
        context: &DecryptionContext,
    ) -> Result<T, EncryptionError> {
        // Verify the ZK proof first
        let verified = self.zk_prover.verify_proof(
            &encrypted.circuit_id,
            &encrypted.zk_proof,
            &encrypted.public_inputs,
        )?;
        
        if !verified {
            return Err(EncryptionError::ZkProofVerificationFailed);
        }
        
        // Decrypt the data
        let plaintext = self.encryption_manager.decrypt(
            &encrypted.encrypted_data,
            context,
        )?;
        
        // Deserialize the object
        let object = T::from_bytes(&plaintext)
            .map_err(|e| EncryptionError::from(e))?;
        
        // Verify the content ID matches
        let content_id = object.content_id();
        if content_id != encrypted.content_id {
            return Err(EncryptionError::ContentIdMismatch);
        }
        
        Ok(object)
    }
}
```

## Usage Examples

### Basic Encryption

```rust
// Get the storage encryption manager
let encryption = system.storage_encryption_manager();

// Data to encrypt
let sensitive_data = "This is sensitive information".as_bytes();

// Create an encryption context
let context = EncryptionContext::new()
    .with_algorithm(EncryptionAlgorithm::AesGcm256)
    .with_additional_data("resource metadata".as_bytes());

// Encrypt the data
let encrypted = encryption.encrypt(sensitive_data, &context)?;

println!("Data encrypted with key: {}", encrypted.key_id);

// Store the encrypted data
store.put("sensitive_resource", &encrypted)?;

// Later, decrypt the data
let retrieved = store.get("sensitive_resource")?;
let decryption_context = DecryptionContext::new()
    .with_additional_data("resource metadata".as_bytes());

let decrypted = encryption.decrypt(&retrieved, &decryption_context)?;
assert_eq!(decrypted, sensitive_data);
```

### Field-Level Encryption

```rust
// Define a document type
#[derive(Serialize, Deserialize)]
struct UserProfile {
    user_id: String,
    name: String,
    email: String,
    phone: String,
    address: Address,
}

#[derive(Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
    postal_code: String,
    country: String,
}

// Create field encryption policies
let mut field_policies = HashMap::new();
field_policies.insert("email".to_string(), FieldEncryptionPolicy {
    algorithm: EncryptionAlgorithm::AesGcm256,
    required: true,
});
field_policies.insert("phone".to_string(), FieldEncryptionPolicy {
    algorithm: EncryptionAlgorithm::AesGcm256,
    required: true,
});
field_policies.insert("address.street".to_string(), FieldEncryptionPolicy {
    algorithm: EncryptionAlgorithm::AesGcm256,
    required: true,
});
field_policies.insert("address.postal_code".to_string(), FieldEncryptionPolicy {
    algorithm: EncryptionAlgorithm::AesGcm256,
    required: true,
});

// Get field-level encryption provider
let field_encryption = system.field_level_encryption_provider();

// Encrypt specific fields in the document
let encrypted_profile = field_encryption.encrypt_fields(
    &user_profile,
    &field_policies,
    &EncryptionContext::new(),
)?;

// Store the partially encrypted document
document_store.store_document(
    "user_profiles",
    user_profile.user_id.clone(),
    &encrypted_profile.document,
    &DocumentOptions::new(),
)?;

// Later, decrypt the fields
let stored_profile = document_store.retrieve_document(
    "user_profiles",
    user_profile.user_id.clone(),
    &RetrieveOptions::new(),
)?;

let decrypted_profile = field_encryption.decrypt_fields(
    &stored_profile,
    &field_policies,
    &DecryptionContext::new(),
)?;
```

### Content-Preserving Encryption

```rust
// Get content-preserving encryption provider
let content_preserving = system.content_preserving_encryption_provider();

// Create a resource
let resource = Resource::new("resource-1", "Sample resource");

// Get the original content ID
let original_id = resource.content_id();

// Encrypt the resource while preserving its content hash
let encrypted = content_preserving.encrypt(&resource)?;

// Verify that the content ID is preserved
assert_eq!(encrypted.content_id, original_id);

// Store the encrypted resource
storage.store_bytes(&encrypted.ciphertext)?;

// Later, decrypt the resource
let decrypted = content_preserving.decrypt(&encrypted)?;

// Verify that the decrypted resource has the same content ID
assert_eq!(decrypted.content_id(), original_id);
```

### Cross-Domain Encryption

```rust
// Get cross-domain encryption manager
let cross_domain = system.cross_domain_encryption_manager();

// Data to share across domains
let shared_data = "Cross-domain shared data".as_bytes();

// Source and target domains
let source_domain = DomainId::new("domain-a");
let target_domain = DomainId::new("domain-b");

// Create encryption context
let context = EncryptionContext::new()
    .with_purpose("cross-domain-sharing")
    .with_sharing_policy("confidential");

// Encrypt for the target domain
let encrypted = cross_domain.encrypt_for_domain(
    shared_data,
    &source_domain,
    &target_domain,
    &context,
)?;

// Send the encrypted data to Domain B...

// On Domain B, receive and decrypt
let decrypted = cross_domain.receive_from_domain(
    &encrypted,
    &source_domain,
    &DecryptionContext::new(),
)?;

assert_eq!(decrypted, shared_data);
```

## Implementation Status

The current implementation status of Storage Encryption:

- ✅ Core encryption interfaces
- ✅ Basic AES-GCM encryption
- ⚠️ Key management (partially implemented)
- ⚠️ Field-level encryption (partially implemented)
- ❌ Content-preserving encryption (not yet implemented)
- ❌ Cross-domain encryption (not yet implemented)
- ❌ ZK encryption integration (not yet implemented)

## Future Enhancements

Planned future enhancements for Storage Encryption:

1. **Threshold Encryption**: Support for decryption requiring multiple parties
2. **Post-Quantum Encryption**: Integration of post-quantum secure algorithms
3. **Hardware Security Module (HSM) Integration**: Support for hardware-backed key storage
4. **Homomorphic Encryption**: Limited support for computing on encrypted data
5. **Key Escrow Mechanisms**: Secure key recovery for organizational needs
6. **Forward Secrecy**: Enhanced key rotation with perfect forward secrecy
7. **Searchable Encryption**: Ability to search encrypted data without decryption
8. **Format-Preserving Encryption**: Encryption that preserves data format
9. **Quantum-Resistant Cryptography**: Migration to quantum-resistant algorithms