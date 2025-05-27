//! ProcessDataflowBlock types for runtime optimization
//!
//! This module defines types for managing complex, multi-step workflows that can be
//! orchestrated by the optimization framework. ProcessDataflowBlocks represent
//! higher-level business processes that span multiple effects and domains.

use crate::{
    core::{
        id::{ExprId, ResourceId},
        str::Str,
        time::Timestamp,
    },
    tel::optimization::TypedDomain,
    expr::value::ValueExpr,
    serialization::{Encode, Decode, DecodeError, SimpleSerialize},
};
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// ProcessDataflowBlock Definition Types
//-----------------------------------------------------------------------------

/// Definition of a ProcessDataflowBlock workflow
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowDefinition {
    /// Unique identifier for this dataflow definition
    pub id: ExprId,
    /// Human-readable name
    pub name: Str,
    /// Input schema definition
    pub input_schema: ValueExpr,
    /// Output schema definition
    pub output_schema: ValueExpr,
    /// State schema definition
    pub state_schema: ValueExpr,
    /// Workflow nodes
    pub nodes: Vec<DataflowNode>,
    /// Workflow edges
    pub edges: Vec<DataflowEdge>,
    /// Conditional logic
    pub conditions: Vec<DataflowCondition>,
    /// Action templates for effects
    pub action_templates: Vec<ActionTemplate>,
    /// TypedDomain policies per node
    pub domain_policies: HashMap<Str, TypedDomain>,
    /// Creation timestamp
    pub created_at: Timestamp,
}

/// A node in the dataflow workflow
#[derive(Debug, Clone, PartialEq)]
pub struct DataflowNode {
    /// Node identifier
    pub id: Str,
    /// Node type (e.g., "action", "condition", "merge", "split")
    pub node_type: Str,
    /// Node configuration
    pub config: ValueExpr,
    /// Required TypedDomain for execution
    pub required_domain: Option<TypedDomain>,
}

/// An edge connecting dataflow nodes
#[derive(Debug, Clone, PartialEq)]
pub struct DataflowEdge {
    /// Source node ID
    pub from_node: Str,
    /// Target node ID
    pub to_node: Str,
    /// Edge condition (optional)
    pub condition: Option<ExprId>,
    /// Data transformation (optional)
    pub transform: Option<ExprId>,
}

/// Conditional logic for dataflow execution
#[derive(Debug, Clone, PartialEq)]
pub struct DataflowCondition {
    /// Condition identifier
    pub id: Str,
    /// Condition expression
    pub expression: ExprId,
    /// True branch action
    pub true_action: Str,
    /// False branch action
    pub false_action: Str,
}

/// Template for generating effects from dataflow nodes
#[derive(Debug, Clone, PartialEq)]
pub struct ActionTemplate {
    /// Template identifier
    pub id: Str,
    /// Effect type to generate
    pub effect_type: Str,
    /// Parameter mapping expression
    pub parameter_mapping: ExprId,
    /// Target TypedDomain
    pub target_domain: TypedDomain,
}

//-----------------------------------------------------------------------------
// ProcessDataflowBlock Instance Types
//-----------------------------------------------------------------------------

/// Runtime state of a ProcessDataflowBlock instance
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowInstanceState {
    /// Instance identifier
    pub id: ResourceId,
    /// Reference to the definition
    pub definition_id: ExprId,
    /// Current node in the workflow
    pub current_node_id: Str,
    /// Current state values
    pub state_values: ValueExpr,
    /// Execution history
    pub execution_history: Vec<ExecutionStep>,
    /// Instance status
    pub status: InstanceStatus,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
}

/// A step in the execution history
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionStep {
    /// Step identifier
    pub step_id: Str,
    /// Node that was executed
    pub node_id: Str,
    /// Input parameters
    pub input_params: ValueExpr,
    /// Output results
    pub output_results: ValueExpr,
    /// Execution timestamp
    pub executed_at: Timestamp,
    /// TypedDomain where execution occurred
    pub execution_domain: TypedDomain,
}

