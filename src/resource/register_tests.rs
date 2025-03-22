#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::resource::register::{
        Register, RegisterId, RegisterContents, RegisterState, RegisterService,
        AuthorizationMethod, RegisterOperation, OperationType, TimeRange
    };
    use crate::resource::register_service::InMemoryRegisterService;
    use crate::types::ResourceId;
    use crate::tel::{Address, Domain};
    use crate::ast::{AstContext, AstNodeId};

    #[tokio::test]
    async fn test_register_creation() {
        // Create a register service
        let service = InMemoryRegisterService::new();
        
        // Create a register
        let owner: Address = "owner123".to_string();
        let domain: Domain = "test_domain".to_string();
        let contents = RegisterContents::string("test data".to_string());
        let auth = AuthorizationMethod::ZKProofAuthorization {
            verification_key: "test_key".to_string(),
            proof: vec![1, 2, 3],
        };
        
        let register_id = service.create_register(
            owner.clone(),
            domain.clone(),
            contents.clone(),
            auth,
            None,
        ).await.unwrap();
        
        // Verify the register exists
        let register = service.get_register(&register_id).await.unwrap();
        
        assert_eq!(register.register_id, register_id);
        assert_eq!(register.owner, owner);
        assert_eq!(register.domain, domain);
        assert!(matches!(register.contents, RegisterContents::String(ref s) if s == "test data"));
        assert_eq!(register.state, RegisterState::Active);
    }
    
    #[tokio::test]
    async fn test_register_update() {
        // Create a register service
        let service = InMemoryRegisterService::new();
        
        // Create a register
        let owner: Address = "owner123".to_string();
        let domain: Domain = "test_domain".to_string();
        let contents = RegisterContents::string("initial data".to_string());
        let auth = AuthorizationMethod::ZKProofAuthorization {
            verification_key: "test_key".to_string(),
            proof: vec![1, 2, 3],
        };
        
        let register_id = service.create_register(
            owner.clone(),
            domain.clone(),
            contents.clone(),
            auth.clone(),
            None,
        ).await.unwrap();
        
        // Update the register
        let new_contents = RegisterContents::string("updated data".to_string());
        service.update_register(
            &register_id,
            new_contents.clone(),
            auth,
            None,
        ).await.unwrap();
        
        // Verify the register was updated
        let register = service.get_register(&register_id).await.unwrap();
        
        assert_eq!(register.register_id, register_id);
        assert!(matches!(register.contents, RegisterContents::String(ref s) if s == "updated data"));
        assert_eq!(register.state, RegisterState::Active);
    }
    
    #[tokio::test]
    async fn test_register_transfer() {
        // Create a register service
        let service = InMemoryRegisterService::new();
        
        // Create a register
        let owner: Address = "owner123".to_string();
        let domain: Domain = "test_domain".to_string();
        let contents = RegisterContents::string("test data".to_string());
        let auth = AuthorizationMethod::ZKProofAuthorization {
            verification_key: "test_key".to_string(),
            proof: vec![1, 2, 3],
        };
        
        let register_id = service.create_register(
            owner.clone(),
            domain.clone(),
            contents.clone(),
            auth.clone(),
            None,
        ).await.unwrap();
        
        // Transfer the register
        let new_owner: Address = "new_owner456".to_string();
        service.transfer_register(
            &register_id,
            new_owner.clone(),
            auth,
            None,
        ).await.unwrap();
        
        // Verify the register was transferred
        let register = service.get_register(&register_id).await.unwrap();
        
        assert_eq!(register.register_id, register_id);
        assert_eq!(register.owner, new_owner);
        assert_eq!(register.state, RegisterState::Active);
        
        // Verify we can get the register by the new owner
        let registers = service.get_registers_by_owner(&new_owner).await.unwrap();
        assert_eq!(registers.len(), 1);
        assert_eq!(registers[0].register_id, register_id);
    }
    
    #[tokio::test]
    async fn test_register_delete() {
        // Create a register service
        let service = InMemoryRegisterService::new();
        
        // Create a register
        let owner: Address = "owner123".to_string();
        let domain: Domain = "test_domain".to_string();
        let contents = RegisterContents::string("test data".to_string());
        let auth = AuthorizationMethod::ZKProofAuthorization {
            verification_key: "test_key".to_string(),
            proof: vec![1, 2, 3],
        };
        
        let register_id = service.create_register(
            owner.clone(),
            domain.clone(),
            contents.clone(),
            auth.clone(),
            None,
        ).await.unwrap();
        
        // Delete the register
        service.delete_register(
            &register_id,
            auth,
            None,
        ).await.unwrap();
        
        // Verify the register is now a tombstone
        let register = service.get_register(&register_id).await.unwrap();
        
        assert_eq!(register.register_id, register_id);
        assert_eq!(register.state, RegisterState::Tombstone);
        
        // Verify we can't find the register by owner anymore
        let registers = service.get_registers_by_owner(&owner).await.unwrap();
        assert_eq!(registers.len(), 0);
    }
    
    #[tokio::test]
    async fn test_apply_operation() {
        // Create a register service
        let service = InMemoryRegisterService::new();
        
        // Create a create operation
        let contents = RegisterContents::string("operation data".to_string());
        let auth = AuthorizationMethod::ZKProofAuthorization {
            verification_key: "test_key".to_string(),
            proof: vec![1, 2, 3],
        };
        
        let create_operation = RegisterOperation {
            op_type: OperationType::CreateRegister,
            registers: vec![],
            new_contents: Some(contents.clone()),
            authorization: auth.clone(),
            proof: None,
            resource_delta: "delta1".to_string(),
            ast_context: None,
        };
        
        // Apply the create operation
        let result_ids = service.apply_operation(create_operation).await.unwrap();
        assert_eq!(result_ids.len(), 1);
        let register_id = result_ids[0];
        
        // Verify the register was created
        let register = service.get_register(&register_id).await.unwrap();
        assert!(matches!(register.contents, RegisterContents::String(ref s) if s == "operation data"));
        
        // Create an update operation
        let new_contents = RegisterContents::string("updated via operation".to_string());
        let update_operation = RegisterOperation {
            op_type: OperationType::UpdateRegister,
            registers: vec![register_id],
            new_contents: Some(new_contents.clone()),
            authorization: auth.clone(),
            proof: None,
            resource_delta: "delta2".to_string(),
            ast_context: None,
        };
        
        // Apply the update operation
        let result_ids = service.apply_operation(update_operation).await.unwrap();
        assert_eq!(result_ids.len(), 1);
        assert_eq!(result_ids[0], register_id);
        
        // Verify the register was updated
        let register = service.get_register(&register_id).await.unwrap();
        assert!(matches!(register.contents, RegisterContents::String(ref s) if s == "updated via operation"));
    }
    
    #[tokio::test]
    async fn test_ast_context_tracking() {
        // Create a register service
        let service = InMemoryRegisterService::new();
        
        // Create a register with AST context
        let owner: Address = "owner123".to_string();
        let domain: Domain = "test_domain".to_string();
        let contents = RegisterContents::string("test data".to_string());
        let auth = AuthorizationMethod::SimpleAuthorization { 
            key: "test_key".to_string() 
        };
        
        let ast_node_id = AstNodeId::new("test_node".to_string());
        let ast_context = AstContext::new(ast_node_id);
        
        let register_id = service.create_register(
            owner.clone(),
            domain.clone(),
            contents.clone(),
            auth,
            Some(ast_context.clone()),
        ).await.unwrap();
        
        // Verify we can get the register by AST context
        let registers = service.get_registers_by_ast_context(&ast_context).await.unwrap();
        assert_eq!(registers.len(), 1);
        assert_eq!(registers[0].register_id, register_id);
    }
    
    #[tokio::test]
    async fn test_register_content_types() {
        // Create a register service
        let service = InMemoryRegisterService::new();
        let owner: Address = "owner123".to_string();
        let domain: Domain = "test_domain".to_string();
        let auth = AuthorizationMethod::ZKProofAuthorization {
            verification_key: "test_key".to_string(),
            proof: vec![1, 2, 3],
        };
        
        // Test binary contents
        let binary_contents = RegisterContents::binary(vec![1, 2, 3, 4]);
        let binary_id = service.create_register(
            owner.clone(), domain.clone(), binary_contents, auth.clone(), None
        ).await.unwrap();
        
        // Test JSON contents
        let json_value = serde_json::json!({
            "name": "test",
            "value": 42
        });
        let json_contents = RegisterContents::json(json_value.clone());
        let json_id = service.create_register(
            owner.clone(), domain.clone(), json_contents, auth.clone(), None
        ).await.unwrap();
        
        // Test token balance contents
        let token_contents = RegisterContents::token_balance(
            "ETH".to_string(), owner.clone(), 1000
        );
        let token_id = service.create_register(
            owner.clone(), domain.clone(), token_contents, auth.clone(), None
        ).await.unwrap();
        
        // Verify all the registers with different content types
        let binary_register = service.get_register(&binary_id).await.unwrap();
        let json_register = service.get_register(&json_id).await.unwrap();
        let token_register = service.get_register(&token_id).await.unwrap();
        
        match binary_register.contents {
            RegisterContents::Binary(ref data) => assert_eq!(data, &vec![1, 2, 3, 4]),
            _ => panic!("Expected Binary content"),
        }
        
        match json_register.contents {
            RegisterContents::Json(ref data) => assert_eq!(data, &json_value),
            _ => panic!("Expected Json content"),
        }
        
        match token_register.contents {
            RegisterContents::TokenBalance { ref token_type, ref address, amount } => {
                assert_eq!(token_type, "ETH");
                assert_eq!(address, &owner);
                assert_eq!(amount, 1000);
            },
            _ => panic!("Expected TokenBalance content"),
        }
    }
    
    #[tokio::test]
    async fn test_register_lifetime() {
        // Create a register
        let register_id = RegisterId::new();
        let owner: Address = "owner123".to_string();
        let domain: Domain = "test_domain".to_string();
        let contents = RegisterContents::string("test data".to_string());
        let tx_id = "tx123".to_string();
        
        let mut register = Register::new(
            register_id,
            owner.clone(),
            domain.clone(),
            contents.clone(),
            1, // epoch
            tx_id.clone(),
        );
        
        // Verify initial state
        assert_eq!(register.state, RegisterState::Active);
        assert!(register.state.is_active());
        assert!(register.consumed_by_tx.is_none());
        assert!(register.successors.is_empty());
        
        // Test consuming the register
        let successor_id = RegisterId::new();
        register.consume("tx456".to_string(), vec![successor_id]).unwrap();
        
        assert_eq!(register.state, RegisterState::Consumed);
        assert!(register.state.is_consumed());
        assert_eq!(register.consumed_by_tx, Some("tx456".to_string()));
        assert_eq!(register.successors.len(), 1);
        assert_eq!(register.successors[0], successor_id);
        
        // Create a new register and test archiving
        let mut register2 = Register::new(
            RegisterId::new(),
            owner.clone(),
            domain.clone(),
            contents.clone(),
            1, // epoch
            tx_id.clone(),
        );
        
        register2.archive("archive-ref-123".to_string()).unwrap();
        
        assert_eq!(register2.state, RegisterState::Archived);
        assert!(register2.state.is_archived());
        assert_eq!(register2.archive_reference, Some("archive-ref-123".to_string()));
    }
    
    #[tokio::test]
    async fn test_time_range() {
        // Test creating a time range
        let time_range = TimeRange::new(100, Some(200));
        
        assert_eq!(time_range.start, 100);
        assert_eq!(time_range.end, Some(200));
        
        // Test containment
        assert!(!time_range.contains(99));
        assert!(time_range.contains(100));
        assert!(time_range.contains(150));
        assert!(!time_range.contains(200));
        assert!(!time_range.contains(201));
        
        // Test creating from now
        let now_range = TimeRange::from_now(Some(1000)); // 1 second
        
        assert!(now_range.end.is_some());
        assert!(!now_range.is_expired()); // Should not be expired yet
        
        // Test infinite range
        let infinite_range = TimeRange::new(100, None);
        
        assert_eq!(infinite_range.start, 100);
        assert_eq!(infinite_range.end, None);
        
        assert!(!infinite_range.contains(99));
        assert!(infinite_range.contains(100));
        assert!(infinite_range.contains(1000000));
        assert!(!infinite_range.is_expired()); // Infinite ranges never expire
    }
} 