use std::collections::HashMap;
use std::sync::Arc;

use crate::address::Address;
use crate::program_account::{
    ProgramAccount, ProgramAccountRegistry, 
    AssetProgramAccount, UtilityProgramAccount, 
    AccountType, TransactionStatus,
    ProgramAccountResource, AvailableEffect, EffectParameter,
    ProgramAccountView, ResourceView, BalanceView, CapabilityView, 
    EffectView, TransactionView, ParameterView,
    ViewTransformer, ProgramAccountViewTransformer,
    serialization::*,
};
use crate::resource::{RegisterId, ResourceCapability};

// Mock implementations for testing
struct MockAssetAccount {
    id: Address,
    owner: Address,
    name: String,
    account_type: AccountType,
    domains: Vec<Address>,
    metadata: HashMap<String, String>,
    assets: HashMap<String, u64>,
    asset_types: HashMap<String, String>,
    asset_info: HashMap<String, AssetInfo>,
    resources: Vec<ProgramAccountResource>,
    capabilities: Vec<AccountCapability>,
    effects: Vec<AvailableEffect>,
    transactions: Vec<AccountTransaction>,
}

struct AssetInfo {
    name: String,
    symbol: Option<String>,
    decimals: Option<u8>,
    icon_url: Option<String>,
}

struct AccountCapability {
    id: String,
    action: String,
    target: Option<String>,
    restrictions: HashMap<String, String>,
    expires_at: Option<u64>,
}

struct AccountTransaction {
    id: String,
    transaction_type: String,
    timestamp: u64,
    status: TransactionStatus,
    resources: Vec<String>,
    effects: Vec<String>,
    domains: Vec<String>,
    metadata: HashMap<String, String>,
    hash: Option<String>,
    block_number: Option<u64>,
    fee: Option<TransactionFee>,
}

struct TransactionFee {
    amount: u64,
    currency: String,
    gas_price: Option<u64>,
    gas_used: Option<u64>,
}

impl ProgramAccount for MockAssetAccount {
    fn id(&self) -> &Address {
        &self.id
    }

