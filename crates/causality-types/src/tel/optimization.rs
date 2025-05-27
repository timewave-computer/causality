//! Core optimization types for runtime strategy evaluation
//!
//! This module defines the fundamental types used by the optimization framework
//! for intent resolution, effect execution planning, and ProcessDataflowBlock orchestration.

use crate::{
    core::{
        id::{DomainId, EntityId, ExprId, ResourceId},
        str::Str,
        time::Timestamp,
        resource::ResourceFlow,
    },
    expr::value::ValueExpr,
    serialization::{Encode, Decode, DecodeError, SimpleSerialize},
};
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// TypedDomain Definitions
//-----------------------------------------------------------------------------

/// Represents different types of execution domains with distinct characteristics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedDomain {
    /// Domain enforcing ZK-compatibility and determinism
    VerifiableDomain(DomainId),
    /// Domain facilitating interactions with external services
    ServiceDomain(DomainId),
}

impl TypedDomain {
    /// Get the underlying domain ID
    pub fn domain_id(&self) -> DomainId {
        match self {
            TypedDomain::VerifiableDomain(id) => *id,
            TypedDomain::ServiceDomain(id) => *id,
        }
    }

    /// Check if this is a verifiable domain
    pub fn is_verifiable(&self) -> bool {
        matches!(self, TypedDomain::VerifiableDomain(_))
    }

    /// Check if this is a service domain
    pub fn is_service(&self) -> bool {
        matches!(self, TypedDomain::ServiceDomain(_))
    }
}

impl Encode for TypedDomain {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            TypedDomain::VerifiableDomain(id) => {
                bytes.push(0); // variant tag
                bytes.extend_from_slice(&id.as_ssz_bytes());
            }
            TypedDomain::ServiceDomain(id) => {
                bytes.push(1); // variant tag
                bytes.extend_from_slice(&id.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for TypedDomain {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for TypedDomain".to_string() });
        }
        
        let variant = bytes[0];
        let id = DomainId::from_ssz_bytes(&bytes[1..])?;
        
        match variant {
            0 => Ok(TypedDomain::VerifiableDomain(id)),
            1 => Ok(TypedDomain::ServiceDomain(id)),
            _ => Err(DecodeError { message: format!("Invalid TypedDomain variant: {}", variant) }),
        }
    }
}

impl SimpleSerialize for TypedDomain {}

impl Default for TypedDomain {
    fn default() -> Self {
        TypedDomain::VerifiableDomain(DomainId::default())
    }
}

//-----------------------------------------------------------------------------
// ProcessDataflowBlock Orchestration Types
//-----------------------------------------------------------------------------

/// Hint for ProcessDataflowBlock initiation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessDataflowInitiationHint {
    /// ID of the dataflow definition to use
    pub df_def_id: ExprId,
    /// Initial parameters for the dataflow
    pub params_expr_id: ExprId,
    /// Target typed domain for execution
    pub target_typed_domain: TypedDomain,
}

impl Encode for ProcessDataflowInitiationHint {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.df_def_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.params_expr_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.target_typed_domain.as_ssz_bytes());
        bytes
    }
}

impl Decode for ProcessDataflowInitiationHint {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 64 {
            return Err(DecodeError { message: "Insufficient bytes for ProcessDataflowInitiationHint".to_string() });
        }
        
        let df_def_id = ExprId::from_ssz_bytes(&bytes[0..32])?;
        let params_expr_id = ExprId::from_ssz_bytes(&bytes[32..64])?;
        let target_typed_domain = TypedDomain::from_ssz_bytes(&bytes[64..])?;
        
        Ok(ProcessDataflowInitiationHint {
            df_def_id,
            params_expr_id,
            target_typed_domain,
        })
    }
}

impl SimpleSerialize for ProcessDataflowInitiationHint {}

/// ProcessDataflowBlock orchestration step
#[derive(Debug, Clone, PartialEq)]
pub enum DataflowOrchestrationStep {
    /// Initiate a new dataflow instance
    InitiateDataflow {
        df_def_id: ExprId,
        params: ValueExpr,
    },
    /// Advance an existing dataflow instance
    AdvanceDataflow {
        df_instance_id: ResourceId,
        action_params: ValueExpr,
    },
}

