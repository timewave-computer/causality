// This module implements the transformation from TEL combinators to TEG fragments.
// It is part of the F functor in the TEL-TEG adjunction.

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use causality_types::{ContentHash, test_content_hash};
use hex;

use crate::{
    TEGFragment, EffectNode, ResourceNode,
    graph::edge::{Condition, TemporalRelation, RelationshipType},
    effect_node::ParameterValue,
    resource_node::ResourceState
};

/// Trait for converting TEL combinators to TEG fragments.
/// 
/// This trait enables the transformation from TEL combinators
/// to the TEG intermediate representation.
pub trait ToTEGFragment {
    /// Convert the implementing type to a TEG fragment.
    /// 
    /// The resulting fragment can be composed with other fragments
    /// to form a complete Temporal Effect Graph.
    fn to_teg_fragment(&self) -> Result<TEGFragment>;
}

/// Helper function to convert Value to ParameterValue
pub fn value_to_parameter_value(value: &Value) -> Result<ParameterValue> {
    match value {
        Value::Null => Ok(ParameterValue::Null),
        Value::Bool(b) => Ok(ParameterValue::Boolean(*b)),
        Value::Number(n) => {
            if n.is_i64() {
                Ok(ParameterValue::Integer(n.as_i64().unwrap()))
            } else {
                Ok(ParameterValue::Float(n.as_f64().unwrap_or(0.0)))
            }
        },
        Value::String(s) => {
            // If the string starts with "0x", try to parse it as hex bytes
            if s.starts_with("0x") {
                match hex::decode(&s[2..]) {
                    Ok(bytes) => Ok(ParameterValue::Bytes(bytes)),
                    Err(_) => Ok(ParameterValue::String(s.clone())),
                }
            } else {
                Ok(ParameterValue::String(s.clone()))
            }
        },
        Value::Array(arr) => {
            let mut items = Vec::new();
            for item in arr {
                items.push(value_to_parameter_value(item)?);
            }
            Ok(ParameterValue::Array(items))
        },
        Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                map.insert(key.clone(), value_to_parameter_value(val)?);
            }
            Ok(ParameterValue::Object(map))
        }
    }
}

/// Core TEL combinators.
pub enum CoreCombinator {
    /// Identity combinator: I x = x
    Identity,
    
    /// Constant combinator: K x y = x
    Constant(Value),
    
    /// Substitution combinator: S f g x = (f x) (g x)
    Substitution { 
        f_id: String, 
        g_id: String 
    },
    
    /// Composition combinator: B f g x = f (g x)
    Composition { 
        f_id: String, 
        g_id: String 
    },
    
    /// Transform combinator: C f x y = f y x
    Transform { 
        f_id: String 
    },
}

/// Implementation of ToTEGFragment for CoreCombinator
impl ToTEGFragment for CoreCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        match self {
            CoreCombinator::Identity => {
                // Create a fragment with a single identity effect
                let mut fragment = TEGFragment::new();
                
                // Create an identity effect node
                let node_id = "identity".to_string();
                let effect = EffectNode::builder()
                    .id(node_id.clone())
                    .effect_type("identity".to_string())
                    .domain("core".to_string())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                // Add to fragment
                fragment.effect_nodes.insert(node_id.clone(), effect);
                fragment.entry_points.push(node_id.clone());
                fragment.exit_points.push(node_id);
                
                Ok(fragment)
            },
            
            CoreCombinator::Constant(value) => {
                // Create a fragment with a single constant effect
                let mut fragment = TEGFragment::new();
                
                // Create a value string for the ID
                let value_str = value.to_string();
                let node_id = format!("constant_{}", value_str);
                
                // Create a constant effect node with the value parameter
                let mut builder = EffectNode::builder()
                    .id(node_id.clone())
                    .effect_type("constant".to_string())
                    .domain("core".to_string());
                
                // Add the value parameter
                if let Ok(param_value) = value_to_parameter_value(value) {
                    builder = builder.parameter("value".to_string(), param_value);
                }
                
                let effect = builder.build().map_err(|e| anyhow!(e))?;
                
                // Add to fragment
                fragment.effect_nodes.insert(node_id.clone(), effect);
                fragment.entry_points.push(node_id.clone());
                fragment.exit_points.push(node_id);
                
                Ok(fragment)
            },
            
