<!-- Tracking resource relationships -->
<!-- Original file: docs/src/resource_relationship_tracking.md -->

# Content-Addressed Resource Relationship Tracking

This document details the tracking and management of content-addressed relationships between ResourceRegisters in the Causality system, demonstrating how the system efficiently maintains, queries, and navigates these relationships.

## Overview

Relationship tracking is the system functionality that allows efficient monitoring, querying, and indexing of relationships between ResourceRegisters. With the unified content-addressed model, relationship tracking is built on immutable, verifiable data structures that enable reliable cross-domain queries and performance optimizations.

## Content-Addressed Relationship Tracker

The core component responsible for relationship tracking is the RelationshipTracker:

```rust
/// Tracks and manages content-addressed relationships between resources
pub struct RelationshipTracker<C: ExecutionContext> {
    /// Content-addressed storage
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// In-memory relationship index for fast queries
    relationship_index: RelationshipIndex<C>,
    
    /// Relationship change notification system
    notification_system: Arc<dyn RelationshipNotificationSystem>,
    
    /// Verification service for relationship validation
    verification_service: Arc<VerificationService>,
    
    /// Capability manager for access control
    capability_manager: Arc<CapabilityManager>,
    
    /// Execution context type
    context_type: PhantomData<C>,
}

/// Fast in-memory index for relationship queries
struct RelationshipIndex<C: ExecutionContext> {
    /// Maps resource ID to its outgoing relationships
    outgoing_relationships: HashMap<RegisterId, Vec<ContentHash>>,
    
    /// Maps resource ID to its incoming relationships
    incoming_relationships: HashMap<RegisterId, Vec<ContentHash>>,
    
    /// Maps relationship type to relationships of that type
    relationships_by_type: HashMap<RelationshipType, Vec<ContentHash>>,
    
    /// Maps domain ID to cross-domain relationships
    cross_domain_relationships: HashMap<DomainId, Vec<ContentHash>>,
    
    /// Context type
    context_type: PhantomData<C>,
}
```

## Three-Layer Effect Architecture for Relationship Tracking

Relationship tracking is implemented using the three-layer effect architecture:

### 1. Algebraic Effect Layer

```rust
/// Relationship tracker effect for tracking operations
pub enum RelationshipTrackerEffect<C: ExecutionContext, R> {
    /// Track a new relationship
    TrackRelationship {
        relationship: ContentRef<ResourceRelationship<C>>,
        continuation: Box<dyn Continuation<TrackResult, R>>,
    },
    
    /// Find relationships for a resource
    FindRelationships {
        resource_id: RegisterId,
        query: RelationshipQuery,
        continuation: Box<dyn Continuation<FindRelationshipsResult, R>>,
    },
    
    /// Subscribe to relationship changes
    SubscribeToChanges {
        resource_id: RegisterId,
        notification_handler: Box<dyn NotificationHandler>,
        continuation: Box<dyn Continuation<SubscriptionId, R>>,
    },
    
    /// Verify relationships between resources
    VerifyRelationshipPath {
        source_id: RegisterId,
        target_id: RegisterId,
        path_constraints: PathConstraints,
        continuation: Box<dyn Continuation<VerifyPathResult, R>>,
    },
}
```

### 2. Effect Constraints Layer

```rust
/// Type constraints for relationship tracker effects
pub trait RelationshipTrackerEffectHandler<C: ExecutionContext>: Send + Sync {
    /// Process a relationship tracker effect
    fn handle_tracker_effect<R>(
        &self,
        effect: RelationshipTrackerEffect<C, R>,
        context: &C,
    ) -> Result<R, TrackerError>;
    
    /// Validate a relationship tracker effect
    fn validate_tracker_effect<R>(
        &self,
        effect: &RelationshipTrackerEffect<C, R>,
        context: &C,
    ) -> Result<ValidationResult, ValidationError>;
}
```

### 3. Domain Implementation Layer (TEL)

