use crate::resource::{
    agent::{
        capability::{
            CapabilityBundle, CapabilityBundleBuilder, CapabilityBundleManager,
            CapabilityBundleScope, DelegationRules, StandardBundleType,
        },
        AgentId,
    },
    content::ContentHash,
    ResourceId, ResourceType,
};

/// Create a test agent ID
fn create_test_agent_id(name: &str) -> AgentId {
    AgentId::from_content_hash(ContentHash::calculate(name.as_bytes()))
}

/// Create a test resource ID
fn create_test_resource_id(name: &str) -> ResourceId {
    ResourceId::from_content_hash(ContentHash::calculate(name.as_bytes()))
}

#[test]
fn test_capability_bundle_comprehensive() {
    // Initialize manager
    let mut manager = CapabilityBundleManager::new();
    
    // Register different standard bundles
    let read_only_id = manager.register_standard_bundle(
        StandardBundleType::ReadOnly,
        Some(ResourceType::new("document")),
    ).unwrap();
    
    let read_write_id = manager.register_standard_bundle(
        StandardBundleType::ReadWrite,
        None,
    ).unwrap();
    
    let admin_id = manager.register_standard_bundle(
        StandardBundleType::Admin,
        Some(ResourceType::new("system")),
    ).unwrap();
    
    let user_id = manager.register_standard_bundle(
        StandardBundleType::UserBasic,
        None,
    ).unwrap();
    
    let committee_id = manager.register_standard_bundle(
        StandardBundleType::CommitteeBasic,
        None,
    ).unwrap();
    
    let operator_id = manager.register_standard_bundle(
        StandardBundleType::OperatorBasic,
        None,
    ).unwrap();
    
    // Create some agents
    let system = create_test_agent_id("system");
    let admin = create_test_agent_id("admin");
    let operator = create_test_agent_id("operator");
    let committee = create_test_agent_id("committee");
    let alice = create_test_agent_id("alice");
    let bob = create_test_agent_id("bob");
    
    let current_time = 1000;
    
    // System has admin privileges and delegates to admin
    manager.delegate_bundle(&admin_id, &system, &admin, current_time).unwrap();
    
    // Admin delegates operator bundle to operator
    manager.delegate_bundle(&operator_id, &admin, &operator, current_time).unwrap();
    
    // Admin delegates committee bundle to committee
    manager.delegate_bundle(&committee_id, &admin, &committee, current_time).unwrap();
    
    // Admin delegates user bundle to alice
    manager.delegate_bundle(&user_id, &admin, &alice, current_time).unwrap();
    
    // Admin delegates read-only to bob
    manager.delegate_bundle(&read_only_id, &admin, &bob, current_time).unwrap();
    
    // Test delegation validation
    assert!(manager.has_bundle(&admin, &admin_id, current_time));
    assert!(manager.has_bundle(&operator, &operator_id, current_time));
    assert!(manager.has_bundle(&committee, &committee_id, current_time));
    assert!(manager.has_bundle(&alice, &user_id, current_time));
    assert!(manager.has_bundle(&bob, &read_only_id, current_time));
    
    // Test resource capabilities
    let document_id = create_test_resource_id("test_document");
    let document_type = ResourceType::new("document");
    
    // Bob should have read capabilities for the document
    let bob_caps = manager.get_agent_capabilities_for_resource(
        &bob,
        &document_id,
        &document_type,
        current_time,
    );
    assert!(!bob_caps.is_empty());
    assert!(bob_caps.iter().any(|c| c.id().as_str() == "read"));
    assert!(bob_caps.iter().any(|c| c.id().as_str() == "list"));
    assert!(!bob_caps.iter().any(|c| c.id().as_str() == "write"));
    
    // Alice has user capabilities but not document specific ones
    let alice_caps = manager.get_agent_capabilities_for_resource(
        &alice,
        &document_id,
        &document_type,
        current_time,
    );
    assert!(alice_caps.iter().any(|c| c.id().as_str() == "resource.list"));
    assert!(!alice_caps.iter().any(|c| c.id().as_str() == "write"));
    
    // Test delegation revocation
    manager.revoke_delegation(&read_only_id, &admin, &bob).unwrap();
    assert!(!manager.has_bundle(&bob, &read_only_id, current_time));
    
    // Test expired delegation
    let custom_bundle = CapabilityBundleBuilder::new(
        "Temporary Access",
        "Bundle with temporary access"
    )
    .scope(CapabilityBundleScope::Global)
    .add_capability(Capability::new("temp.access", vec!["access"]))
    .build();
    
    let temp_id = custom_bundle.id.clone();
    manager.register_bundle(custom_bundle).unwrap();
    
    // Create delegation with explicit expiration
    let delegation = manager.delegate_bundle(&temp_id, &admin, &bob, current_time).unwrap()
        .with_expiration(current_time + 50);
    
    // Manually update the delegation to include expiration
    manager.revoke_delegation(&temp_id, &admin, &bob).unwrap();
    let delegations = manager.delegations.entry((bob.clone(), temp_id.clone())).or_insert_with(Vec::new);
    delegations.push(delegation);
    
    // Before expiration, bob should have the bundle
    assert!(manager.has_bundle(&bob, &temp_id, current_time + 49));
    
    // After expiration, bob should not have the bundle
    assert!(!manager.has_bundle(&bob, &temp_id, current_time + 50));
    
    // Test custom bundle creation
    let custom_capabilities = vec![
        ("custom.read".to_string(), vec!["read".to_string()]),
        ("custom.execute".to_string(), vec!["execute".to_string()]),
    ];
    
    let custom_id = manager.register_standard_bundle(
        StandardBundleType::Custom("Custom Bundle".to_string(), custom_capabilities),
        None
    ).unwrap();
    
    let custom_bundle = manager.get_bundle(&custom_id).unwrap();
    assert_eq!(custom_bundle.name(), "Custom Bundle");
    assert_eq!(custom_bundle.capabilities().len(), 2);
    assert!(custom_bundle.capabilities().iter().any(|c| c.id().as_str() == "custom.read"));
    assert!(custom_bundle.capabilities().iter().any(|c| c.id().as_str() == "custom.execute"));
}

