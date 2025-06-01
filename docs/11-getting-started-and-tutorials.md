# Getting Started and Tutorials

The Causality framework provides a comprehensive foundation for building linear resource applications through its three-layer architecture: a minimal register machine (Layer 0), structured types with row polymorphism (Layer 1), and declarative effect-based programming (Layer 2). This guide introduces the essential concepts and practical techniques needed to begin productive development while understanding the architectural principles that enable safe, verifiable resource management.

The framework emphasizes linearity as a core principle - resources are consumed exactly once by default, ensuring conservation laws and preventing common errors like double-spending. This linearity, combined with static verification and content addressing, forms the foundation for building reliable distributed applications.

## Development Environment Setup

Setting up a productive development environment for the Causality framework involves configuring the necessary tools and dependencies while ensuring compatibility with the three-layer architecture. The framework provides comprehensive tooling support through Nix-based environment management that includes all components needed for register machine development, type system exploration, and effect handler implementation.

The recommended approach uses the provided Nix flake configuration that includes all necessary dependencies for both Rust and OCaml development, specialized tools for ZK circuit generation, and the complete Layer 1 Causality Lisp development environment. This approach ensures consistency across different development machines while providing immediate access to all framework capabilities.

```bash
# Clone the repository
git clone <repository-url>
cd causality

# Enter the Nix development environment
nix develop

# Build all crates in the workspace
cargo build

# Run tests for key architectural components
cargo test -p causality-types             # Foundational types for all layers
cargo test -p causality-vm                # Layer 0: Typed Register Machine
cargo test -p causality-lisp-ast          # Layer 1: Lisp AST and primitives
cargo test -p causality-lisp-compiler     # Layer 1: Lisp to Layer 0 compiler
cargo test -p causality-effects-engine    # Layer 2: Effects, Intents, TEG

# Build and test OCaml components (if using ml_causality)
cd ml_causality
dune build
dune test
```

Environment verification involves building all three architectural layers and running the comprehensive test suite to ensure proper configuration. The framework includes specialized test scripts for each layer that validate different aspects of the system - from register machine instruction execution to row type inference to effect handler composition.

## Understanding Linear Resources

The Causality framework centers around linear resources - values that must be consumed exactly once. This constraint, enforced by the type system, ensures conservation laws are maintained and prevents common errors in distributed systems. Understanding linearity and its implications provides the foundation for building safe applications.

Linear resources represent quantifiable assets or capabilities that cannot be duplicated or discarded without explicit handling. When a resource is consumed, it becomes unavailable for further use, ensuring that value is neither created nor destroyed, only transformed.

```rust
use causality_types::core::resource::*;
use causality_types::core::object::*;
use causality_types::primitive::ids::*;

fn linear_resource_basics() -> Result<(), Box<dyn std::error::Error>> {
    // Create a linear resource (consumed exactly once)
    let token = Resource {
        id: ResourceId::new([1u8; 32]),
        data: TokenData { amount: 100 },
        owner: alice_address(),
        consumed: false,
        capabilities: row_type!{
            transfer: TransferCapability,
            balance: BalanceCapability
        },
        timestamp: Timestamp::now(),
    };
    
    // Resources can be wrapped in Objects with different linearity
    let shared_config = Object::<ConfigData, Unrestricted> {
        data: ConfigData { timeout: 30 },
        linearity: Unrestricted,  // Can be used multiple times
        capabilities: Set::new(),
    };
    
    let optional_perm = Object::<Permission, Affine> {
        data: Permission::Read,
        linearity: Affine,       // Can be used at most once
        capabilities: Set::from([Capability::Read]),
    };
    
    println!("Linear token: {:?}", token);
    println!("Shared config: {:?}", shared_config);
    println!("Optional permission: {:?}", optional_perm);
    
    Ok(())
}
```

The framework provides four linearity qualifiers:
- **Linear**: Must be used exactly once (default for resources)
- **Affine**: Can be used at most once (optional consumption)
- **Relevant**: Must be used at least once (required consumption)
- **Unrestricted**: Can be used any number of times (shared data)

## Row Types and Capabilities

Row types enable compile-time tracking of capabilities and extensible records. They form Layer 1 of the architecture, providing type-safe access to resource capabilities while maintaining the benefits of static verification. Row operations occur at compile time, ensuring zero runtime overhead.