            CoreCombinator::Substitution { f_id, g_id } => {
                // Create a fragment with two reference nodes and a substitution effect
                let mut fragment = TEGFragment::new();
                
                // Create reference nodes for f and g
                let f_ref_id = format!("ref_{}", f_id);
                let f_ref = EffectNode::builder()
                    .id(f_ref_id.clone())
                    .effect_type("reference".to_string())
                    .domain("core".to_string())
                    .string_parameter("name", f_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                let g_ref_id = format!("ref_{}", g_id);
                let g_ref = EffectNode::builder()
                    .id(g_ref_id.clone())
                    .effect_type("reference".to_string())
                    .domain("core".to_string())
                    .string_parameter("name", g_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                // Create a substitution effect
                let node_id = format!("S_{}_{}", f_id, g_id);
                let effect = EffectNode::builder()
                    .id(node_id.clone())
                    .effect_type("substitution".to_string())
                    .domain("core".to_string())
                    .string_parameter("f_id", f_id.clone())
                    .string_parameter("g_id", g_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                // Add to fragment
                fragment.effect_nodes.insert(f_ref_id, f_ref);
                fragment.effect_nodes.insert(g_ref_id, g_ref);
                fragment.effect_nodes.insert(node_id.clone(), effect);
                fragment.entry_points.push(node_id.clone());
                fragment.exit_points.push(node_id);
                
                Ok(fragment)
            },
            
            CoreCombinator::Composition { f_id, g_id } => {
                // Create a fragment with two reference nodes and a composition effect
                let mut fragment = TEGFragment::new();
                
                // Create reference nodes for f and g
                let f_ref_id = format!("ref_{}", f_id);
                let f_ref = EffectNode::builder()
                    .id(f_ref_id.clone())
                    .effect_type("reference".to_string())
                    .domain("core".to_string())
                    .string_parameter("name", f_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                let g_ref_id = format!("ref_{}", g_id);
                let g_ref = EffectNode::builder()
                    .id(g_ref_id.clone())
                    .effect_type("reference".to_string())
                    .domain("core".to_string())
                    .string_parameter("name", g_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                // Create a composition effect
                let node_id = format!("B_{}_{}", f_id, g_id);
                let effect = EffectNode::builder()
                    .id(node_id.clone())
                    .effect_type("composition".to_string())
                    .domain("core".to_string())
                    .string_parameter("f_id", f_id.clone())
                    .string_parameter("g_id", g_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                // Add to fragment
                fragment.effect_nodes.insert(f_ref_id, f_ref);
                fragment.effect_nodes.insert(g_ref_id, g_ref);
                fragment.effect_nodes.insert(node_id.clone(), effect);
                fragment.entry_points.push(node_id.clone());
                fragment.exit_points.push(node_id);
                
                Ok(fragment)
            },
            
            CoreCombinator::Transform { f_id } => {
                // Create a fragment with a reference node and a transform effect
                let mut fragment = TEGFragment::new();
                
                // Create reference node for f
                let f_ref_id = format!("ref_{}", f_id);
                let f_ref = EffectNode::builder()
                    .id(f_ref_id.clone())
                    .effect_type("reference".to_string())
                    .domain("core".to_string())
                    .string_parameter("name", f_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                // Create a transform effect
                let node_id = format!("C_{}", f_id);
                let effect = EffectNode::builder()
                    .id(node_id.clone())
                    .effect_type("transform".to_string())
                    .domain("core".to_string())
                    .string_parameter("f_id", f_id.clone())
                    .build()
                    .map_err(|e| anyhow!(e))?;
                
                // Add to fragment
                fragment.effect_nodes.insert(f_ref_id, f_ref);
                fragment.effect_nodes.insert(node_id.clone(), effect);
                fragment.entry_points.push(node_id.clone());
                fragment.exit_points.push(node_id);
                
                Ok(fragment)
            },
        }
    }
}

/// Application combinator.
pub struct ApplicationCombinator {
    /// Function to apply
    pub function_id: String,
    /// Argument to apply the function to
    pub argument_id: String,
}

/// Implementation of ToTEGFragment for ApplicationCombinator
impl ToTEGFragment for ApplicationCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with an application effect
        let mut fragment = TEGFragment::new();
        
        // Create an application effect node
        let node_id = format!("apply_{}_{}", self.function_id, self.argument_id);
        let effect = EffectNode::builder()
            .id(node_id.clone())
            .effect_type("apply".to_string())
            .domain("core".to_string())
            .string_parameter("function_id", self.function_id.clone())
            .string_parameter("argument_id", self.argument_id.clone())
            .build()
            .map_err(|e| anyhow!(e))?;
        
        // Add to fragment
        fragment.effect_nodes.insert(node_id.clone(), effect);
        fragment.entry_points.push(node_id.clone());
        fragment.exit_points.push(node_id);
        
        Ok(fragment)
    }
}

/// Literal combinator.
pub struct LiteralCombinator {
    /// Value of the literal
    pub value: Value,
}

/// Implementation of ToTEGFragment for LiteralCombinator
impl ToTEGFragment for LiteralCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with a literal effect
        let mut fragment = TEGFragment::new();
        
        // Create a literal effect node
        let value_str = self.value.to_string();
        let node_id = format!("literal_{}", value_str);
        let effect = EffectNode::builder()
            .id(node_id.clone())
            .effect_type("literal".to_string())
            .domain("core".to_string())
            .parameter("value", value_to_parameter_value(&self.value)?)
            .build()
            .map_err(|e| anyhow!(e))?;
        
        // Add to fragment
        fragment.effect_nodes.insert(node_id.clone(), effect);
        fragment.entry_points.push(node_id.clone());
        fragment.exit_points.push(node_id);
        
        Ok(fragment)
    }
}

/// Reference combinator.
pub struct ReferenceCombinator {
    /// Name of the referenced entity
    pub name: String,
}

/// Implementation of ToTEGFragment for ReferenceCombinator
impl ToTEGFragment for ReferenceCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with a reference effect
        let mut fragment = TEGFragment::new();
        