impl Encode for DataflowOrchestrationStep {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            DataflowOrchestrationStep::InitiateDataflow { df_def_id, params } => {
                bytes.push(0); // variant tag
                bytes.extend_from_slice(&df_def_id.as_ssz_bytes());
                bytes.extend_from_slice(&params.as_ssz_bytes());
            }
            DataflowOrchestrationStep::AdvanceDataflow { df_instance_id, action_params } => {
                bytes.push(1); // variant tag
                bytes.extend_from_slice(&df_instance_id.as_ssz_bytes());
                bytes.extend_from_slice(&action_params.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for DataflowOrchestrationStep {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for DataflowOrchestrationStep".to_string() });
        }
        
        let variant = bytes[0];
        match variant {
            0 => {
                if bytes.len() < 33 {
                    return Err(DecodeError { message: "Insufficient bytes for InitiateDataflow".to_string() });
                }
                let df_def_id = ExprId::from_ssz_bytes(&bytes[1..33])?;
                let params = ValueExpr::from_ssz_bytes(&bytes[33..])?;
                Ok(DataflowOrchestrationStep::InitiateDataflow { df_def_id, params })
            }
            1 => {
                if bytes.len() < 33 {
                    return Err(DecodeError { message: "Insufficient bytes for AdvanceDataflow".to_string() });
                }
                let df_instance_id = ResourceId::from_ssz_bytes(&bytes[1..33])?;
                let action_params = ValueExpr::from_ssz_bytes(&bytes[33..])?;
                Ok(DataflowOrchestrationStep::AdvanceDataflow { df_instance_id, action_params })
            }
            _ => Err(DecodeError { message: format!("Invalid DataflowOrchestrationStep variant: {}", variant) }),
        }
    }
}

impl SimpleSerialize for DataflowOrchestrationStep {}

//-----------------------------------------------------------------------------
// Resource Transfer Types
//-----------------------------------------------------------------------------

/// Types of resource transfers in optimization plans
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferType {
    /// Direct transfer between domains
    Direct,
    /// Transfer through an intermediary
    Bridged,
    /// Atomic swap
    Swap,
    /// Batched transfer
    Batch,
}

impl Encode for TransferType {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        vec![match self {
            TransferType::Direct => 0,
            TransferType::Bridged => 1,
            TransferType::Swap => 2,
            TransferType::Batch => 3,
        }]
    }
}

impl Decode for TransferType {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for TransferType".to_string() });
        }
        
        match bytes[0] {
            0 => Ok(TransferType::Direct),
            1 => Ok(TransferType::Bridged),
            2 => Ok(TransferType::Swap),
            3 => Ok(TransferType::Batch),
            _ => Err(DecodeError { message: format!("Invalid TransferType variant: {}", bytes[0]) }),
        }
    }
}

impl SimpleSerialize for TransferType {}

/// Resource transfer specification for optimization plans
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceTransfer {
    /// Source typed domain
    pub source_typed_domain: TypedDomain,
    /// Target typed domain
    pub target_typed_domain: TypedDomain,
    /// Resource flow specification
    pub resource_flow: ResourceFlow,
    /// Type of transfer
    pub transfer_type: TransferType,
    /// Estimated cost
    pub estimated_cost: u64,
    /// Estimated time (in milliseconds)
    pub estimated_time_ms: u64,
}

impl Encode for ResourceTransfer {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.source_typed_domain.as_ssz_bytes());
        bytes.extend_from_slice(&self.target_typed_domain.as_ssz_bytes());
        bytes.extend_from_slice(&self.resource_flow.as_ssz_bytes());
        bytes.extend_from_slice(&self.transfer_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.estimated_cost.to_le_bytes());
        bytes.extend_from_slice(&self.estimated_time_ms.to_le_bytes());
        bytes
    }
}

impl Decode for ResourceTransfer {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 16 {
            return Err(DecodeError { message: "Insufficient bytes for ResourceTransfer".to_string() });
        }
        
