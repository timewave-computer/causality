# 105: Hybrid Value Model - Direct vs Content-Addressed Values

Causality employs a hybrid value model that strategically combines direct values (stored inline) with content-addressed references. This document explains the design rationale, usage patterns, and benefits of this approach.

## 1. Overview: Why a Hybrid Model?

The hybrid value model balances performance and structural sharing by storing small values directly while content-addressing larger or reusable components:

### Direct Values (Inline Storage)
- **Small primitive values**: `Unit`, `Bool`, `Int`, `Symbol`  
- **Immediate computation**: No indirection overhead
- **Zero allocation**: Values stored directly in registers/memory

### Content-Addressed Values (Reference-Based)
- **Complex expressions**: Lambda terms, large records, effect definitions
- **Structural sharing**: Identical values share storage globally
- **Global optimization**: Cross-function deduplication and caching

## 2. Value Classification Rules

### 2.1 Always Direct (Inline)

```rust
// Base types are always stored directly
pub enum DirectValue {
    Unit,                    // 0 bytes (unit type)
    Bool(bool),              // 1 byte
    Int(u32),                // 4 bytes (RISC-V native word)
    Symbol(InternedString),  // 8 bytes (pointer to string table)
}

// Examples of direct storage
let unit_val = Value::Unit;                    // Stored inline
let bool_val = Value::Bool(true);              // Stored inline  
let int_val = Value::Int(42);                  // Stored inline
let symbol_val = Value::Symbol("hello");       // Stored inline (interned)
```

### 2.2 Always Content-Addressed (Referenced)

```rust
// Complex constructs are always content-addressed
pub enum ContentAddressedValue {
    ExprId(EntityId),        // Lambda calculus expressions
    ResourceId(EntityId),    // Linear resources
    EffectId(EntityId),      // Effect definitions
    HandlerId(EntityId),     // Effect handlers
    CircuitId(EntityId),     // ZK circuit definitions
    ProofId(EntityId),       // ZK proofs
}

// Examples of content-addressed storage
let lambda_expr = ExprId(expr.content_id());           // Always referenced
let user_resource = ResourceId(resource.content_id()); // Always referenced
let transfer_effect = EffectId(effect.content_id());   // Always referenced
```

### 2.3 Hybrid (Context-Dependent)

```rust
// Product and Sum types use hybrid approach
pub enum HybridValue {
    // Small products stored directly
    Product(Box<Value>, Box<Value>),     // Direct if both values are small
    
    // Large products content-addressed
    ProductRef(EntityId),                // Referenced if total size > threshold
    
    // Sums follow same pattern
    Sum { tag: u8, value: Box<Value> },  // Direct if value is small
    SumRef(EntityId),                    // Referenced if value is large
}
```

## 3. Size-Based Thresholds

### 3.1 Storage Decision Algorithm

```rust
const DIRECT_STORAGE_THRESHOLD: usize = 64; // bytes

fn storage_strategy(value: &Value) -> StorageStrategy {
    match value.estimated_size() {
        size if size <= DIRECT_STORAGE_THRESHOLD => StorageStrategy::Direct,
        _ => StorageStrategy::ContentAddressed,
    }
}

pub enum StorageStrategy {
    Direct,           // Store value inline
    ContentAddressed, // Store hash reference, value in global store
}
```

### 3.2 Size Estimation Examples

```rust
// Small values → Direct storage
Value::Unit                              // 0 bytes → Direct
Value::Bool(true)                        // 1 byte → Direct  
Value::Int(12345)                        // 4 bytes → Direct
Value::Symbol("user_id")                 // 8 bytes → Direct

// Medium values → Context dependent
Value::Product(
    Box::new(Value::Int(1)),             // 4 + 4 = 8 bytes → Direct
    Box::new(Value::Int(2))
)

Value::Product(
    Box::new(Value::Product(/* ... */)), // > 64 bytes → Content-addressed
    Box::new(Value::Product(/* ... */))
)

// Large values → Always content-addressed
Lambda { params: vec![...], body: ... }  // Complex AST → Content-addressed
Effect { inputs: ..., outputs: ... }     // Complex structure → Content-addressed
```

## 4. Implementation Patterns

### 4.1 Value Type Hierarchy

