// Comprehensive tests for the content-addressed code system
//
// This file contains tests for the entire content-addressed code system,
// including the repository, executor, compatibility checking, and RISC-V metadata.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tempfile::TempDir;

use causality::code::{
    CodeMetadata, CodeMetadataBuilder, CodeRepository, CompatibilityChecker,
    ContentAddressableExecutor, ContentHash, ContentHasher, ExecutionContext, HashAlgorithm,
    RiscVCompatibilityChecker, RiscVMetadata, Value,
};
use causality::effect::EffectType;
use causality::error::Result;
use causality::resource::ResourceManager;

// Test objects for storing in the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCode {
    name: String,
    version: String,
    code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestEffect {
    name: String,
    effect_type: String,
    parameters: HashMap<String, Value>,
}

#[test]
fn test_repository_store_and_load() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);

    // Create a test code object
    let test_code = TestCode {
        name: "test_function".to_string(),
        version: "1.0.0".to_string(),
        code: "function test() { return 42; }".to_string(),
    };

    // Create metadata
    let metadata = CodeMetadataBuilder::new()
        .with_name(Some("test_function"))
        .with_description(Some("A test function"))
        .with_format("json")
        .with_version("1.0.0")
        .build();

    // Serialize and store the code
    let serialized = bincode::serialize(&test_code)?;
    let hash = repository.store_with_metadata(serialized, metadata)?;

    // Load the code back from the repository
    let loaded_data = repository.load_by_hash(&hash)?;
    let loaded_code: TestCode = bincode::deserialize(&loaded_data.data)?;

    // Verify the loaded code matches the original
    assert_eq!(loaded_code.name, test_code.name);
    assert_eq!(loaded_code.version, test_code.version);
    assert_eq!(loaded_code.code, test_code.code);

    // Verify metadata
    assert_eq!(loaded_data.metadata.name, Some("test_function".to_string()));
    assert_eq!(
        loaded_data.metadata.description,
        Some("A test function".to_string())
    );
    assert_eq!(loaded_data.metadata.format, "json");
    assert_eq!(loaded_data.metadata.version, "1.0.0");

    Ok(())
}

#[test]
fn test_name_registry() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);

    // Create test code objects
    let test_code_v1 = TestCode {
        name: "test_function".to_string(),
        version: "1.0.0".to_string(),
        code: "function test() { return 42; }".to_string(),
    };

    let test_code_v2 = TestCode {
        name: "test_function".to_string(),
        version: "2.0.0".to_string(),
        code: "function test() { return 84; }".to_string(),
    };

    // Create metadata for v1
    let metadata_v1 = CodeMetadataBuilder::new()
        .with_name(Some("test_function"))
        .with_format("json")
        .with_version("1.0.0")
        .build();

    // Create metadata for v2
    let metadata_v2 = CodeMetadataBuilder::new()
        .with_name(Some("test_function"))
        .with_format("json")
        .with_version("2.0.0")
        .build();

    // Store v1
    let serialized_v1 = bincode::serialize(&test_code_v1)?;
    let hash_v1 = repository.store_with_metadata(serialized_v1, metadata_v1)?;

    // Store v2
    let serialized_v2 = bincode::serialize(&test_code_v2)?;
    let hash_v2 = repository.store_with_metadata(serialized_v2, metadata_v2)?;

    // Register names
    repository.register_name("test_function", &hash_v1)?;
    repository.register_name("test_function_v2", &hash_v2)?;

    // Look up by name
    let resolved_hash_v1 = repository.resolve_name("test_function")?;
    let resolved_hash_v2 = repository.resolve_name("test_function_v2")?;

    // Verify resolved hashes
    assert_eq!(resolved_hash_v1, hash_v1);
    assert_eq!(resolved_hash_v2, hash_v2);

    // Update name registry to point to v2
    repository.register_name("test_function", &hash_v2)?;

    // Look up by name again
    let updated_hash = repository.resolve_name("test_function")?;

    // Verify updated hash
    assert_eq!(updated_hash, hash_v2);

    // Test error when resolving unknown name
    let result = repository.resolve_name("unknown_function");
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_version_resolution() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);

    // Create a series of versioned code objects
    let versions = vec!["1.0.0", "1.1.0", "1.2.0", "2.0.0"];
    let mut hashes = HashMap::new();

    for version in &versions {
        let test_code = TestCode {
            name: "versioned_function".to_string(),
            version: version.to_string(),
            code: format!("function test() {{ return '{}'; }}", version),
        };

        let metadata = CodeMetadataBuilder::new()
            .with_name(Some("versioned_function"))
            .with_format("json")
            .with_version(version)
            .build();

        let serialized = bincode::serialize(&test_code)?;
        let hash = repository.store_with_metadata(serialized, metadata)?;

        // Register version-specific name
        let version_name = format!("versioned_function@{}", version);
        repository.register_name(&version_name, &hash)?;

        hashes.insert(version.to_string(), hash);
    }

    // Register the latest version as the default
    repository.register_name("versioned_function", &hashes["2.0.0"])?;

    // Resolve specific versions
    for version in &versions {
        let version_name = format!("versioned_function@{}", version);
        let hash = repository.resolve_name(&version_name)?;
        assert_eq!(hash, hashes[*version]);
    }

    // Resolve latest version
    let latest_hash = repository.resolve_name("versioned_function")?;
    assert_eq!(latest_hash, hashes["2.0.0"]);

    // Update latest version
    repository.register_name("versioned_function", &hashes["1.2.0"])?;
    let updated_latest = repository.resolve_name("versioned_function")?;
    assert_eq!(updated_latest, hashes["1.2.0"]);

    Ok(())
}

