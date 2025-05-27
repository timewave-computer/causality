//! Optimization system for TEL graph execution
//!
//! Defines optimization strategies, cost models, and resource usage estimation
//! for efficient execution of Temporal Effect Language graphs.

use crate::primitive::ids::{DomainId, ResourceId, ExprId, EntityId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::system::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Optimization Strategy Types (from tel/strategy.rs)
//-----------------------------------------------------------------------------

/// Optimization strategy for effect execution
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationStrategy {
    pub name: Str,
    pub description: Str,
    pub priority_weight: f64,
    pub latency_weight: f64,
    pub cost_weight: f64,
}

impl OptimizationStrategy {
    /// Capital efficiency optimization strategy
    pub fn capital_efficiency() -> Self {
        Self {
            name: Str::from("CapitalEfficiency"),
            description: Str::from("Maximize resource utilization efficiency"),
            priority_weight: 0.8,
            latency_weight: 0.3,
            cost_weight: 0.5,
        }
    }

    /// Latency optimization strategy
    pub fn latency_optimization() -> Self {
        Self {
            name: Str::from("LatencyOptimization"),
            description: Str::from("Minimize execution latency"),
            priority_weight: 0.5,
            latency_weight: 0.9,
            cost_weight: 0.2,
        }
    }

    /// Cost minimization strategy
    pub fn cost_minimization() -> Self {
        Self {
            name: Str::from("CostMinimization"),
            description: Str::from("Minimize computational and resource costs"),
            priority_weight: 0.4,
            latency_weight: 0.2,
            cost_weight: 0.9,
        }
    }

    /// Balanced optimization strategy
    pub fn balanced() -> Self {
        Self {
            name: Str::from("Balanced"),
            description: Str::from("Balance all optimization objectives"),
            priority_weight: 0.6,
            latency_weight: 0.6,
            cost_weight: 0.6,
        }
    }
}

//-----------------------------------------------------------------------------
// Cost Model Types (from tel/cost_model.rs)
//-----------------------------------------------------------------------------

/// Cost model for effect execution
#[derive(Debug, Clone, PartialEq)]
pub struct EffectCostModel {
    pub base_cost: u64,
    pub resource_cost_per_unit: u64,
    pub complexity_multiplier: f64,
    pub domain_cost_factor: f64,
    pub estimated_duration_ms: u64,
}

impl Default for EffectCostModel {
    fn default() -> Self {
        Self {
            base_cost: 100,
            resource_cost_per_unit: 10,
            complexity_multiplier: 1.0,
            domain_cost_factor: 1.0,
            estimated_duration_ms: 1000,
        }
    }
}

impl Eq for EffectCostModel {}

impl EffectCostModel {
    /// Calculate total cost for a given number of resources
    pub fn calculate_total_cost(&self, resource_count: u64) -> u64 {
        let base = self.base_cost;
        let resource_cost = self.resource_cost_per_unit * resource_count;
        let complexity_adjusted = (base + resource_cost) as f64 * self.complexity_multiplier;
        let domain_adjusted = complexity_adjusted * self.domain_cost_factor;
        domain_adjusted as u64
    }

    /// Estimate execution time based on cost model
    pub fn estimate_execution_time(&self, resource_count: u64) -> u64 {
        let base_time = self.estimated_duration_ms;
        let resource_factor = (resource_count as f64).sqrt();
        let complexity_factor = self.complexity_multiplier;
        (base_time as f64 * resource_factor * complexity_factor) as u64
    }
}

/// Resource usage estimation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceUsageEstimate {
    pub cpu_time_ms: u64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub network_bytes: u64,
    pub estimated_completion_time: Timestamp,
}

impl Default for ResourceUsageEstimate {
    fn default() -> Self {
        Self {
            cpu_time_ms: 1000,
            memory_bytes: 1024 * 1024, // 1MB
            storage_bytes: 1024 * 512,  // 512KB
            network_bytes: 1024 * 256,  // 256KB
            estimated_completion_time: Timestamp::now(),
        }
    }
}

//-----------------------------------------------------------------------------
// Optimization Types (from tel/optimization.rs)
//-----------------------------------------------------------------------------