        // Create a reference effect node
        let node_id = format!("ref_{}", self.name);
        let effect = EffectNode::builder()
            .id(node_id.clone())
            .effect_type("reference".to_string())
            .domain("core".to_string())
            .string_parameter("name", self.name.clone())
            .build()
            .map_err(|e| anyhow!(e))?;
        
        // Add to fragment
        fragment.effect_nodes.insert(node_id.clone(), effect);
        fragment.entry_points.push(node_id.clone());
        fragment.exit_points.push(node_id);
        
        Ok(fragment)
    }
}

/// Effect combinator.
pub struct EffectCombinator {
    /// Name of the effect
    pub effect_name: String,
    /// Domain of the effect
    pub domain_id: String,
    /// Parameters for the effect
    pub parameters: HashMap<String, Value>,
    /// Required capabilities for executing the effect
    pub required_capabilities: Vec<String>,
}

/// Implementation of ToTEGFragment for EffectCombinator
impl ToTEGFragment for EffectCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with an effect node
        let mut fragment = TEGFragment::new();
        
        // Create an effect node
        let node_id = format!("effect_{}", self.effect_name);
        let mut builder = EffectNode::builder()
            .id(node_id.clone())
            .effect_type(format!("effect_{}", self.effect_name))
            .domain(self.domain_id.clone());
            
        // Add required capabilities
        for cap in &self.required_capabilities {
            builder = builder.requires_capability(cap.clone());
        }
            
        let effect = builder.build().map_err(|e| anyhow!(e))?;
        
        // Add parameters
        for (name, value) in &self.parameters {
            let param_value = value_to_parameter_value(value)?;
            fragment.effect_nodes.get_mut(&node_id).unwrap().parameters.insert(name.clone(), param_value);
        }
        
        // Add to fragment
        fragment.effect_nodes.insert(node_id.clone(), effect);
        fragment.entry_points.push(node_id.clone());
        fragment.exit_points.push(node_id);
        
        Ok(fragment)
    }
}

