# Getting Started and Tutorials

The Causality framework provides a comprehensive foundation for building resource-based applications through its type system, serialization infrastructure, and development tools. This guide introduces the essential concepts and practical techniques needed to begin productive development with the framework while understanding the architectural principles that guide its design.

The framework emphasizes type safety, deterministic behavior, and content addressing as fundamental principles that enable reliable resource management applications. Understanding these principles and their practical implications forms the foundation for effective use of the framework's capabilities.

## Development Environment Setup

Setting up a productive development environment for the Causality framework involves configuring the necessary tools and dependencies while ensuring compatibility with the framework's build system and testing infrastructure. The framework provides comprehensive tooling support through Nix-based environment management that eliminates common setup problems.

The recommended approach uses the provided Nix flake configuration that includes all necessary dependencies, tools, and environment settings for both Rust and OCaml development. This approach ensures consistency across different development machines while providing immediate access to all framework capabilities.

```bash
# Clone the repository
git clone <repository-url>
cd causality

# Enter the Nix development environment
nix develop

# Build the project
cargo build

# Run tests to verify installation
cargo test
```

Environment verification involves building all framework components and running the comprehensive test suite to ensure that the development environment is properly configured. The framework includes several test scripts that validate different aspects of the system while providing confidence that the environment supports productive development.

Alternative installation approaches are available for developers who prefer manual dependency management, though these require careful attention to tool versions and dependency compatibility. The framework specifies exact dependency versions to ensure consistent behavior across different environments.

## Core Type System Understanding

The Causality framework centers around a collection of core types that represent the fundamental concepts of resource management, transformation requests, and execution effects. Understanding these types and their relationships provides the foundation for building sophisticated applications while maintaining the safety and correctness properties that define the framework.

Resource types serve as the fundamental building blocks for representing quantifiable assets or capabilities within applications. These types capture the essential properties needed for resource tracking while supporting the content addressing and serialization requirements that enable reliable resource management.

```rust
use causality_types::core::resource::*;
use causality_types::primitive::ids::*;
use causality_types::core::timestamp::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Resource
    let resource = Resource {
        id: EntityId::new([1u8; 32]),
        name: Str::from("my_token"),
        domain_id: DomainId::new([0u8; 32]),
        resource_type: Str::from("token"),
        quantity: 1000,
        timestamp: Timestamp::now(),
    };
    
    println!("Created Resource:");
    println!("  ID: {:?}", resource.id);
    println!("  Name: {}", resource.name);
    println!("  Type: {}", resource.resource_type);
    println!("  Quantity: {}", resource.quantity);
    
    // Test serialization
    let serialized = resource.as_ssz_bytes();
    println!("  Serialized size: {} bytes", serialized.len());
    
    // Test deserialization
    let deserialized = Resource::from_ssz_bytes(&serialized)?;
    assert_eq!(resource, deserialized);
    println!("  Serialization roundtrip: OK");
    
    Ok(())
}
```

Resource creation involves specifying identification information, descriptive metadata, domain association, type classification, quantity tracking, and temporal information. This comprehensive structure enables sophisticated resource management while maintaining the simplicity needed for practical applications.

Serialization testing demonstrates the framework's commitment to deterministic data representation through SSZ encoding that enables content addressing and reliable data exchange. Every framework type implements consistent serialization that supports both storage and transmission use cases.

## Intent Processing Fundamentals

Intent types represent requests for resource transformations that specify desired inputs, outputs, and processing requirements. Understanding Intent construction and processing provides the foundation for building applications that can express complex resource management requirements while taking advantage of the framework's optimization and execution capabilities.

Intent construction involves specifying the transformation requirements through resource flows that describe the inputs and outputs of the desired operation. The Intent structure captures both the essential transformation logic and additional metadata that enables sophisticated optimization and routing decisions.

