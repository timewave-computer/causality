//! Examples for Domain Effects
//!
//! This module contains examples of how to use domain effects.

/// Example showing how to query facts from a domain adapter
pub async fn query_domain_example() {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use causality_domain::domain::DomainId;
    use causality_domain::fact::FactQuery;
    
    use crate::effect::EffectContext;
    use crate::domain_effect::{
        DomainQueryEffect, query_domain_fact,
        create_domain_handler_with_new_registry,
        EffectDomainRegistry
    };
    use crate::handler::EffectHandler;
    
    // Create a new registry and handler
    let (registry, handler) = create_domain_handler_with_new_registry();
    
    // Register your domain adapter factories here
    // registry.register_factory(Arc::new(YourDomainAdapterFactory::new()));
    
    // Define a domain ID
    let domain_id = "ethereum:mainnet".to_string();
    
    // Create a domain query effect
    let effect = query_domain_fact(domain_id, "balance")
        .with_parameter("address", "0x123456789abcdef");
    
    // Create an effect context
    let context = EffectContext::new()
        .with_param("requester", "user_123");
    
    // Execute the effect
    let result = handler.handle(Arc::new(effect), &context).await;
    
    // Process the result
    match result {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Query succeeded: {:?}", outcome.data);
            
            // Access the balance value
            if let Some(balance) = outcome.data.get("balance") {
                println!("Balance: {}", balance);
            }
        },
        crate::handler::HandlerResult::Error(err) => {
            println!("Query failed: {:?}", err);
        },
        _ => {
            println!("Handler couldn't process the effect");
        }
    }
}

/// Example showing how to submit a transaction to a domain
pub async fn submit_transaction_example() {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use causality_domain::domain::{DomainId, Transaction};
    
    use crate::effect::EffectContext;
    use crate::domain_effect::{
        DomainTransactionEffect, submit_domain_transaction,
        create_domain_handler_with_new_registry
    };
    use crate::handler::EffectHandler;
    
    // Create a new registry and handler
    let (registry, handler) = create_domain_handler_with_new_registry();
    
    // Register your domain adapter factories here
    // registry.register_factory(Arc::new(YourDomainAdapterFactory::new()));
    
    // Define a domain ID
    let domain_id = "ethereum:goerli".to_string();
    
    // Create a domain transaction effect
    let mut effect = submit_domain_transaction(domain_id, "transfer")
        .with_parameter("to", "0x123456789abcdef")
        .with_parameter("amount", "1000000000000000000") // 1 ETH
        .with_confirmation(true); // Wait for confirmation
    
    // Create an effect context
    let context = EffectContext::new()
        .with_param("requester", "user_123");
    
    // Execute the effect
    let result = handler.handle(Arc::new(effect), &context).await;
    
    // Process the result
    match result {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Transaction submitted: {:?}", outcome.data);
            
            // Access the transaction ID
            if let Some(tx_id) = outcome.data.get("transaction_id") {
                println!("Transaction ID: {}", tx_id);
            }
        },
        crate::handler::HandlerResult::Error(err) => {
            println!("Transaction failed: {:?}", err);
        },
        _ => {
            println!("Handler couldn't process the effect");
        }
    }
}

/// Example showing how to check domain capabilities
pub async fn check_capability_example() {
    use std::sync::Arc;
    
    use causality_domain::domain::DomainId;
    
    use crate::effect::EffectContext;
    use crate::domain_effect::{
        check_domain_capability,
        create_domain_handler_with_new_registry
    };
    use crate::handler::EffectHandler;
    
    // Create a new registry and handler
    let (registry, handler) = create_domain_handler_with_new_registry();
    
    // Register your domain adapter factories here
    // registry.register_factory(Arc::new(YourDomainAdapterFactory::new()));
    
    // Define a domain ID
    let domain_id = "solana:mainnet".to_string();
    
    // Create capability check effects for different capabilities
    let effect1 = check_domain_capability(domain_id.clone(), "tokens");
    let effect2 = check_domain_capability(domain_id.clone(), "smart_contracts");
    let effect3 = check_domain_capability(domain_id.clone(), "streaming_payments");
    
    // Create an effect context
    let context = EffectContext::new();
    
    // Execute the effects and print results
    for (i, effect) in [effect1, effect2, effect3].iter().enumerate() {
        let result = handler.handle(Arc::new(effect.clone()), &context).await;
        
        match result {
            crate::handler::HandlerResult::Handled(outcome) => {
                if let Some(has_capability) = outcome.data.get("has_capability") {
                    println!("Capability {} check: {}", effect.capability(), has_capability);
                }
            },
            _ => {
                println!("Capability check failed");
            }
        }
    }
}