    fn owner(&self) -> &Address {
        &self.owner
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn account_type(&self) -> AccountType {
        self.account_type.clone()
    }

    fn supported_domains(&self) -> &[Address] {
        &self.domains
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn resources(&self) -> &[ProgramAccountResource] {
        &self.resources
    }

    fn capabilities(&self) -> &[AccountCapability] {
        &self.capabilities
    }

    fn available_effects(&self) -> &[AvailableEffect] {
        &self.effects
    }

    fn transaction_history(&self) -> &[AccountTransaction] {
        &self.transactions
    }

    fn as_asset_account(&self) -> Option<&dyn AssetProgramAccount> {
        Some(self as &dyn AssetProgramAccount)
    }

    fn as_utility_account(&self) -> Option<&dyn UtilityProgramAccount> {
        None
    }

    // Other trait methods would be implemented here...
}

impl AssetProgramAccount for MockAssetAccount {
    fn balances(&self) -> &HashMap<String, u64> {
        &self.assets
    }

    fn asset_type(&self, asset_id: &str) -> &str {
        self.asset_types.get(asset_id).map(|s| s.as_str()).unwrap_or("unknown")
    }

    fn asset_info(&self, asset_id: &str) -> Option<&AssetInfo> {
        self.asset_info.get(asset_id)
    }

    // Other trait methods would be implemented here...
}

#[test]
fn test_program_account_to_view_transformation() {
    // Create a mock asset account
    let mock_account = create_mock_asset_account();
    
    // Create transformer
    let transformer = ProgramAccountViewTransformer::new();
    
    // Transform to view
    let view = transformer.to_view(&mock_account as &dyn ProgramAccount);
    
    // Verify the view has the correct data
    assert_eq!(view.id, mock_account.id.to_string());
    assert_eq!(view.owner, mock_account.owner.to_string());
    assert_eq!(view.name, mock_account.name);
    assert_eq!(view.account_type, "Asset");
    assert_eq!(view.domains.len(), mock_account.domains.len());
    assert_eq!(view.balances.len(), mock_account.assets.len());
    assert_eq!(view.resources.len(), mock_account.resources.len());
    assert_eq!(view.capabilities.len(), mock_account.capabilities.len());
    assert_eq!(view.effects.len(), mock_account.effects.len());
    assert_eq!(view.transactions.len(), mock_account.transactions.len());
    
    // Verify specific fields
    let btc_balance = view.balances.iter().find(|b| b.asset_id == "btc").unwrap();
    assert_eq!(btc_balance.asset_name, "Bitcoin");
    assert_eq!(btc_balance.amount, "100");
    assert_eq!(btc_balance.asset_type, "cryptocurrency");
    assert_eq!(btc_balance.symbol, Some("BTC".to_string()));
    
    // Check resources
    let resource = &view.resources[0];
    assert_eq!(resource.resource_type, "document");
    assert_eq!(resource.state, "active");
    
    // Check capabilities
    let capability = &view.capabilities[0];
    assert_eq!(capability.action, "read");
    assert!(capability.target.is_some());
    
    // Check effects
    let effect = &view.effects[0];
    assert_eq!(effect.name, "Transfer");
    assert!(effect.available);
    assert_eq!(effect.parameters.len(), 2);
    
    // Check transactions
    let transaction = &view.transactions[0];
    assert_eq!(transaction.transaction_type, "transfer");
    assert_eq!(transaction.status, "confirmed");
}

#[test]
fn test_serialization_and_deserialization() {
    // Create a mock asset account
    let mock_account = create_mock_asset_account();
    
    // Create transformer
    let transformer = ProgramAccountViewTransformer::new();
    
    // Transform to view
    let view = transformer.to_view(&mock_account as &dyn ProgramAccount);
    
    // Serialize to JSON
    let json = to_json(&view).unwrap();
    
    // Deserialize back to view
    let deserialized_view: ProgramAccountView = from_json(&json).unwrap();
    
    // Verify the deserialized view matches the original
    assert_eq!(deserialized_view.id, view.id);
    assert_eq!(deserialized_view.name, view.name);
    assert_eq!(deserialized_view.account_type, view.account_type);
    assert_eq!(deserialized_view.balances.len(), view.balances.len());
    assert_eq!(deserialized_view.resources.len(), view.resources.len());
    assert_eq!(deserialized_view.effects.len(), view.effects.len());
    
    // Check pretty printing
    let pretty_json = to_pretty_json(&view).unwrap();
    assert!(pretty_json.contains("\n"));
    
    // Deserialize from pretty JSON
    let pretty_deserialized: ProgramAccountView = from_json(&pretty_json).unwrap();
    assert_eq!(pretty_deserialized.id, view.id);
}

// Helper function to create a mock asset account for testing
fn create_mock_asset_account() -> MockAssetAccount {
    let mut assets = HashMap::new();
    assets.insert("btc".to_string(), 100);
    assets.insert("eth".to_string(), 5000);
    
    let mut asset_types = HashMap::new();
    asset_types.insert("btc".to_string(), "cryptocurrency".to_string());
    asset_types.insert("eth".to_string(), "cryptocurrency".to_string());
    
    let mut asset_info = HashMap::new();
    asset_info.insert("btc".to_string(), AssetInfo {
        name: "Bitcoin".to_string(),
        symbol: Some("BTC".to_string()),
        decimals: Some(8),
        icon_url: Some("https://example.com/icons/btc.png".to_string()),
    });
    asset_info.insert("eth".to_string(), AssetInfo {
        name: "Ethereum".to_string(),
        symbol: Some("ETH".to_string()),
        decimals: Some(18),
        icon_url: Some("https://example.com/icons/eth.png".to_string()),
    });
    
    let resources = vec![
        ProgramAccountResource {
            id: "doc-1".to_string(),
            resource_type: "document".to_string(),
            domain: Some("domain1".to_string()),
            preview: Some("Document preview".to_string()),
            state: "active".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("created_at".to_string(), "2023-01-01".to_string());
                map
            },
        },
        ProgramAccountResource {
            id: "img-1".to_string(),
            resource_type: "image".to_string(),
            domain: Some("domain2".to_string()),
            preview: Some("Image thumbnail".to_string()),
            state: "active".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("created_at".to_string(), "2023-02-01".to_string());
                map
            },
        },
    ];
    