```rust
use causality_types::row_types::*;

fn row_type_example() -> Result<(), Box<dyn std::error::Error>> {
    // Define a row type for token capabilities
    type TokenCapabilities = Row!{
        transfer: TransferPermission,
        balance: BalanceQuery,
        mint: MintPermission,
        burn: BurnPermission
    };
    
    // Create a token with specific capabilities
    let admin_token = Token::<TokenCapabilities> {
        data: TokenData { amount: 1000 },
        capabilities: row!{
            transfer: TransferPermission::new(),
            balance: BalanceQuery::new(),
            mint: MintPermission::new(),
            burn: BurnPermission::new(),
        },
    };
    
    // Extract transfer capability (compile-time operation)
    let (transfer_cap, remaining_token) = row_extract!(
        admin_token, 
        transfer
    );
    
    // remaining_token now has type Token<{balance, mint, burn}>
    // transfer capability has been linearly extracted
    
    println!("Extracted transfer capability");
    println!("Remaining capabilities: balance, mint, burn");
    
    Ok(())
}
```

Row type operations are verified at compile time, ensuring that:
- Capabilities cannot be extracted twice
- Only available capabilities can be accessed
- Type signatures accurately reflect available capabilities

## Effects and Handlers

Effects represent specific, potentially state-changing operations and are the building blocks of Layer 2. Each `Effect` is content-addressable via its SSZ hash. Handlers are pure transformations acting on effects.

```rust
// Assuming prelude imports for EntityId, Str, DomainId, ExprId, ResourceFlow, Timestamp
// And helper functions like alice_address(), bob_address(), token_resource_id(), current_timestamp()
// And a conceptual ResourceFlow::new_linear_input/output helper.

// Define an Effect struct (as per causality-types)
pub struct Effect {
    pub id: EntityId,           // Unique, content-addressed identifier (SSZ hash)
    pub name: Str,              // User-friendly name
    pub domain_id: DomainId,    // Application domain
    pub effect_type: Str,       // Categorizes the effect (e.g., "transfer")
    pub inputs: Vec<ResourceFlow>, // Input resources/data
    pub outputs: Vec<ResourceFlow>,// Output resources/data
    pub expression: Option<ExprId>,// Optional Layer 1 Lisp expression for core logic
    pub timestamp: Timestamp,
    pub hint: Option<ExprId>,   // Optional Layer 1 Lisp expression for hints
}

// Conceptual Handler trait
pub trait Handler<InEffect, OutEffect> {
    fn name(&self) -> &str;
    fn transform(&self, effect: InEffect) -> OutEffect;
}

// Example: A simple batching handler for a specific Effect type
struct BatchTokenTransferOptimizer;

// Assume MyCustomEffect is a type that fits the general Effect structure or is transformable to/from it.
// For simplicity, let's assume it operates on our defined Effect struct.
fn should_batch(effect: &Effect) -> bool { /* ... logic ... */ true }
fn transform_to_batched_effect(effect: Effect) -> Effect { /* ... logic ... */ effect }

impl Handler<Effect, Effect> for BatchTokenTransferOptimizer {
    fn name(&self) -> &str { "BatchTokenTransferOptimizer" }

    fn transform(&self, effect: Effect) -> Effect {
        if effect.effect_type == "token_transfer" && should_batch(&effect) {
            transform_to_batched_effect(effect)
        } else {
            effect
        }
    }
}

fn effect_example() -> Result<(), Box<dyn std::error::Error>> {
    let effect_id = EntityId::from_slice(&[0u8; 32]); // Placeholder ID
    let domain_id = DomainId::from_slice(&[1u8; 32]); // Placeholder DomainId

    let transfer_effect = Effect {
        id: effect_id, // In reality, derived from SSZ hash of content
        name: "Transfer 50 TKN from Alice to Bob".into(),
        domain_id,
        effect_type: "token_transfer".into(),
        inputs: vec![ResourceFlow::new_linear_input(token_resource_id(), "TKN", alice_address(), 50u64)],
        outputs: vec![ResourceFlow::new_linear_output(token_resource_id(), "TKN", bob_address(), 50u64)],
        expression: None, // Logic might be handled by a dedicated runtime module for "token_transfer" effect_type
        timestamp: current_timestamp(),
        hint: None,
    };

    let batch_optimizer = BatchTokenTransferOptimizer;
    let potentially_batched_effect = batch_optimizer.transform(transfer_effect);

    println!("Original or Batched Effect ID: {:?}", potentially_batched_effect.id);
    println!("Handler used: {}", batch_optimizer.name());

    Ok(())
}
```
Pre/post conditions are no longer direct fields in the `Effect` struct. They are typically enforced by the logic within an associated Layer 1 `expression`, or validated by the effects engine based on `inputs`, `outputs`, and `effect_type`.

