<!-- Unified model for programs -->
<!-- Original file: docs/src/unified_program_model.md -->

# Unified Program Model

This document outlines the unified program model in Causality, which consolidates logic programs and account programs into a single coherent concept. This unification provides a more flexible and consistent programming model while simplifying the developer experience.

## Core Concepts

### The Program Model

A **Program** in the unified model represents a modular unit of logic and/or state with well-defined capabilities rather than being distinguished by type. Key characteristics include:

1. **Identity**: Each program has a unique identifier
2. **Capabilities**: Programs explicitly declare what they can do
3. **State**: Programs may maintain state (optional)
4. **Logic**: Programs define execution logic
5. **Schema**: Programs describe their interface
6. **Security**: Programs specify their security model

This unification eliminates duplicate functionality, provides consistent developer experience, simplifies deployment, and enables more flexible program design.

### Structure

```rust
/// A unified program that can contain logic, state, or both
struct Program {
    /// Unique identifier for the program
    id: ProgramId,
    
    /// Type of program (for compatibility and optimization)
    program_type: ProgramType,
    
    /// Capabilities this program has
    capabilities: ProgramCapabilities,
    
    /// State management (optional)
    state: Option<ProgramState>,
    
    /// Program logic implementation
    logic: ProgramLogic,
    
    /// Interface schema
    schema: ProgramSchema,
    
    /// Security settings
    security: ProgramSecurity,
    
    /// Metadata for the program
    metadata: Metadata,
}

/// Program capabilities
struct ProgramCapabilities {
    /// Whether the program can manage state
    has_state: bool,
    
    /// Whether the program can be invoked by other programs
    invocable: bool,
    
    /// Whether the program can invoke other programs
    can_invoke_others: bool,
    
    /// Whether the program can manage resources
    can_manage_resources: bool,
    
    /// Whether the program can observe external facts
    can_observe_facts: bool,
    
    /// Whether the program can be upgraded
    upgradable: bool,
}

/// Program state configuration
struct ProgramState {
    /// Storage strategy for program state
    storage_strategy: StorageStrategy,
    
    /// State schema definition
    schema: StateSchema,
    
    /// Initial state (if any)
    initial_state: Option<State>,
}

/// Program logic implementation
enum ProgramLogic {
    /// Rust implementation
    Rust(RustImplementation),
    
    /// WebAssembly implementation
    Wasm(WasmImplementation),
    
    /// Temporal Effect Language implementation
    TEL(TELImplementation),
    
    /// External implementation (e.g., Solidity contract)
    External(ExternalImplementation),
}
```

## Program Lifecycle

Programs flow through a consistent lifecycle:

1. **Creation**: Program is defined with necessary capabilities
2. **Deployment**: Program is installed in the system
3. **Initialization**: Program state is initialized (if applicable)
4. **Execution**: Program is invoked for computation
5. **Upgrades**: Program may be upgraded (if upgradable)
6. **Retirement**: Program is eventually retired

Each phase has specific validation requirements and state transitions, ensuring consistency and correctness throughout the process.

## Integration with Resource System

Programs interact with the resource system through operations:

1. Programs can create, update, and consume resources
2. Programs can establish resource relationships
3. Programs can validate resource operations
4. Programs can implement resource logic

## Integration with Operation Model

Programs are both the targets and sources of operations:

1. Operations can deploy, update, and invoke programs
2. Programs can generate operations
3. Programs can validate operations
4. Programs can compose multiple operations

## Usage Examples

### Creating a Stateful Program