/// Example showing how to integrate domain effects with your application
pub async fn integration_example() {
    use std::sync::Arc;
    
    // Import types we need
    use crate::effect::{EffectManager, EffectContext, EffectRegistry};
    use crate::domain_effect::{
        DomainEffectHandlerAdapter, EffectDomainRegistry,
        query_domain_fact, submit_domain_transaction, check_domain_capability
    };
    use crate::handler::EffectHandler;
    
    // Create domain registry
    let domain_registry = Arc::new(EffectDomainRegistry::new());
    
    // Create domain handler
    let domain_handler = Arc::new(DomainEffectHandlerAdapter::new(domain_registry.clone()));
    
    // Create effect manager
    let mut effect_manager = EffectManager::new();
    
    // Register domain handlers
    // (Note: This is hypothetical as the real EffectManager might have different registration methods)
    // effect_manager.register_handler(domain_handler);
    
    // Define a domain ID
    let domain_id = "eth:mainnet".to_string();
    
    // Create domain effects
    let query_effect = query_domain_fact(domain_id.clone(), "balance")
        .with_parameter("address", "0xabcdef123456789");
        
    let tx_effect = submit_domain_transaction(domain_id.clone(), "transfer")
        .with_parameter("to", "0x987654321fedcba")
        .with_parameter("amount", "5000000000000000000"); // 5 ETH
        
    let capability_effect = check_domain_capability(domain_id.clone(), "tokens");
    
    // Create context
    let context = EffectContext::new()
        .with_param("user_id", "alice");
    
    // Execute effects directly with the handler
    println!("Executing domain query effect...");
    let query_result = domain_handler.handle(Arc::new(query_effect), &context).await;
    
    println!("Executing domain transaction effect...");
    let tx_result = domain_handler.handle(Arc::new(tx_effect), &context).await;
    
    println!("Executing domain capability effect...");
    let capability_result = domain_handler.handle(Arc::new(capability_effect), &context).await;
    
    // Process results
    println!("Query result: {:?}", query_result);
    println!("Transaction result: {:?}", tx_result);
    println!("Capability check result: {:?}", capability_result);
}

/// Example showing how to use domain selection effects
pub async fn domain_selection_example() {
    use std::sync::Arc;
    
    use crate::effect::EffectContext;
    use crate::domain_effect::{
        select_domains_by_type, select_domains_by_capability,
        select_domains_by_name, create_domain_handler_with_new_registry
    };
    use crate::handler::EffectHandler;
    
    // Create a new registry and handler
    let (registry, handler) = create_domain_handler_with_new_registry();
    
    // Register your domain adapter factories here
    // registry.register_factory(Arc::new(YourDomainAdapterFactory::new()));
    
    // Create an effect context
    let context = EffectContext::new();
    
    // Example 1: Find all Ethereum domains
    let effect1 = select_domains_by_type("ethereum")
        .with_parameter("include_testnets", "true");
    
    // Example 2: Find domains with smart contract capability
    let effect2 = select_domains_by_capability("smart_contracts");
    
    // Example 3: Find domains by name pattern
    let effect3 = select_domains_by_name("mainnet");
    
    // Execute the effects
    println!("Searching for Ethereum domains:");
    match handler.handle(Arc::new(effect1), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            let count = outcome.data.get("count").unwrap_or(&"0".to_string()).parse::<usize>().unwrap_or(0);
            println!("Found {} Ethereum domains", count);
            
            for i in 0..count {
                let id = outcome.data.get(&format!("domain_{}_id", i)).unwrap_or(&"unknown".to_string());
                let name = outcome.data.get(&format!("domain_{}_name", i)).unwrap_or(&"unknown".to_string());
                println!("  Domain: {} ({})", name, id);
            }
        },
        _ => println!("No Ethereum domains found or search failed"),
    }
    
    println!("\nSearching for domains with smart contract capability:");
    match handler.handle(Arc::new(effect2), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            let count = outcome.data.get("count").unwrap_or(&"0".to_string()).parse::<usize>().unwrap_or(0);
            println!("Found {} domains with smart contract capability", count);
            
            for i in 0..count {
                let id = outcome.data.get(&format!("domain_{}_id", i)).unwrap_or(&"unknown".to_string());
                let type_name = outcome.data.get(&format!("domain_{}_type", i)).unwrap_or(&"unknown".to_string());
                println!("  Domain: {} ({})", id, type_name);
            }
        },
        _ => println!("No domains with smart contract capability found or search failed"),
    }
    
    println!("\nSearching for mainnet domains:");
    match handler.handle(Arc::new(effect3), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            let count = outcome.data.get("count").unwrap_or(&"0".to_string()).parse::<usize>().unwrap_or(0);
            println!("Found {} mainnet domains", count);
            
            for i in 0..count {
                let id = outcome.data.get(&format!("domain_{}_id", i)).unwrap_or(&"unknown".to_string());
                let type_name = outcome.data.get(&format!("domain_{}_type", i)).unwrap_or(&"unknown".to_string());
                println!("  Domain: {} ({})", id, type_name);
            }
        },
        _ => println!("No mainnet domains found or search failed"),
    }
}