        let mut offset = 0;
        
        let source_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        offset += source_typed_domain.as_ssz_bytes().len();
        
        let target_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        offset += target_typed_domain.as_ssz_bytes().len();
        
        let resource_flow = ResourceFlow::from_ssz_bytes(&bytes[offset..])?;
        offset += resource_flow.as_ssz_bytes().len();
        
        let transfer_type = TransferType::from_ssz_bytes(&bytes[offset..])?;
        offset += transfer_type.as_ssz_bytes().len();
        
        if bytes.len() < offset + 16 {
            return Err(DecodeError { message: "Insufficient bytes for cost and time".to_string() });
        }
        
        let estimated_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let estimated_time_ms = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        
        Ok(ResourceTransfer {
            source_typed_domain,
            target_typed_domain,
            resource_flow,
            transfer_type,
            estimated_cost,
            estimated_time_ms,
        })
    }
}

impl SimpleSerialize for ResourceTransfer {}

//-----------------------------------------------------------------------------
// Effect Compatibility Types
//-----------------------------------------------------------------------------

/// Effect compatibility metadata for optimization
#[derive(Debug, Clone, PartialEq)]
pub struct EffectCompatibility {
    /// Source typed domain
    pub source_typed_domain: TypedDomain,
    /// Target typed domain
    pub target_typed_domain: TypedDomain,
    /// Compatibility score (0.0 to 1.0)
    pub compatibility_score: f64,
    /// Required adaptations
    pub required_adaptations: Vec<Str>,
    /// Estimated adaptation cost
    pub adaptation_cost: u64,
}

impl Encode for EffectCompatibility {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.source_typed_domain.as_ssz_bytes());
        bytes.extend_from_slice(&self.target_typed_domain.as_ssz_bytes());
        bytes.extend_from_slice(&self.compatibility_score.to_le_bytes());
        bytes.extend_from_slice(&self.required_adaptations.as_ssz_bytes());
        bytes.extend_from_slice(&self.adaptation_cost.to_le_bytes());
        bytes
    }
}

impl Decode for EffectCompatibility {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 16 {
            return Err(DecodeError { message: "Insufficient bytes for EffectCompatibility".to_string() });
        }
        
        let mut offset = 0;
        
        let source_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        offset += source_typed_domain.as_ssz_bytes().len();
        
        let target_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        offset += target_typed_domain.as_ssz_bytes().len();
        
        if bytes.len() < offset + 8 {
            return Err(DecodeError { message: "Insufficient bytes for compatibility_score".to_string() });
        }
        
        let compatibility_score = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let required_adaptations = Vec::<Str>::from_ssz_bytes(&bytes[offset..])?;
        offset += required_adaptations.as_ssz_bytes().len();
        
        if bytes.len() < offset + 8 {
            return Err(DecodeError { message: "Insufficient bytes for adaptation_cost".to_string() });
        }
        
        let adaptation_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        
        Ok(EffectCompatibility {
            source_typed_domain,
            target_typed_domain,
            compatibility_score,
            required_adaptations,
            adaptation_cost,
        })
    }
}

impl SimpleSerialize for EffectCompatibility {}

//-----------------------------------------------------------------------------
// Resource Preference Types
//-----------------------------------------------------------------------------

/// Resource preference for optimization strategies
#[derive(Debug, Clone, PartialEq)]
pub struct ResourcePreference {
    /// Preferred typed domain
    pub preferred_typed_domain: TypedDomain,
    /// Resource type pattern
    pub resource_type: Str,
    /// Preference weight (0.0 to 1.0)
    pub preference_weight: f64,
    /// Cost multiplier for this preference
    pub cost_multiplier: f64,
    /// Additional constraints
    pub constraints: HashMap<Str, ValueExpr>,
}