```rust
use causality_types::core::intent::*;
use causality_types::core::resource::ResourceFlow;

fn create_intent_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create resource flows
    let input_flow = ResourceFlow {
        resource_type: Str::from("token"),
        quantity: 100,
        domain_id: DomainId::new([0u8; 32]),
    };
    
    let output_flow = ResourceFlow {
        resource_type: Str::from("token"),
        quantity: 100,
        domain_id: DomainId::new([1u8; 32]),
    };
    
    // Create an Intent
    let intent = Intent {
        id: EntityId::new([2u8; 32]),
        name: Str::from("transfer_tokens"),
        domain_id: DomainId::new([0u8; 32]),
        priority: 1,
        inputs: vec![input_flow],
        outputs: vec![output_flow],
        expression: None,
        timestamp: Timestamp::now(),
        optimization_hint: None,
        target_typed_domain: None,
        process_dataflow_hint: None,
    };
    
    println!("Created Intent:");
    println!("  ID: {:?}", intent.id);
    println!("  Name: {}", intent.name);
    println!("  Priority: {}", intent.priority);
    println!("  Inputs: {} flows", intent.inputs.len());
    println!("  Outputs: {} flows", intent.outputs.len());
    
    // Test serialization
    let serialized = intent.as_ssz_bytes();
    let deserialized = Intent::from_ssz_bytes(&serialized)?;
    assert_eq!(intent, deserialized);
    println!("  Serialization roundtrip: OK");
    
    Ok(())
}
```

Resource flow specification enables precise description of the resource movements involved in transformations while supporting validation of conservation properties and domain constraints. The ResourceFlow type captures the essential information needed for resource tracking while enabling composition of complex transformation patterns.

Priority handling enables sophisticated scheduling of Intent processing based on business requirements and resource availability. The priority system supports both simple numeric priorities and complex priority functions that consider multiple factors in scheduling decisions.

Optimization hints provide applications with the ability to influence execution strategies while maintaining the framework's ability to make optimal decisions based on current system state and resource availability. These hints enable performance optimization without compromising the deterministic properties of the framework.

## Effect Execution Patterns

Effect types represent the actual execution of resource transformations, capturing both the transformation logic and the resource flows involved in the operation. Understanding Effect construction and execution provides insight into how the framework translates high-level Intent specifications into concrete resource transformations.

Effect creation involves specifying the transformation logic, resource flows, execution context, and metadata needed for proper execution and verification. The Effect structure captures both the immediate transformation requirements and the broader context needed for integration with the framework's execution infrastructure.

```rust
use causality_types::core::effect::*;
use causality_types::core::typed_domain::*;

fn create_effect_example() -> Result<(), Box<dyn std::error::Error>> {
    let source_domain = TypedDomain::VerifiableDomain {
        domain_id: DomainId::new([0u8; 32]),
        zk_constraints: true,
        deterministic_only: true,
    };
    
    let target_domain = TypedDomain::ServiceDomain {
        domain_id: DomainId::new([1u8; 32]),
        external_apis: vec![Str::from("payment_api")],
        non_deterministic_allowed: false,
    };
    
    let effect = Effect {
        id: EntityId::new([3u8; 32]),
        name: Str::from("token_transfer"),
        domain_id: DomainId::new([0u8; 32]),
        effect_type: Str::from("transfer"),
        inputs: vec![],
        outputs: vec![],
        expression: None,
        timestamp: Timestamp::now(),
        resources: vec![],
        nullifiers: vec![],
        scoped_by: HandlerId::new([4u8; 32]),
        intent_id: None,
        source_typed_domain: source_domain,
        target_typed_domain: target_domain,
        cost_model: None,
        resource_usage_estimate: None,
        originating_dataflow_instance: None,
    };
    
    println!("Created Effect:");
    println!("  ID: {:?}", effect.id);
    println!("  Name: {}", effect.name);
    println!("  Type: {}", effect.effect_type);
    println!("  Source domain: {:?}", effect.source_typed_domain);
    println!("  Target domain: {:?}", effect.target_typed_domain);
    
    Ok(())
}
```

Domain specification enables sophisticated routing of Effects to appropriate execution environments based on the computational requirements and constraints of the transformation. The typed domain system provides both isolation and optimization opportunities while maintaining the correctness guarantees needed for reliable execution.

Nullifier generation enables proper tracking of consumed resources while preventing double-spending and other resource management errors. The nullifier system provides cryptographic proof of resource consumption while maintaining the privacy properties needed for sophisticated applications.

Cost modeling enables sophisticated resource allocation and optimization decisions based on the computational and resource costs of different operations. The cost model system supports both simple cost estimates and complex cost functions that consider multiple factors in optimization decisions.

## Expression System Integration

The framework includes a comprehensive expression system that enables sophisticated transformation logic while maintaining the deterministic properties required for reliable execution. Understanding expression construction and evaluation provides access to the framework's most powerful capabilities for building complex applications.

Expression construction involves building abstract syntax trees that represent transformation logic using the framework's expression types. The expression system provides both low-level primitives for maximum flexibility and high-level abstractions for common patterns.