```rust
// Create a program with state
let token_program = Program::new("token_program")
    .with_capabilities(ProgramCapabilities {
        has_state: true,
        invocable: true,
        can_invoke_others: false,
        can_manage_resources: true,
        can_observe_facts: false,
        upgradable: true,
    })
    .with_state(ProgramState {
        storage_strategy: StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
        schema: StateSchema::new()
            .with_field("total_supply", FieldType::U64)
            .with_field("owner", FieldType::Address),
        initial_state: Some(
            State::new()
                .with_value("total_supply", 1_000_000u64)
                .with_value("owner", deployer_address)
        ),
    })
    .with_logic(ProgramLogic::Rust(
        RustImplementation::new()
            .with_function("mint", mint_implementation)
            .with_function("transfer", transfer_implementation)
            .with_function("burn", burn_implementation)
    ))
    .with_schema(
        ProgramSchema::new()
            .with_method("mint", 
                MethodSchema::new()
                    .with_input("to", FieldType::Address)
                    .with_input("amount", FieldType::U64)
                    .with_output("success", FieldType::Bool)
            )
            .with_method("transfer", 
                MethodSchema::new()
                    .with_input("from", FieldType::Address)
                    .with_input("to", FieldType::Address)
                    .with_input("amount", FieldType::U64)
                    .with_output("success", FieldType::Bool)
            )
            .with_method("burn", 
                MethodSchema::new()
                    .with_input("from", FieldType::Address)
                    .with_input("amount", FieldType::U64)
                    .with_output("success", FieldType::Bool)
            )
    )
    .with_security(
        ProgramSecurity::new()
            .with_upgrade_authority(deployer_address)
            .with_method_access("mint", AccessPolicy::OnlyOwner)
    );

// Deploy the program
let deploy_operation = Operation::new(OperationType::DeployProgram)
    .with_output(token_program.clone())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(deployer.clone()));

let result = execute_operation(deploy_operation, &context).await?;
let program_id = result.outputs[0].id();
```

### Invoking a Program

```rust
// Invoke the token program to mint tokens
let invoke_operation = Operation::new(OperationType::InvokeProgram)
    .with_input(Program::reference(program_id))
    .with_parameter("method", "mint")
    .with_parameter("args", json!({
        "to": recipient_address,
        "amount": 1000u64
    }))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(token_owner.clone()));

// Validate and execute
let validation_result = validator.validate(&invoke_operation)?;
if validation_result.is_valid {
    let result = execute_operation(invoke_operation, &context).await?;
    let success = result.return_value::<bool>()?;
    // Process result
}
```

### Stateless Logic Program

```rust
// Create a pure logic program without state
let math_program = Program::new("math_program")
    .with_capabilities(ProgramCapabilities {
        has_state: false,
        invocable: true,
        can_invoke_others: false,
        can_manage_resources: false,
        can_observe_facts: false,
        upgradable: true,
    })
    .with_logic(ProgramLogic::Rust(
        RustImplementation::new()
            .with_function("add", |args: json::Value| -> Result<json::Value> {
                let a = args["a"].as_u64().ok_or(Error::InvalidArgument("a"))?;
                let b = args["b"].as_u64().ok_or(Error::InvalidArgument("b"))?;
                Ok(json::to_value(a + b)?)
            })
            .with_function("multiply", |args: json::Value| -> Result<json::Value> {
                let a = args["a"].as_u64().ok_or(Error::InvalidArgument("a"))?;
                let b = args["b"].as_u64().ok_or(Error::InvalidArgument("b"))?;
                Ok(json::to_value(a * b)?)
            })
    ))
    .with_schema(
        ProgramSchema::new()
            .with_method("add", 
                MethodSchema::new()
                    .with_input("a", FieldType::U64)
                    .with_input("b", FieldType::U64)
                    .with_output("result", FieldType::U64)
            )
            .with_method("multiply", 
                MethodSchema::new()
                    .with_input("a", FieldType::U64)
                    .with_input("b", FieldType::U64)
                    .with_output("result", FieldType::U64)
            )
    );

// Deploy the program
let deploy_operation = Operation::new(OperationType::DeployProgram)
    .with_output(math_program.clone())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(deployer.clone()));

let result = execute_operation(deploy_operation, &context).await?;
```

### Program with TEL Logic