/// Status of a ProcessDataflowBlock instance
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceStatus {
    /// Instance is ready to start
    Ready,
    /// Instance is currently executing
    Running,
    /// Instance is waiting for external input
    Waiting,
    /// Instance completed successfully
    Completed,
    /// Instance failed with error
    Failed,
    /// Instance was cancelled
    Cancelled,
}

//-----------------------------------------------------------------------------
// ProcessDataflowBlock Reference Types
//-----------------------------------------------------------------------------

/// Reference to a ProcessDataflowBlock (definition or instance)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessDataflowReference {
    /// Reference to a definition
    DefinitionId(ExprId),
    /// Reference to an instance
    InstanceId(ResourceId),
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

impl Encode for ProcessDataflowDefinition {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize each field with length prefix
        let id_bytes = self.id.as_ssz_bytes();
        bytes.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&id_bytes);
        
        let name_bytes = self.name.as_ssz_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&name_bytes);
        
        let input_schema_bytes = self.input_schema.as_ssz_bytes();
        bytes.extend_from_slice(&(input_schema_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&input_schema_bytes);
        
        let output_schema_bytes = self.output_schema.as_ssz_bytes();
        bytes.extend_from_slice(&(output_schema_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&output_schema_bytes);
        
        let state_schema_bytes = self.state_schema.as_ssz_bytes();
        bytes.extend_from_slice(&(state_schema_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&state_schema_bytes);
        
        let nodes_bytes = self.nodes.as_ssz_bytes();
        bytes.extend_from_slice(&(nodes_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&nodes_bytes);
        
        let edges_bytes = self.edges.as_ssz_bytes();
        bytes.extend_from_slice(&(edges_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&edges_bytes);
        
        let conditions_bytes = self.conditions.as_ssz_bytes();
        bytes.extend_from_slice(&(conditions_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&conditions_bytes);
        
        let action_templates_bytes = self.action_templates.as_ssz_bytes();
        bytes.extend_from_slice(&(action_templates_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&action_templates_bytes);
        
        // Convert HashMap to Vec of pairs for serialization with length prefix
        let policy_pairs: Vec<(Str, TypedDomain)> = self.domain_policies.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let policy_pairs_bytes = policy_pairs.as_ssz_bytes();
        bytes.extend_from_slice(&(policy_pairs_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&policy_pairs_bytes);
        
        let created_at_bytes = self.created_at.as_ssz_bytes();
        bytes.extend_from_slice(&(created_at_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&created_at_bytes);
        
        bytes
    }
}

impl Decode for ProcessDataflowDefinition {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError::new("Insufficient data for ProcessDataflowDefinition"));
        }
        
        let mut offset = 0;
        
        // Read id with length prefix
        let id_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + id_len > bytes.len() {
            return Err(DecodeError::new("Invalid id length"));
        }
        let id = ExprId::from_ssz_bytes(&bytes[offset..offset + id_len])?;
        offset += id_len;
        
        // Read name with length prefix
        let name_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + name_len > bytes.len() {
            return Err(DecodeError::new("Invalid name length"));
        }
        let name = Str::from_ssz_bytes(&bytes[offset..offset + name_len])?;
        offset += name_len;
        
        // Read input_schema with length prefix
        let input_schema_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + input_schema_len > bytes.len() {
            return Err(DecodeError::new("Invalid input_schema length"));
        }
        let input_schema = ValueExpr::from_ssz_bytes(&bytes[offset..offset + input_schema_len])?;
        offset += input_schema_len;
        
        // Read output_schema with length prefix
        let output_schema_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + output_schema_len > bytes.len() {
            return Err(DecodeError::new("Invalid output_schema length"));
        }
        let output_schema = ValueExpr::from_ssz_bytes(&bytes[offset..offset + output_schema_len])?;
        offset += output_schema_len;
        
        // Read state_schema with length prefix
        let state_schema_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + state_schema_len > bytes.len() {
            return Err(DecodeError::new("Invalid state_schema length"));
        }
        let state_schema = ValueExpr::from_ssz_bytes(&bytes[offset..offset + state_schema_len])?;
        offset += state_schema_len;
        
        // Read nodes with length prefix
        let nodes_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + nodes_len > bytes.len() {
            return Err(DecodeError::new("Invalid nodes length"));
        }
        let nodes = Vec::<DataflowNode>::from_ssz_bytes(&bytes[offset..offset + nodes_len])?;
        offset += nodes_len;
        
        // Read edges with length prefix
        let edges_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + edges_len > bytes.len() {
            return Err(DecodeError::new("Invalid edges length"));
        }
        let edges = Vec::<DataflowEdge>::from_ssz_bytes(&bytes[offset..offset + edges_len])?;
        offset += edges_len;
        
        // Read conditions with length prefix
        let conditions_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + conditions_len > bytes.len() {
            return Err(DecodeError::new("Invalid conditions length"));
        }
        let conditions = Vec::<DataflowCondition>::from_ssz_bytes(&bytes[offset..offset + conditions_len])?;
        offset += conditions_len;
        
        // Read action_templates with length prefix
        let action_templates_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + action_templates_len > bytes.len() {
            return Err(DecodeError::new("Invalid action_templates length"));
        }
        let action_templates = Vec::<ActionTemplate>::from_ssz_bytes(&bytes[offset..offset + action_templates_len])?;
        offset += action_templates_len;
        
        // Read domain_policies with length prefix
        let policy_pairs_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + policy_pairs_len > bytes.len() {
            return Err(DecodeError::new("Invalid domain_policies length"));
        }
        let policy_pairs: Vec<(Str, TypedDomain)> = Vec::from_ssz_bytes(&bytes[offset..offset + policy_pairs_len])?;
        let domain_policies: HashMap<Str, TypedDomain> = policy_pairs.into_iter().collect();
        offset += policy_pairs_len;
        
        // Read created_at with length prefix
        let created_at_len = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + created_at_len > bytes.len() {
            return Err(DecodeError::new("Invalid created_at length"));
        }
        let created_at = Timestamp::from_ssz_bytes(&bytes[offset..offset + created_at_len])?;
        
        Ok(ProcessDataflowDefinition {
            id,
            name,
            input_schema,
            output_schema,
            state_schema,
            nodes,
            edges,
            conditions,
            action_templates,
            domain_policies,
            created_at,
        })
    }
}

impl SimpleSerialize for ProcessDataflowDefinition {}

impl Encode for DataflowNode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.node_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.config.as_ssz_bytes());
        
        // Handle optional required domain
        if let Some(ref domain) = self.required_domain {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&domain.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        bytes
    }
}

impl Decode for DataflowNode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let node_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += node_type.as_ssz_bytes().len();
        
        let config = ValueExpr::from_ssz_bytes(&bytes[offset..])?;
        offset += config.as_ssz_bytes().len();
        
        // Handle optional required domain
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for required domain marker".to_string() });
        }
        
        let required_domain = if bytes[offset] == 1 {
            offset += 1;
            Some(TypedDomain::from_ssz_bytes(&bytes[offset..])?)
        } else {
            offset += 1;
            None
        };
        
        Ok(DataflowNode {
            id,
            node_type,
            config,
            required_domain,
        })
    }
}