The handler/interpreter separation ensures:
- Handlers are pure functions (easy to test and compose)
- State changes occur only in the interpreter
- Effect transformations can be optimized before execution
- Multiple handlers can process the same effect differently

## Intent-Based Programming (Layer 2)

Intents are declarative requests for desired state transformations. They specify inputs, outputs, and optionally, computational logic (as a Layer 1 Lisp expression) or hints. The Layer 2 effects engine processes intents, often constructing a Temporal Effect Graph (TEG) to manage causal dependencies between the resulting effects.

```rust
// Assuming prelude imports for EntityId, Str, DomainId, ExprId, ResourceFlow, Timestamp
// And helper functions like alice_address(), bob_address(), token_resource_id(), current_timestamp()
// And conceptual ResourceFlow::new_escrow_input/output, alice_btc_resource_id(), bob_eth_resource_id() helpers.

// Define an Intent struct (as per causality-types)
pub struct Intent {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub priority: u32,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>, // Optional Layer 1 Lisp expression for the intent's logic
    pub timestamp: Timestamp,
    pub hint: Option<ExprId>,       // Optional Layer 1 Lisp expression for hints
}

fn intent_example() -> Result<(), Box<dyn std::error::Error>> {
    let intent_id = EntityId::from_slice(&[2u8; 32]); // Placeholder ID
    let domain_id = DomainId::from_slice(&[1u8; 32]); // Placeholder DomainId
    // Optionally, an ExprId for a Lisp program defining swap logic
    let swap_logic_expr_id: Option<ExprId> = None; 

    let atomic_swap_intent = Intent {
        id: intent_id, // In reality, derived from SSZ hash of content
        name: "Atomic Swap BTC for ETH".into(),
        domain_id,
        priority: 10,
        inputs: vec![
            ResourceFlow::new_escrow_input(alice_btc_resource_id(), "BTC", alice_address(), 1.0),
            ResourceFlow::new_escrow_input(bob_eth_resource_id(), "ETH", bob_address(), 20.0),
        ],
        outputs: vec![
            ResourceFlow::new_escrow_output(alice_btc_resource_id(), "BTC", bob_address(), 1.0),
            ResourceFlow::new_escrow_output(bob_eth_resource_id(), "ETH", alice_address(), 20.0),
        ],
        expression: swap_logic_expr_id, // This Lisp expr would define conditions and exact transformations
        timestamp: current_timestamp(),
        hint: None,
    };

    // The framework's effects engine processes the intent to build a TEG
    // let teg = effects_engine.process_intent(atomic_swap_intent)?;
    
    println!("Created atomic swap intent: {}", atomic_swap_intent.name);
    // println!("TEG generated with {} nodes and {} edges", teg.nodes.len(), teg.edges.len());
    
    Ok(())
}
```
Constraints and specific effects are no longer direct fields in `Intent`. Instead, the `inputs`, `outputs`, and the optional `expression` (a Layer 1 Lisp program) guide the effects engine in constructing and validating the necessary effects and their causal links within a TEG.

The runtime's flow synthesis:
1. Analyzes resource requirements
2. Searches for valid effect sequences
3. Verifies linear safety and conservation
4. Optimizes based on hints
5. Produces a Temporal Effect Graph (TEG)

## Layer 1 Lisp and Layer 0 Register Machine

Layer 1 Causality Lisp provides the primary way to define custom logic for resource transformations, effect implementations, and intent expressions. It features **11 core primitives** that compile down to the **9 instructions** of the Layer 0 typed register machine. This architecture ensures predictable performance, facilitates formal verification, and allows for efficient compilation to ZK circuits.