#[test]
fn test_dependency_tracking() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);

    // Create dependency code objects
    let dep1 = TestCode {
        name: "dependency1".to_string(),
        version: "1.0.0".to_string(),
        code: "function dep1() { return 'dep1'; }".to_string(),
    };

    let dep2 = TestCode {
        name: "dependency2".to_string(),
        version: "1.0.0".to_string(),
        code: "function dep2() { return 'dep2'; }".to_string(),
    };

    // Create main code object with dependencies
    let main_code = TestCode {
        name: "main_function".to_string(),
        version: "1.0.0".to_string(),
        code: "function main() { return dep1() + dep2(); }".to_string(),
    };

    // Store dependencies
    let dep1_metadata = CodeMetadataBuilder::new()
        .with_name(Some("dependency1"))
        .with_format("json")
        .with_version("1.0.0")
        .build();

    let dep2_metadata = CodeMetadataBuilder::new()
        .with_name(Some("dependency2"))
        .with_format("json")
        .with_version("1.0.0")
        .build();

    let serialized_dep1 = bincode::serialize(&dep1)?;
    let hash_dep1 = repository.store_with_metadata(serialized_dep1, dep1_metadata)?;

    let serialized_dep2 = bincode::serialize(&dep2)?;
    let hash_dep2 = repository.store_with_metadata(serialized_dep2, dep2_metadata)?;

    // Create main code's dependencies map
    let mut dependencies = HashMap::new();
    dependencies.insert("dependency1".to_string(), hash_dep1.to_string());
    dependencies.insert("dependency2".to_string(), hash_dep2.to_string());

    // Store main code with dependencies
    let main_metadata = CodeMetadataBuilder::new()
        .with_name(Some("main_function"))
        .with_format("json")
        .with_version("1.0.0")
        .with_dependencies(Some(dependencies))
        .build();

    let serialized_main = bincode::serialize(&main_code)?;
    let hash_main = repository.store_with_metadata(serialized_main, main_metadata)?;

    // Register names
    repository.register_name("dependency1", &hash_dep1)?;
    repository.register_name("dependency2", &hash_dep2)?;
    repository.register_name("main_function", &hash_main)?;

    // Load main code
    let main_entry = repository.load_by_hash(&hash_main)?;

    // Check dependencies
    let deps = main_entry.metadata.dependencies.unwrap();
    assert_eq!(deps.len(), 2);
    assert_eq!(deps["dependency1"], hash_dep1.to_string());
    assert_eq!(deps["dependency2"], hash_dep2.to_string());

    Ok(())
}