impl SimpleSerialize for DataflowNode {}

impl Encode for DataflowEdge {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.from_node.as_ssz_bytes());
        bytes.extend_from_slice(&self.to_node.as_ssz_bytes());
        
        // Handle optional condition
        if let Some(ref condition) = self.condition {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&condition.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        // Handle optional transform
        if let Some(ref transform) = self.transform {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&transform.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        bytes
    }
}

impl Decode for DataflowEdge {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let from_node = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += from_node.as_ssz_bytes().len();
        
        let to_node = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += to_node.as_ssz_bytes().len();
        
        // Handle optional condition
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for condition marker".to_string() });
        }
        
        let condition = if bytes[offset] == 1 {
            offset += 1;
            Some(ExprId::from_ssz_bytes(&bytes[offset..])?)
        } else {
            offset += 1;
            None
        };
        
        if let Some(ref cond) = condition {
            offset += cond.as_ssz_bytes().len();
        }
        
        // Handle optional transform
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for transform marker".to_string() });
        }
        
        let transform = if bytes[offset] == 1 {
            offset += 1;
            Some(ExprId::from_ssz_bytes(&bytes[offset..])?)
        } else {
            offset += 1;
            None
        };
        
        Ok(DataflowEdge {
            from_node,
            to_node,
            condition,
            transform,
        })
    }
}