```rust
effect EVMRelationshipTracker implements RelationshipTrackerEffect {
    // State fields
    domain_id: DomainId
    contract_address: Address
    
    // Implementation methods for tracking relationships in EVM storage
    fn track_evm_relationship(relationship, context) -> Result<TrackResult, TrackerError> {
        // Get relationship data
        let source_id = relationship.source.resolve(context.storage).id;
        let target_id = relationship.target.resolve(context.storage).id;
        let relationship_type = relationship.relationship_type;
        
        // Create EVM storage key for the relationship
        let storage_key = keccak256(
            abi.encode(
                "relationship",
                source_id.to_string(),
                target_id.to_string(),
                relationship_type.to_string()
            )
        );
        
        // Store relationship hash in EVM storage
        context.evm_client.set_storage(
            this.contract_address,
            storage_key,
            relationship.content_hash.as_bytes()
        );
        
        // Index the relationship for event emission
        context.evm_client.emit_event(
            "RelationshipCreated",
            [
                source_id.to_string(),
                target_id.to_string(),
                relationship_type.to_string(),
                relationship.content_hash.to_string()
            ]
        );
        
        return Ok(TrackResult {
            tracked: true,
            storage_key: storage_key.to_string(),
        });
    }
    
    fn find_evm_relationships(resource_id, query, context) -> Result<FindRelationshipsResult, TrackerError> {
        // Create filter for EVM logs
        let filter = {
            fromBlock: "0x0",
            toBlock: "latest",
            address: this.contract_address,
            topics: [
                keccak256("RelationshipCreated(string,string,string,string)"),
                keccak256(resource_id.to_string()),
                query.relationship_type.map(|t| keccak256(t.to_string())),
                null
            ]
        };
        
        // Query EVM logs
        let logs = context.evm_client.get_logs(filter);
        
        // Extract relationship content hashes from logs
        let content_hashes = logs.map(|log| {
            let values = abi.decode(
                ["string", "string", "string", "string"],
                log.data
            );
            return ContentHash::from_string(values[3]);
        });
        
        // Resolve relationships from content hashes
        let relationships = content_hashes.map(|hash| {
            context.storage.get::<ResourceRelationship<C>>(hash)
        }).filter_map(|result| result.ok());
        
        return Ok(FindRelationshipsResult {
            relationships: relationships,
            total_found: relationships.len(),
        });
    }
}
```

## Tracking and Indexing Relationships

The RelationshipTracker provides methods for tracking, indexing, and querying relationships:

```rust
impl<C: ExecutionContext> RelationshipTracker<C> {
    /// Tracks a new relationship
    pub async fn track_relationship(
        &self,
        relationship: ContentRef<ResourceRelationship<C>>,
        context: &C,
    ) -> Result<(), TrackerError> {
        // Resolve the relationship
        let relationship_obj = relationship.resolve(&self.storage)?;
        
        // Verify access capability
        self.capability_manager.verify_capability(
            &context.initiator,
            &Capability::TrackRelationship,
            &relationship_obj.source.resolve(&self.storage)?.id,
        )?;
        
        // Verify the relationship is valid
        self.verification_service.verify(&relationship_obj)?;
        
        // Execute the track effect
        let track_result = execute_effect(RelationshipTrackerEffect::TrackRelationship {
            relationship: relationship.clone(),
            continuation: Box::new(|result| Ok(result)),
        }, context).await?;
        
        // Update in-memory index
        self.relationship_index.add_relationship(&relationship_obj);
        
        // Send notifications
        self.notification_system.notify_relationship_created(
            relationship_obj.source.resolve(&self.storage)?.id,
            relationship_obj.target.resolve(&self.storage)?.id,
            relationship_obj.relationship_type.clone(),
            relationship.clone(),
        ).await?;
        
        Ok(())
    }
    
    /// Finds relationships for a resource
    pub async fn find_relationships(
        &self,
        resource_id: &RegisterId,
        query: &RelationshipQuery,
        context: &C,
    ) -> Result<Vec<ResourceRelationship<C>>, TrackerError> {
        // Verify access capability
        self.capability_manager.verify_capability(
            &context.initiator,
            &Capability::ReadRelationships,
            resource_id,
        )?;
        
        // Check if we can serve from the in-memory index
        if query.can_be_served_from_index() && !query.requires_full_content_resolution() {
            // Use in-memory index for fast lookup
            let content_hashes = match query.direction {
                RelationshipDirection::Directed => {
                    if query.outgoing {
                        self.relationship_index.outgoing_relationships.get(resource_id)
                    } else {
                        self.relationship_index.incoming_relationships.get(resource_id)
                    }
                },
                RelationshipDirection::Bidirectional => {
                    // Combine outgoing and incoming for bidirectional queries
                    let mut combined = Vec::new();
                    if let Some(outgoing) = self.relationship_index.outgoing_relationships.get(resource_id) {
                        combined.extend(outgoing);
                    }
                    if let Some(incoming) = self.relationship_index.incoming_relationships.get(resource_id) {
                        combined.extend(incoming);
                    }
                    Some(&combined)
                }
            }.unwrap_or(&Vec::new());
            
            // Filter by relationship type if specified
            let filtered_hashes = if let Some(rel_type) = &query.relationship_type {
                content_hashes.iter()
                    .filter(|&&hash| {
                        if let Ok(rel) = self.storage.get::<ResourceRelationship<C>>(&hash) {
                            rel.relationship_type == *rel_type
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                content_hashes.clone()
            };
            
            // Resolve relationships
            let mut relationships = Vec::new();
            for hash in filtered_hashes {
                if let Ok(rel) = self.storage.get::<ResourceRelationship<C>>(&hash) {
                    relationships.push(rel);
                }
            }
            
            return Ok(relationships);
        }
        
        // For complex queries, use the effect system
        let find_result = execute_effect(RelationshipTrackerEffect::FindRelationships {
            resource_id: resource_id.clone(),
            query: query.clone(),
            continuation: Box::new(|result| Ok(result)),
        }, context).await?;
        
        Ok(find_result.relationships)
    }
    
    /// Subscribes to relationship changes
    pub async fn subscribe_to_changes(
        &self,
        resource_id: &RegisterId,
        handler: Box<dyn NotificationHandler>,
        context: &C,
    ) -> Result<SubscriptionId, TrackerError> {
        // Verify access capability
        self.capability_manager.verify_capability(
            &context.initiator,
            &Capability::SubscribeToRelationships,
            resource_id,
        )?;
        
        // Execute the subscribe effect
        let subscription_id = execute_effect(RelationshipTrackerEffect::SubscribeToChanges {
            resource_id: resource_id.clone(),
            notification_handler: handler,
            continuation: Box::new(|id| Ok(id)),
        }, context).await?;
        
        Ok(subscription_id)
    }
}
```

## Optimized Relationship Queries

The system provides optimized relationship query capabilities:

```rust
/// Finds resources related to a source resource through specific relationship types
pub async fn find_related_resources<C: ExecutionContext>(
    tracker: &RelationshipTracker<C>,
    source_id: &RegisterId,
    relationship_types: Vec<RelationshipType>,
    max_depth: u32,
    context: &C,
) -> Result<Vec<ResourceRegister<C>>, TrackerError> {
    // Query for first-level relationships
    let initial_query = RelationshipQuery {
        relationship_type: None,
        direction: RelationshipDirection::Directed,
        outgoing: true,
        limit: None,
    };
    
    // Find direct relationships
    let direct_relationships = tracker.find_relationships(
        source_id,
        &initial_query,
        context,
    ).await?;
    
    // Filter by relationship types
    let filtered_relationships = direct_relationships.into_iter()
        .filter(|rel| relationship_types.contains(&rel.relationship_type))
        .collect::<Vec<_>>();
    
    // Extract target resources
    let mut related_resources = Vec::new();
    for relationship in &filtered_relationships {
        if let Ok(target) = relationship.target.resolve(&tracker.storage) {
            related_resources.push(target);
        }
    }
    
    // If max_depth > 1, perform recursive search
    if max_depth > 1 {
        for relationship in filtered_relationships {
            if let Ok(target) = relationship.target.resolve(&tracker.storage) {
                // Recursively find related resources
                let nested_related = find_related_resources(
                    tracker,
                    &target.id,
                    relationship_types.clone(),
                    max_depth - 1,
                    context,
                ).await?;
                
                // Add to result set (avoiding duplicates)
                for resource in nested_related {
                    if !related_resources.iter().any(|r| r.id == resource.id) {
                        related_resources.push(resource);
                    }
                }
            }
        }
    }
    
    Ok(related_resources)
}
```