#[test]
fn test_compatibility_checker() -> Result<()> {
    // Set up a compatibility checker
    let mut allowed_effects = HashSet::new();
    allowed_effects.insert(EffectType::Read);
    allowed_effects.insert(EffectType::Write);

    let checker = CompatibilityChecker::default()
        .with_version("0.2.0".to_string())
        .add_supported_format("json".to_string())
        .add_supported_format("risc-v".to_string())
        .allow_effect(EffectType::Read)
        .allow_effect(EffectType::Write);

    // Test compatible metadata
    let mut required_effects = HashSet::new();
    required_effects.insert(EffectType::Read);

    let compatible_metadata = CodeMetadataBuilder::new()
        .with_format("json")
        .with_required_version(Some("0.2.0".to_string()))
        .with_required_effects(Some(required_effects))
        .build();

    assert!(checker.check_compatibility(&compatible_metadata).is_ok());

    // Test incompatible format
    let incompatible_format = CodeMetadataBuilder::new().with_format("unknown").build();

    assert!(checker.check_compatibility(&incompatible_format).is_err());

    // Test incompatible version
    let incompatible_version = CodeMetadataBuilder::new()
        .with_format("json")
        .with_required_version(Some("0.3.0".to_string()))
        .build();

    assert!(checker.check_compatibility(&incompatible_version).is_err());

    // Test incompatible effects
    let mut unauthorized_effects = HashSet::new();
    unauthorized_effects.insert(EffectType::Delete);

    let incompatible_effects = CodeMetadataBuilder::new()
        .with_format("json")
        .with_required_version(Some("0.2.0".to_string()))
        .with_required_effects(Some(unauthorized_effects))
        .build();

    assert!(checker.check_compatibility(&incompatible_effects).is_err());

    Ok(())
}

#[test]
fn test_riscv_metadata() -> Result<()> {
    // Create RISC-V metadata
    let metadata = RiscVMetadata::new()
        .with_isa_extension("RV32I".to_string())
        .with_isa_version("2.1".to_string())
        .with_max_memory(8 * 1024 * 1024)
        .with_max_stack_depth(512)
        .with_max_instructions(500_000)
        .with_floating_point(false)
        .with_atomics(false)
        .with_mul_div(true);

    // Create compatibility checker
    let checker = RiscVCompatibilityChecker::default();

    // Check compatibility
    assert!(checker.check_compatibility(&metadata).is_ok());

    // Test incompatible metadata
    let incompatible = RiscVMetadata::new().with_isa_extension("RV64G".to_string());

    assert!(checker.check_compatibility(&incompatible).is_err());

    Ok(())
}

#[test]
fn test_executor_with_riscv_metadata() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);
    let resource_manager = Arc::new(ResourceManager::new());

    // Create an executor
    let executor = ContentAddressableExecutor::new(repository.clone(), resource_manager.clone());

    // Create a RISC-V enabled test code object
    let test_code = TestCode {
        name: "riscv_function".to_string(),
        version: "1.0.0".to_string(),
        code: "addi x1, x0, 10".to_string(),
    };

    // Create RISC-V metadata
    let riscv_metadata = RiscVMetadata::new()
        .with_isa_extension("RV32I".to_string())
        .with_isa_version("2.1".to_string())
        .with_max_memory(1024 * 1024)
        .with_max_stack_depth(256)
        .with_max_instructions(10_000)
        .with_floating_point(false)
        .with_atomics(false)
        .with_mul_div(false);

    // Create code metadata
    let metadata = CodeMetadataBuilder::new()
        .with_name(Some("riscv_function"))
        .with_format("risc-v")
        .with_version("1.0.0")
        .with_riscv_metadata(Some(riscv_metadata))
        .build();

    // Serialize and store the code
    let serialized = bincode::serialize(&test_code)?;
    let hash = repository.store_with_metadata(serialized, metadata)?;

    // Register the name
    repository.register_name("riscv_function", &hash)?;

    // Create a context and attempt to execute
    let context = executor.create_context("test_execution".to_string(), None)?;

    // This will fail because we haven't implemented RISC-V execution yet,
    // but we should successfully verify the RISC-V metadata.
    let result = executor.execute_by_name("riscv_function", vec![], &context);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not yet implemented"));

    Ok(())
}