/// State transition combinator.
pub struct StateTransitionCombinator {
    /// Resource ID to transition
    pub resource_id: String,
    /// Source state
    pub from_state: Value,
    /// Target state
    pub to_state: Value,
    /// Domain of the resource
    pub domain_id: String,
}

/// Implementation of ToTEGFragment for StateTransitionCombinator
impl ToTEGFragment for StateTransitionCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with a state transition effect
        let mut fragment = TEGFragment::new();
        
        // Create a unique ID for this transition
        let from_state = self.from_state.to_string();
        let to_state = self.to_state.to_string();
        let effect_id = format!("transition_{}_{}_{}",
            self.resource_id, from_state, to_state);
        
        let mut builder = EffectNode::builder()
            .id(effect_id.clone())
            .effect_type("state_transition".to_string())
            .domain(self.domain_id.clone())
            .string_parameter("resource_id", self.resource_id.clone())
            .parameter("from_state", value_to_parameter_value(&self.from_state)?)
            .parameter("to_state", value_to_parameter_value(&self.to_state)?);
            
        // Add resource access and required capability
        builder = builder.accesses_resource(self.resource_id.clone())
                        .requires_capability("state_transition".to_string());
            
        let effect = builder.build().map_err(|e| anyhow!(e))?;
        
        // Add to fragment
        fragment.effect_nodes.insert(effect_id.clone(), effect);
        fragment.entry_points.push(effect_id.clone());
        fragment.exit_points.push(effect_id);
        
        Ok(fragment)
    }
}

/// Content addressing combinator.
pub struct ContentAddressingCombinator {
    /// Content to address
    pub content: Value,
    /// Addressing method
    pub method: String,
}

/// Implementation of ToTEGFragment for ContentAddressingCombinator
impl ToTEGFragment for ContentAddressingCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with a content addressing effect
        let mut fragment = TEGFragment::new();
        
        // Create a unique ID for this operation
        let content_str = self.content.to_string();
        let node_id = format!("content_address_{}_{}", self.method, content_str);
        
        let effect = EffectNode::builder()
            .id(node_id.clone())
            .effect_type("content_addressing".to_string())
            .domain("cryptography".to_string())
            .requires_capability("content_addressing".to_string())
            .parameter("content", value_to_parameter_value(&self.content)?)
            .parameter("method", ParameterValue::String(self.method.clone()))
            .build()
            .map_err(|e| anyhow!(e))?;
        
        // Add to fragment
        fragment.effect_nodes.insert(node_id.clone(), effect);
        fragment.entry_points.push(node_id.clone());
        fragment.exit_points.push(node_id);
        
        Ok(fragment)
    }
}

/// Resource operation combinator
struct ResourceOpCombinator {
    resource_id: String,
    resource_type: String,
    operation: String,
    initial_state: Value,
    metadata: HashMap<String, String>,
    domain_id: String,
}