/// Example showing how to use EVM-specific effects
pub async fn evm_effects_example() {
    use std::sync::Arc;
    
    use crate::effect::EffectContext;
    use crate::domain_effect::{
        create_domain_handler_with_new_registry,
        evm_view_call, evm_transaction_call, evm_balance, evm_storage, evm_estimate_gas
    };
    use crate::handler::EffectHandler;
    
    // Create a new registry and handler
    let (registry, handler) = create_domain_handler_with_new_registry();
    
    // Register your domain adapter factories here
    // registry.register_factory(Arc::new(YourEvmAdapterFactory::new()));
    
    // Create an effect context
    let context = EffectContext::new();
    
    // Define a domain ID for Ethereum mainnet
    let domain_id = "ethereum:mainnet".to_string();
    
    // Create an ERC-20 token contract address
    let erc20_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH contract
    
    // Example 1: Query token balance using view call
    println!("Example 1: Query ERC-20 token balance");
    let balance_of_func = evm_view_call(
        domain_id.clone(),
        erc20_address,
        "balanceOf(address)",
        vec!["0x1234567890123456789012345678901234567890".to_string()]
    );
    
    match handler.handle(Arc::new(balance_of_func), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Token balance: {}", outcome.data.get("result").unwrap_or(&"0".to_string()));
        },
        _ => println!("Failed to get token balance"),
    }
    
    // Example 2: Get ETH balance directly
    println!("\nExample 2: Query ETH balance");
    let eth_balance = evm_balance(
        domain_id.clone(), 
        "0x1234567890123456789012345678901234567890"
    );
    
    match handler.handle(Arc::new(eth_balance), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("ETH balance: {}", outcome.data.get("result").unwrap_or(&"0".to_string()));
        },
        _ => println!("Failed to get ETH balance"),
    }
    
    // Example 3: Estimate gas for a transaction
    println!("\nExample 3: Estimate gas for token transfer");
    let gas_estimate = evm_estimate_gas(
        domain_id.clone(),
        erc20_address,
        "transfer(address,uint256)",
        vec![
            "0x1234567890123456789012345678901234567890".to_string(),
            "1000000000000000000".to_string() // 1 token with 18 decimals
        ]
    ).with_from("0x9876543210987654321098765432109876543210");
    
    match handler.handle(Arc::new(gas_estimate), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Estimated gas: {}", outcome.data.get("gas_estimate").unwrap_or(&"0".to_string()));
        },
        _ => println!("Failed to estimate gas"),
    }
    
    // Example 4: Submit a transaction
    println!("\nExample 4: Submit token transfer transaction");
    let transfer_tx = evm_transaction_call(
        domain_id.clone(),
        erc20_address,
        "transfer(address,uint256)",
        vec![
            "0x1234567890123456789012345678901234567890".to_string(),
            "1000000000000000000".to_string() // 1 token with 18 decimals
        ]
    )
    .with_gas_limit("100000")
    .with_gas_price("20000000000"); // 20 gwei
    
    match handler.handle(Arc::new(transfer_tx), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Transaction submitted:");
            println!("  Success: {}", outcome.success);
            if let Some(tx_hash) = outcome.data.get("result") {
                println!("  Transaction hash: {}", tx_hash);
            }
        },
        _ => println!("Failed to submit transaction"),
    }
    
    // Example 5: Query contract storage directly
    println!("\nExample 5: Query contract storage");
    let storage_query = evm_storage(
        domain_id.clone(),
        erc20_address,
        "0x0" // First storage slot (typically token name in ERC-20)
    );
    
    match handler.handle(Arc::new(storage_query), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Storage value: {}", outcome.data.get("result").unwrap_or(&"0x0".to_string()));
        },
        _ => println!("Failed to query storage"),
    }
    
    println!("\nNote: In a real application, you would register an actual EVM domain adapter");
    println!("      These examples would fail without a properly registered adapter.");
}