## Content-Addressed Relationship Paths

The system supports verifying and navigating relationship paths:

```rust
/// A path of relationships connecting resources
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipPath<C: ExecutionContext> {
    /// Content hash that uniquely identifies this path
    pub content_hash: ContentHash,
    
    /// The starting resource
    pub source: ContentRef<ResourceRegister<C>>,
    
    /// The ending resource
    pub target: ContentRef<ResourceRegister<C>>,
    
    /// The sequence of relationships forming the path
    pub relationships: Vec<ContentRef<ResourceRelationship<C>>>,
    
    /// Path properties
    pub properties: HashMap<String, Value>,
    
    /// Verification information
    pub verification: VerificationInfo,
    
    /// Execution context type
    pub context: PhantomData<C>,
}

impl<C: ExecutionContext> ContentAddressed for RelationshipPath<C> {
    fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }
    
    fn verify(&self) -> bool {
        // Calculate hash from contents and verify it matches the stored hash
        calculate_content_hash(self) == self.content_hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        // Serialize to canonical binary format
        serialize_canonical(self).expect("Failed to serialize RelationshipPath")
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        // Deserialize from binary format
        deserialize_canonical(bytes)
    }
}
```

### Discovering and Verifying Relationship Paths

```rust
/// Finds and verifies a path between two resources
pub async fn find_relationship_path<C: ExecutionContext>(
    tracker: &RelationshipTracker<C>,
    source_id: &RegisterId,
    target_id: &RegisterId,
    constraints: &PathConstraints,
    context: &C,
) -> Result<Option<RelationshipPath<C>>, TrackerError> {
    // Verify access capabilities
    tracker.capability_manager.verify_capability(
        &context.initiator,
        &Capability::FindRelationshipPaths,
        source_id,
    )?;
    
    // Execute the verify path effect
    let verify_result = execute_effect(RelationshipTrackerEffect::VerifyRelationshipPath {
        source_id: source_id.clone(),
        target_id: target_id.clone(),
        path_constraints: constraints.clone(),
        continuation: Box::new(|result| Ok(result)),
    }, context).await?;
    
    if !verify_result.path_exists {
        return Ok(None);
    }
    
    // Get the source and target resources
    let source_resource = context.storage.get_resource(source_id)?;
    let target_resource = context.storage.get_resource(target_id)?;
    
    // Construct the relationship path
    let mut path = RelationshipPath {
        content_hash: ContentHash::default(), // Will be calculated later
        source: ContentRef::new(&source_resource),
        target: ContentRef::new(&target_resource),
        relationships: verify_result.relationship_path,
        properties: verify_result.path_properties,
        verification: VerificationInfo {
            content_hash: ContentHash::default(), // Will be calculated later
            status: VerificationStatus::Verified,
            method: VerificationMethod::PathVerification,
            proof: if let Some(proof) = verify_result.proof {
                Some(ContentRef::new(&proof))
            } else {
                None
            },
            last_verified: Some(context.time_snapshot.clone()),
        },
        context: PhantomData,
    };
    
    // Calculate content hash
    path.content_hash = calculate_content_hash(&path)?;
    path.verification.content_hash = calculate_content_hash(&path.verification)?;
    
    // Store the path
    context.storage.store(&path)?;
    
    Ok(Some(path))
}
```

## Cross-Domain Relationship Tracking

The system supports tracking relationships across domains:

```rust
/// Tracks relationships across multiple domains
pub struct CrossDomainRelationshipTracker<C: ExecutionContext> {
    /// Per-domain trackers
    domain_trackers: HashMap<DomainId, Arc<RelationshipTracker<C>>>,
    
    /// Cross-domain synchronization service
    sync_service: Arc<CrossDomainSyncService>,
    
    /// Content-addressed storage
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// Execution context type
    context_type: PhantomData<C>,
}

impl<C: ExecutionContext> CrossDomainRelationshipTracker<C> {
    /// Tracks a cross-domain relationship
    pub async fn track_cross_domain_relationship(
        &self,
        relationship: ContentRef<ResourceRelationship<C>>,
        source_domain: &DomainId,
        target_domain: &DomainId,
        context: &C,
    ) -> Result<(), TrackerError> {
        // Verify that this is a cross-domain relationship
        let relationship_obj = relationship.resolve(&self.storage)?;
        if relationship_obj.relationship_type != RelationshipType::CrossDomain {
            return Err(TrackerError::InvalidRelationshipType);
        }
        
        // Get domain trackers
        let source_tracker = self.domain_trackers.get(source_domain)
            .ok_or(TrackerError::DomainNotFound(source_domain.clone()))?;
        let target_tracker = self.domain_trackers.get(target_domain)
            .ok_or(TrackerError::DomainNotFound(target_domain.clone()))?;
        
        // Track in source domain
        source_tracker.track_relationship(relationship.clone(), context).await?;
        
        // Track in target domain
        target_tracker.track_relationship(relationship.clone(), context).await?;
        
        // Register with cross-domain sync service
        self.sync_service.register_cross_domain_relationship(
            relationship.clone(),
            source_domain.clone(),
            target_domain.clone(),
        ).await?;
        
        Ok(())
    }
    
    /// Finds cross-domain paths between resources
    pub async fn find_cross_domain_path(
        &self,
        source_id: &RegisterId,
        source_domain: &DomainId,
        target_id: &RegisterId,
        target_domain: &DomainId,
        constraints: &PathConstraints,
        context: &C,
    ) -> Result<Option<RelationshipPath<C>>, TrackerError> {
        // If same domain, delegate to single domain tracker
        if source_domain == target_domain {
            let tracker = self.domain_trackers.get(source_domain)
                .ok_or(TrackerError::DomainNotFound(source_domain.clone()))?;
                
            return tracker.find_relationship_path(
                source_id,
                target_id,
                constraints,
                context,
            ).await;
        }
        
        // For cross-domain paths, use the cross-domain sync service
        let path_result = self.sync_service.find_cross_domain_path(
            source_id,
            source_domain,
            target_id,
            target_domain,
            constraints,
            context,
        ).await?;
        
        if path_result.path_exists {
            // Construct the relationship path
            let source_resource = context.storage.get_resource(source_id)?;
            let target_resource = context.storage.get_resource(target_id)?;
            
            let mut path = RelationshipPath {
                content_hash: ContentHash::default(), // Will be calculated later
                source: ContentRef::new(&source_resource),
                target: ContentRef::new(&target_resource),
                relationships: path_result.relationship_path,
                properties: path_result.path_properties,
                verification: VerificationInfo {
                    content_hash: ContentHash::default(), // Will be calculated later
                    status: VerificationStatus::Verified,
                    method: VerificationMethod::CrossDomainPathVerification,
                    proof: if let Some(proof) = path_result.proof {
                        Some(ContentRef::new(&proof))
                    } else {
                        None
                    },
                    last_verified: Some(context.time_snapshot.clone()),
                },
                context: PhantomData,
            };
            
            // Calculate content hash
            path.content_hash = calculate_content_hash(&path)?;
            path.verification.content_hash = calculate_content_hash(&path.verification)?;
            
            // Store the path
            context.storage.store(&path)?;
            
            return Ok(Some(path));
        }
        
        Ok(None)
    }
}
```

## Querying and Monitoring Capabilities

The relationship tracking system provides capability-based access to relationship information:

```rust
/// Creates a capability to monitor relationships for a resource
pub fn create_relationship_monitoring_capability<C: ExecutionContext>(
    resource_id: &RegisterId,
    relationship_types: Option<Vec<RelationshipType>>,
    target_resources: Option<Vec<RegisterId>>,
    context: &C,
) -> Result<Capability, CapabilityError> {
    // Check if initiator has authority to create monitoring capability
    require_capability(
        &context.initiator,
        resource_id,
        &Capability::MonitorRelationships,
        context,
    )?;
    
    // Create monitoring conditions
    let mut conditions = vec![
        CapabilityCondition::ResourceCondition(resource_id.clone()),
    ];
    
    // Add relationship type conditions if specified
    if let Some(types) = relationship_types {
        conditions.push(CapabilityCondition::RelationshipTypesCondition(types));
    }
    
    // Add target resource conditions if specified
    if let Some(targets) = target_resources {
        conditions.push(CapabilityCondition::TargetResourcesCondition(targets));
    }
    
    // Create the capability
    let capability = Capability {
        content_hash: ContentHash::default(), // Will be calculated
        operation: OperationType::MonitorRelationships,
        resource: resource_id.clone(),
        conditions,
        issuer: context.initiator.clone(),
        expires_at: Some(context.time_snapshot.clone() + context.default_capability_duration),
        signature: context.sign_capability(&resource_id)?,
    };
    
    // Calculate content hash
    let capability_with_hash = capability.with_calculated_hash();
    
    Ok(capability_with_hash)
}
```

## Auditing and Change History

The tracking system provides a complete, content-addressed history of relationship changes:

```rust
/// A content-addressed relationship change record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipChangeRecord<C: ExecutionContext> {
    /// Content hash that uniquely identifies this change record
    pub content_hash: ContentHash,
    
    /// Type of change
    pub change_type: ChangeType,
    
    /// The relationship being changed
    pub relationship: ContentRef<ResourceRelationship<C>>,
    
    /// Previous state (for updates and deletions)
    pub previous_state: Option<ContentRef<ResourceRelationship<C>>>,
    
    /// Initiator of the change
    pub initiator: EntityId,
    
    /// Timestamp when the change occurred
    pub timestamp: ContentRef<TimeMapSnapshot>,
    
    /// Reason for the change
    pub reason: String,
    
    /// Execution context type
    pub context: PhantomData<C>,
}

impl<C: ExecutionContext> ContentAddressed for RelationshipChangeRecord<C> {
    fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }
    
    fn verify(&self) -> bool {
        // Calculate hash from contents and verify it matches the stored hash
        calculate_content_hash(self) == self.content_hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        // Serialize to canonical binary format
        serialize_canonical(self).expect("Failed to serialize RelationshipChangeRecord")
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        // Deserialize from binary format
        deserialize_canonical(bytes)
    }
}
```

### Auditing Relationship Changes

```rust
/// Gets the audit history for a relationship
pub async fn get_relationship_audit_history<C: ExecutionContext>(
    tracker: &RelationshipTracker<C>,
    relationship_id: &ContentHash,
    context: &C,
) -> Result<Vec<RelationshipChangeRecord<C>>, TrackerError> {
    // Verify access capability
    require_capability(
        &context.initiator,
        relationship_id,
        &Capability::AuditRelationships,
        context,
    )?;
    
    // Create a pattern to find change records for this relationship
    let pattern = ContentPattern::new()
        .with_tag("type", "RelationshipChangeRecord")
        .with_tag("relationship_id", relationship_id.to_string());
    
    // Query the content-addressed storage
    let record_hashes = context.storage.list(&pattern)?;
    
    // Resolve change records
    let mut records = Vec::new();
    for hash in record_hashes {
        if let Ok(record) = context.storage.get::<RelationshipChangeRecord<C>>(&hash) {
            records.push(record);
        }
    }
    
    // Sort by timestamp (oldest first)
    records.sort_by(|a, b| {
        let a_time = a.timestamp.resolve(&context.storage)
            .map(|t| t.timestamp)
            .unwrap_or(0);
        let b_time = b.timestamp.resolve(&context.storage)
            .map(|t| t.timestamp)
            .unwrap_or(0);
        a_time.cmp(&b_time)
    });
    
    Ok(records)
}
```

## Relationship Networks and Graph Analysis

The tracking system provides functionality for analyzing relationship networks:

```rust
/// Analyzes the relationship network for a set of resources
pub async fn analyze_relationship_network<C: ExecutionContext>(
    tracker: &RelationshipTracker<C>,
    seed_resources: Vec<RegisterId>,
    relationship_types: Vec<RelationshipType>,
    max_depth: u32,
    analysis_options: NetworkAnalysisOptions,
    context: &C,
) -> Result<NetworkAnalysisResult, TrackerError> {
    // Verify access capability for all seed resources
    for resource_id in &seed_resources {
        require_capability(
            &context.initiator,
            resource_id,
            &Capability::AnalyzeNetwork,
            context,
        )?;
    }
    
    // Build the relationship graph starting from seed resources
    let mut graph = RelationshipGraph::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    // Add seed resources to the queue
    for resource_id in seed_resources {
        queue.push_back((resource_id, 0));
    }
    
    // Breadth-first search of the relationship network
    while let Some((resource_id, depth)) = queue.pop_front() {
        // Skip if already visited or max depth reached
        if visited.contains(&resource_id) || depth >= max_depth {
            continue;
        }
        
        // Mark as visited
        visited.insert(resource_id.clone());
        
        // Add node to graph
        let resource = context.storage.get_resource(&resource_id)?;
        graph.add_node(resource_id.clone(), resource);
        
        // Find relationships for this resource
        let query = RelationshipQuery {
            relationship_type: None,
            direction: RelationshipDirection::Bidirectional,
            outgoing: true,
            limit: None,
        };
        
        let relationships = tracker.find_relationships(
            &resource_id,
            &query,
            context,
        ).await?;
        
        // Filter by relationship types
        let filtered_relationships = relationships.into_iter()
            .filter(|rel| relationship_types.contains(&rel.relationship_type))
            .collect::<Vec<_>>();
        
        // Add edges to graph and queue next resources
        for relationship in filtered_relationships {
            let source = relationship.source.resolve(&context.storage)?;
            let target = relationship.target.resolve(&context.storage)?;
            
            // Add edge to graph
            graph.add_edge(
                source.id.clone(),
                target.id.clone(),
                relationship.relationship_type.clone(),
                relationship.direction.clone(),
                ContentRef::new(&relationship),
            );
            
            // Add connected resources to queue if not visited
            let next_resource_id = if source.id == resource_id {
                target.id
            } else {
                source.id
            };
            
            if !visited.contains(&next_resource_id) {
                queue.push_back((next_resource_id, depth + 1));
            }
        }
    }
    
    // Run selected analyses on the graph
    let mut results = NetworkAnalysisResult {
        node_count: graph.node_count(),
        edge_count: graph.edge_count(),
        analysis_results: HashMap::new(),
    };
    
    if analysis_options.compute_centrality {
        let centrality = graph.compute_centrality();
        results.analysis_results.insert("centrality".to_string(), centrality.into());
    }
    
    if analysis_options.find_communities {
        let communities = graph.find_communities();
        results.analysis_results.insert("communities".to_string(), communities.into());
    }
    
    if analysis_options.compute_path_statistics {
        let path_stats = graph.compute_path_statistics();
        results.analysis_results.insert("path_statistics".to_string(), path_stats.into());
    }
    
    Ok(results)
}
```

## Conclusion

The content-addressed resource relationship tracking system provides robust capabilities for managing, querying, and analyzing relationships between resources in the Causality system. By leveraging content addressing, the three-layer effect architecture, and capability-based access control, the tracking system ensures that relationship information is immutable, verifiable, and securely accessible.

Key benefits of this approach include:

1. **Content-Addressed Audit Trail**: Complete, immutable history of relationship changes
2. **Efficient Querying**: High-performance relationship queries through optimized indexing
3. **Path Verification**: Cryptographic verification of relationship paths
4. **Cross-Domain Tracking**: Seamless tracking of relationships across domain boundaries
5. **Network Analysis**: Advanced relationship network analysis capabilities
6. **Capability-Based Access Control**: Fine-grained access control to relationship information

The unified model simplifies relationship tracking while providing powerful capabilities for resource relationship management in complex multi-domain environments.