/// Real Almanac schema format (matching indexer-core::ContractSchema)
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacSchema {
    pub contract_address: String,
    pub chain_id: String,
    pub contract_name: Option<String>,
    pub abi: Option<serde_json::Value>,
    pub layout_commitment: String,
    pub storage_layout: Vec<StorageSlot>,
    pub events: Vec<EventSchema>,
    pub queries: Vec<QuerySchema>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Real Almanac storage slot format
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSlot {
    pub slot: String,
    pub offset: u32,
    pub type_name: String,
    pub label: String,
    pub is_mapping: bool,
    pub mapping_key_type: Option<String>,
    pub mapping_value_type: Option<String>,
}

/// Real Almanac event schema format
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSchema {
    pub event_name: String,
    pub signature: String,
    pub inputs: Vec<EventInput>,
    pub anonymous: bool,
    pub topic_hash: String,
}

/// Real Almanac event input format
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInput {
    pub name: String,
    pub type_name: String,
    pub indexed: bool,
    pub internal_type: Option<String>,
}

/// Real Almanac query schema format
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySchema {
    pub query_name: String,
    pub query_type: QueryType,
    pub target_slots: Vec<String>,
    pub filters: Vec<QueryFilter>,
    pub aggregations: Vec<QueryAggregation>,
}

/// Real Almanac query type enum
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    StorageSlot,
    TokenBalance,
    TokenAllowance,
    Event,
    ContractCall,
}

/// Real Almanac query filter format
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Real Almanac filter operator enum
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Contains,
    StartsWith,
    EndsWith,
}

/// Real Almanac query aggregation format
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAggregation {
    pub field: String,
    pub aggregation_type: AggregationType,
    pub alias: Option<String>,
}

/// Real Almanac aggregation type enum
#[cfg(feature = "almanac")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    First,
    Last,
}

// Mock schema format for development (when almanac feature is not enabled)
#[cfg(not(feature = "almanac"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacSchema {
    pub contract_address: String,
    pub chain_id: String,
    pub layout_commitment: String,
    pub queries: Vec<String>, // Simplified for mocks
} 