#[test]
fn test_capability_bundle_scope() {
    // Initialize manager
    let mut manager = CapabilityBundleManager::new();
    
    // Create bundles with different scopes
    let global_bundle = CapabilityBundleBuilder::new(
        "Global Access", 
        "Access to everything"
    )
    .scope(CapabilityBundleScope::Global)
    .add_capability(Capability::new("global.access", vec!["access"]))
    .build();
    
    let document_bundle = CapabilityBundleBuilder::new(
        "Document Access", 
        "Access to documents"
    )
    .scope(CapabilityBundleScope::ResourceType(ResourceType::new("document")))
    .add_capability(Capability::new("document.access", vec!["access"]))
    .build();
    
    let specific_doc_id = create_test_resource_id("specific_document");
    let specific_doc_bundle = CapabilityBundleBuilder::new(
        "Specific Document", 
        "Access to a specific document"
    )
    .scope(CapabilityBundleScope::Resource(
        specific_doc_id.clone(), 
        ResourceType::new("document")
    ))
    .add_capability(Capability::new("specific.access", vec!["access"]))
    .build();
    
    // Register bundles
    let global_id = global_bundle.id.clone();
    let document_id = document_bundle.id.clone();
    let specific_id = specific_doc_bundle.id.clone();
    
    manager.register_bundle(global_bundle).unwrap();
    manager.register_bundle(document_bundle).unwrap();
    manager.register_bundle(specific_doc_bundle).unwrap();
    
    // Create agents
    let admin = create_test_agent_id("admin");
    let user = create_test_agent_id("user");
    
    let current_time = 1000;
    
    // Delegate all bundles to user
    manager.delegate_bundle(&global_id, &admin, &user, current_time).unwrap();
    manager.delegate_bundle(&document_id, &admin, &user, current_time).unwrap();
    manager.delegate_bundle(&specific_id, &admin, &user, current_time).unwrap();
    
    // Test global scope
    let system_id = create_test_resource_id("system");
    let system_type = ResourceType::new("system");
    
    let caps = manager.get_agent_capabilities_for_resource(
        &user,
        &system_id,
        &system_type,
        current_time,
    );
    
    // Global bundle should apply
    assert!(caps.iter().any(|c| c.id().as_str() == "global.access"));
    
    // Document type bundle should not apply
    assert!(!caps.iter().any(|c| c.id().as_str() == "document.access"));
    
    // Specific document bundle should not apply
    assert!(!caps.iter().any(|c| c.id().as_str() == "specific.access"));
    
    // Test document type scope
    let random_doc_id = create_test_resource_id("random_document");
    let doc_type = ResourceType::new("document");
    
    let caps = manager.get_agent_capabilities_for_resource(
        &user,
        &random_doc_id,
        &doc_type,
        current_time,
    );
    
    // Global bundle should apply
    assert!(caps.iter().any(|c| c.id().as_str() == "global.access"));
    
    // Document type bundle should apply
    assert!(caps.iter().any(|c| c.id().as_str() == "document.access"));
    
    // Specific document bundle should not apply
    assert!(!caps.iter().any(|c| c.id().as_str() == "specific.access"));
    
    // Test specific resource scope
    let caps = manager.get_agent_capabilities_for_resource(
        &user,
        &specific_doc_id,
        &doc_type,
        current_time,
    );
    
    // Global bundle should apply
    assert!(caps.iter().any(|c| c.id().as_str() == "global.access"));
    
    // Document type bundle should apply
    assert!(caps.iter().any(|c| c.id().as_str() == "document.access"));
    
    // Specific document bundle should apply
    assert!(caps.iter().any(|c| c.id().as_str() == "specific.access"));
}