```rust
/// Unified value type that handles both direct and referenced values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    // Direct values (stored inline)
    Unit,
    Bool(bool),
    Int(u32),
    Symbol(InternedString),
    
    // Small composite values (stored inline)
    SmallProduct(Box<Value>, Box<Value>),
    SmallSum { tag: u8, value: Box<Value> },
    
    // Content-addressed references
    ExprRef(EntityId),      // References to expressions
    ResourceRef(EntityId),  // References to resources
    ProductRef(EntityId),   // References to large products
    SumRef(EntityId),       // References to large sums
    RecordRef(EntityId),    // References to records
}

impl Value {
    /// Get the storage strategy for this value
    pub fn storage_strategy(&self) -> StorageStrategy {
        match self {
            Value::Unit | Value::Bool(_) | Value::Int(_) | Value::Symbol(_) => 
                StorageStrategy::Direct,
            
            Value::SmallProduct(left, right) => {
                let total_size = left.estimated_size() + right.estimated_size();
                if total_size <= DIRECT_STORAGE_THRESHOLD {
                    StorageStrategy::Direct
                } else {
                    StorageStrategy::ContentAddressed
                }
            }
            
            Value::ExprRef(_) | Value::ResourceRef(_) | Value::ProductRef(_) | 
            Value::SumRef(_) | Value::RecordRef(_) => 
                StorageStrategy::ContentAddressed,
            
            _ => StorageStrategy::ContentAddressed,
        }
    }
}
```

### 4.2 Transparent Value Operations

```rust
impl Value {
    /// Access a value, transparently handling direct vs referenced storage
    pub fn resolve(&self, store: &ContentStore) -> ResolvedValue {
        match self {
            // Direct values return immediately
            Value::Unit => ResolvedValue::Unit,
            Value::Bool(b) => ResolvedValue::Bool(*b),
            Value::Int(i) => ResolvedValue::Int(*i),
            Value::Symbol(s) => ResolvedValue::Symbol(s.clone()),
            
            // Small composite values decompose directly
            Value::SmallProduct(left, right) => {
                ResolvedValue::Product(
                    Box::new(left.resolve(store)),
                    Box::new(right.resolve(store))
                )
            }
            
            // Content-addressed values require store lookup
            Value::ExprRef(id) => {
                let expr = store.get_expression(*id)
                    .expect("Expression not found in store");
                ResolvedValue::Expression(expr)
            }
            
            Value::ResourceRef(id) => {
                let resource = store.get_resource(*id)
                    .expect("Resource not found in store");
                ResolvedValue::Resource(resource)
            }
            
            Value::ProductRef(id) => {
                let product = store.get_product(*id)
                    .expect("Product not found in store");
                ResolvedValue::Product(
                    Box::new(product.left.resolve(store)),
                    Box::new(product.right.resolve(store))
                )
            }
        }
    }
}
```

## 5. Benefits of the Hybrid Approach

### 5.1 Performance Benefits

| Value Type | Storage | Access Cost | Memory Usage |
|------------|---------|-------------|--------------|
| `Bool`, `Int` | Direct | O(1) immediate | Minimal |
| Small `Product` | Direct | O(1) immediate | Sum of components |
| Large `Product` | Content-addressed | O(1) hash lookup | Shared globally |
| `Lambda` expression | Content-addressed | O(1) hash lookup | Shared globally |

### 5.2 Memory Efficiency

```rust
// Example: Common mathematical operations
let add_expr = Lambda {
    params: vec!["x", "y"],
    body: Apply(
        Apply(Var("+"), Var("x")),
        Var("y")
    )
};

// Content-addressed → Only stored once globally
let add_id = add_expr.content_id(); // EntityId

// Multiple uses reference the same storage
let program1 = Apply(add_id, Product(Int(1), Int(2)));  // References add_id
let program2 = Apply(add_id, Product(Int(3), Int(4)));  // References same add_id
let program3 = Apply(add_id, Product(Int(5), Int(6)));  // References same add_id

// All three programs share the addition lambda definition
```

### 5.3 Compilation Optimization

```rust
// Content-addressed expressions enable global optimization
fn optimize_expression(expr_id: EntityId, optimizer: &Optimizer) -> EntityId {
    // Check if optimized version already exists
    if let Some(optimized_id) = optimizer.get_cached_optimization(expr_id) {
        return optimized_id; // Reuse previous optimization work
    }
    
    // Perform optimization and cache result
    let optimized_expr = optimizer.optimize(expr_id);
    let optimized_id = optimized_expr.content_id();
    
    optimizer.cache_optimization(expr_id, optimized_id);
    optimized_id
}

// Same optimizations apply to all instances of the same expression
```

## 6. Layer-Specific Usage Patterns

### 6.1 Layer 0: Register Machine Values

```rust
// Register machine primarily uses direct values for efficiency
pub enum RegisterValue {
    // Direct storage for primitive operations
    Unit,
    Bool(bool),
    Int(u32),
    Symbol(InternedString),
    
    // References to complex objects
    ResourceHandle(EntityId),  // Reference to linear resource
    FunctionHandle(EntityId),  // Reference to compiled function
}

// Register operations work directly with primitive values
Instruction::Move { src: RegisterId(0), dst: RegisterId(1) }; // Direct value copy
Instruction::Add { left: RegisterId(1), right: RegisterId(2), out: RegisterId(3) }; // Direct arithmetic
```

