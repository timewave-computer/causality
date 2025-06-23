# 014: Row Types and Capability System

Row types form the **foundation of Causality's capability system**, providing static type-level guarantees for resource field access while enabling polymorphic record operations. This document explains how row types integrate with the three-layer architecture to provide fine-grained access control and efficient ZK circuit compilation.

## 1. Overview: Row Types as the Capability Foundation

**Row types are central to Causality's architecture** because they enable:

1. **Static Capability Resolution**: Polymorphic field access compiles to concrete operations
2. **Zero-Runtime-Cost Access Control**: All capability checks happen at compile time
3. **ZK Circuit Compatibility**: Fixed record structures enable efficient proof generation
4. **Content-Addressed Schema Management**: Record schemas are content-addressed for global reuse

### Integration Across Layers

| Layer | Row Type Role | Purpose |
|-------|---------------|---------|
| **Layer 0** | Static Record Layout | Compiled operations access fixed offsets |
| **Layer 1** | Polymorphic Operations | `project`, `restrict`, `extend`, `diff` operations |
| **Layer 2** | Capability-Based Effects | Dynamic capability requirements resolve to static schemas |

## 2. Row Type System Fundamentals

### 2.1 Row Type Structure

```rust
pub struct RowType {
    /// Named fields in the row (ordered for deterministic comparison)
    pub fields: BTreeMap<String, TypeInner>,
    
    /// Optional row variable for open row types
    pub extension: Option<RowVariable>,
}

pub struct RowVariable {
    /// Variable name
    pub name: String,
    
    /// Optional constraint on what fields this variable can contain
    pub constraint: Option<RowConstraint>,
}

pub enum RowConstraint {
    /// Variable must not contain these field names
    Lacks(Vec<String>),
    
    /// Variable must contain these field types
    Contains(BTreeMap<String, TypeInner>),
}
```

### 2.2 Row Polymorphism in Action

**Open Row Types** enable polymorphic operations:

```rust
// Open row type: { name: String, age: Int | r }
// The "|r" means "any additional fields"
let user_row = RowType::open(
    btreemap! {
        "name".to_string() => TypeInner::Base(BaseType::Symbol),
        "age".to_string() => TypeInner::Base(BaseType::Int),
    },
    RowVariable::new("r")
);

// Can project 'name' from any record containing at least these fields
let name_projection = user_row.project("name"); // ✅ Always succeeds
```

**Closed Row Types** provide exact field specifications:

```rust
// Closed row type: { x: Int, y: Int }
let point_row = RowType::with_fields(btreemap! {
    "x".to_string() => TypeInner::Base(BaseType::Int),
    "y".to_string() => TypeInner::Base(BaseType::Int),
});

// Operations must match exactly
let z_projection = point_row.project("z"); // ❌ Compile error: field not found
```

## 3. Row Operations and Compile-Time Resolution

### 3.1 Core Row Operations

All row operations are **compile-time** operations that produce static type information:

| Operation | Type Signature | Purpose | Example |
|-----------|----------------|---------|---------|
| `project` | `Row → FieldName → Type` | Extract field type | `{ x: Int, y: Int }.project("x") → Int` |
| `restrict` | `Row → FieldName → Row'` | Remove field | `{ x: Int, y: Int }.restrict("y") → { x: Int }` |
| `extend` | `Row → (FieldName, Type) → Row'` | Add field | `{ x: Int }.extend("y", Int) → { x: Int, y: Int }` |
| `diff` | `Row → Row → Row` | Field difference | `{ x: Int, y: Int }.diff({ y: Int }) → { x: Int }` |

### 3.2 Example: Polymorphic Access Pattern

```rust
// Define a polymorphic function that works on any record with 'name' field
fn get_name<R>(record: Record<{ name: String | R }>) -> String 
where 
    R: RowType + Lacks("name")  // R cannot already contain 'name'
{
    record.project("name")  // Compiles to static offset access
}

// Usage with different record types
let user = Record::new(btreemap! {
    "name" => "Alice",
    "age" => 30,
    "email" => "alice@example.com"
});

let product = Record::new(btreemap! {
    "name" => "Widget",
    "price" => 1299,
    "category" => "Hardware"
});

let name1 = get_name(user);    // ✅ Compiles: user has 'name' + other fields
let name2 = get_name(product); // ✅ Compiles: product has 'name' + other fields
```