/// Typed domain for optimization decisions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TypedDomain {
    /// Domain identifier
    pub domain_id: DomainId,
    /// Domain type (e.g., "verifiable", "service", "local")
    pub domain_type: Str,
    /// Domain capabilities
    pub capabilities: Vec<Str>,
    /// Performance characteristics
    pub performance_profile: DomainPerformanceProfile,
}

impl TypedDomain {
    /// Create a new typed domain
    pub fn new(domain_id: DomainId, domain_type: Str) -> Self {
        Self {
            domain_id,
            domain_type,
            capabilities: Vec::new(),
            performance_profile: DomainPerformanceProfile::default(),
        }
    }

    /// Add a capability to the domain
    pub fn with_capability(mut self, capability: Str) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Set the performance profile
    pub fn with_performance_profile(mut self, profile: DomainPerformanceProfile) -> Self {
        self.performance_profile = profile;
        self
    }
}

/// Performance profile for a domain
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainPerformanceProfile {
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: u64,
    /// Reliability score (0-100)
    pub reliability_score: u8,
    /// Cost factor (relative to baseline)
    pub cost_factor: u64,
}

impl Default for DomainPerformanceProfile {
    fn default() -> Self {
        Self {
            avg_latency_ms: 100,
            throughput_ops_per_sec: 1000,
            reliability_score: 95,
            cost_factor: 100,
        }
    }
}

/// Effect compatibility metadata for optimization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectCompatibility {
    /// Effect type this compatibility applies to
    pub effect_type: Str,
    /// Compatibility score (0-100)
    pub compatibility_score: u8,
    /// Required capabilities
    pub required_capabilities: Vec<Str>,
    /// Performance impact
    pub performance_impact: PerformanceImpact,
}

/// Performance impact assessment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformanceImpact {
    /// Latency impact in milliseconds
    pub latency_impact_ms: i64,
    /// Throughput impact (percentage change)
    pub throughput_impact_percent: i8,
    /// Resource usage impact
    pub resource_impact: ResourceImpact,
}

/// Resource usage impact
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceImpact {
    /// CPU usage change (percentage)
    pub cpu_impact_percent: i8,
    /// Memory usage change (percentage)
    pub memory_impact_percent: i8,
    /// Storage usage change (percentage)
    pub storage_impact_percent: i8,
}

/// Resource preference for optimization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePreference {
    /// Resource type
    pub resource_type: Str,
    /// Preference weight (0-100)
    pub preference_weight: u8,
    /// Minimum required quantity
    pub min_quantity: u64,
    /// Maximum acceptable quantity
    pub max_quantity: Option<u64>,
    /// Cost sensitivity
    pub cost_sensitivity: CostSensitivity,
}

/// Cost sensitivity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostSensitivity {
    /// Very sensitive to cost changes
    High,
    /// Moderately sensitive to cost changes
    Medium,
    /// Not very sensitive to cost changes
    Low,
    /// Cost is not a factor
    None,
}

/// Process dataflow initiation hint
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowInitiationHint {
    /// Preferred strategy identifier
    pub preferred_strategy_id: Str,
    /// Priority level (0-100)
    pub priority: u32,
    /// Dataflow definition identifier
    pub dataflow_definition_id: Option<ResourceId>,
    /// Resource constraints
    pub resource_constraints: ResourceUsageEstimate,
    /// Domain preference order
    pub domain_preference_order: Vec<TypedDomain>,
    /// Whether to prefer parallel execution
    pub prefer_parallel_execution: bool,
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Cost budget
    pub cost_budget: Option<u64>,
}

impl Default for ProcessDataflowInitiationHint {
    fn default() -> Self {
        Self {
            preferred_strategy_id: Str::from("balanced"),
            priority: 50,
            dataflow_definition_id: None,
            resource_constraints: ResourceUsageEstimate::default(),
            domain_preference_order: Vec::new(),
            prefer_parallel_execution: false,
            max_latency_ms: None,
            cost_budget: None,
        }
    }
}

//-----------------------------------------------------------------------------
// Serialization Implementations
//-----------------------------------------------------------------------------