### 6.2 Layer 1: Causality Lisp Expressions

```rust
// Lambda calculus expressions are always content-addressed
pub enum LispValue {
    // Leaf values can be direct
    Const(Value),           // Direct for small constants
    
    // Composite expressions are content-addressed
    Lambda(EntityId),       // Always referenced
    Apply(EntityId),        // Always referenced
    Product(EntityId),      // Referenced if large, direct if small
    Sum(EntityId),          // Referenced if large, direct if small
}

// Example: Expression construction
let small_product = Product(
    Const(Value::Int(1)),   // Direct storage
    Const(Value::Int(2))    // Direct storage
); // → Stored directly as SmallProduct

let complex_lambda = Lambda {
    params: vec!["x", "y", "z"],
    body: /* complex expression tree */
}; // → Always content-addressed
```

### 6.3 Layer 2: Effects and Intents

```rust
// Effects are always content-addressed for global reuse
pub struct Effect {
    pub inputs: Vec<TypeBinding>,    // Schema definitions
    pub outputs: Vec<TypeBinding>,   // Schema definitions
    pub constraints: Vec<Constraint>, // Logical constraints
    pub implementation: EffectImpl,   // Implementation reference
}

// Effect usage always involves content-addressed references
pub enum EffectValue {
    EffectRef(EntityId),        // Reference to effect definition
    HandlerRef(EntityId),       // Reference to effect handler
    IntentRef(EntityId),        // Reference to intent specification
}

// Benefits: Same effect definition reused across many applications
let transfer_effect_id = transfer_effect.content_id();
let escrow_intent = Intent {
    effects: vec![transfer_effect_id, transfer_effect_id], // Same effect, different params
    constraints: vec![/* ... */],
};
```

## 7. Content Store Architecture

### 7.1 Unified Content Store

```rust
/// Global content store managing all content-addressed values
pub struct ContentStore {
    /// Expressions indexed by content hash
    expressions: BTreeMap<EntityId, Expression>,
    
    /// Resources indexed by content hash
    resources: BTreeMap<EntityId, Resource>,
    
    /// Large composite values indexed by content hash
    products: BTreeMap<EntityId, ProductValue>,
    sums: BTreeMap<EntityId, SumValue>,
    records: BTreeMap<EntityId, RecordValue>,
    
    /// Effect definitions indexed by content hash
    effects: BTreeMap<EntityId, Effect>,
    handlers: BTreeMap<EntityId, Handler>,
    intents: BTreeMap<EntityId, Intent>,
    
    /// ZK artifacts indexed by content hash
    circuits: BTreeMap<EntityId, ZkCircuit>,
    proofs: BTreeMap<EntityId, ZkProof>,
}

impl ContentStore {
    /// Store a value and return its content-addressed ID
    pub fn store<T: ContentAddressable>(&mut self, value: T) -> EntityId {
        let id = value.content_id();
        // Store in appropriate collection based on type
        self.store_by_type(id, value);
        id
    }
    
    /// Retrieve a value by its content hash
    pub fn get<T>(&self, id: EntityId) -> Option<&T> 
    where 
        T: ContentAddressable,
    {
        // Lookup in appropriate collection based on type
        self.get_by_type(id)
    }
}
```

### 7.2 Lazy Resolution

```rust
/// Values that support lazy resolution from content store
pub trait Resolvable {
    type Resolved;
    
    /// Resolve this value using the content store
    fn resolve(&self, store: &ContentStore) -> Result<Self::Resolved, ResolveError>;
    
    /// Check if this value needs resolution
    fn needs_resolution(&self) -> bool;
}

impl Resolvable for Value {
    type Resolved = ResolvedValue;
    
    fn resolve(&self, store: &ContentStore) -> Result<ResolvedValue, ResolveError> {
        match self {
            // Direct values resolve immediately
            Value::Unit => Ok(ResolvedValue::Unit),
            Value::Bool(b) => Ok(ResolvedValue::Bool(*b)),
            Value::Int(i) => Ok(ResolvedValue::Int(*i)),
            
            // Referenced values require store lookup
            Value::ExprRef(id) => {
                let expr = store.get_expression(*id)
                    .ok_or(ResolveError::NotFound(*id))?;
                Ok(ResolvedValue::Expression(expr.clone()))
            }
            
            Value::ResourceRef(id) => {
                let resource = store.get_resource(*id)
                    .ok_or(ResolveError::NotFound(*id))?;
                Ok(ResolvedValue::Resource(resource.clone()))
            }
        }
    }
    
    fn needs_resolution(&self) -> bool {
        matches!(self, 
            Value::ExprRef(_) | Value::ResourceRef(_) | 
            Value::ProductRef(_) | Value::SumRef(_) | Value::RecordRef(_)
        )
    }
}
```