**Layer 1: Causality Lisp Core Primitives (11 total):**
1.  `lambda (params...) body...`: Defines a function.
2.  `app func args...`: Applies a function.
3.  `let (bindings...) body...`: Local bindings.
4.  `if cond then-expr else-expr`: Conditional execution.
5.  `quote datum`: Returns datum literally.
6.  `cons head tail`: Constructs a pair (list cell).
7.  `car pair`: Gets the head of a pair.
8.  `cdr pair`: Gets the tail of a pair.
9.  `nil? obj`: Checks if an object is nil.
10. `eq? obj1 obj2`: Checks for equality of basic Lisp values (e.g., symbols, numbers).
11. `primitive-op "op-name" args...`: Accesses built-in operations. These include:
    *   Arithmetic: `+`, `-`, `*`, `/`, `=`, `<`, `>` etc.
    *   Type predicates: `integer?`, `symbol?`, `pair?` etc.
    *   Layer 0 resource/object interactions: `alloc-resource`, `consume-resource`, `read-field`, `write-field`, `perform-effect`, `check-constraint`. These map more directly to Layer 0 instructions.

**Example: Simple Lisp for an Effect's Expression**

This Lisp code could be the `expression` associated with an `Effect`. It might define how to calculate an output value based on an input resource's field and a parameter.

```lisp
;; Assume 'input-resource-id' and 'multiplication-factor' are available 
;; in the Lisp environment when this expression is evaluated.
;; (e.g., set up by the effects engine based on Effect.inputs)

(define (calculate-new-output input-resource-id multiplication-factor)
  (let ((current-amount (primitive-op "read-field" input-resource-id "amount")))
    (primitive-op "multiply" current-amount multiplication-factor)))

;; This Lisp code is compiled by the 'causality-lisp-compiler'
;; into a sequence of Layer 0 register machine instructions.
;; The 'causality-vm' then executes these instructions.
```

**Layer 0: The 9-Instruction Typed Register Machine**

The Layer 0 machine provides a minimal, deterministic execution environment. Its 9 instructions are:
1.  **Load `rd, rs, offset`**: Loads a value from memory (identified by `rs` + `offset`) into register `rd`.
2.  **Store `rs1, offset, rs2`**: Stores the value from register `rs2` into memory (identified by `rs1` + `offset`).
3.  **Move `rd, rs`**: Moves the value from register `rs` to register `rd`.
4.  **Call `target_reg`**: Calls a function whose address is in `target_reg`. Pushes return address.
5.  **Return**: Returns from a function call. Pops return address.
6.  **Alloc `rd, type_id_reg, data_reg`**: Allocates a new resource or object. `type_id_reg` holds its type, `data_reg` its initial data. Result (ID or pointer) in `rd`.
7.  **Consume `rd, resource_id_reg`**: Consumes a linear resource. `resource_id_reg` identifies the resource. Result (e.g., extracted value) in `rd`.
8.  **Perform `effect_id_reg, inputs_reg`**: Initiates a Layer 2 effect. `effect_id_reg` identifies the effect, `inputs_reg` points to its inputs.
9.  **Check `constraint_id_reg, inputs_reg`**: Verifies a constraint. `constraint_id_reg` identifies the constraint, `inputs_reg` points to relevant data.

```rust
// Conceptual Rust representation of Layer 0 instructions
// (Actual enum in causality-vm crate might differ slightly in field names or structure)
pub enum Register { /* ... */ }

pub enum Instruction {
    Load { dest_reg: Register, src_addr_reg: Register, offset: i32 },
    Store { base_addr_reg: Register, offset: i32, src_val_reg: Register },
    Move { dest_reg: Register, src_reg: Register },
    Call { target_addr_reg: Register },
    Return,
    Alloc { dest_reg: Register, type_id_reg: Register, data_reg: Register },
    Consume { dest_reg: Register, resource_id_reg: Register },
    Perform { effect_id_reg: Register, inputs_ptr_reg: Register },
    Check { constraint_id_reg: Register, inputs_ptr_reg: Register },
}

fn register_machine_example_conceptual() -> Result<(), Box<dyn std::error::Error>> {
    // Lisp: (primitive-op "add" (primitive-op "read-field" R_INPUT "amount") R_FACTOR)
    // This is highly conceptual as direct compilation is complex.
    // A Lisp 'primitive-op' for arithmetic or resource interaction translates
    // into a sequence of these 9 core Layer 0 instructions by the Lisp compiler.
    
    // For instance, a 'read-field' might involve:
    // 1. Move resource_id to a specific register for a runtime call.
    // 2. Move field_name_id to another register.
    // 3. Call a runtime function (implemented in Layer 0 instructions) that handles field lookup.
    // 4. The result (field value) would be returned in a designated register.

    // An arithmetic operation like '+' would similarly call a runtime function or be inlined
    // as a sequence of more fundamental bit/byte manipulations if not directly supported
    // (though basic arithmetic is usually a primitive or a very short sequence).
    
    println!("Lisp's 11 primitives compile to sequences of the 9 Layer 0 instructions.");
    Ok(())
}
```
The Lisp compiler (`causality-lisp-compiler`) is responsible for translating Layer 1 Lisp programs, including the 11 core primitives and `primitive-op` calls, into efficient sequences of these 9 Layer 0 instructions. The `causality-vm` then executes these instruction sequences.

