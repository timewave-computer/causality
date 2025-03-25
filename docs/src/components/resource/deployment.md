<!-- Deployment of resources -->
<!-- Original file: docs/src/resource_deployment.md -->

# Resource Deployment in Causality

## Overview

This document describes the comprehensive process of resource deployment within the Causality architecture. Resource deployment encompasses the stages and mechanisms required to prepare, package, validate, deploy, and activate resources within the system. The deployment process ensures that resources are properly installed, configured, and integrated with the broader system in a consistent and secure manner.

## Core Concepts

### Deployment Model

The fundamental deployment model includes these key components:

```rust
pub struct ResourceDeployment {
    /// Unique identifier for this deployment
    id: DeploymentId,
    
    /// Resource being deployed
    resource: Resource,
    
    /// Deployment status
    status: DeploymentStatus,
    
    /// Deployment configuration
    config: DeploymentConfig,
    
    /// Deployment manifest
    manifest: DeploymentManifest,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Deployment metadata
    metadata: DeploymentMetadata,
}

pub enum DeploymentStatus {
    /// Deployment has been created but not started
    Created,
    
    /// Deployment is being prepared
    Preparing,
    
    /// Deployment is being validated
    Validating,
    
    /// Deployment is being executed
    Deploying,
    
    /// Deployment is being activated
    Activating,
    
    /// Deployment is complete
    Completed,
    
    /// Deployment failed
    Failed(String),
    
    /// Deployment was rolled back
    RolledBack,
}
```

### Deployment Manifest

The deployment manifest specifies deployment requirements and dependencies:

```rust
pub struct DeploymentManifest {
    /// Resource metadata
    resource_metadata: ResourceMetadata,
    
    /// Dependencies required by this resource
    dependencies: Vec<ResourceDependency>,
    
    /// Required capabilities for deployment
    required_capabilities: Vec<CapabilityRequirement>,
    
    /// Configuration schema
    config_schema: ConfigSchema,
    
    /// Deployment hooks
    hooks: DeploymentHooks,
    
    /// Validation rules
    validation_rules: Vec<ValidationRule>,
}
```

## Deployment Stages

### 1. Resource Packaging

Resources are packaged for deployment:

```rust
/// Package a resource for deployment
pub fn package_resource(
    resource: Resource,
    package_options: PackageOptions,
) -> Result<ResourcePackage, PackageError> {
    // Create a new package
    let mut package = ResourcePackage::new(resource.id());
    
    // Add core resource definition
    package.add_resource(resource.clone())?;
    
    // Add resource code
    if let Some(code) = resource.code() {
        package.add_code(code)?;
    }
    
    // Add resource binaries
    if let Some(binaries) = resource.binaries() {
        package.add_binaries(binaries)?;
    }
    
    // Add resource assets
    if let Some(assets) = resource.assets() {
        package.add_assets(assets)?;
    }
    
    // Add deployment manifest
    let manifest = create_deployment_manifest(&resource, &package_options)?;
    package.set_manifest(manifest)?;
    
    // Sign package
    package.sign(package_options.signing_key())?;
    
    Ok(package)
}
```

### 2. Deployment Preparation

Preparing the system for resource deployment:

```rust
/// Prepare for resource deployment
pub fn prepare_deployment(
    package: ResourcePackage,
    config: DeploymentConfig,
) -> Result<ResourceDeployment, DeploymentError> {
    // Verify package signature
    package.verify_signature()?;
    
    // Extract deployment manifest
    let manifest = package.manifest()?;
    
    // Create deployment
    let deployment = ResourceDeployment {
        id: DeploymentId::generate(),
        resource: package.resource()?,
        status: DeploymentStatus::Created,
        config,
        manifest,
        created_at: system.current_time(),
        metadata: DeploymentMetadata::new(),
    };
    
    // Store the deployment
    system.deployment_registry().register_deployment(deployment.clone())?;
    
    // Update status
    let mut deployment = deployment;
    deployment.status = DeploymentStatus::Preparing;
    system.deployment_registry().update_deployment(&deployment)?;
    
    // Prepare environment
    prepare_environment(&deployment)?;
    
    // Resolve dependencies
    resolve_dependencies(&deployment)?;
    
    Ok(deployment)
}
```

### 3. Deployment Validation

Validating the deployment before execution:

```rust
/// Validate a deployment
pub fn validate_deployment(
    deployment: &ResourceDeployment,
) -> Result<ValidationResult, ValidationError> {
    // Update status
    let mut deployment = deployment.clone();
    deployment.status = DeploymentStatus::Validating;
    system.deployment_registry().update_deployment(&deployment)?;
    
    // Create validation context
    let context = ValidationContext::new()
        .with_deployment(deployment.clone())
        .with_system_info(system.system_info());
    
    // Execute pre-validation hooks
    execute_hooks(&deployment, HookType::PreValidation, &context)?;
    
    // Validate resource definition
    let resource_result = validate_resource_definition(&deployment.resource, &context)?;
    if !resource_result.is_valid() {
        return Ok(resource_result);
    }
    
    // Validate dependencies
    let dependency_result = validate_dependencies(&deployment, &context)?;
    if !dependency_result.is_valid() {
        return Ok(dependency_result);
    }
    
    // Validate capabilities
    let capability_result = validate_capabilities(&deployment, &context)?;
    if !capability_result.is_valid() {
        return Ok(capability_result);
    }
    
    // Validate configuration
    let config_result = validate_configuration(&deployment, &context)?;
    if !config_result.is_valid() {
        return Ok(config_result);
    }
    
    // Validate system compatibility
    let compatibility_result = validate_compatibility(&deployment, &context)?;
    if !compatibility_result.is_valid() {
        return Ok(compatibility_result);
    }
    
    // Execute post-validation hooks
    execute_hooks(&deployment, HookType::PostValidation, &context)?;
    
    // All validations passed
    Ok(ValidationResult::valid())
}
```

### 4. Deployment Execution

Executing the deployment:

```rust
/// Execute a deployment
pub fn execute_deployment(
    deployment: &ResourceDeployment,
) -> Result<DeploymentResult, DeploymentError> {
    // Update status
    let mut deployment = deployment.clone();
    deployment.status = DeploymentStatus::Deploying;
    system.deployment_registry().update_deployment(&deployment)?;
    
    // Create deployment context
    let context = DeploymentContext::new()
        .with_deployment(deployment.clone());
    
    // Execute pre-deployment hooks
    execute_hooks(&deployment, HookType::PreDeployment, &context)?;
    
    // Deploy resource code
    deploy_resource_code(&deployment, &context)?;
    
    // Deploy resource binaries
    deploy_resource_binaries(&deployment, &context)?;
    
    // Deploy resource assets
    deploy_resource_assets(&deployment, &context)?;
    
    // Register resource in registry
    register_resource(&deployment, &context)?;
    
    // Configure resource
    configure_resource(&deployment, &context)?;
    
    // Execute post-deployment hooks
    execute_hooks(&deployment, HookType::PostDeployment, &context)?;
    
    // Return result
    Ok(DeploymentResult {
        deployment_id: deployment.id,
        resource_id: deployment.resource.id(),
        status: DeploymentResultStatus::Success,
        timestamp: system.current_time(),
    })
}
```

### 5. Resource Activation

Activating the deployed resource:

```rust
/// Activate a deployed resource
pub fn activate_resource(
    deployment: &ResourceDeployment,
) -> Result<ActivationResult, ActivationError> {
    // Update status
    let mut deployment = deployment.clone();
    deployment.status = DeploymentStatus::Activating;
    system.deployment_registry().update_deployment(&deployment)?;
    
    // Create activation context
    let context = ActivationContext::new()
        .with_deployment(deployment.clone());
    
    // Execute pre-activation hooks
    execute_hooks(&deployment, HookType::PreActivation, &context)?;
    
    // Activate resource
    let resource_id = deployment.resource.id();
    system.resource_registry().activate_resource(resource_id)?;
    
    // Register resource interfaces
    register_resource_interfaces(&deployment, &context)?;
    
    // Initialize resource state
    initialize_resource_state(&deployment, &context)?;
    
    // Establish resource relationships
    establish_resource_relationships(&deployment, &context)?;
    
    // Register capabilities
    register_resource_capabilities(&deployment, &context)?;
    
    // Execute post-activation hooks
    execute_hooks(&deployment, HookType::PostActivation, &context)?;
    
    // Update deployment status
    let mut final_deployment = deployment.clone();
    final_deployment.status = DeploymentStatus::Completed;
    system.deployment_registry().update_deployment(&final_deployment)?;
    
    // Return result
    Ok(ActivationResult {
        deployment_id: deployment.id,
        resource_id,
        status: ActivationStatus::Active,
        timestamp: system.current_time(),
    })
}
```

## Deployment Infrastructure

### Deployment Registry

Tracks and manages deployments:

```rust
pub struct DeploymentRegistry {
    /// Core registry functionality
    registry: Registry<ResourceDeployment>,
    
    /// Deployment observers
    observers: Vec<Box<dyn DeploymentObserver>>,
}

impl DeploymentRegistry {
    /// Register a new deployment
    pub fn register_deployment(
        &self,
        deployment: ResourceDeployment,
    ) -> Result<DeploymentId, RegistryError> {
        let deployment_id = deployment.id;
        self.registry.register(deployment.clone())?;
        
        // Notify observers
        for observer in &self.observers {
            observer.on_deployment_registered(&deployment)?;
        }
        
        Ok(deployment_id)
    }
    
    /// Update an existing deployment
    pub fn update_deployment(
        &self,
        deployment: &ResourceDeployment,
    ) -> Result<(), RegistryError> {
        // Get current deployment
        let current = self.registry.get(&deployment.id.into())?;
        
        // Update deployment
        self.registry.update(deployment.clone())?;
        
        // Notify observers
        for observer in &self.observers {
            observer.on_deployment_updated(&current, deployment)?;
        }
        
        Ok(())
    }
    
    /// Get a deployment by ID
    pub fn get_deployment(
        &self,
        id: DeploymentId,
    ) -> Result<ResourceDeployment, RegistryError> {
        self.registry.get(&id.into())
    }
    
    /// Query deployments
    pub fn query_deployments(
        &self,
        query: DeploymentQuery,
    ) -> Result<Vec<ResourceDeployment>, RegistryError> {
        self.registry.query(query)
    }
}
```

### Deployment Manager

Orchestrates the deployment process:

```rust
pub struct DeploymentManager {
    /// Deployment registry
    deployment_registry: Arc<DeploymentRegistry>,
    
    /// Resource registry
    resource_registry: Arc<ResourceRegistry>,
    
    /// Deployment validators
    validators: Vec<Box<dyn DeploymentValidator>>,
    
    /// Deployment executors
    executors: HashMap<ResourceType, Box<dyn DeploymentExecutor>>,
    
    /// Deployment event handlers
    event_handlers: Vec<Box<dyn DeploymentEventHandler>>,
}

impl DeploymentManager {
    /// Deploy a resource package
    pub fn deploy_resource(
        &self,
        package: ResourcePackage,
        config: DeploymentConfig,
    ) -> Result<DeploymentResult, DeploymentError> {
        // Prepare deployment
        let deployment = prepare_deployment(package, config)?;
        
        // Validate deployment
        let validation_result = validate_deployment(&deployment)?;
        if !validation_result.is_valid() {
            // Update status to failed
            let mut failed_deployment = deployment.clone();
            failed_deployment.status = DeploymentStatus::Failed(
                validation_result.error_messages().join(", ")
            );
            self.deployment_registry.update_deployment(&failed_deployment)?;
            
            return Err(DeploymentError::ValidationFailed(validation_result));
        }
        
        // Execute deployment
        let deployment_result = execute_deployment(&deployment)?;
        
        // Activate resource
        let activation_result = activate_resource(&deployment)?;
        
        Ok(deployment_result)
    }
    
    /// Roll back a deployment
    pub fn rollback_deployment(
        &self,
        deployment_id: DeploymentId,
        reason: &str,
    ) -> Result<RollbackResult, RollbackError> {
        // Get the deployment
        let deployment = self.deployment_registry.get_deployment(deployment_id)?;
        
        // Ensure deployment can be rolled back
        if !can_rollback(&deployment) {
            return Err(RollbackError::CannotRollback(
                format!("Deployment cannot be rolled back: {}", 
                    deployment.status_message()),
            ));
        }
        
        // Create rollback context
        let context = RollbackContext::new()
            .with_deployment(deployment.clone())
            .with_reason(reason);
        
        // Execute pre-rollback hooks
        execute_hooks(&deployment, HookType::PreRollback, &context)?;
        
        // Deactivate resource
        deactivate_resource(&deployment, &context)?;
        
        // Unregister resource
        unregister_resource(&deployment, &context)?;
        
        // Remove resource artifacts
        remove_resource_artifacts(&deployment, &context)?;
        
        // Restore previous state if applicable
        restore_previous_state(&deployment, &context)?;
        
        // Execute post-rollback hooks
        execute_hooks(&deployment, HookType::PostRollback, &context)?;
        
        // Update deployment status
        let mut rolled_back = deployment.clone();
        rolled_back.status = DeploymentStatus::RolledBack;
        self.deployment_registry.update_deployment(&rolled_back)?;
        
        // Return result
        Ok(RollbackResult {
            deployment_id,
            status: RollbackStatus::Success,
            timestamp: system.current_time(),
        })
    }
}
```

## Resource-Specific Deployment

### Interface-Based Deployment

Deploying resources that implement interfaces:

```rust
/// Deploy a resource implementing interfaces
pub fn deploy_interface_resource(
    deployment: &ResourceDeployment,
    context: &DeploymentContext,
) -> Result<(), DeploymentError> {
    // Get the resource
    let resource = &deployment.resource;
    
    // Get implemented interfaces
    let interfaces = resource.implemented_interfaces();
    
    // Deploy resource base
    deploy_resource_base(resource, context)?;
    
    // For each interface, register implementation
    for interface_impl in interfaces {
        let interface_id = interface_impl.interface_id();
        
        // Verify interface exists
        system.interface_registry().get_interface(interface_id)?;
        
        // Register implementation
        system.interface_registry().register_implementation(
            resource.id(),
            interface_id,
            interface_impl.implementation_data().clone(),
        )?;
    }
    
    Ok(())
}
```

### Service Resource Deployment

Deploying service resources:

```rust
/// Deploy a service resource
pub fn deploy_service_resource(
    deployment: &ResourceDeployment,
    context: &DeploymentContext,
) -> Result<(), DeploymentError> {
    // Get the resource
    let resource = &deployment.resource;
    
    // Check resource type
    if resource.resource_type() != ResourceType::new("service") {
        return Err(DeploymentError::InvalidResourceType(
            format!("Expected service resource, got {}", resource.resource_type()),
        ));
    }
    
    // Deploy resource base
    deploy_resource_base(resource, context)?;
    
    // Extract service configuration
    let service_config = extract_service_config(resource, &deployment.config)?;
    
    // Prepare service environment
    prepare_service_environment(resource, &service_config, context)?;
    
    // Deploy service binary
    deploy_service_binary(resource, context)?;
    
    // Configure service endpoints
    configure_service_endpoints(resource, &service_config, context)?;
    
    // Register service with service registry
    register_service(resource, &service_config, context)?;
    
    Ok(())
}
```

## Deployment Security

### Capability Validation

Validating capabilities during deployment:

```rust
/// Validate deployment capabilities
pub fn validate_capabilities(
    deployment: &ResourceDeployment,
    context: &ValidationContext,
) -> Result<ValidationResult, ValidationError> {
    // Get required capabilities from manifest
    let required_capabilities = &deployment.manifest.required_capabilities;
    
    // Check deployer's capabilities
    let deployer_identity = context.identity();
    
    for required in required_capabilities {
        // Check if deployer has required capability
        let has_capability = system.capability_registry().check_capability(
            deployer_identity,
            required.capability_type(),
            required.capability_target(),
        )?;
        
        if !has_capability {
            return Ok(ValidationResult::invalid(format!(
                "Deployer lacks required capability: {:?} for target {:?}",
                required.capability_type(),
                required.capability_target(),
            )));
        }
    }
    
    // All capability checks passed
    Ok(ValidationResult::valid())
}
```

### Deployment Signing

Signing deployment artifacts:

```rust
/// Sign a deployment package
pub fn sign_deployment_package(
    package: &mut ResourcePackage,
    signing_key: &SigningKey,
) -> Result<(), SigningError> {
    // Generate package hash
    let package_hash = package.calculate_hash()?;
    
    // Sign the hash
    let signature = signing_key.sign(&package_hash)?;
    
    // Add signature to package
    package.set_signature(SignatureInfo {
        signature,
        signer: signing_key.identity(),
        algorithm: signing_key.algorithm(),
        timestamp: system.current_time(),
    })?;
    
    Ok(())
}

/// Verify a deployment package signature
pub fn verify_package_signature(
    package: &ResourcePackage,
) -> Result<bool, SigningError> {
    // Get signature
    let signature_info = package.signature()?;
    
    // Calculate package hash
    let package_hash = package.calculate_hash()?;
    
    // Get signer's verification key
    let verification_key = system.key_registry().get_verification_key(signature_info.signer)?;
    
    // Verify signature
    verification_key.verify(&package_hash, &signature_info.signature)
}
```

## Deployment Monitoring

Monitoring deployment status:

```rust
pub trait DeploymentObserver: Send + Sync {
    /// Called when a deployment is registered
    fn on_deployment_registered(&self, deployment: &ResourceDeployment) -> Result<(), ObserverError>;
    
    /// Called when a deployment is updated
    fn on_deployment_updated(&self, old: &ResourceDeployment, new: &ResourceDeployment) -> Result<(), ObserverError>;
    
    /// Called when a deployment is completed
    fn on_deployment_completed(&self, deployment: &ResourceDeployment, result: &DeploymentResult) -> Result<(), ObserverError>;
    
    /// Called when a deployment fails
    fn on_deployment_failed(&self, deployment: &ResourceDeployment, error: &DeploymentError) -> Result<(), ObserverError>;
    
    /// Called when a deployment is rolled back
    fn on_deployment_rolled_back(&self, deployment: &ResourceDeployment, result: &RollbackResult) -> Result<(), ObserverError>;
}
```