/// Example showing how to use CosmWasm-specific effects
pub async fn cosmwasm_effects_example() {
    use std::sync::Arc;
    
    use crate::effect::EffectContext;
    use crate::domain_effect::{
        create_domain_handler_with_new_registry,
        cosmwasm_query, cosmwasm_execute, cosmwasm_instantiate, cosmwasm_upload
    };
    use crate::handler::EffectHandler;
    
    // Create a new registry and handler
    let (registry, handler) = create_domain_handler_with_new_registry();
    
    // Register your domain adapter factories here
    // registry.register_factory(Arc::new(YourCosmWasmAdapterFactory::new()));
    
    // Create an effect context
    let context = EffectContext::new();
    
    // Define a domain ID for a Cosmos chain
    let domain_id = "cosmos:osmosis-1".to_string();
    
    // Example 1: Query a CosmWasm contract
    println!("Example 1: Query CosmWasm contract");
    let query_msg = r#"{"balance":{"address":"osmo1..."}}"#;
    let query_effect = cosmwasm_query(
        domain_id.clone(),
        "osmo1abc...", // Contract address
        query_msg
    ).at_height(5000000);
    
    match handler.handle(Arc::new(query_effect), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Query result: {}", outcome.data.get("result").unwrap_or(&"{}".to_string()));
        },
        _ => println!("Failed to query contract"),
    }
    
    // Example 2: Execute a CosmWasm contract
    println!("\nExample 2: Execute CosmWasm contract");
    let execute_msg = r#"{"transfer":{"recipient":"osmo1xyz...","amount":"1000000"}}"#;
    let execute_effect = cosmwasm_execute(
        domain_id.clone(),
        "osmo1abc...", // Contract address
        execute_msg
    )
    .with_sender("osmo1sender...")
    .with_funds("uosmo", 1000u128) // Send 1000 microOSMO with the execution
    .with_gas_limit("200000");
    
    match handler.handle(Arc::new(execute_effect), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Execution successful:");
            println!("  Success: {}", outcome.success);
            if let Some(tx_hash) = outcome.data.get("tx_hash") {
                println!("  Transaction hash: {}", tx_hash);
            }
        },
        _ => println!("Failed to execute contract"),
    }
    
    // Example 3: Upload a CosmWasm contract
    println!("\nExample 3: Upload CosmWasm code");
    // In a real application, this would be read from a file
    let wasm_bytecode = "AGFzbQEAAAABpAI..."; // Base64 encoded WASM binary (truncated for example)
    let upload_effect = cosmwasm_upload(
        domain_id.clone(),
        wasm_bytecode
    )
    .with_sender("osmo1admin...")
    .with_gas_limit("1000000")
    .with_fee("5000", "uosmo");
    
    match handler.handle(Arc::new(upload_effect), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Upload successful:");
            if let Some(code_id) = outcome.data.get("code_id") {
                println!("  Code ID: {}", code_id);
            }
            if let Some(tx_hash) = outcome.data.get("tx_hash") {
                println!("  Transaction hash: {}", tx_hash);
            }
        },
        _ => println!("Failed to upload contract code"),
    }
    
    // Example 4: Instantiate a CosmWasm contract
    println!("\nExample 4: Instantiate CosmWasm contract");
    let code_id = 123; // The code ID from a previous upload
    let init_msg = r#"{"name":"My Token","symbol":"TKN","decimals":6,"initial_balances":[{"address":"osmo1owner...","amount":"1000000000"}]}"#;
    let instantiate_effect = cosmwasm_instantiate(
        domain_id.clone(),
        code_id,
        init_msg,
        "My CW20 Token" // Contract label
    )
    .with_sender("osmo1admin...")
    .with_admin("osmo1admin...") // Optional admin for contract migration
    .with_gas_limit("500000")
    .with_fee("10000", "uosmo");
    
    match handler.handle(Arc::new(instantiate_effect), &context).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            println!("Instantiation successful:");
            if let Some(contract_address) = outcome.data.get("contract_address") {
                println!("  Contract Address: {}", contract_address);
            }
            if let Some(tx_hash) = outcome.data.get("tx_hash") {
                println!("  Transaction hash: {}", tx_hash);
            }
        },
        _ => println!("Failed to instantiate contract"),
    }
    
    println!("\nNote: In a real application, you would register an actual CosmWasm domain adapter");
    println!("      These examples would fail without a properly registered adapter.");
}