## 8. Decision Guidelines

### 8.1 When to Use Direct Storage

 **Use direct storage for**:
- Base types: `Unit`, `Bool`, `Int`, `Symbol`
- Small composite values (< 64 bytes total)
- Frequently accessed primitive values
- Register machine operands
- Intermediate computation results

### 8.2 When to Use Content-Addressed Storage

 **Use content-addressed storage for**:
- Lambda expressions and functions
- Large data structures (> 64 bytes)
- Reusable components (effects, handlers)
- ZK circuits and proofs
- Complex type definitions
- Anything that benefits from global deduplication

### 8.3 Decision Flow Chart

```
Value Creation
    ↓
Is it a base type? ────Yes───→ Direct Storage
    ↓ No
Is total size < 64 bytes? ────Yes───→ Direct Storage
    ↓ No
Is it reusable? ────Yes───→ Content-Addressed Storage
    ↓ No
Is it complex structure? ────Yes───→ Content-Addressed Storage
    ↓ No
Default ───→ Content-Addressed Storage
```

## 9. Implementation Examples

### 9.1 Smart Product Construction

```rust
impl Value {
    /// Smart constructor that chooses storage strategy automatically
    pub fn product(left: Value, right: Value) -> Value {
        let total_size = left.estimated_size() + right.estimated_size();
        
        if total_size <= DIRECT_STORAGE_THRESHOLD {
            // Store directly for small products
            Value::SmallProduct(Box::new(left), Box::new(right))
        } else {
            // Content-address large products
            let product_data = ProductValue { left, right };
            let product_id = product_data.content_id();
            
            // Store in global content store
            CONTENT_STORE.with(|store| {
                store.borrow_mut().store_product(product_id, product_data);
            });
            
            Value::ProductRef(product_id)
        }
    }
}
```

### 9.2 Transparent Value Arithmetic

```rust
impl Value {
    /// Add two values, handling both direct and referenced cases
    pub fn add(&self, other: &Value, store: &ContentStore) -> Result<Value, ArithmeticError> {
        // Resolve both values to their direct representations
        let left_resolved = self.resolve(store)?;
        let right_resolved = other.resolve(store)?;
        
        match (left_resolved, right_resolved) {
            (ResolvedValue::Int(a), ResolvedValue::Int(b)) => {
                Ok(Value::Int(a + b)) // Result is direct
            }
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
}
```

## 10. Performance Characteristics

### 10.1 Access Patterns

| Operation | Direct Value | Content-Addressed Value |
|-----------|--------------|-------------------------|
| **Read** | O(1) immediate | O(1) hash table lookup |
| **Copy** | O(size) memcpy | O(1) reference copy |
| **Equality** | O(size) comparison | O(1) hash comparison |
| **Storage** | O(size) allocation | O(1) global dedup |

### 10.2 Memory Usage Patterns

```rust
// Example: Expression trees
let common_subexpr = Lambda { /* complex definition */ };
let expr1 = Apply(common_subexpr.clone(), Arg1);
let expr2 = Apply(common_subexpr.clone(), Arg2);
let expr3 = Apply(common_subexpr.clone(), Arg3);

// Direct storage: 3 × common_subexpr size
// Content-addressed: 1 × common_subexpr size + 3 × reference size

// Benefit increases with:
// - Size of common_subexpr
// - Number of references
// - Frequency of reuse across programs
```

## 11. Best Practices

### 11.1 Value Construction Guidelines

```rust
//  Good: Use smart constructors
let result = Value::product(left, right);  // Automatically chooses storage strategy

//  Avoid: Manual storage decisions without size consideration
let result = Value::ProductRef(manually_computed_id); // May be inefficient for small values
```

### 11.2 Performance Optimization

```rust
//  Good: Batch resolve operations
let resolved_values: Vec<ResolvedValue> = values
    .iter()
    .map(|v| v.resolve(store))
    .collect::<Result<Vec<_>, _>>()?;

//  Avoid: Repeated individual resolutions
for value in &values {
    let resolved = value.resolve(store)?; // Repeated store access
    // ... use resolved value
}
```

## 12. Conclusion

The hybrid value model provides Causality with:

1. **Performance**: Direct storage for small, frequently accessed values
2. **Efficiency**: Content addressing for large, reusable components  
3. **Transparency**: Uniform API regardless of storage strategy
4. **Optimization**: Global deduplication and shared computation results
5. **Scalability**: Memory usage grows sub-linearly with program complexity

This design enables Causality to handle both high-performance computation (through direct values) and large-scale optimization (through content addressing) within a single coherent framework.

The key insight is that different value types have different optimal storage strategies, and the system should automatically choose the best approach rather than forcing developers to make these decisions manually. 