```rust
use causality_types::expr::*;

fn expression_examples() -> Result<(), Box<dyn std::error::Error>> {
    // Simple value expression
    let number_expr = Expr::Const(ValueExpr::Int(42));
    
    // Variable reference
    let var_expr = Expr::Var(Str::from("balance"));
    
    // Lambda expression
    let lambda_expr = Expr::Lambda(
        vec![Str::from("x")],
        Box::new(Expr::Var(Str::from("x"))),
    );
    
    // Combinator expression
    let add_expr = Expr::Combinator(AtomicCombinator::Add);
    
    // Function application
    let app_expr = Expr::Apply(
        Box::new(add_expr),
        vec![
            Expr::Const(ValueExpr::Int(10)),
            Expr::Const(ValueExpr::Int(20)),
        ],
    );
    
    println!("Created expressions:");
    println!("  Number: {:?}", number_expr);
    println!("  Variable: {:?}", var_expr);
    println!("  Lambda: {:?}", lambda_expr);
    println!("  Application: {:?}", app_expr);
    
    Ok(())
}
```

Expression evaluation provides deterministic execution of transformation logic while maintaining proper isolation and resource accounting. The evaluation system supports both immediate evaluation for simple operations and lazy evaluation for complex computations that may not be needed immediately.

Combinator integration enables powerful composition patterns that leverage the mathematical properties of combinators to ensure correctness and enable optimization. The combinator system provides both primitive combinators and higher-level abstractions that simplify common patterns.

## Testing and Development Workflow

The framework includes comprehensive testing infrastructure that enables thorough validation of applications while maintaining the deterministic properties required for reliable testing. Understanding the testing approach and available utilities enables confident development of complex applications.

Testing workflow involves both unit testing of individual components and integration testing of complete workflows. The framework provides testing utilities that simplify test data generation while ensuring that tests exercise realistic scenarios and edge cases.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use causality_toolkit::testing::fixtures::*;
    
    #[test]
    fn test_resource_creation() -> Result<(), Box<dyn std::error::Error>> {
        // Create test resource
        let resource = create_test_resource(
            "token",
            1000,
            DomainId::new([0u8; 32]),
        );
        
        // Verify properties
        assert_eq!(resource.resource_type, Str::from("token"));
        assert_eq!(resource.quantity, 1000);
        
        // Test serialization
        let serialized = resource.as_ssz_bytes();
        let deserialized = Resource::from_ssz_bytes(&serialized)?;
        assert_eq!(resource, deserialized);
        
        Ok(())
    }
    
    #[test]
    fn test_intent_processing() -> Result<(), Box<dyn std::error::Error>> {
        // Create test intent
        let intent = create_test_intent(
            "test_transfer",
            DomainId::new([0u8; 32]),
            1,
        );
        
        // Verify intent properties
        assert_eq!(intent.name, Str::from("test_transfer"));
        assert_eq!(intent.priority, 1);
        
        // Test serialization roundtrip
        let serialized = intent.as_ssz_bytes();
        let deserialized = Intent::from_ssz_bytes(&serialized)?;
        assert_eq!(intent, deserialized);
        
        Ok(())
    }
}
```

Test data generation utilities provide convenient methods for creating realistic test scenarios while ensuring that test data exercises the full range of framework capabilities. These utilities handle the complexity of test setup while providing natural interfaces for test development.

Assertion utilities enable comprehensive validation of framework behavior while providing clear feedback about test failures. The testing infrastructure includes both basic assertions and sophisticated validation functions that check complex properties and invariants.

## Building and Deployment Preparation

The framework includes comprehensive build system support that enables efficient development workflows while preparing for eventual production deployment. Understanding the build system and its capabilities enables optimization of development productivity and preparation for production use.

Build system organization through Cargo workspaces enables efficient compilation of the various framework components while maintaining clear dependency relationships. The workspace structure supports both individual component development and comprehensive system building.

```bash
# Build all components
cargo build

# Build specific component
cargo build -p causality-types

# Build with optimizations
cargo build --release

# Run comprehensive tests
cargo test

# Run tests with features
cargo test --features testing

# Build OCaml components
cd ml_causality
dune build
dune test
```

Development workflow optimization through incremental compilation and caching enables efficient iteration during development while maintaining the correctness guarantees needed for reliable applications. The build system takes advantage of Rust's compilation model to minimize rebuild times.

Testing integration with the build system enables comprehensive validation during development while providing confidence that changes maintain compatibility and correctness. The testing infrastructure includes both automated testing and manual testing tools that support various development scenarios.