## 4. Capability System Integration

### 4.1 Capabilities as Row Type Constraints

**Capabilities control which row operations are permitted**:

```rust
pub enum RecordCapability {
    ReadField(FieldName),
    WriteField(FieldName),
    ProjectFields(Vec<FieldName>),
    ExtendRecord(RecordSchema),
    RestrictRecord(Vec<FieldName>),
    CreateRecord(RecordSchema),
    DeleteRecord,
    FullRecordAccess,
}

pub struct Capability {
    pub name: String,
    pub level: CapabilityLevel,
    pub record_capability: Option<RecordCapability>,
}
```

### 4.2 Static Capability Resolution Process

The capability system performs **static analysis** to resolve dynamic operations:

```
1. Capability Analysis
   Intent → Required Capabilities → Static Schema Resolution

2. Schema Monomorphization  
   Polymorphic field access → Concrete field operations with fixed layouts

3. Effect Compilation
   Layer 2 capability effects → Layer 1 row operations

4. Code Generation
   Layer 1 row operations → Layer 0 register operations
```

### 4.3 Example: Capability-Controlled Access

```rust
// Layer 2: Declare capability requirements
let read_user_name = Effect::AccessField {
    resource_id: user_resource,
    field_name: "name",
    required_capability: Capability::ReadField("name"),
};

// Compilation process:
// 1. Check: Does execution context have ReadField("name") capability?
// 2. Resolve: What is the concrete schema of user_resource?
// 3. Monomorphize: Replace polymorphic access with static offset
// 4. Generate: Layer 0 instructions with fixed memory layout

// Layer 1: Compiled to concrete row operation
let concrete_access = project_field(
    user_record,           // Concrete type: { name: String, age: Int, email: String }
    "name",               // Field: known at compile time
    offset_16             // Memory offset: computed statically
);

// Layer 0: Generated register machine code
[
    Move { src: user_reg, dst: temp_reg },
    LoadFieldAtOffset { src: temp_reg, offset: 16, dst: name_reg },
    // Field access becomes simple memory load with static offset
]
```

## 5. Location-Aware Row Types

### 5.1 Row Types with Location Information

**Location-aware row types** extend the traditional row type system to include location information, enabling the same field operations to work seamlessly across local and remote data:

```rust
pub struct LocationAwareRowType {
    /// Base row type with field definitions
    pub base_row: RowType,
    
    /// Location information for each field
    pub field_locations: BTreeMap<String, Location>,
    
    /// Default location for new fields
    pub default_location: Location,
    
    /// Migration specifications for moving fields between locations
    pub migration_specs: Vec<MigrationSpec>,
}

pub enum Location {
    Local,
    Remote(String),
    Domain(String),
}
```

### 5.2 Location-Transparent Operations

The same row operations work regardless of data location:

```rust
// Local row access - traditional operation
let local_row = LocationAwareRowType::new(
    user_row_type,
    btreemap! {
        "name" => Location::Local,
        "email" => Location::Local,
        "preferences" => Location::Local,
    }
);

let local_name = local_row.project_local("name");

// Remote row access - same API, different location
let distributed_row = LocationAwareRowType::new(
    user_row_type,
    btreemap! {
        "name" => Location::Remote("user_service".to_string()),
        "email" => Location::Remote("user_service".to_string()),
        "preferences" => Location::Remote("preferences_service".to_string()),
    }
);

let remote_name = distributed_row.project_remote("name", &Location::Remote("user_service".to_string()));

// Mixed location access
let hybrid_row = LocationAwareRowType::new(
    user_row_type,
    btreemap! {
        "name" => Location::Local,                                    // Cached locally
        "email" => Location::Remote("user_service".to_string()),     // Authoritative remote
        "preferences" => Location::Remote("prefs_service".to_string()), // Separate service
    }
);
```

### 5.3 Automatic Data Migration

Location-aware row types support automatic migration when data needs to move:

```rust
// Define migration specification
let migration_spec = MigrationSpec {
    from: Location::Local,
    to: Location::Remote("fast_storage".to_string()),
    fields: vec!["large_dataset".to_string()],
    strategy: MigrationStrategy::Move,
    protocol: TypeInner::Base(BaseType::Unit), // Auto-derived
};

// Migration is triggered automatically when needed
let migrated_row = hybrid_row.migrate(
    "large_dataset",
    Location::Remote("fast_storage".to_string()),
    MigrationStrategy::Copy
)?;

// Protocol is automatically derived for the migration:
// 1. Send migration request
// 2. Stream data chunks  
// 3. Receive confirmation
// 4. Update location metadata
```

### 5.4 Distributed Row Updates

Updates can span multiple locations with automatic coordination:

```rust
// Distributed update across multiple locations
let field_updates = btreemap! {
    "name" => ("Alice Smith".to_string(), Location::Remote("user_service".to_string())),
    "email" => ("alice.smith@example.com".to_string(), Location::Remote("user_service".to_string())),
    "preferences" => (new_preferences, Location::Remote("prefs_service".to_string())),
};

let updated_row = hybrid_row.distributed_update(field_updates)?;

// Automatically coordinates:
// 1. Batches updates by target location
// 2. Derives communication protocols for each location
// 3. Executes updates with proper ordering and consistency
// 4. Returns unified result
```

## 6. Distributed Capability System

### 6.1 Location-Aware Capabilities

Capabilities now include location constraints and delegation mechanisms:

```rust
pub enum LocationConstraint {
    /// Field must be accessed locally only
    LocalOnly,
    
    /// Field can be accessed from specific locations
    AllowedLocations(Vec<Location>),
    
    /// Field can be accessed from any location
    AnyLocation,
    
    /// Field requires specific protocol for remote access
    RequiresProtocol(SessionType),
}

pub enum RecordCapability {
    // Traditional capabilities
    ReadField(FieldName),
    WriteField(FieldName),
    
    // Location-aware capabilities
    DistributedAccess {
        fields: Vec<FieldName>,
        allowed_locations: Vec<Location>,
        required_protocols: Vec<SessionType>,
    },
    
    // Session-based delegation
    SessionDelegation {
        delegated_capabilities: Vec<RecordCapability>,
        session_duration: Option<u64>,
        target_location: Location,
    },
    
    // Cross-location operations
    CrossLocationUpdate {
        source_location: Location,
        target_location: Location,
        fields: Vec<FieldName>,
        consistency_model: ConsistencyModel,
    },
}
```

### 6.2 Cross-Location Capability Verification

The capability system verifies distributed access across locations:

```rust
impl CapabilitySet {
    /// Verify capability for cross-location access
    pub fn verify_distributed_access(
        &self,
        field_name: &str,
        from_location: &Location,
        to_location: &Location,
        operation: &str,
    ) -> Result<bool, CapabilityError> {
        // 1. Check if we have the basic capability for the field
        let base_capability = self.get_field_capability(field_name)?;
        
        // 2. Verify location constraints
        match &base_capability.location_constraint {
            LocationConstraint::LocalOnly => {
                if from_location != to_location {
                    return Err(CapabilityError::RemoteAccessDenied);
                }
            }
            LocationConstraint::AllowedLocations(allowed) => {
                if !allowed.contains(to_location) {
                    return Err(CapabilityError::LocationNotAllowed);
                }
            }
            LocationConstraint::RequiresProtocol(required_protocol) => {
                // Verify that the operation uses the required protocol
                self.verify_protocol_compliance(required_protocol, operation)?;
            }
            LocationConstraint::AnyLocation => {
                // No additional constraints
            }
        }
        
        // 3. Check for session delegation if needed
        if from_location != to_location {
            self.verify_session_delegation(from_location, to_location)?;
        }
        
        Ok(true)
    }
}
```

### 6.3 Protocol Derivation from Row Operations

Row operations automatically derive communication protocols:

```rust
// Field access pattern
let field_access = FieldAccess {
    field_name: "balance".to_string(),
    access_type: AccessType::Read,
    source_location: Location::Local,
    target_location: Location::Remote("database".to_string()),
};

// Automatically derived protocol
let derived_protocol = derive_field_access_protocol(&field_access)?;

// Results in:
// SessionType::Send(
//     Box::new(TypeInner::Base(BaseType::Symbol)), // Field query
//     Box::new(SessionType::Receive(
//         Box::new(TypeInner::Base(BaseType::Int)), // Field value
//         Box::new(SessionType::End)
//     ))
// )

// Multi-field access pattern
let multi_field_access = vec![
    FieldAccess { field_name: "name".to_string(), access_type: AccessType::Read, .. },
    FieldAccess { field_name: "email".to_string(), access_type: AccessType::Read, .. },
    FieldAccess { field_name: "preferences".to_string(), access_type: AccessType::Write, .. },
];

// Automatically derives batched protocol
let batched_protocol = derive_multi_field_protocol(&multi_field_access)?;

// Results in optimized protocol:
// SessionType::Send(
//     Box::new(TypeInner::Product(
//         Box::new(TypeInner::Base(BaseType::Symbol)), // Field list
//         Box::new(TypeInner::Base(BaseType::Symbol))  // Update data
//     )),
//     Box::new(SessionType::Receive(
//         Box::new(TypeInner::Product(
//             Box::new(TypeInner::Base(BaseType::Symbol)), // Read results
//             Box::new(TypeInner::Base(BaseType::Bool))     // Write confirmation
//         )),
//         Box::new(SessionType::End)
//     ))
// )
```

### 6.4 Cross-Location Constraint Examples

Complex distributed operations with unified constraints:

```rust
// Example 1: Distributed transaction across multiple services
let distributed_transaction = TransformConstraint::DistributedSync {
    locations: vec![
        Location::Remote("payment_service".to_string()),
        Location::Remote("inventory_service".to_string()),
        Location::Remote("shipping_service".to_string()),
    ],
    sync_type: TypeInner::Base(BaseType::Symbol),
    consistency_model: "two_phase_commit".to_string(),
};

// Example 2: Cross-location data migration with consistency
let migration_constraint = TransformConstraint::DataMigration {
    from_location: Location::Remote("old_database".to_string()),
    to_location: Location::Remote("new_database".to_string()),
    data_type: TypeInner::Record(user_record_type),
    migration_strategy: "online_migration".to_string(),
};

// Example 3: Capability delegation across security domains
let delegation_constraint = TransformConstraint::CapabilityAccess {
    resource: "sensitive_user_data".to_string(),
    required_capability: Some(Capability {
        name: "delegate_read_access".to_string(),
        level: CapabilityLevel::High,
        location_constraint: Some(LocationConstraint::RequiresProtocol(
            SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Symbol)), // Delegation request
                Box::new(SessionType::Receive(
                    Box::new(TypeInner::Base(BaseType::Bool)), // Approval
                    Box::new(SessionType::End)
                ))
            )
        )),
    }),
    access_pattern: "time_limited_delegation".to_string(),
};

// Example 4: Multi-location aggregation query
let aggregation_constraint = TransformConstraint::RemoteTransform {
    source_location: Location::Local,
    target_location: Location::Remote("analytics_cluster".to_string()),
    source_type: TypeInner::Record(query_record_type),
    target_type: TypeInner::Record(result_record_type),
    protocol: TypeInner::Session(Box::new(SessionType::Send(
        Box::new(TypeInner::Record(query_record_type)), // Aggregation query
        Box::new(SessionType::Receive(
            Box::new(TypeInner::Record(result_record_type)), // Aggregated results
            Box::new(SessionType::End)
        ))
    ))),
};
```

### 6.5 Unified Constraint Resolution

All constraints are resolved through the same unified system:

```rust
// Single constraint solver handles all operation types
let mut constraint_system = TransformConstraintSystem::new();

// Add local field access
constraint_system.add_constraint(TransformConstraint::LocalTransform {
    source_type: TypeInner::Record(user_record_type.clone()),
    target_type: TypeInner::Base(BaseType::Symbol),
    transform: TransformDefinition::FunctionApplication {
        function: "extract_name".to_string(),
        argument: "user_record".to_string(),
    },
});

// Add remote field access - same constraint language!
constraint_system.add_constraint(TransformConstraint::RemoteTransform {
    source_location: Location::Local,
    target_location: Location::Remote("user_service".to_string()),
    source_type: TypeInner::Base(BaseType::Symbol),
    target_type: TypeInner::Base(BaseType::Symbol),
    protocol: TypeInner::Session(Box::new(derived_protocol)),
});

// Add cross-location migration - same constraint language!
constraint_system.add_constraint(migration_constraint);

// Single solver resolves all constraints together
let mut det_sys = DeterministicSystem::new();
let resolved_operations = constraint_system.solve_constraints(&mut det_sys)?;

// Results in unified execution plan that handles:
// 1. Local field extraction
// 2. Remote field access with proper protocol
// 3. Data migration with consistency guarantees
// 4. All using the same mathematical framework
```