/// Example showing how to use ZK-specific effects
pub async fn zk_effects_example() {
    // Create a new registry and handler
    let (registry, handler) = create_domain_handler_with_new_registry();
    
    // Define a domain ID for a ZK/Succinct chain
    let zk_domain_id = "zk:succinct:1";
    
    // Example 1: Generate a ZK proof
    println!("Generating a ZK proof...");
    let effect = zk_prove(zk_domain_id, "factorial_circuit", r#"{"n": 5}"#)
        .with_public_input("120") // 5! = 120
        .with_parameter("max_iterations", "100");
    
    match handler.handle(Arc::new(effect), &EffectContext::new()).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            if outcome.success {
                println!("Proof generation successful!");
                println!("Proof hash: {}", outcome.data.get("proof_hash").unwrap_or(&"unknown".to_string()));
            } else {
                println!("Proof generation failed: {}", outcome.error.unwrap_or_else(|| "unknown error".to_string()));
            }
        },
        crate::handler::HandlerResult::Error(err) => println!("Error: {}", err),
        crate::handler::HandlerResult::NotApplicable => println!("Handler could not handle the effect"),
    }
    
    // Example 2: Verify a ZK proof
    println!("\nVerifying a ZK proof...");
    let proof_hash = "proof123"; // In a real example, this would be from the previous step
    let effect = zk_verify(zk_domain_id, "factorial_vk", proof_hash)
        .with_public_input("120"); // 5! = 120
    
    match handler.handle(Arc::new(effect), &EffectContext::new()).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            if outcome.success {
                let verification_result = outcome.data.get("success").unwrap_or(&"false".to_string());
                println!("Proof verification result: {}", verification_result);
            } else {
                println!("Verification processing failed: {}", outcome.error.unwrap_or_else(|| "unknown error".to_string()));
            }
        },
        crate::handler::HandlerResult::Error(err) => println!("Error: {}", err),
        crate::handler::HandlerResult::NotApplicable => println!("Handler could not handle the effect"),
    }
    
    // Example 3: Create a witness for a circuit
    println!("\nCreating a witness...");
    let effect = zk_witness(zk_domain_id, "merkle_circuit", r#"{"leaves": ["a", "b", "c", "d"], "proof_path": [1, 0]}"#);
    
    match handler.handle(Arc::new(effect), &EffectContext::new()).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            if outcome.success {
                println!("Witness creation successful!");
                println!("Witness hash: {}", outcome.data.get("witness_hash").unwrap_or(&"unknown".to_string()));
            } else {
                println!("Witness creation failed: {}", outcome.error.unwrap_or_else(|| "unknown error".to_string()));
            }
        },
        crate::handler::HandlerResult::Error(err) => println!("Error: {}", err),
        crate::handler::HandlerResult::NotApplicable => println!("Handler could not handle the effect"),
    }
    
    // Example 4: Compose ZK proofs
    println!("\nComposing ZK proofs...");
    let effect = zk_compose(zk_domain_id, "recursive_circuit")
        .with_source_proof_hash("proof123")
        .with_source_proof_hash("proof456");
    
    match handler.handle(Arc::new(effect), &EffectContext::new()).await {
        crate::handler::HandlerResult::Handled(outcome) => {
            if outcome.success {
                println!("Proof composition successful!");
                println!("Result proof hash: {}", outcome.data.get("result_proof_hash").unwrap_or(&"unknown".to_string()));
            } else {
                println!("Proof composition failed: {}", outcome.error.unwrap_or_else(|| "unknown error".to_string()));
            }
        },
        crate::handler::HandlerResult::Error(err) => println!("Error: {}", err),
        crate::handler::HandlerResult::NotApplicable => println!("Handler could not handle the effect"),
    }
    
    println!("\nNote: These examples require a proper ZK domain adapter to be registered for the domain ID '{}'", zk_domain_id);
} 