impl Encode for ResourcePreference {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.preferred_typed_domain.as_ssz_bytes());
        bytes.extend_from_slice(&self.resource_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.preference_weight.to_le_bytes());
        bytes.extend_from_slice(&self.cost_multiplier.to_le_bytes());
        
        // Convert HashMap to Vec of pairs for serialization
        let constraint_pairs: Vec<(Str, ValueExpr)> = self.constraints.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let constraint_pairs_bytes = constraint_pairs.as_ssz_bytes();
        bytes.extend_from_slice(&constraint_pairs_bytes);
        
        bytes
    }
}

impl Decode for ResourcePreference {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 16 {
            return Err(DecodeError::new("Insufficient data for ResourcePreference"));
        }
        
        let mut offset = 0;
        
        let preferred_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        offset += preferred_typed_domain.as_ssz_bytes().len();
        
        let resource_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += resource_type.as_ssz_bytes().len();
        
        if bytes.len() < offset + 16 {
            return Err(DecodeError::new("Insufficient data for ResourcePreference weights"));
        }
        
        let preference_weight = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let cost_multiplier = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        // Deserialize constraints from Vec of pairs
        let constraint_pairs: Vec<(Str, ValueExpr)> = Vec::from_ssz_bytes(&bytes[offset..])?;
        let constraints: HashMap<Str, ValueExpr> = constraint_pairs.into_iter().collect();
        
        Ok(ResourcePreference {
            preferred_typed_domain,
            resource_type,
            preference_weight,
            cost_multiplier,
            constraints,
        })
    }
}

impl SimpleSerialize for ResourcePreference {}

//-----------------------------------------------------------------------------
// Resolution Plan Types
//-----------------------------------------------------------------------------

/// A complete plan for resolving a set of intents
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionPlan {
    /// Unique identifier for this plan
    pub plan_id: EntityId,
    /// Intent bundles to be resolved
    pub intent_bundles: Vec<EntityId>,
    /// Sequence of effects to execute
    pub effect_sequence: Vec<EntityId>,
    /// ProcessDataflowBlock orchestration steps
    pub dataflow_steps: Vec<DataflowOrchestrationStep>,
    /// Resource transfers required
    pub resource_transfers: Vec<ResourceTransfer>,
    /// Target typed domain for execution
    pub target_typed_domain: TypedDomain,
    /// Estimated total cost
    pub estimated_cost: u64,
    /// Estimated execution time (in milliseconds)
    pub estimated_time_ms: u64,
    /// Plan metadata
    pub metadata: HashMap<Str, ValueExpr>,
}