## 7. Content-Addressed Schema Management

### 7.1 Schema Content Addressing

**All record schemas are content-addressed**, enabling global reuse:

```rust
// Schema definition becomes content-addressed
let user_schema = RecordSchema {
    fields: btreemap! {
        "name" => TypeInner::Base(BaseType::Symbol),
        "age" => TypeInner::Base(BaseType::Int),
        "email" => TypeInner::Base(BaseType::Symbol),
    },
    capabilities: hashset! {
        "read_public_fields",
        "update_email",
    },
};

// Schema gets content-addressed ID
let schema_id = user_schema.content_id(); // SchemaId(EntityId)

// Same logical schema → same ID across the entire system
assert_eq!(
    user_schema.content_id(),
    another_identical_user_schema.content_id()
);
```

### 7.2 Benefits of Content-Addressed Schemas

1. **Global Deduplication**: Identical schemas share implementation across applications
2. **Version Management**: Schema changes create new content IDs automatically
3. **Cache Efficiency**: Common schemas can be cached by ID
4. **ZK Optimization**: Identical schemas use same circuit patterns

## 8. ZK Circuit Integration

### 8.1 Static Layout for ZK Compatibility

Row type resolution ensures **fixed memory layouts** for ZK circuit generation:

```rust
// Before resolution: Polymorphic access
fn transfer_with_fee<R>(account: Record<{ balance: Int, fee_rate: Float | R }>) 
where R: Lacks("balance", "fee_rate")
{
    let balance = account.project("balance");    // Polymorphic
    let fee_rate = account.project("fee_rate");  // Polymorphic
    // ... transfer logic
}

// After resolution: Static layout
struct ConcreteAccountLayout {
    balance: Int,      // Offset: 0
    fee_rate: Float,   // Offset: 8  
    owner: Address,    // Offset: 16
    created_at: Int,   // Offset: 48
}

// ZK circuit generation
let circuit_constraints = vec![
    // Balance access: static memory load
    LoadField { base_addr: account_reg, offset: 0, dst: balance_reg },
    // Fee rate access: static memory load  
    LoadField { base_addr: account_reg, offset: 8, dst: fee_rate_reg },
    // Transfer constraints use concrete layout
    TransferConstraint { 
        src_balance_offset: 0,
        dst_balance_offset: 0,
        amount: amount_reg,
    },
];
```

### 8.2 Circuit Optimization Through Schema Sharing

```rust
// Multiple functions using same schema → same circuit pattern
fn get_account_balance(account: StandardAccount) -> Int;  // Schema A
fn update_account_balance(account: StandardAccount, new_balance: Int) -> StandardAccount; // Schema A  
fn validate_account(account: StandardAccount) -> Bool;   // Schema A

// All compile to circuits using StandardAccount layout
// → Circuit pattern can be cached and reused
let standard_account_circuit = CircuitCache::get("StandardAccount_operations");
```

## 9. Row Type Operations at Each Layer

### 9.1 Layer 2: Capability-Based Effects

```rust
// High-level capability-controlled operations
let user_name = AccessField {
    resource: user_resource,
    field: "name", 
    capability: ReadField("name"),
}.perform()?;

let updated_user = UpdateField {
    resource: user_resource,
    field: "email",
    new_value: "new@example.com",
    capability: WriteField("email"),
}.perform()?;
```

### 9.2 Layer 1: Row Type Operations

```rust
// Direct row type manipulation (post-capability-resolution)
let user_row = RowType::with_fields(user_fields);
let name_type = user_row.project("name")?;           // → TypeInner::Base(Symbol)
let restricted = user_row.restrict("internal_id")?;  // Remove sensitive field
let extended = user_row.extend("preferences", preferences_type)?; // Add field
```

### 9.3 Layer 0: Static Memory Operations

```rust
// Compiled to register machine operations
[
    Move { src: user_reg, dst: temp_reg },
    // Static offset computed from row type resolution
    LoadFieldAtOffset { src: temp_reg, offset: 16, dst: name_reg }, 
    // Field access becomes simple memory load
]
```