impl ToTEGFragment for ResourceOpCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with a resource operation effect
        let mut fragment = TEGFragment::new();
        
        // Determine the effect type based on operation
        let effect_type = match self.operation.as_str() {
            "create" => "resource_creation",
            "update" => "resource_update",
            "delete" => "resource_deletion",
            _ => return Err(anyhow!("Unsupported resource operation: {}", self.operation)),
        };
        
        // Create a resource effect node
        let effect_id = format!("{}_{}", effect_type, self.resource_id);
        let mut builder = EffectNode::builder()
            .id(effect_id.clone())
            .effect_type(effect_type.to_string())
            .domain(self.domain_id.clone())
            .requires_capability(format!("{}", effect_type))
            .accesses_resource(self.resource_id.clone())
            .string_parameter("resource_id", self.resource_id.clone())
            .string_parameter("resource_type", self.resource_type.clone());
            
        // Add initial state for creation operations
        if effect_type == "resource_creation" {
            builder = builder.parameter("initial_state", value_to_parameter_value(&self.initial_state)?);
        }
            
        // Add metadata
        for (key, value) in &self.metadata {
            builder = builder.string_parameter(format!("metadata_{}", key), value.clone());
        }
        
        let effect = builder.build().map_err(|e| anyhow!(e))?;
        
        // Add to fragment
        fragment.effect_nodes.insert(effect_id.clone(), effect);
        fragment.entry_points.push(effect_id.clone());
        fragment.exit_points.push(effect_id);
        
        Ok(fragment)
    }
}

/// Query combinator.
pub struct QueryCombinator {
    /// Resource ID to query
    pub resource_id: String,
    /// Query type
    pub query_type: String,
    /// Query parameters
    pub parameters: HashMap<String, Value>,
    /// Domain of the resource
    pub domain_id: String,
}

/// Implementation of ToTEGFragment for QueryCombinator
impl ToTEGFragment for QueryCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with a query effect
        let mut fragment = TEGFragment::new();
        
        // Create a query effect node
        let node_id = format!("query_{}_{}", self.resource_id, self.query_type);
        let mut builder = EffectNode::builder()
            .id(node_id.clone())
            .effect_type("resource_query".to_string())
            .domain(self.domain_id.clone())
            .requires_capability(format!("resource_query_{}", self.domain_id))
            .accesses_resource(self.resource_id.clone())
            .parameter("resource_id", ParameterValue::String(self.resource_id.clone()))
            .parameter("query_type", ParameterValue::String(self.query_type.clone()));
            
        // Add query parameters
        for (name, value) in &self.parameters {
            builder = builder.parameter(name.clone(), value_to_parameter_value(value)?);
        }
        
        let effect = builder.build().map_err(|e| anyhow!(e))?;
        
        // Add to fragment
        fragment.effect_nodes.insert(node_id.clone(), effect);
        fragment.entry_points.push(node_id.clone());
        fragment.exit_points.push(node_id);
        
        Ok(fragment)
    }
}