impl Encode for ResolutionPlan {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize each field with length prefix
        let plan_id_bytes = self.plan_id.as_ssz_bytes();
        bytes.extend_from_slice(&(plan_id_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&plan_id_bytes);
        
        let intent_bundles_bytes = self.intent_bundles.as_ssz_bytes();
        bytes.extend_from_slice(&(intent_bundles_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&intent_bundles_bytes);
        
        let effect_sequence_bytes = self.effect_sequence.as_ssz_bytes();
        bytes.extend_from_slice(&(effect_sequence_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&effect_sequence_bytes);
        
        let dataflow_steps_bytes = self.dataflow_steps.as_ssz_bytes();
        bytes.extend_from_slice(&(dataflow_steps_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&dataflow_steps_bytes);
        
        let resource_transfers_bytes = self.resource_transfers.as_ssz_bytes();
        bytes.extend_from_slice(&(resource_transfers_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&resource_transfers_bytes);
        
        let target_typed_domain_bytes = self.target_typed_domain.as_ssz_bytes();
        bytes.extend_from_slice(&(target_typed_domain_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&target_typed_domain_bytes);
        
        bytes.extend_from_slice(&self.estimated_cost.to_le_bytes());
        bytes.extend_from_slice(&self.estimated_time_ms.to_le_bytes());
        
        // Convert HashMap to Vec of pairs for serialization
        let metadata_pairs: Vec<(Str, ValueExpr)> = self.metadata.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let metadata_pairs_bytes = metadata_pairs.as_ssz_bytes();
        bytes.extend_from_slice(&(metadata_pairs_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&metadata_pairs_bytes);
        
        bytes
    }
}

impl Decode for ResolutionPlan {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError::new("Insufficient data for ResolutionPlan"));
        }
        
        let mut offset = 0;
        
        // Read plan_id with length prefix
        let plan_id_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + plan_id_len > bytes.len() {
            return Err(DecodeError::new("Invalid plan_id length"));
        }
        let plan_id = EntityId::from_ssz_bytes(&bytes[offset..offset + plan_id_len])?;
        offset += plan_id_len;
        
        // Read intent_bundles with length prefix
        let intent_bundles_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + intent_bundles_len > bytes.len() {
            return Err(DecodeError::new("Invalid intent_bundles length"));
        }
        let intent_bundles = Vec::<EntityId>::from_ssz_bytes(&bytes[offset..offset + intent_bundles_len])?;
        offset += intent_bundles_len;
        
        // Read effect_sequence with length prefix
        let effect_sequence_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + effect_sequence_len > bytes.len() {
            return Err(DecodeError::new("Invalid effect_sequence length"));
        }
        let effect_sequence = Vec::<EntityId>::from_ssz_bytes(&bytes[offset..offset + effect_sequence_len])?;
        offset += effect_sequence_len;
        
        // Read dataflow_steps with length prefix
        let dataflow_steps_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + dataflow_steps_len > bytes.len() {
            return Err(DecodeError::new("Invalid dataflow_steps length"));
        }
        let dataflow_steps = Vec::<DataflowOrchestrationStep>::from_ssz_bytes(&bytes[offset..offset + dataflow_steps_len])?;
        offset += dataflow_steps_len;
        
        // Read resource_transfers with length prefix
        let resource_transfers_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + resource_transfers_len > bytes.len() {
            return Err(DecodeError::new("Invalid resource_transfers length"));
        }
        let resource_transfers = Vec::<ResourceTransfer>::from_ssz_bytes(&bytes[offset..offset + resource_transfers_len])?;
        offset += resource_transfers_len;
        
        // Read target_typed_domain with length prefix
        let target_typed_domain_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + target_typed_domain_len > bytes.len() {
            return Err(DecodeError::new("Invalid target_typed_domain length"));
        }
        let target_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..offset + target_typed_domain_len])?;
        offset += target_typed_domain_len;
        
        if bytes.len() < offset + 16 {
            return Err(DecodeError::new("Insufficient data for ResolutionPlan costs"));
        }
        
        let estimated_cost = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let estimated_time_ms = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        // Read metadata with length prefix
        let metadata_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + metadata_len > bytes.len() {
            return Err(DecodeError::new("Invalid metadata length"));
        }
        let metadata_pairs: Vec<(Str, ValueExpr)> = Vec::from_ssz_bytes(&bytes[offset..offset + metadata_len])?;
        let metadata: HashMap<Str, ValueExpr> = metadata_pairs.into_iter().collect();
        
        Ok(ResolutionPlan {
            plan_id,
            intent_bundles,
            effect_sequence,
            dataflow_steps,
            resource_transfers,
            target_typed_domain,
            estimated_cost,
            estimated_time_ms,
            metadata,
        })
    }
}

impl SimpleSerialize for ResolutionPlan {}

//-----------------------------------------------------------------------------
// Scored Plan Types
//-----------------------------------------------------------------------------

/// A resolution plan with associated scoring metrics
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredPlan {
    /// The underlying resolution plan
    pub plan: ResolutionPlan,
    /// Overall score (0.0 to 1.0, higher is better)
    pub overall_score: f64,
    /// Cost efficiency score
    pub cost_efficiency_score: f64,
    /// Time efficiency score
    pub time_efficiency_score: f64,
    /// Resource utilization score
    pub resource_utilization_score: f64,
    /// TypedDomain compatibility score
    pub domain_compatibility_score: f64,
    /// Strategy that generated this plan
    pub strategy_name: Str,
    /// Evaluation timestamp
    pub evaluated_at: Timestamp,
}

impl Encode for ScoredPlan {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize plan with length prefix
        let plan_bytes = self.plan.as_ssz_bytes();
        bytes.extend_from_slice(&(plan_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&plan_bytes);
        
        bytes.extend_from_slice(&self.overall_score.to_le_bytes());
        bytes.extend_from_slice(&self.cost_efficiency_score.to_le_bytes());
        bytes.extend_from_slice(&self.time_efficiency_score.to_le_bytes());
        bytes.extend_from_slice(&self.resource_utilization_score.to_le_bytes());
        bytes.extend_from_slice(&self.domain_compatibility_score.to_le_bytes());
        
        // Serialize strategy_name with length prefix
        let strategy_name_bytes = self.strategy_name.as_ssz_bytes();
        bytes.extend_from_slice(&(strategy_name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&strategy_name_bytes);
        
        // Serialize evaluated_at with length prefix
        let evaluated_at_bytes = self.evaluated_at.as_ssz_bytes();
        bytes.extend_from_slice(&(evaluated_at_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&evaluated_at_bytes);
        
        bytes
    }
}

impl Decode for ScoredPlan {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError::new("Insufficient bytes for ScoredPlan"));
        }
        