impl Encode for TypedDomain {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.capabilities.as_ssz_bytes());
        bytes.extend_from_slice(&self.performance_profile.as_ssz_bytes());
        bytes
    }
}

impl Decode for TypedDomain {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_id.as_ssz_bytes().len();
        
        let domain_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_type.as_ssz_bytes().len();
        
        let capabilities = Vec::<Str>::from_ssz_bytes(&bytes[offset..])?;
        offset += capabilities.as_ssz_bytes().len();
        
        let performance_profile = DomainPerformanceProfile::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(TypedDomain {
            domain_id,
            domain_type,
            capabilities,
            performance_profile,
        })
    }
}

impl SimpleSerialize for TypedDomain {}

impl Encode for DomainPerformanceProfile {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.avg_latency_ms.to_le_bytes());
        bytes.extend_from_slice(&self.throughput_ops_per_sec.to_le_bytes());
        bytes.push(self.reliability_score);
        bytes.extend_from_slice(&self.cost_factor.to_le_bytes());
        bytes
    }
}

impl Decode for DomainPerformanceProfile {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 25 { // 8 + 8 + 1 + 8
            return Err(DecodeError { message: "Insufficient bytes for DomainPerformanceProfile".to_string() });
        }
        
        let mut offset = 0;
        
        let avg_latency_ms = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let throughput_ops_per_sec = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let reliability_score = bytes[offset];
        offset += 1;
        
        let cost_factor = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        
        Ok(DomainPerformanceProfile {
            avg_latency_ms,
            throughput_ops_per_sec,
            reliability_score,
            cost_factor,
        })
    }
}

impl SimpleSerialize for DomainPerformanceProfile {}

impl Encode for EffectCompatibility {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.effect_type.as_ssz_bytes());
        bytes.push(self.compatibility_score);
        bytes.extend_from_slice(&self.required_capabilities.as_ssz_bytes());
        bytes.extend_from_slice(&self.performance_impact.as_ssz_bytes());
        bytes
    }
}

impl Decode for EffectCompatibility {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let effect_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += effect_type.as_ssz_bytes().len();
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for compatibility_score".to_string() });
        }
        let compatibility_score = bytes[offset];
        offset += 1;
        
        let required_capabilities = Vec::<Str>::from_ssz_bytes(&bytes[offset..])?;
        offset += required_capabilities.as_ssz_bytes().len();
        
        let performance_impact = PerformanceImpact::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(EffectCompatibility {
            effect_type,
            compatibility_score,
            required_capabilities,
            performance_impact,
        })
    }
}

impl SimpleSerialize for EffectCompatibility {}

impl Encode for PerformanceImpact {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.latency_impact_ms.to_le_bytes());
        bytes.push(self.throughput_impact_percent as u8);
        bytes.extend_from_slice(&self.resource_impact.as_ssz_bytes());
        bytes
    }
}

impl Decode for PerformanceImpact {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 9 { // 8 + 1
            return Err(DecodeError { message: "Insufficient bytes for PerformanceImpact".to_string() });
        }
        
        let mut offset = 0;
        
        let latency_impact_ms = i64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let throughput_impact_percent = bytes[offset] as i8;
        offset += 1;
        
        let resource_impact = ResourceImpact::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(PerformanceImpact {
            latency_impact_ms,
            throughput_impact_percent,
            resource_impact,
        })
    }
}

impl SimpleSerialize for PerformanceImpact {}

impl Encode for ResourceImpact {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        vec![
            self.cpu_impact_percent as u8,
            self.memory_impact_percent as u8,
            self.storage_impact_percent as u8,
        ]
    }
}

impl Decode for ResourceImpact {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 3 {
            return Err(DecodeError { message: "Insufficient bytes for ResourceImpact".to_string() });
        }
        
        Ok(ResourceImpact {
            cpu_impact_percent: bytes[0] as i8,
            memory_impact_percent: bytes[1] as i8,
            storage_impact_percent: bytes[2] as i8,
        })
    }
}

impl SimpleSerialize for ResourceImpact {}