## Testing Linear Programs

Testing linear resource programs requires special attention to resource consumption. The framework provides testing utilities that help manage linear resources in tests while ensuring that linearity constraints are properly verified.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use causality_testing::*;
    
    #[test]
    fn test_linear_transfer() -> Result<(), Box<dyn std::error::Error>> {
        // Create test resources with linear tracking
        let test_env = LinearTestEnvironment::new();
        
        let alice_tokens = test_env.create_resource(
            TokenData { amount: 100 },
            row_type!{transfer: true, balance: true}
        );
        
        let bob_account = test_env.create_account(bob_address());
        
        // Execute transfer - consumes alice_tokens
        let receipt = execute_transfer(
            alice_tokens,  // Moved, not borrowed
            bob_account,
            50
        )?;
        
        // Verify linearity - alice_tokens is consumed
        assert!(test_env.is_consumed(&alice_tokens));
        
        // Verify conservation
        assert_eq!(
            receipt.transferred_amount, 
            50
        );
        
        // This would fail - can't use consumed resource
        // let invalid = execute_transfer(alice_tokens, carol, 25);
        
        Ok(())
    }
    
    #[test] 
    fn test_effect_handler_purity() {
        let effect = create_test_effect();
        let handler = create_test_handler();
        
        // Handlers are pure - same input, same output
        let result1 = handler.transform(effect.clone());
        let result2 = handler.transform(effect.clone());
        
        assert_eq!(result1, result2);
    }
}
```

Testing best practices:
- Use `LinearTestEnvironment` for resource tracking
- Verify conservation laws in every test
- Test handler purity with repeated applications
- Use property-based testing for type system invariants

## Building Complete Applications

Building complete applications involves combining all three layers effectively. Start with clear resource definitions, design your effects with appropriate constraints, and let the framework handle the compilation and optimization.

```rust
// Example: Decentralized Exchange Application

// Layer 1: Define resource types with capabilities
type TokenWithDEX = Token<Row!{
    transfer: TransferCap,
    swap: SwapCap,
    liquidity: LiquidityCap,
}>;

// Layer 2: Define effects for DEX operations
define_effect!(
    AddLiquidity,
    params: {
        token_a: TokenWithDEX,
        token_b: TokenWithDEX,
        lp_recipient: Address,
    },
    pre: constraint!(
        token_a.amount > 0 && 
        token_b.amount > 0 &&
        same_pool(token_a, token_b)
    ),
    post: constraint!(
        lp_tokens_minted &&
        pool_invariant_maintained
    ),
    hints: [
        Hint::BatchWith(SamePool),
        Hint::MinimizeSlippage,
    ]
);

// Define handlers for optimization
let dex_handler_pipeline = compose_handlers(
    slippage_protection_handler,
    batch_swap_handler,
    route_optimization_handler,
);

// Layer 0: Everything compiles to register machine
// The framework handles this automatically
```

## Performance Considerations

The three-layer architecture enables various optimization opportunities:

**Compile-Time Optimizations (Layer 1)**:
- Row type operations have zero runtime cost
- Static register allocation for known values
- Dead code elimination for unused capabilities

**Effect Optimization (Layer 2)**:
- Handler composition enables modular optimization
- TEG analysis identifies parallelizable effects
- Hints guide execution strategy selection

**Register Machine (Layer 0)**:
- Minimal instruction set enables efficient execution
- Static allocation reduces memory pressure
- Direct compilation to ZK circuits

## Next Steps

After mastering the basics:

1. **Explore Advanced Types**: Fractional capabilities, private resources, dependent types
2. **Build Custom Handlers**: Create domain-specific effect transformations
3. **Optimize for ZK**: Understand circuit generation and optimization
4. **Contribute**: The framework is open for contributions in all three layers

The Causality framework provides a solid foundation for building verifiable, distributed applications with strong safety guarantees. By embracing linearity and the three-layer architecture, you can build systems that are both powerful and provably correct.