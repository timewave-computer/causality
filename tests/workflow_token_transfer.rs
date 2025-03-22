use std::sync::{Arc, Mutex};

use causality::boundary::{
    annotation::{Boundary, BoundaryCrossing, BoundaryType},
    crossing::{AuthType, BoundarySafe, CrossingProtocol, VerificationResult},
    metrics::BoundaryMetrics,
    off_chain::{OffChainComponent, OffChainRequest, OffChainResponse, ServiceAvailability},
    on_chain::{ContractInterface, SubmitResult, TransactionReceipt, TransactionStatus},
    BoundarySystem,
};

// Import helper types from our mocks
use causality::tests::boundary_mocks::{
    MockComponentFactory, MockData, MockContract, MockStorageService,
    create_mock_auth_token, verify_mock_auth_token,
};

// Define token-specific data structures
#[derive(Debug, Clone)]
struct TokenData {
    token_id: String,
    amount: u64,
    owner: String,
    metadata: Option<String>,
}

impl BoundarySafe for TokenData {
    fn prepare_for_boundary(&self) -> Vec<u8> {
        // Serialize token data for boundary crossing
        format!("{}:{}:{}:{}",
            self.token_id,
            self.amount,
            self.owner,
            self.metadata.clone().unwrap_or_default()
        ).into_bytes()
    }

    fn from_boundary(data: Vec<u8>) -> Result<Self, String> {
        // Deserialize token data from boundary crossing
        let s = String::from_utf8(data).map_err(|e| e.to_string())?;
        let parts: Vec<&str> = s.split(':').collect();
        
        if parts.len() < 4 {
            return Err("Invalid TokenData format".to_string());
        }
        
        let token_id = parts[0].to_string();
        let amount = parts[1].parse::<u64>().map_err(|e| e.to_string())?;
        let owner = parts[2].to_string();
        let metadata = if parts[3].is_empty() { None } else { Some(parts[3].to_string()) };
        
        Ok(TokenData { token_id, amount, owner, metadata })
    }
}

// Token contract implementation for on-chain components
struct TokenContract {
    inner: MockContract,
    balances: Arc<Mutex<std::collections::HashMap<String, u64>>>,
}

