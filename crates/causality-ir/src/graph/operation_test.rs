// Tests for the Operation struct
// This is in a separate file from the main implementation to avoid
// being affected by other issues in the codebase

use super::operation::{Operation, OperationType};
use causality_types::{ContentAddressed, HashError};

#[test]
fn test_operation_creation() {
    let op = Operation::new(
        "resource:123".into(),
        OperationType::Create,
        Some("agent:456".into()),
    );
    
    assert_eq!(op.target_resource_id, "resource:123");
    assert_eq!(op.operation_type, OperationType::Create);
    assert_eq!(op.agent_id.as_ref().unwrap(), "agent:456");
    assert!(op.parameters.is_none());
    assert!(op.metadata.is_none());
}

#[test]
fn test_operation_with_parameters() {
    let op = Operation::new(
        "resource:123".into(),
        OperationType::Update,
        Some("agent:456".into()),
    )
    .with_parameter("key1", "value1")
    .with_parameter("key2", "value2");
    
    let params = op.parameters.as_ref().unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params.get("key1").unwrap(), "value1");
    assert_eq!(params.get("key2").unwrap(), "value2");
}

#[test]
fn test_operation_with_metadata() {
    let op = Operation::new(
        "resource:123".into(),
        OperationType::Delete,
        None,
    )
    .with_metadata("origin", "system")
    .with_metadata("timestamp", "123456789");
    
    let meta = op.metadata.as_ref().unwrap();
    assert_eq!(meta.len(), 2);
    assert_eq!(meta.get("origin").unwrap(), "system");
    assert_eq!(meta.get("timestamp").unwrap(), "123456789");
    assert_eq!(op.has_agent(), false);
}

#[test]
fn test_content_addressed() -> Result<(), HashError> {
    let op1 = Operation::new(
        "resource:123".into(),
        OperationType::Create,
        Some("agent:456".into()),
    )
    .with_parameter("param", "value");
    
    let op2 = Operation::new(
        "resource:123".into(),
        OperationType::Create,
        Some("agent:456".into()),
    )
    .with_parameter("param", "value");
    
    // Same content should produce same hash
    let hash1 = op1.content_hash()?;
    let hash2 = op2.content_hash()?;
    assert_eq!(hash1, hash2);
    
    // Different content should produce different hash
    let op3 = op1.clone().with_parameter("param2", "value2");
    let hash3 = op3.content_hash()?;
    assert_ne!(hash1, hash3);
    
    Ok(())
} 