impl Encode for ResourcePreference {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.resource_type.as_ssz_bytes());
        bytes.push(self.preference_weight);
        bytes.extend_from_slice(&self.min_quantity.to_le_bytes());
        
        // Handle optional max_quantity
        if let Some(max_qty) = self.max_quantity {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&max_qty.to_le_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        // Encode cost sensitivity
        let sensitivity_byte = match self.cost_sensitivity {
            CostSensitivity::High => 0,
            CostSensitivity::Medium => 1,
            CostSensitivity::Low => 2,
            CostSensitivity::None => 3,
        };
        bytes.push(sensitivity_byte);
        
        bytes
    }
}

impl Decode for ResourcePreference {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let resource_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += resource_type.as_ssz_bytes().len();
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for preference_weight".to_string() });
        }
        let preference_weight = bytes[offset];
        offset += 1;
        
        if offset + 8 > bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for min_quantity".to_string() });
        }
        let min_quantity = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for max_quantity marker".to_string() });
        }
        
        let max_quantity = if bytes[offset] == 1 {
            offset += 1;
            if offset + 8 > bytes.len() {
                return Err(DecodeError { message: "Insufficient bytes for max_quantity value".to_string() });
            }
            let max_qty = u64::from_le_bytes([
                bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
            ]);
            offset += 8;
            Some(max_qty)
        } else {
            offset += 1;
            None
        };
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for cost_sensitivity".to_string() });
        }
        
        let cost_sensitivity = match bytes[offset] {
            0 => CostSensitivity::High,
            1 => CostSensitivity::Medium,
            2 => CostSensitivity::Low,
            3 => CostSensitivity::None,
            _ => return Err(DecodeError { message: "Invalid cost_sensitivity value".to_string() }),
        };
        
        Ok(ResourcePreference {
            resource_type,
            preference_weight,
            min_quantity,
            max_quantity,
            cost_sensitivity,
        })
    }
}

impl SimpleSerialize for ResourcePreference {}

impl Encode for ProcessDataflowInitiationHint {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.preferred_strategy_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.priority.to_le_bytes());
        
        // Handle optional dataflow_definition_id
        if let Some(ref def_id) = self.dataflow_definition_id {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&def_id.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        bytes.extend_from_slice(&self.resource_constraints.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_preference_order.as_ssz_bytes());
        bytes.push(if self.prefer_parallel_execution { 1 } else { 0 });
        
        // Handle optional max_latency_ms
        if let Some(max_latency) = self.max_latency_ms {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&max_latency.to_le_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        // Handle optional cost_budget
        if let Some(budget) = self.cost_budget {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&budget.to_le_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        bytes
    }
}

impl Decode for ProcessDataflowInitiationHint {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let preferred_strategy_id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += preferred_strategy_id.as_ssz_bytes().len();
        
        if offset + 4 > bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for priority".to_string() });
        }
        let priority = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]);
        offset += 4;
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for dataflow_definition_id marker".to_string() });
        }
        
        let dataflow_definition_id = if bytes[offset] == 1 {
            offset += 1;
            Some(ResourceId::from_ssz_bytes(&bytes[offset..])?)
        } else {
            offset += 1;
            None
        };
        
        if let Some(ref def_id) = dataflow_definition_id {
            offset += def_id.as_ssz_bytes().len();
        }
        
        let resource_constraints = ResourceUsageEstimate::from_ssz_bytes(&bytes[offset..])?;
        offset += resource_constraints.as_ssz_bytes().len();
        
        let domain_preference_order = Vec::<TypedDomain>::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_preference_order.as_ssz_bytes().len();
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for prefer_parallel_execution".to_string() });
        }
        let prefer_parallel_execution = bytes[offset] == 1;
        offset += 1;
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for max_latency_ms marker".to_string() });
        }
        
        let max_latency_ms = if bytes[offset] == 1 {
            offset += 1;
            if offset + 8 > bytes.len() {
                return Err(DecodeError { message: "Insufficient bytes for max_latency_ms value".to_string() });
            }
            let latency = u64::from_le_bytes([
                bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
            ]);
            offset += 8;
            Some(latency)
        } else {
            offset += 1;
            None
        };
        
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for cost_budget marker".to_string() });
        }
        
        let cost_budget = if bytes[offset] == 1 {
            offset += 1;
            if offset + 8 > bytes.len() {
                return Err(DecodeError { message: "Insufficient bytes for cost_budget value".to_string() });
            }
            let budget = u64::from_le_bytes([
                bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
            ]);
            Some(budget)
        } else {
            None
        };
        
        Ok(ProcessDataflowInitiationHint {
            preferred_strategy_id,
            priority,
            dataflow_definition_id,
            resource_constraints,
            domain_preference_order,
            prefer_parallel_execution,
            max_latency_ms,
            cost_budget,
        })
    }
}