impl SimpleSerialize for DataflowEdge {}

impl Encode for DataflowCondition {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.expression.as_ssz_bytes());
        bytes.extend_from_slice(&self.true_action.as_ssz_bytes());
        bytes.extend_from_slice(&self.false_action.as_ssz_bytes());
        bytes
    }
}

impl Decode for DataflowCondition {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let expression = ExprId::from_ssz_bytes(&bytes[offset..])?;
        offset += expression.as_ssz_bytes().len();
        
        let true_action = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += true_action.as_ssz_bytes().len();
        
        let false_action = Str::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(DataflowCondition {
            id,
            expression,
            true_action,
            false_action,
        })
    }
}

impl SimpleSerialize for DataflowCondition {}

impl Encode for ActionTemplate {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.effect_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.parameter_mapping.as_ssz_bytes());
        bytes.extend_from_slice(&self.target_domain.as_ssz_bytes());
        bytes
    }
}

impl Decode for ActionTemplate {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let effect_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += effect_type.as_ssz_bytes().len();
        
        let parameter_mapping = ExprId::from_ssz_bytes(&bytes[offset..])?;
        offset += parameter_mapping.as_ssz_bytes().len();
        
        let target_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ActionTemplate {
            id,
            effect_type,
            parameter_mapping,
            target_domain,
        })
    }
}

impl SimpleSerialize for ActionTemplate {}

impl Encode for ProcessDataflowInstanceState {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.definition_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.current_node_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.state_values.as_ssz_bytes());
        bytes.extend_from_slice(&self.execution_history.as_ssz_bytes());
        bytes.extend_from_slice(&self.status.as_ssz_bytes());
        bytes.extend_from_slice(&self.created_at.as_ssz_bytes());
        bytes.extend_from_slice(&self.updated_at.as_ssz_bytes());
        bytes
    }
}

impl Decode for ProcessDataflowInstanceState {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let id = ResourceId::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let definition_id = ExprId::from_ssz_bytes(&bytes[offset..])?;
        offset += definition_id.as_ssz_bytes().len();
        
        let current_node_id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += current_node_id.as_ssz_bytes().len();
        
        let state_values = ValueExpr::from_ssz_bytes(&bytes[offset..])?;
        offset += state_values.as_ssz_bytes().len();
        
        let execution_history = Vec::<ExecutionStep>::from_ssz_bytes(&bytes[offset..])?;
        offset += execution_history.as_ssz_bytes().len();
        
        let status = InstanceStatus::from_ssz_bytes(&bytes[offset..])?;
        offset += status.as_ssz_bytes().len();
        
        let created_at = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        offset += created_at.as_ssz_bytes().len();
        
        let updated_at = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ProcessDataflowInstanceState {
            id,
            definition_id,
            current_node_id,
            state_values,
            execution_history,
            status,
            created_at,
            updated_at,
        })
    }
}

impl SimpleSerialize for ProcessDataflowInstanceState {}

impl Encode for ExecutionStep {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.step_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.node_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.input_params.as_ssz_bytes());
        bytes.extend_from_slice(&self.output_results.as_ssz_bytes());
        bytes.extend_from_slice(&self.executed_at.as_ssz_bytes());
        bytes.extend_from_slice(&self.execution_domain.as_ssz_bytes());
        bytes
    }
}

impl Decode for ExecutionStep {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let step_id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += step_id.as_ssz_bytes().len();
        
        let node_id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += node_id.as_ssz_bytes().len();
        
        let input_params = ValueExpr::from_ssz_bytes(&bytes[offset..])?;
        offset += input_params.as_ssz_bytes().len();
        
        let output_results = ValueExpr::from_ssz_bytes(&bytes[offset..])?;
        offset += output_results.as_ssz_bytes().len();
        
        let executed_at = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        offset += executed_at.as_ssz_bytes().len();
        