## Usage Examples

### Basic Resource Deployment

```rust
// Create resource definition
let resource = Resource::new(
    ResourceType::new("document_store"),
    ResourceAttributes::new()
        .with_attribute("name", "Customer Document Storage")
        .with_attribute("version", "1.0.0")
        .with_attribute("description", "Secure storage for customer documents"),
);

// Create resource code
let resource_code = ResourceCode::from_directory("/path/to/code")?;

// Set code for resource
let resource = resource.with_code(resource_code);

// Create package options
let package_options = PackageOptions::new()
    .with_signing_key(system.identity_manager().get_signing_key()?)
    .with_compression(CompressionType::Gzip);

// Package the resource
let package = package_resource(resource, package_options)?;

// Create deployment configuration
let config = DeploymentConfig::new()
    .with_parameter("storage_path", "/data/customer_documents")
    .with_parameter("max_document_size", "10MB")
    .with_parameter("allowed_document_types", vec!["pdf", "docx", "xlsx"]);

// Deploy the resource
let result = deployment_manager.deploy_resource(package, config)?;

println!("Deployment completed with status: {:?}", result.status);
```

### Deploying a Service Resource

```rust
// Create service resource
let service = Resource::new(
    ResourceType::new("service"),
    ResourceAttributes::new()
        .with_attribute("name", "Document Processor Service")
        .with_attribute("version", "2.1.0")
        .with_attribute("service_type", "background_worker")
        .with_attribute("language", "rust"),
);

// Set service binary
let binary_data = std::fs::read("/path/to/document_processor_binary")?;
let service = service.with_binary("document_processor", binary_data);

// Create service configuration
let service_config = ServiceConfig::new()
    .with_port(8080)
    .with_environment_variables(HashMap::from([
        ("LOG_LEVEL".to_string(), "info".to_string()),
        ("DB_CONNECTION".to_string(), "postgres://user:pass@localhost:5432/docs".to_string()),
    ]))
    .with_resource_limits(ResourceLimits::new()
        .with_memory("512Mi")
        .with_cpu("0.5"));

// Create deployment configuration
let config = DeploymentConfig::new()
    .with_service_config(service_config)
    .with_parameter("worker_threads", "4")
    .with_parameter("queue_name", "document_processing");

// Package and deploy
let package = package_resource(service, package_options)?;
let result = deployment_manager.deploy_resource(package, config)?;

println!("Service deployment completed with ID: {}", result.deployment_id);
```

### Rolling Back a Deployment

```rust
// Attempt to roll back a deployment
let rollback_result = deployment_manager.rollback_deployment(
    deployment_id,
    "Service performance degradation observed",
)?;

println!("Rollback completed with status: {:?}", rollback_result.status);

// Get deployment history
let deployment_history = deployment_registry.query_deployments(
    DeploymentQuery::new()
        .with_resource_id(resource_id)
        .with_status_in(vec![
            DeploymentStatus::Completed,
            DeploymentStatus::Failed,
            DeploymentStatus::RolledBack,
        ])
        .with_order_by(OrderBy::Created, SortOrder::Descending),
)?;

println!("Deployment history for resource:");
for deployment in deployment_history {
    println!("  {} - Status: {:?}, Created: {}", 
        deployment.id, deployment.status, deployment.created_at);
}
```

## Implementation Status

The current implementation status of Resource Deployment:

- ✅ Core deployment model
- ✅ Resource packaging
- ✅ Deployment validation
- ⚠️ Deployment execution (partially implemented)
- ⚠️ Resource activation (partially implemented)
- ⚠️ Deployment rollback (partially implemented)
- ❌ Deployment monitoring (not yet implemented)
- ❌ Distributed deployment coordination (not yet implemented)

## Future Enhancements

Planned future enhancements for Resource Deployment:

1. **Canary Deployments**: Support for gradual resource rollout with monitoring
2. **Blue-Green Deployments**: Zero-downtime resource upgrades
3. **Deployment Versioning**: Enhanced versioning for resource deployments
4. **Dependency Management**: Improved resolution of complex dependency graphs
5. **Runtime Adaptation**: Resources that adapt to runtime environment conditions
6. **Deployment Templates**: Reusable templates for common deployment patterns
7. **Enhanced Monitoring**: Advanced monitoring of deployed resources
8. **Automated Rollback**: Automatic rollback based on health metrics 