impl TokenContract {
    fn new(name: &str) -> Self {
        TokenContract {
            inner: MockComponentFactory::create_mock_contract(name, 2, 0.95),
            balances: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    fn mint(&self, recipient: &str, amount: u64) -> Result<(), String> {
        let mut balances = self.balances.lock().unwrap();
        let current = balances.entry(recipient.to_string()).or_insert(0);
        *current += amount;
        
        // Store in the contract state as well
        let key = format!("balance:{}", recipient);
        self.inner.set_state(&key, amount.to_string().into_bytes());
        
        Ok(())
    }
    
    fn get_balance(&self, address: &str) -> u64 {
        let balances = self.balances.lock().unwrap();
        *balances.get(address).unwrap_or(&0)
    }
}

impl ContractInterface for TokenContract {
    fn contract_address(&self) -> &str {
        self.inner.contract_address()
    }

    fn contract_name(&self) -> &str {
        self.inner.contract_name()
    }

    fn call(&self, method: &str, params: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
        match method {
            "balanceOf" => {
                if params.is_empty() {
                    return Err("Missing address parameter".to_string());
                }
                let address = String::from_utf8(params[0].clone()).map_err(|e| e.to_string())?;
                let balance = self.get_balance(&address);
                Ok(balance.to_string().into_bytes())
            },
            "totalSupply" => {
                // Sum of all balances
                let balances = self.balances.lock().unwrap();
                let total: u64 = balances.values().sum();
                Ok(total.to_string().into_bytes())
            },
            _ => Err(format!("Unknown method: {}", method)),
        }
    }

    fn submit_transaction(&self, method: &str, params: Vec<Vec<u8>>, _value: Option<u64>) -> Result<SubmitResult, String> {
        match method {
            "transfer" => {
                if params.len() < 2 {
                    return Err("Missing recipient or amount parameters".to_string());
                }
                
                let to = String::from_utf8(params[0].clone()).map_err(|e| e.to_string())?;
                let amount_str = String::from_utf8(params[1].clone()).map_err(|e| e.to_string())?;
                let amount = amount_str.parse::<u64>().map_err(|e| e.to_string())?;
                
                // Submit transaction through the mock contract
                let tx_result = self.inner.submit_transaction(
                    "transfer",
                    vec![to.as_bytes().to_vec(), amount.to_string().into_bytes()],
                    None,
                )?;
                
                Ok(tx_result)
            },
            _ => Err(format!("Unknown method: {}", method)),
        }
    }

    fn get_transaction_status(&self, transaction_id: &str) -> Result<TransactionStatus, String> {
        self.inner.get_transaction_status(transaction_id)
    }
}

// User wallet service implementation for off-chain components
struct UserWalletService {
    inner: MockStorageService,
    balances: Arc<Mutex<std::collections::HashMap<String, std::collections::HashMap<String, u64>>>>,
}

impl UserWalletService {
    fn new() -> Self {
        UserWalletService {
            inner: MockComponentFactory::create_storage_service(5, 0.01), // 5ms latency, 1% error rate
            balances: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    fn update_balance(&self, user_id: &str, token_id: &str, amount: u64) -> Result<(), String> {
        let mut balances = self.balances.lock().unwrap();
        let user_balances = balances.entry(user_id.to_string()).or_insert_with(std::collections::HashMap::new);
        let current = user_balances.entry(token_id.to_string()).or_insert(0);
        *current = amount;
        Ok(())
    }
    
    fn get_balance(&self, user_id: &str, token_id: &str) -> u64 {
        let balances = self.balances.lock().unwrap();
        balances.get(user_id)
            .and_then(|user_balances| user_balances.get(token_id))
            .copied()
            .unwrap_or(0)
    }
}

impl OffChainComponent for UserWalletService {
    fn initialize(&mut self) -> Result<(), String> {
        self.inner.initialize()
    }

    fn check_availability(&self) -> ServiceAvailability {
        self.inner.check_availability()
    }

    fn execute_request(&self, request: OffChainRequest) -> Result<OffChainResponse, String> {
        match request {
            OffChainRequest::Custom(name, payload) => {
                match name.as_str() {
                    "get_balance" => {
                        let data = String::from_utf8(payload).map_err(|e| e.to_string())?;
                        let parts: Vec<&str> = data.split(':').collect();
                        
                        if parts.len() < 2 {
                            return Err("Invalid payload format, expected user_id:token_id".to_string());
                        }
                        
                        let user_id = parts[0];
                        let token_id = parts[1];
                        
                        let balance = self.get_balance(user_id, token_id);
                        Ok(OffChainResponse::Success(balance.to_string().into_bytes()))
                    },
                    "update_balance" => {
                        let data = String::from_utf8(payload).map_err(|e| e.to_string())?;
                        let parts: Vec<&str> = data.split(':').collect();
                        
                        if parts.len() < 3 {
                            return Err("Invalid payload format, expected user_id:token_id:amount".to_string());
                        }
                        
                        let user_id = parts[0];
                        let token_id = parts[1];
                        let amount = parts[2].parse::<u64>().map_err(|e| e.to_string())?;
                        
                        self.update_balance(user_id, token_id, amount)?;
                        Ok(OffChainResponse::Success(vec![1])) // Success
                    },
                    _ => self.inner.execute_request(request), // Fall back to inner service
                }
            },
            _ => self.inner.execute_request(request), // Fall back to inner service
        }
    }
}

// Token transfer service implementation
struct TokenTransferService {
    boundary_system: BoundarySystem,
    token_contract: Arc<TokenContract>,
    wallet_service: Arc<UserWalletService>,
}

impl TokenTransferService {
    fn new(boundary_system: BoundarySystem, token_contract: Arc<TokenContract>, wallet_service: Arc<UserWalletService>) -> Self {
        TokenTransferService {
            boundary_system,
            token_contract,
            wallet_service,
        }
    }
    
    // On-chain to off-chain transfer (withdraw)
    #[Boundary(BoundaryType::OnChain)]
    fn withdraw_tokens(&self, token_data: &TokenData, auth_token: Option<Vec<u8>>) -> Result<(), String> {
        println!("Initiating withdrawal of {} tokens for user {}", token_data.amount, token_data.owner);
        
        // Verify on-chain balance
        let on_chain_balance = self.token_contract.get_balance(&token_data.owner);
        if on_chain_balance < token_data.amount {
            return Err(format!("Insufficient balance: {} < {}", on_chain_balance, token_data.amount));
        }
        
        // Submit transaction to transfer tokens to contract
        let tx_result = self.token_contract.submit_transaction(
            "transfer",
            vec![
                self.token_contract.contract_address().as_bytes().to_vec(),
                token_data.amount.to_string().into_bytes(),
            ],
            None,
        )?;
        
        println!("On-chain withdrawal transaction submitted: {}", tx_result.transaction_id);
        
        // Wait for confirmation (this would be async in a real implementation)
        let mut status = self.token_contract.get_transaction_status(&tx_result.transaction_id)?;
        println!("Initial transaction status: {:?}", status);
        
        // Simple polling to check confirmation
        for _ in 0..5 {
            if let TransactionStatus::Confirmed(_) = status {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            status = self.token_contract.get_transaction_status(&tx_result.transaction_id)?;
            println!("Updated transaction status: {:?}", status);
        }
        
        if !matches!(status, TransactionStatus::Confirmed(_)) {
            return Err("Transaction did not confirm within timeout".to_string());
        }
        
        // Cross boundary to update off-chain balance
        let protocol = CrossingProtocol {
            from_boundary: BoundaryType::OnChain,
            to_boundary: BoundaryType::OffChain,
            auth_type: match auth_token {
                Some(_) => AuthType::Token,
                None => AuthType::None,
            },
        };
        
        let crossed_data = self.boundary_system.cross_boundary(token_data, protocol, auth_token)?;
        println!("Token data crossed from on-chain to off-chain: {:?}", crossed_data);
        
        // Update off-chain wallet balance
        let update_req = OffChainRequest::Custom(
            "update_balance".to_string(),
            format!("{}:{}:{}", crossed_data.owner, crossed_data.token_id, crossed_data.amount).into_bytes(),
        );
        
        let response = self.wallet_service.execute_request(update_req)?;
        println!("Off-chain wallet updated: {:?}", response);
        
        Ok(())
    }
    
    // Off-chain to on-chain transfer (deposit)
    #[Boundary(BoundaryType::OffChain)]
    fn deposit_tokens(&self, token_data: &TokenData, auth_token: Option<Vec<u8>>) -> Result<(), String> {
        println!("Initiating deposit of {} tokens for user {}", token_data.amount, token_data.owner);
        
        // Verify off-chain balance
        let balance_req = OffChainRequest::Custom(
            "get_balance".to_string(),
            format!("{}:{}", token_data.owner, token_data.token_id).into_bytes(),
        );
        
        let response = self.wallet_service.execute_request(balance_req)?;
        if let OffChainResponse::Success(data) = response {
            let off_chain_balance = String::from_utf8(data)
                .map_err(|e| e.to_string())?
                .parse::<u64>()
                .map_err(|e| e.to_string())?;
                
            if off_chain_balance < token_data.amount {
                return Err(format!("Insufficient off-chain balance: {} < {}", off_chain_balance, token_data.amount));
            }
        } else {
            return Err("Failed to get off-chain balance".to_string());
        }
        
        // Cross boundary to update on-chain balance
        let protocol = CrossingProtocol {
            from_boundary: BoundaryType::OffChain,
            to_boundary: BoundaryType::OnChain,
            auth_type: match auth_token {
                Some(_) => AuthType::Token,
                None => AuthType::None,
            },
        };
        
        let crossed_data = self.boundary_system.cross_boundary(token_data, protocol, auth_token)?;
        println!("Token data crossed from off-chain to on-chain: {:?}", crossed_data);
        
        // Mint tokens to the user on-chain (in a real system, this would release locked tokens)
        self.token_contract.mint(&crossed_data.owner, crossed_data.amount)?;
        println!("On-chain tokens minted to {}: {}", crossed_data.owner, crossed_data.amount);
        
        // Update off-chain wallet balance (subtract)
        let update_req = OffChainRequest::Custom(
            "update_balance".to_string(),
            format!("{}:{}:{}",
                crossed_data.owner,
                crossed_data.token_id,
                self.wallet_service.get_balance(&crossed_data.owner, &crossed_data.token_id) - crossed_data.amount
            ).into_bytes(),
        );
        
        let response = self.wallet_service.execute_request(update_req)?;
        println!("Off-chain wallet updated after deposit: {:?}", response);
        
        Ok(())
    }
    
    // Transfer tokens between users (off-chain)
    #[Boundary(BoundaryType::OffChain)]
    fn transfer_off_chain(&self, 
                         from_user: &str, 
                         to_user: &str, 
                         token_id: &str, 
                         amount: u64, 
                         auth_token: Option<Vec<u8>>) -> Result<(), String> {
        println!("Initiating off-chain transfer of {} tokens from {} to {}", amount, from_user, to_user);
        
        // Verify sender's off-chain balance
        let balance_req = OffChainRequest::Custom(
            "get_balance".to_string(),
            format!("{}:{}", from_user, token_id).into_bytes(),
        );
        
        let response = self.wallet_service.execute_request(balance_req)?;
        if let OffChainResponse::Success(data) = response {
            let off_chain_balance = String::from_utf8(data)
                .map_err(|e| e.to_string())?
                .parse::<u64>()
                .map_err(|e| e.to_string())?;
                
            if off_chain_balance < amount {
                return Err(format!("Insufficient off-chain balance: {} < {}", off_chain_balance, amount));
            }
        } else {
            return Err("Failed to get off-chain balance".to_string());
        }
        
        // Update sender's off-chain wallet (subtract)
        let sender_balance = self.wallet_service.get_balance(from_user, token_id);
        let update_sender_req = OffChainRequest::Custom(
            "update_balance".to_string(),
            format!("{}:{}:{}", from_user, token_id, sender_balance - amount).into_bytes(),
        );
        
        self.wallet_service.execute_request(update_sender_req)?;
        
        // Update recipient's off-chain wallet (add)
        let recipient_balance = self.wallet_service.get_balance(to_user, token_id);
        let update_recipient_req = OffChainRequest::Custom(
            "update_balance".to_string(),
            format!("{}:{}:{}", to_user, token_id, recipient_balance + amount).into_bytes(),
        );
        
        self.wallet_service.execute_request(update_recipient_req)?;
        
        println!("Off-chain transfer completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Define a macro to simplify running async tests
    macro_rules! workflow_test {
        ($name:ident, $body:expr) => {
            #[test]
            fn $name() {
                $body
            }
        };
    }
    
    // Helper function to set up test environment
    fn setup_token_transfer_environment() -> TokenTransferService {
        // Create boundary system with metrics collection
        let boundary_system = BoundarySystem::new(Some(Arc::new(Mutex::new(BoundaryMetrics::new()))));
        
        // Create on-chain token contract
        let token_contract = Arc::new(TokenContract::new("TestToken"));
        
        // Create off-chain wallet service
        let wallet_service = Arc::new(UserWalletService::new());
        
        // Create token transfer service
        TokenTransferService::new(boundary_system, token_contract, wallet_service)
    }
    
    workflow_test!(test_basic_token_deposit, {
        let transfer_service = setup_token_transfer_environment();
        
        // Initialize user with off-chain balance
        transfer_service.wallet_service.update_balance("user1", "TEST", 1000).unwrap();
        
        // Create auth token
        let auth_token = create_mock_auth_token("user1", 3600);
        
        // Create token data for deposit
        let token_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 500,
            owner: "user1".to_string(),
            metadata: None,
        };
        
        // Deposit tokens to on-chain
        let result = transfer_service.deposit_tokens(&token_data, Some(auth_token));
        assert!(result.is_ok(), "Token deposit failed: {:?}", result.err());
        
        // Verify on-chain balance
        let on_chain_balance = transfer_service.token_contract.get_balance("user1");
        assert_eq!(on_chain_balance, 500, "On-chain balance mismatch after deposit");
        
        // Verify off-chain balance
        let off_chain_balance = transfer_service.wallet_service.get_balance("user1", "TEST");
        assert_eq!(off_chain_balance, 500, "Off-chain balance mismatch after deposit");
    });
    
    workflow_test!(test_token_withdraw, {
        let transfer_service = setup_token_transfer_environment();
        
        // Initialize user with on-chain balance
        transfer_service.token_contract.mint("user1", 1000).unwrap();
        
        // Create auth token
        let auth_token = create_mock_auth_token("user1", 3600);
        
        // Create token data for withdrawal
        let token_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 700,
            owner: "user1".to_string(),
            metadata: None,
        };
        
        // Withdraw tokens from on-chain
        let result = transfer_service.withdraw_tokens(&token_data, Some(auth_token));
        assert!(result.is_ok(), "Token withdrawal failed: {:?}", result.err());
        
        // Verify balances
        let on_chain_balance = transfer_service.token_contract.get_balance("user1");
        let off_chain_balance = transfer_service.wallet_service.get_balance("user1", "TEST");
        
        assert_eq!(on_chain_balance, 300, "On-chain balance mismatch after withdrawal");
        assert_eq!(off_chain_balance, 700, "Off-chain balance mismatch after withdrawal");
    });
    
    workflow_test!(test_off_chain_transfer, {
        let transfer_service = setup_token_transfer_environment();
        
        // Initialize user1 with off-chain balance
        transfer_service.wallet_service.update_balance("user1", "TEST", 1000).unwrap();
        
        // Create auth token
        let auth_token = create_mock_auth_token("user1", 3600);
        
        // Transfer tokens off-chain from user1 to user2
        let result = transfer_service.transfer_off_chain(
            "user1", "user2", "TEST", 350, Some(auth_token)
        );
        
        assert!(result.is_ok(), "Off-chain transfer failed: {:?}", result.err());
        
        // Verify balances
        let user1_balance = transfer_service.wallet_service.get_balance("user1", "TEST");
        let user2_balance = transfer_service.wallet_service.get_balance("user2", "TEST");
        
        assert_eq!(user1_balance, 650, "User1 balance mismatch after transfer");
        assert_eq!(user2_balance, 350, "User2 balance mismatch after transfer");
    });
    
    workflow_test!(test_full_token_workflow, {
        let transfer_service = setup_token_transfer_environment();
        
        // Step 1: Initialize user1 with on-chain balance
        transfer_service.token_contract.mint("user1", 2000).unwrap();
        
        // Step 2: User1 withdraws tokens to off-chain wallet
        let auth_token = create_mock_auth_token("user1", 3600);
        let withdraw_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 1500,
            owner: "user1".to_string(),
            metadata: None,
        };
        
        let result = transfer_service.withdraw_tokens(&withdraw_data, Some(auth_token.clone()));
        assert!(result.is_ok(), "Step 2 withdrawal failed: {:?}", result.err());
        
        // Step 3: User1 transfers tokens to user2 off-chain
        let result = transfer_service.transfer_off_chain(
            "user1", "user2", "TEST", 800, Some(auth_token.clone())
        );
        assert!(result.is_ok(), "Step 3 off-chain transfer failed: {:?}", result.err());
        
        // Step 4: User2 deposits tokens to their on-chain account
        let auth_token2 = create_mock_auth_token("user2", 3600);
        let deposit_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 500,
            owner: "user2".to_string(),
            metadata: None,
        };
        
        let result = transfer_service.deposit_tokens(&deposit_data, Some(auth_token2));
        assert!(result.is_ok(), "Step 4 deposit failed: {:?}", result.err());
        
        // Step 5: User1 deposits remaining tokens back to on-chain
        let deposit_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 700,
            owner: "user1".to_string(),
            metadata: None,
        };
        
        let result = transfer_service.deposit_tokens(&deposit_data, Some(auth_token));
        assert!(result.is_ok(), "Step 5 deposit failed: {:?}", result.err());
        
        // Verify final balances
        let user1_on_chain = transfer_service.token_contract.get_balance("user1");
        let user2_on_chain = transfer_service.token_contract.get_balance("user2");
        let user1_off_chain = transfer_service.wallet_service.get_balance("user1", "TEST");
        let user2_off_chain = transfer_service.wallet_service.get_balance("user2", "TEST");
        
        assert_eq!(user1_on_chain, 1200, "User1 on-chain balance mismatch");
        assert_eq!(user2_on_chain, 500, "User2 on-chain balance mismatch");
        assert_eq!(user1_off_chain, 0, "User1 off-chain balance mismatch");
        assert_eq!(user2_off_chain, 300, "User2 off-chain balance mismatch");
        
        // Verify total supply conservation (2000 tokens throughout the workflow)
        let total_tokens = user1_on_chain + user2_on_chain + user1_off_chain + user2_off_chain;
        assert_eq!(total_tokens, 2000, "Token conservation violated");
    });
    
    workflow_test!(test_failed_operations, {
        let transfer_service = setup_token_transfer_environment();
        
        // Initialize user with insufficient balance
        transfer_service.wallet_service.update_balance("user1", "TEST", 100).unwrap();
        transfer_service.token_contract.mint("user1", 200).unwrap();
        
        // Attempt to deposit more than available
        let auth_token = create_mock_auth_token("user1", 3600);
        let deposit_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 500,
            owner: "user1".to_string(),
            metadata: None,
        };
        
        let result = transfer_service.deposit_tokens(&deposit_data, Some(auth_token.clone()));
        assert!(result.is_err(), "Deposit should fail with insufficient balance");
        
        // Attempt to withdraw more than available
        let withdraw_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 500,
            owner: "user1".to_string(),
            metadata: None,
        };
        
        let result = transfer_service.withdraw_tokens(&withdraw_data, Some(auth_token.clone()));
        assert!(result.is_err(), "Withdrawal should fail with insufficient balance");
        
        // Attempt to use expired auth token
        let expired_token = create_mock_auth_token("user1", 0);
        std::thread::sleep(std::time::Duration::from_secs(1)); // Ensure token expires
        
        let withdraw_data = TokenData {
            token_id: "TEST".to_string(),
            amount: 100,
            owner: "user1".to_string(),
            metadata: None,
        };
        
        let result = transfer_service.withdraw_tokens(&withdraw_data, Some(expired_token));
        // The error may come from different places depending on implementation
        assert!(result.is_err(), "Operation with expired token should fail");
        
        // Verify balances haven't changed
        let on_chain_balance = transfer_service.token_contract.get_balance("user1");
        let off_chain_balance = transfer_service.wallet_service.get_balance("user1", "TEST");
        
        assert_eq!(on_chain_balance, 200, "On-chain balance should be unchanged");
        assert_eq!(off_chain_balance, 100, "Off-chain balance should be unchanged");
    });
} 