        let execution_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ExecutionStep {
            step_id,
            node_id,
            input_params,
            output_results,
            executed_at,
            execution_domain,
        })
    }
}

impl SimpleSerialize for ExecutionStep {}

impl Encode for InstanceStatus {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        vec![match self {
            InstanceStatus::Ready => 0,
            InstanceStatus::Running => 1,
            InstanceStatus::Waiting => 2,
            InstanceStatus::Completed => 3,
            InstanceStatus::Failed => 4,
            InstanceStatus::Cancelled => 5,
        }]
    }
}

impl Decode for InstanceStatus {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for InstanceStatus".to_string() });
        }
        
        match bytes[0] {
            0 => Ok(InstanceStatus::Ready),
            1 => Ok(InstanceStatus::Running),
            2 => Ok(InstanceStatus::Waiting),
            3 => Ok(InstanceStatus::Completed),
            4 => Ok(InstanceStatus::Failed),
            5 => Ok(InstanceStatus::Cancelled),
            _ => Err(DecodeError { message: format!("Invalid InstanceStatus variant: {}", bytes[0]) }),
        }
    }
}

impl SimpleSerialize for InstanceStatus {}

impl Encode for ProcessDataflowReference {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ProcessDataflowReference::DefinitionId(id) => {
                bytes.push(0); // variant tag
                bytes.extend_from_slice(&id.as_ssz_bytes());
            }
            ProcessDataflowReference::InstanceId(id) => {
                bytes.push(1); // variant tag
                bytes.extend_from_slice(&id.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for ProcessDataflowReference {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for ProcessDataflowReference".to_string() });
        }
        
        let variant = bytes[0];
        match variant {
            0 => {
                let id = ExprId::from_ssz_bytes(&bytes[1..])?;
                Ok(ProcessDataflowReference::DefinitionId(id))
            }
            1 => {
                let id = ResourceId::from_ssz_bytes(&bytes[1..])?;
                Ok(ProcessDataflowReference::InstanceId(id))
            }
            _ => Err(DecodeError { message: format!("Invalid ProcessDataflowReference variant: {}", variant) }),
        }
    }
}

impl SimpleSerialize for ProcessDataflowReference {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    
    fn test_dataflow_definition_creation() {
        let definition = ProcessDataflowDefinition {
            id: ExprId::new([1u8; 32]),
            name: Str::from("test_dataflow"),
            input_schema: ValueExpr::Number(42i64.into()),
            output_schema: ValueExpr::Number(43i64.into()),
            state_schema: ValueExpr::Number(44i64.into()),
            nodes: vec![],
            edges: vec![],
            conditions: vec![],
            action_templates: vec![],
            domain_policies: HashMap::new(),
            created_at: Timestamp::now(),
        };

        // Test serialization
        let bytes = definition.as_ssz_bytes();
        let decoded = ProcessDataflowDefinition::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(definition, decoded);
    }

    #[test]
    fn test_instance_status_serialization() {
        let statuses = vec![
            InstanceStatus::Ready,
            InstanceStatus::Running,
            InstanceStatus::Waiting,
            InstanceStatus::Completed,
            InstanceStatus::Failed,
            InstanceStatus::Cancelled,
        ];

        for status in statuses {
            let bytes = status.as_ssz_bytes();
            let decoded = InstanceStatus::from_ssz_bytes(&bytes).unwrap();
            assert_eq!(status, decoded);
        }
    }

    #[test]
    fn test_dataflow_reference_serialization() {
        let def_ref = ProcessDataflowReference::DefinitionId(ExprId::new([1u8; 32]));
        let inst_ref = ProcessDataflowReference::InstanceId(ResourceId::new([2u8; 32]));

        let def_bytes = def_ref.as_ssz_bytes();
        let inst_bytes = inst_ref.as_ssz_bytes();

        let def_decoded = ProcessDataflowReference::from_ssz_bytes(&def_bytes).unwrap();
        let inst_decoded = ProcessDataflowReference::from_ssz_bytes(&inst_bytes).unwrap();

        assert_eq!(def_ref, def_decoded);
        assert_eq!(inst_ref, inst_decoded);
    }
} 