#[test]
fn test_delegation_rules() {
    // Initialize manager
    let mut manager = CapabilityBundleManager::new();
    
    // Create a bundle with strict delegation rules
    let mut strict_bundle = CapabilityBundleBuilder::new(
        "Strict Bundle",
        "Bundle with strict delegation rules"
    )
    .scope(CapabilityBundleScope::Global)
    .add_capability(Capability::new("strict.access", vec!["access"]))
    .build();
    
    // Set delegation rules
    let allowed_agent = create_test_agent_id("allowed");
    let mut allowed_agents = Vec::new();
    allowed_agents.push(allowed_agent.clone());
    
    strict_bundle.delegation_rules = DelegationRules {
        allow_delegation: true,
        allow_sub_delegation: false,
        time_limit: Some(100),
        allowed_delegatees: allowed_agents,
    };
    
    let strict_id = strict_bundle.id.clone();
    manager.register_bundle(strict_bundle).unwrap();
    
    // Create agents
    let admin = create_test_agent_id("admin");
    let allowed = create_test_agent_id("allowed");
    let disallowed = create_test_agent_id("disallowed");
    
    let current_time = 1000;
    
    // Admin can delegate to allowed agent
    let result = manager.delegate_bundle(&strict_id, &admin, &allowed, current_time);
    assert!(result.is_ok());
    
    // Admin cannot delegate to disallowed agent
    let result = manager.delegate_bundle(&strict_id, &admin, &disallowed, current_time);
    assert!(result.is_err());
    
    // Allowed agent cannot sub-delegate
    let result = manager.delegate_bundle(&strict_id, &allowed, &disallowed, current_time);
    assert!(result.is_err());
    
    // Create a bundle with permissive delegation rules
    let mut permissive_bundle = CapabilityBundleBuilder::new(
        "Permissive Bundle",
        "Bundle with permissive delegation rules"
    )
    .scope(CapabilityBundleScope::Global)
    .add_capability(Capability::new("permissive.access", vec!["access"]))
    .build();
    
    // Set delegation rules
    permissive_bundle.delegation_rules = DelegationRules {
        allow_delegation: true,
        allow_sub_delegation: true,
        time_limit: None,
        allowed_delegatees: Vec::new(),
    };
    
    let permissive_id = permissive_bundle.id.clone();
    manager.register_bundle(permissive_bundle).unwrap();
    
    // Admin can delegate to anyone
    let result = manager.delegate_bundle(&permissive_id, &admin, &disallowed, current_time);
    assert!(result.is_ok());
    
    // Disallowed can sub-delegate
    let other = create_test_agent_id("other");
    let result = manager.delegate_bundle(&permissive_id, &disallowed, &other, current_time);
    assert!(result.is_ok());
} 