#[test]
fn test_end_to_end_workflow() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);
    let resource_manager = Arc::new(ResourceManager::new());

    // Create an executor
    let executor = ContentAddressableExecutor::new(repository.clone(), resource_manager.clone());

    // 1. Store several related code objects

    // A utility function
    let util_code = TestCode {
        name: "util_function".to_string(),
        version: "1.0.0".to_string(),
        code: "function util() { return 'utility'; }".to_string(),
    };

    let util_metadata = CodeMetadataBuilder::new()
        .with_name(Some("util_function"))
        .with_format("json")
        .with_version("1.0.0")
        .build();

    let serialized_util = bincode::serialize(&util_code)?;
    let hash_util = repository.store_with_metadata(serialized_util, util_metadata)?;
    repository.register_name("util_function", &hash_util)?;

    // A component that uses the utility
    let component_code = TestCode {
        name: "component".to_string(),
        version: "1.0.0".to_string(),
        code: "function component() { return 'component-' + util(); }".to_string(),
    };

    let mut dependencies = HashMap::new();
    dependencies.insert("util_function".to_string(), hash_util.to_string());

    let component_metadata = CodeMetadataBuilder::new()
        .with_name(Some("component"))
        .with_format("json")
        .with_version("1.0.0")
        .with_dependencies(Some(dependencies))
        .build();

    let serialized_component = bincode::serialize(&component_code)?;
    let hash_component =
        repository.store_with_metadata(serialized_component, component_metadata)?;
    repository.register_name("component", &hash_component)?;

    // 2. Create effect that uses the component
    let effect_code = TestEffect {
        name: "test_effect".to_string(),
        effect_type: "read".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert(
                "component".to_string(),
                Value::String("component".to_string()),
            );
            params
        },
    };

    let mut dependencies = HashMap::new();
    dependencies.insert("component".to_string(), hash_component.to_string());

    let mut required_effects = HashSet::new();
    required_effects.insert(EffectType::Read);

    let effect_metadata = CodeMetadataBuilder::new()
        .with_name(Some("test_effect"))
        .with_format("json")
        .with_version("1.0.0")
        .with_dependencies(Some(dependencies))
        .with_required_effects(Some(required_effects))
        .build();

    let serialized_effect = bincode::serialize(&effect_code)?;
    let hash_effect = repository.store_with_metadata(serialized_effect, effect_metadata)?;
    repository.register_name("test_effect", &hash_effect)?;

    // 3. Execute by name
    let context = executor.create_context("test_execution".to_string(), None)?;
    let result = executor.execute_by_name("test_effect", vec![], &context);

    // The execution is expected to return a placeholder value since we haven't
    // implemented the actual execution logic
    assert!(result.is_ok());

    // 4. Get the execution trace
    let trace = context.execution_trace()?;
    assert!(trace.len() >= 2); // At least function invocation and return

    // 5. Update the utility function to a new version
    let util_code_v2 = TestCode {
        name: "util_function".to_string(),
        version: "2.0.0".to_string(),
        code: "function util() { return 'improved utility'; }".to_string(),
    };

    let util_metadata_v2 = CodeMetadataBuilder::new()
        .with_name(Some("util_function"))
        .with_format("json")
        .with_version("2.0.0")
        .build();

    let serialized_util_v2 = bincode::serialize(&util_code_v2)?;
    let hash_util_v2 = repository.store_with_metadata(serialized_util_v2, util_metadata_v2)?;

    // Register as versioned name
    repository.register_name("util_function@2.0.0", &hash_util_v2)?;

    // Both versions should be available
    let v1_hash = repository.resolve_name("util_function")?;
    let v2_hash = repository.resolve_name("util_function@2.0.0")?;

    assert_eq!(v1_hash, hash_util); // Default still points to v1
    assert_eq!(v2_hash, hash_util_v2);

    // 6. Update the default to point to v2
    repository.register_name("util_function", &hash_util_v2)?;

    // Now default should point to v2
    let new_default = repository.resolve_name("util_function")?;
    assert_eq!(new_default, hash_util_v2);

    // Component still uses v1 since it references by hash
    let component_entry = repository.load_by_hash(&hash_component)?;
    let component_deps = component_entry.metadata.dependencies.unwrap();
    assert_eq!(component_deps["util_function"], hash_util.to_string());

    Ok(())
}