```rust
// Create a program using Temporal Effect Language
let tel_program = Program::new("tel_program")
    .with_capabilities(ProgramCapabilities {
        has_state: true,
        invocable: true,
        can_invoke_others: true,
        can_manage_resources: true,
        can_observe_facts: true,
        upgradable: true,
    })
    .with_state(ProgramState {
        storage_strategy: StorageStrategy::FullyOnChain { 
            visibility: StateVisibility::Public 
        },
        schema: StateSchema::new()
            .with_field("counter", FieldType::U64),
        initial_state: Some(
            State::new()
                .with_value("counter", 0u64)
        ),
    })
    .with_logic(ProgramLogic::TEL(
        TELImplementation::new()
            .with_script(r#"
                define increment(amount: u64) -> u64 {
                    let current = get_state("counter") as u64;
                    let new_value = current + amount;
                    set_state("counter", new_value);
                    return new_value;
                }
                
                define create_resource(name: string, amount: u64) -> ResourceId {
                    let resource = create_fungible_resource(name, amount);
                    return resource.id;
                }
            "#)
    ))
    .with_schema(
        ProgramSchema::new()
            .with_method("increment", 
                MethodSchema::new()
                    .with_input("amount", FieldType::U64)
                    .with_output("new_value", FieldType::U64)
            )
            .with_method("create_resource", 
                MethodSchema::new()
                    .with_input("name", FieldType::String)
                    .with_input("amount", FieldType::U64)
                    .with_output("resource_id", FieldType::ResourceId)
            )
    );

// Deploy the program
let deploy_operation = Operation::new(OperationType::DeployProgram)
    .with_output(tel_program.clone())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(deployer.clone()));

let result = execute_operation(deploy_operation, &context).await?;
```

### Program Composition

```rust
// Create a program that composes functionality from other programs
let composed_program = Program::new("composed_program")
    .with_capabilities(ProgramCapabilities {
        has_state: false,
        invocable: true,
        can_invoke_others: true,
        can_manage_resources: false,
        can_observe_facts: false,
        upgradable: true,
    })
    .with_logic(ProgramLogic::Rust(
        RustImplementation::new()
            .with_function("create_and_transfer", |ctx, args: json::Value| -> Result<json::Value> {
                // First create a resource using token_program
                let create_result = ctx.invoke_program(
                    "token_program",
                    "mint",
                    json!({
                        "to": ctx.invoker(),
                        "amount": args["amount"]
                    })
                )?;
                
                // Then transfer it using transfer_program
                let transfer_result = ctx.invoke_program(
                    "transfer_program",
                    "transfer",
                    json!({
                        "resource_id": create_result["resource_id"],
                        "to": args["recipient"]
                    })
                )?;
                
                Ok(json!({
                    "created": create_result["success"],
                    "transferred": transfer_result["success"]
                }))
            })
    ))
    .with_schema(
        ProgramSchema::new()
            .with_method("create_and_transfer", 
                MethodSchema::new()
                    .with_input("amount", FieldType::U64)
                    .with_input("recipient", FieldType::Address)
                    .with_output("result", FieldType::Object)
            )
    );
```

## Program Types

While the unified model treats all programs equally based on capabilities, for compatibility and optimization, programs can still have type designations:

- **LogicProgram**: Pure logic, no state (former Logic Programs)
- **StatefulProgram**: Logic with state (former Account Programs)
- **ResourceLogicProgram**: Specialized for implementing resource logic
- **TELProgram**: Temporal Effect Language programs
- **ExternalProgram**: References to programs on external domains
- **CompositeProgram**: Programs that primarily compose other programs

## Best Practices

1. **Focus on Capabilities**: Design programs based on their capabilities rather than rigid types.

2. **Minimize State**: Only use state when necessary, prefer immutable patterns.

3. **Define Clear Interfaces**: Always provide a complete schema for your program's interface.

4. **Isolate Concerns**: Each program should have a focused purpose.

5. **Compose Rather Than Duplicate**: Use program composition to build complex functionality.

6. **Secure Upgrade Paths**: For upgradable programs, define clear upgrade governance.

7. **Document Capabilities**: Clearly document what your program can and cannot do.

8. **Test Edge Cases**: Test programs with boundary conditions and invalid inputs.

9. **Consider Resource Logic**: Use the resource logic interface for resource-specific behavior.

10. **Define Security Policies**: Set appropriate access controls for sensitive operations.

## Implementation Status

The unified program model is fully implemented in the Causality system:

- ✅ Core `Program` data structure
- ✅ Deployment and invocation operations
- ✅ State management
- ✅ Program capabilities system
- ✅ Program composition
- ✅ Security model

## Future Enhancements

1. **Enhanced Composition**: More powerful program composition patterns
2. **Capability Constraints**: More fine-grained capability control
3. **Formal Verification**: Tools for verifying program behavior
4. **Optimized State Access**: More efficient state access patterns
5. **Improved Developer Tools**: Better tooling for program development 