/// Convert a TEL combinator to a TEG fragment.
/// 
/// This is a convenience function that wraps the trait method.
pub fn to_teg_fragment<T: ToTEGFragment>(tel_combinator: &T) -> Result<TEGFragment> {
    tel_combinator.to_teg_fragment()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use crate::resource_node::ResourceState;
    use crate::effect_node::ParameterValue;
    
    /// Test for the test_content_hash function
    #[test]
    fn test_content_hash_directly() {
        let hash = test_content_hash();
        assert_eq!(hash.algorithm, "sha256");
        assert_eq!(hash.bytes.len(), 32);
        // All bytes should be zero
        for byte in &hash.bytes {
            assert_eq!(*byte, 0);
        }
    }
    
    #[test]
    fn test_core_identity() {
        let combinator = CoreCombinator::Identity;
        let fragment = combinator.to_teg_fragment().unwrap();
        
        // Check that we have the identity effect node
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.entry_points.len(), 1);
        assert_eq!(fragment.exit_points.len(), 1);
        
        let effect = &fragment.effect_nodes[&fragment.entry_points[0]];
        assert_eq!(effect.effect_type, "identity");
    }
    
    #[test]
    fn test_core_constant() {
        let value = json!("test");
        let combinator = CoreCombinator::Constant(value.clone());
        let fragment = combinator.to_teg_fragment().unwrap();
        
        // Check that we have the constant effect node
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.entry_points.len(), 1);
        assert_eq!(fragment.exit_points.len(), 1);
        
        let effect = &fragment.effect_nodes[&fragment.entry_points[0]];
        assert_eq!(effect.effect_type, "constant");
        assert_eq!(effect.parameters["value"], value_to_parameter_value(&value).unwrap());
    }
    
    #[test]
    fn test_literal_combinator() {
        let value = json!({
            "key": "value",
            "number": 42
        });
        
        let literal = LiteralCombinator {
            value: value.clone(),
        };
        
        let fragment = literal.to_teg_fragment().unwrap();
        
        // Check that we have the literal effect node
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.entry_points.len(), 1);
        assert_eq!(fragment.exit_points.len(), 1);
        
        let effect = &fragment.effect_nodes[&fragment.entry_points[0]];
        assert_eq!(effect.effect_type, "literal");
        assert_eq!(effect.parameters["value"], value_to_parameter_value(&value).unwrap());
    }
    
    #[test]
    fn test_reference_combinator() {
        let reference = ReferenceCombinator {
            name: "test_name".to_string(),
        };
        
        let fragment = reference.to_teg_fragment().unwrap();
        
        // Check that we have the reference effect node
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.entry_points.len(), 1);
        assert_eq!(fragment.exit_points.len(), 1);
        
        let effect = &fragment.effect_nodes[&fragment.entry_points[0]];
        assert_eq!(effect.effect_type, "reference");
        assert_eq!(effect.parameters["name"], ParameterValue::String("test_name".to_string()));
    }
    
    #[test]
    fn test_effect_combinator() {
        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), json!("value1"));
        parameters.insert("param2".to_string(), json!(42));
        
        let effect = EffectCombinator {
            effect_name: "test_effect".to_string(),
            parameters,
            required_capabilities: vec!["capability1".to_string()],
            domain_id: "test_domain".to_string(),
        };
        
        let fragment = effect.to_teg_fragment().unwrap();
        
        // Check that we have the effect node
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.entry_points.len(), 1);
        assert_eq!(fragment.exit_points.len(), 1);
        
        let effect_node = &fragment.effect_nodes[&fragment.entry_points[0]];
        assert_eq!(effect_node.effect_type, "effect_test_effect");
        assert_eq!(effect_node.domain_id, "test_domain");
        assert!(effect_node.required_capabilities.contains(&"capability1".to_string()));
    }
    
    #[test]
    fn test_resource_operation_combinator() {
        let initial_state = json!({});
        let mut metadata = HashMap::new();
        metadata.insert("owner".to_string(), "test_user".to_string());
        
        let resource_op = ResourceOpCombinator {
            resource_id: "resource1".to_string(),
            resource_type: "test_resource".to_string(),
            operation: "create".to_string(),
            initial_state: initial_state.clone(),
            metadata: metadata.clone(),
            domain_id: "test_domain".to_string(),
        };
        
        let fragment = resource_op.to_teg_fragment().unwrap();
        
        // Check that we have the right nodes
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.entry_points.len(), 1);
        assert_eq!(fragment.exit_points.len(), 1);
        
        let effect = &fragment.effect_nodes[&fragment.entry_points[0]];
        assert_eq!(effect.effect_type, "resource_creation");
        assert_eq!(effect.domain_id, "test_domain");
        assert_eq!(effect.parameters["resource_id"], ParameterValue::String("resource1".to_string()));
        assert_eq!(effect.parameters["resource_type"], ParameterValue::String("test_resource".to_string()));
        
        // Convert and compare initial_state
        let expected = value_to_parameter_value(&initial_state).unwrap();
        assert_eq!(effect.parameters["initial_state"], expected);
    }

    #[test]
    fn test_content_hash_usage() {
        // Just check that test_content_hash function can be used successfully
        let hash = test_content_hash();
        assert_eq!(hash.algorithm, "sha256");
        assert_eq!(hash.bytes.len(), 32);
        for byte in &hash.bytes {
            assert_eq!(*byte, 0);
        }
    }
}