impl SimpleSerialize for ProcessDataflowInitiationHint {}

impl Encode for ResourceUsageEstimate {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.cpu_time_ms.to_le_bytes());
        bytes.extend_from_slice(&self.memory_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.storage_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.network_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.estimated_completion_time.as_ssz_bytes());
        bytes
    }
}

impl Decode for ResourceUsageEstimate {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 32 { // 4 * 8 bytes
            return Err(DecodeError { message: "Insufficient bytes for ResourceUsageEstimate".to_string() });
        }
        
        let mut offset = 0;
        
        let cpu_time_ms = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let memory_bytes = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let storage_bytes = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let network_bytes = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let estimated_completion_time = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ResourceUsageEstimate {
            cpu_time_ms,
            memory_bytes,
            storage_bytes,
            network_bytes,
            estimated_completion_time,
        })
    }
}

impl SimpleSerialize for ResourceUsageEstimate {}

//-----------------------------------------------------------------------------
// Plan Types for Optimization
//-----------------------------------------------------------------------------

/// A resolution plan for executing a set of intents
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionPlan {
    /// Unique identifier for this plan
    pub plan_id: EntityId,
    /// Intent bundles to be processed
    pub intent_bundles: Vec<ExprId>,
    /// Sequence of effects to execute
    pub effect_sequence: Vec<EntityId>,
    /// Dataflow orchestration steps
    pub dataflow_steps: Vec<DataflowOrchestrationStep>,
    /// Resource transfers required
    pub resource_transfers: Vec<ResourceTransfer>,
    /// Target typed domain for execution
    pub target_typed_domain: TypedDomain,
    /// Estimated execution cost
    pub estimated_cost: u64,
    /// Estimated execution time in milliseconds
    pub estimated_time_ms: u64,
    /// Additional metadata
    pub metadata: HashMap<Str, String>,
}

/// A scored resolution plan with evaluation metrics
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredPlan {
    /// The underlying resolution plan
    pub plan: ResolutionPlan,
    /// Overall score (0.0 to 1.0)
    pub overall_score: f64,
    /// Cost efficiency score
    pub cost_efficiency_score: f64,
    /// Time efficiency score
    pub time_efficiency_score: f64,
    /// Resource utilization score
    pub resource_utilization_score: f64,
    /// Domain compatibility score
    pub domain_compatibility_score: f64,
    /// Name of the strategy that generated this plan
    pub strategy_name: Str,
    /// When this plan was evaluated
    pub evaluated_at: Timestamp,
}

/// A dataflow orchestration step
#[derive(Debug, Clone, PartialEq)]
pub struct DataflowOrchestrationStep {
    /// Step identifier
    pub step_id: EntityId,
    /// Step type (e.g., "compute", "transfer", "verify")
    pub step_type: Str,
    /// Required resources
    pub required_resources: Vec<ResourceId>,
    /// Produced resources
    pub produced_resources: Vec<ResourceId>,
    /// Estimated execution time
    pub estimated_duration_ms: u64,
    /// Dependencies on other steps
    pub dependencies: Vec<EntityId>,
}

/// A resource transfer specification
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceTransfer {
    /// Resource being transferred
    pub resource_id: ResourceId,
    /// Source domain
    pub source_domain: DomainId,
    /// Target domain
    pub target_domain: DomainId,
    /// Transfer cost
    pub transfer_cost: u64,
    /// Transfer time estimate
    pub transfer_time_ms: u64,
} 