## 10. Advanced Row Type Features

### 10.1 Row Constraints and Polymorphism

```rust
// Constraint: R must lack 'id' field (prevents conflicts)
fn add_id<R>(record: Record<R>, id: String) -> Record<{ id: String | R }>
where 
    R: RowType + Lacks("id")
{
    record.extend("id", id)
}

// Constraint: R must contain 'timestamp' field
fn audit<R>(record: Record<R>) -> AuditLog
where 
    R: RowType + Contains("timestamp": Int)
{
    let timestamp = record.project("timestamp"); // Guaranteed to exist
    AuditLog::new(timestamp, record.content_id())
}
```

### 10.2 Row Variables and Type Inference

```rust
// Open row types enable flexible composition
let base_user: Record<{ name: String, email: String | r }> = create_user();

// Add fields while preserving polymorphism
let user_with_prefs: Record<{ name: String, email: String, preferences: Prefs | r }> = 
    base_user.extend("preferences", user_preferences);

// Type inference maintains row variable
// r can still be extended with additional fields
```

## 11. Implementation Benefits

### 11.1 Performance Benefits

1. **Zero Runtime Cost**: All capability checks at compile time
2. **Static Memory Layout**: No dynamic field lookup
3. **Optimal ZK Circuits**: Fixed layouts enable constraint optimization
4. **Cache Efficiency**: Content-addressed schemas enable global caching

### 11.2 Safety Benefits

1. **Compile-Time Capability Verification**: No runtime access control failures
2. **Type Safety**: Row operations prevent invalid field access
3. **Linearity Preservation**: Row operations respect resource linearity
4. **Content Integrity**: Schema content addressing prevents tampering

### 11.3 Developer Experience Benefits

1. **Polymorphic Functions**: Write once, work with many record types
2. **Static Error Detection**: Catch capability/schema errors at compile time
3. **IDE Support**: Complete type information enables rich tooling
4. **Gradual Typing**: Start with open rows, close as requirements clarify

## 12. Best Practices

### 12.1 Schema Design

```rust
// ✅ Good: Modular schema with clear capabilities
let user_public_schema = RecordSchema {
    fields: btreemap! {
        "name" => TypeInner::Base(BaseType::Symbol),
        "email" => TypeInner::Base(BaseType::Symbol),
    },
    capabilities: hashset!["read_public"],
};

let user_private_schema = RecordSchema {
    fields: btreemap! {
        "internal_id" => TypeInner::Base(BaseType::Int),
        "created_at" => TypeInner::Base(BaseType::Int),
    },
    capabilities: hashset!["admin_access"],
};

// ❌ Avoid: Monolithic schema with mixed access levels
let monolithic_schema = RecordSchema {
    fields: btreemap! {
        "name" => TypeInner::Base(BaseType::Symbol),          // Public
        "email" => TypeInner::Base(BaseType::Symbol),         // Public  
        "internal_id" => TypeInner::Base(BaseType::Int),      // Private
        "admin_notes" => TypeInner::Base(BaseType::Symbol),   // Admin only
    },
    capabilities: hashset!["mixed_access"], // Too broad
};
```

### 12.2 Capability Granularity

```rust
// ✅ Good: Fine-grained capabilities
let capabilities = vec![
    Capability::ReadField("name"),
    Capability::ReadField("email"),
    Capability::WriteField("email"),        // Can update email
    // Cannot write name or read internal fields
];

// ❌ Avoid: Overly broad capabilities
let broad_capability = Capability::FullRecordAccess; // Too permissive
```

## 13. Conclusion

Row types are **fundamental to Causality's architecture** because they provide the bridge between:

- **Layer 2's dynamic capability requirements** and **Layer 1's static operations**
- **Polymorphic record manipulation** and **efficient ZK circuit generation**
- **Flexible schema evolution** and **deterministic content addressing**
- **Compile-time safety guarantees** and **zero-runtime-cost enforcement**

This design enables Causality to provide both **developer productivity** (through polymorphic operations) and **system efficiency** (through static compilation), making it unique among verifiable computation platforms.

The row type system demonstrates how **sophisticated type-level programming** can provide practical benefits: better performance, stronger safety guarantees, and more efficient ZK proof generation, all while maintaining developer ergonomics. 