        let mut offset = 0;
        
        // Read plan with length prefix
        let plan_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + plan_len > bytes.len() {
            return Err(DecodeError::new("Invalid plan length"));
        }
        let plan = ResolutionPlan::from_ssz_bytes(&bytes[offset..offset + plan_len])?;
        offset += plan_len;
        
        if bytes.len() < offset + 40 {
            return Err(DecodeError::new("Insufficient bytes for scores"));
        }
        
        let overall_score = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let cost_efficiency_score = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let time_efficiency_score = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let resource_utilization_score = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        let domain_compatibility_score = f64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;
        
        // Read strategy_name with length prefix
        let strategy_name_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + strategy_name_len > bytes.len() {
            return Err(DecodeError::new("Invalid strategy_name length"));
        }
        let strategy_name = Str::from_ssz_bytes(&bytes[offset..offset + strategy_name_len])?;
        offset += strategy_name_len;
        
        // Read evaluated_at with length prefix
        let evaluated_at_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + evaluated_at_len > bytes.len() {
            return Err(DecodeError::new("Invalid evaluated_at length"));
        }
        let evaluated_at = Timestamp::from_ssz_bytes(&bytes[offset..offset + evaluated_at_len])?;
        
        Ok(ScoredPlan {
            plan,
            overall_score,
            cost_efficiency_score,
            time_efficiency_score,
            resource_utilization_score,
            domain_compatibility_score,
            strategy_name,
            evaluated_at,
        })
    }
}

impl SimpleSerialize for ScoredPlan {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::{DomainId, EntityId};

    #[test]
    fn test_typed_domain_serialization() {
        let domain_id = DomainId::new([1u8; 32]);
        let verifiable = TypedDomain::VerifiableDomain(domain_id);
        let service = TypedDomain::ServiceDomain(domain_id);

        // Test serialization round-trip
        let verifiable_bytes = verifiable.as_ssz_bytes();
        let service_bytes = service.as_ssz_bytes();

        let verifiable_decoded = TypedDomain::from_ssz_bytes(&verifiable_bytes).unwrap();
        let service_decoded = TypedDomain::from_ssz_bytes(&service_bytes).unwrap();

        assert_eq!(verifiable, verifiable_decoded);
        assert_eq!(service, service_decoded);
        assert!(verifiable.is_verifiable());
        assert!(service.is_service());
    }

    #[test]
    
    fn test_resolution_plan_creation() {
        let plan_id = EntityId::new([1u8; 32]);
        let domain_id = DomainId::new([2u8; 32]);
        let typed_domain = TypedDomain::VerifiableDomain(domain_id);

        let plan = ResolutionPlan {
            plan_id,
            intent_bundles: vec![],
            effect_sequence: vec![],
            dataflow_steps: vec![],
            resource_transfers: vec![],
            target_typed_domain: typed_domain,
            estimated_cost: 1000,
            estimated_time_ms: 5000,
            metadata: HashMap::new(),
        };

        // Test serialization
        let bytes = plan.as_ssz_bytes();
        let decoded = ResolutionPlan::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(plan, decoded);
    }
} 