    let capabilities = vec![
        AccountCapability {
            id: "cap-1".to_string(),
            action: "read".to_string(),
            target: Some("doc-1".to_string()),
            restrictions: {
                let mut map = HashMap::new();
                map.insert("expires".to_string(), "2024-01-01".to_string());
                map
            },
            expires_at: Some(1704067200), // 2024-01-01
        },
        AccountCapability {
            id: "cap-2".to_string(),
            action: "write".to_string(),
            target: Some("img-1".to_string()),
            restrictions: HashMap::new(),
            expires_at: None,
        },
    ];
    
    let effects = vec![
        AvailableEffect {
            id: "effect-1".to_string(),
            name: "Transfer".to_string(),
            description: "Transfer asset to another account".to_string(),
            domain: Some("domain1".to_string()),
            parameters: vec![
                EffectParameter {
                    name: "recipient".to_string(),
                    param_type: "address".to_string(),
                    description: "Recipient address".to_string(),
                    required: true,
                    default_value: None,
                    constraints: HashMap::new(),
                },
                EffectParameter {
                    name: "amount".to_string(),
                    param_type: "number".to_string(),
                    description: "Amount to transfer".to_string(),
                    required: true,
                    default_value: None,
                    constraints: {
                        let mut map = HashMap::new();
                        map.insert("min".to_string(), "0".to_string());
                        map
                    },
                },
            ],
            available: true,
            requirements: vec!["minimum_balance".to_string()],
        },
        AvailableEffect {
            id: "effect-2".to_string(),
            name: "Swap".to_string(),
            description: "Swap one asset for another".to_string(),
            domain: Some("domain1".to_string()),
            parameters: vec![
                EffectParameter {
                    name: "from_asset".to_string(),
                    param_type: "string".to_string(),
                    description: "Asset to swap from".to_string(),
                    required: true,
                    default_value: None,
                    constraints: HashMap::new(),
                },
                EffectParameter {
                    name: "to_asset".to_string(),
                    param_type: "string".to_string(),
                    description: "Asset to swap to".to_string(),
                    required: true,
                    default_value: None,
                    constraints: HashMap::new(),
                },
                EffectParameter {
                    name: "amount".to_string(),
                    param_type: "number".to_string(),
                    description: "Amount to swap".to_string(),
                    required: true,
                    default_value: None,
                    constraints: {
                        let mut map = HashMap::new();
                        map.insert("min".to_string(), "0".to_string());
                        map
                    },
                },
            ],
            available: true,
            requirements: vec!["liquidity_available".to_string()],
        },
    ];
    
    let transactions = vec![
        AccountTransaction {
            id: "tx-1".to_string(),
            transaction_type: "transfer".to_string(),
            timestamp: 1672531200, // 2023-01-01
            status: TransactionStatus::Confirmed,
            resources: vec!["btc".to_string()],
            effects: vec!["effect-1".to_string()],
            domains: vec!["domain1".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("note".to_string(), "Payment for services".to_string());
                map
            },
            hash: Some("0x123456789abcdef".to_string()),
            block_number: Some(12345),
            fee: Some(TransactionFee {
                amount: 10,
                currency: "BTC".to_string(),
                gas_price: Some(5),
                gas_used: Some(2),
            }),
        },
        AccountTransaction {
            id: "tx-2".to_string(),
            transaction_type: "swap".to_string(),
            timestamp: 1675209600, // 2023-02-01
            status: TransactionStatus::Pending,
            resources: vec!["btc".to_string(), "eth".to_string()],
            effects: vec!["effect-2".to_string()],
            domains: vec!["domain1".to_string()],
            metadata: HashMap::new(),
            hash: None,
            block_number: None,
            fee: None,
        },
    ];
    
    MockAssetAccount {
        id: Address::new_random(),
        owner: Address::new_random(),
        name: "Test Asset Account".to_string(),
        account_type: AccountType::Asset,
        domains: vec![Address::new_random(), Address::new_random()],
        metadata: {
            let mut map = HashMap::new();
            map.insert("created_at".to_string(), "2023-01-01".to_string());
            map.insert("description".to_string(), "Test account for UI model testing".to_string());
            map
        },
        assets,
        asset_types,
        asset_info,
        resources,
        capabilities,
        effects,
        transactions,
    }
} 
