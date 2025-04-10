// This module implements the transformation from TEL combinators to TEG fragments.
// It is part of the F functor in the TEL-TEG adjunction.

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use causality_types::ContentHash;

use crate::{
    TEGFragment, EffectNode, ResourceNode,
    graph::edge::{Condition, TemporalRelation, RelationshipType},
    effect_node::ParameterValue,
    resource_node::ResourceState
};

/// Create a test content hash
fn test_content_hash() -> ContentHash {
    ContentHash::new("sha256", vec![0; 32])
}

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
fn value_to_parameter_value(value: &Value) -> Result<ParameterValue> {
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
        Value::String(s) => Ok(ParameterValue::String(s.clone())),
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
                let effect = EffectNode {
                    id: node_id.clone(),
                    effect_type: "identity".to_string(),
                    parameters: HashMap::new(),
                    required_capabilities: vec![],
                    resources_accessed: vec![],
                    fact_dependencies: vec![],
                    domain_id: "core".to_string(),
                    content_hash: test_content_hash(),
                };
                
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
                
                // Add the value parameter
                let mut parameters = HashMap::new();
                parameters.insert("value".to_string(), value_to_parameter_value(value)?);
                
                // Create a constant effect node
                let effect = EffectNode {
                    id: node_id.clone(),
                    effect_type: "constant".to_string(),
                    parameters,
                    required_capabilities: vec![],
                    resources_accessed: vec![],
                    fact_dependencies: vec![],
                    domain_id: "core".to_string(),
                    content_hash: test_content_hash(),
                };
                
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
                let mut f_ref_params = HashMap::new();
                f_ref_params.insert("name".to_string(), ParameterValue::String(f_id.clone()));
                
                let f_ref = EffectNode {
                    id: f_ref_id.clone(),
                    effect_type: "reference".to_string(),
                    parameters: f_ref_params,
                    required_capabilities: vec![],
                    resources_accessed: vec![],
                    fact_dependencies: vec![],
                    domain_id: "core".to_string(),
                    content_hash: test_content_hash(),
                };
                
                let g_ref_id = format!("ref_{}", g_id);
                let mut g_ref_params = HashMap::new();
                g_ref_params.insert("name".to_string(), ParameterValue::String(g_id.clone()));
                
                let g_ref = EffectNode {
                    id: g_ref_id.clone(),
                    effect_type: "reference".to_string(),
                    parameters: g_ref_params,
                    required_capabilities: vec![],
                    resources_accessed: vec![],
                    fact_dependencies: vec![],
                    domain_id: "core".to_string(),
                    content_hash: test_content_hash(),
                };
                
                // Create a substitution effect
                let node_id = format!("S_{}_{}", f_id, g_id);
                let mut effect_params = HashMap::new();
                effect_params.insert("f_id".to_string(), ParameterValue::String(f_id.clone()));
                effect_params.insert("g_id".to_string(), ParameterValue::String(g_id.clone()));
                
                let effect = EffectNode {
                    id: node_id.clone(),
                    effect_type: "substitution".to_string(),
                    parameters: effect_params,
                    required_capabilities: vec![],
                    resources_accessed: vec![],
                    fact_dependencies: vec![],
                    domain_id: "core".to_string(),
                    content_hash: test_content_hash(),
                };
                
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
                let mut f_ref = EffectNode::new(f_ref_id.clone());
                f_ref.effect_type = "reference".to_string();
                f_ref.domain_id = "core".to_string();
                f_ref.parameters.insert("name".to_string(), ParameterValue::String(f_id.clone()));
                
                let g_ref_id = format!("ref_{}", g_id);
                let mut g_ref = EffectNode::new(g_ref_id.clone());
                g_ref.effect_type = "reference".to_string();
                g_ref.domain_id = "core".to_string();
                g_ref.parameters.insert("name".to_string(), ParameterValue::String(g_id.clone()));
                
                // Create a composition effect
                let node_id = format!("B_{}_{}", f_id, g_id);
                let mut effect = EffectNode::new(node_id.clone());
                effect.effect_type = "composition".to_string();
                effect.domain_id = "core".to_string();
                effect.parameters.insert("f_id".to_string(), ParameterValue::String(f_id.clone()));
                effect.parameters.insert("g_id".to_string(), ParameterValue::String(g_id.clone()));
                
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
                let mut f_ref = EffectNode::new(f_ref_id.clone());
                f_ref.effect_type = "reference".to_string();
                f_ref.domain_id = "core".to_string();
                f_ref.parameters.insert("name".to_string(), ParameterValue::String(f_id.clone()));
                
                // Create a transform effect
                let node_id = format!("C_{}", f_id);
                let mut effect = EffectNode::new(node_id.clone());
                effect.effect_type = "transform".to_string();
                effect.domain_id = "core".to_string();
                effect.parameters.insert("f_id".to_string(), ParameterValue::String(f_id.clone()));
                
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
        let mut effect = EffectNode::new(node_id.clone());
        effect.effect_type = "apply".to_string();
        effect.domain_id = "core".to_string();
        effect.parameters.insert("function_id".to_string(), ParameterValue::String(self.function_id.clone()));
        effect.parameters.insert("argument_id".to_string(), ParameterValue::String(self.argument_id.clone()));
        
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
        let mut effect = EffectNode::new(node_id.clone());
        effect.effect_type = "literal".to_string();
        effect.domain_id = "core".to_string();
        effect.parameters.insert("value".to_string(), value_to_parameter_value(&self.value)?);
        
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
        let mut effect = EffectNode::new(node_id.clone());
        effect.effect_type = "reference".to_string();
        effect.domain_id = "core".to_string();
        effect.parameters.insert("name".to_string(), ParameterValue::String(self.name.clone()));
        
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
        let mut effect = EffectNode::new(node_id.clone());
        effect.effect_type = format!("effect_{}", self.effect_name);
        effect.domain_id = self.domain_id.clone();
        effect.required_capabilities = self.required_capabilities.clone();
        
        // Add parameters
        for (name, value) in &self.parameters {
            effect.parameters.insert(name.clone(), value_to_parameter_value(value)?);
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
        // Create a fragment with a state transition effect and resource nodes
        let mut fragment = TEGFragment::new();
        
        // Convert Value to ResourceState
        let from_state = match &self.from_state {
            Value::String(s) => match s.as_str() {
                "active" => ResourceState::Active,
                "frozen" => ResourceState::Frozen,
                "locked" => ResourceState::Locked,
                "inactive" => ResourceState::Inactive,
                _ => ResourceState::Custom(s.clone()),
            },
            _ => ResourceState::Custom(self.from_state.to_string()),
        };
        
        let to_state = match &self.to_state {
            Value::String(s) => match s.as_str() {
                "active" => ResourceState::Active,
                "frozen" => ResourceState::Frozen,
                "locked" => ResourceState::Locked,
                "inactive" => ResourceState::Inactive,
                _ => ResourceState::Custom(s.clone()),
            },
            _ => ResourceState::Custom(self.to_state.to_string()),
        };
        
        // Create resource nodes for from and to states
        let from_resource_id = format!("{}_from", self.resource_id);
        let mut from_resource = ResourceNode::new(
            from_resource_id.clone(),
            format!("{}_type", self.resource_id),
            from_state,
            self.domain_id.clone(),
        );
        
        let to_resource_id = format!("{}_to", self.resource_id);
        let mut to_resource = ResourceNode::new(
            to_resource_id.clone(),
            format!("{}_type", self.resource_id),
            to_state,
            self.domain_id.clone(),
        );
        
        // Create a state transition effect node
        let effect_id = format!("transition_{}_{}_{}", 
            self.resource_id, from_state, to_state);
        
        let mut effect = EffectNode::new(effect_id.clone());
        effect.effect_type = "state_transition".to_string();
        effect.domain_id = self.domain_id.clone();
        effect.resources_accessed = vec![self.resource_id.clone()];
        effect.required_capabilities = vec!["state_transition".to_string()];
        
        // Add parameters
        effect.parameters.insert("resource_id".to_string(), ParameterValue::String(self.resource_id.clone()));
        effect.parameters.insert("from_state".to_string(), value_to_parameter_value(&self.from_state)?);
        effect.parameters.insert("to_state".to_string(), value_to_parameter_value(&self.to_state)?);
        
        // Add to fragment
        fragment.resource_nodes.insert(from_resource_id.clone(), from_resource);
        fragment.resource_nodes.insert(to_resource_id.clone(), to_resource);
        fragment.effect_nodes.insert(effect_id.clone(), effect);
        
        // Add relationship between resources
        fragment.resource_relationships.insert(
            from_resource_id.clone(),
            vec![(to_resource_id.clone(), RelationshipType::StateTransition)],
        );
        
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
        
        // Create a content addressing effect node
        let content_str = self.content.to_string();
        let node_id = format!("content_address_{}_{}", self.method, content_str);
        let mut effect = EffectNode::new(node_id.clone());
        effect.effect_type = "content_addressing".to_string();
        effect.domain_id = "cryptography".to_string();
        effect.required_capabilities = vec!["content_addressing".to_string()];
        
        // Add parameters
        effect.parameters.insert("content".to_string(), value_to_parameter_value(&self.content)?);
        effect.parameters.insert("method".to_string(), ParameterValue::String(self.method.clone()));
        
        // Add to fragment
        fragment.effect_nodes.insert(node_id.clone(), effect);
        fragment.entry_points.push(node_id.clone());
        fragment.exit_points.push(node_id);
        
        Ok(fragment)
    }
}

/// Resource combinator.
pub struct ResourceCombinator {
    /// Resource ID
    pub resource_id: String,
    /// Resource type
    pub resource_type: String,
    /// Initial state
    pub initial_state: Value,
    /// Domain of the resource
    pub domain_id: String,
    /// Metadata for the resource
    pub metadata: HashMap<String, String>,
}

/// Implementation of ToTEGFragment for ResourceCombinator
impl ToTEGFragment for ResourceCombinator {
    fn to_teg_fragment(&self) -> Result<TEGFragment> {
        // Create a fragment with a resource creation effect and resource node
        let mut fragment = TEGFragment::new();
        
        // Convert resource state
        let resource_state = ResourceState::Custom(self.initial_state.to_string());
        
        // Create resource node
        let mut resource = ResourceNode::new(
            self.resource_id.clone(),
            self.resource_type.clone(),
            resource_state,
            self.domain_id.clone(),
        );
        
        // Add metadata
        for (key, value) in &self.metadata {
            resource.metadata.insert(key.clone(), ParameterValue::String(value.clone()));
        }
        
        // Create a resource creation effect node
        let effect_id = format!("create_resource_{}", self.resource_id);
        let mut effect = EffectNode::new(effect_id.clone());
        effect.effect_type = "resource_creation".to_string();
        effect.domain_id = self.domain_id.clone();
        effect.resources_accessed = vec![self.resource_id.clone()];
        effect.required_capabilities = vec!["resource_creation".to_string()];
        
        // Add parameters
        effect.parameters.insert("resource_id".to_string(), ParameterValue::String(self.resource_id.clone()));
        effect.parameters.insert("resource_type".to_string(), ParameterValue::String(self.resource_type.clone()));
        effect.parameters.insert("initial_state".to_string(), value_to_parameter_value(&self.initial_state)?);
        
        // Add metadata parameters
        for (key, value) in &self.metadata {
            effect.parameters.insert(format!("metadata_{}", key), ParameterValue::String(value.clone()));
        }
        
        // Add to fragment
        fragment.resource_nodes.insert(self.resource_id.clone(), resource);
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
        let mut effect = EffectNode::new(node_id.clone());
        effect.effect_type = "resource_query".to_string();
        effect.domain_id = self.domain_id.clone();
        effect.resources_accessed = vec![self.resource_id.clone()];
        effect.required_capabilities = vec![format!("resource_query_{}", self.domain_id)];
        
        // Add fixed parameters
        effect.parameters.insert("resource_id".to_string(), ParameterValue::String(self.resource_id.clone()));
        effect.parameters.insert("query_type".to_string(), ParameterValue::String(self.query_type.clone()));
        
        // Add query parameters
        for (name, value) in &self.parameters {
            effect.parameters.insert(name.clone(), value_to_parameter_value(value)?);
        }
        
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
    use causality_types::ContentHash;
    
    // Create a test content hash
    fn test_content_hash() -> ContentHash {
        ContentHash::new("sha256", vec![0; 32])
    }

    #[test]
    fn test_identity_combinator() {
        let identity = CoreCombinator::Identity;
        let fragment = identity.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert!(fragment.effect_nodes.contains_key("identity"));
        assert_eq!(fragment.effect_nodes["identity"].effect_type, "identity");
    }
    
    #[test]
    fn test_constant_combinator() {
        let constant = CoreCombinator::Constant(json!("test"));
        let fragment = constant.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert!(fragment.effect_nodes.contains_key("constant_\"test\""));
        assert_eq!(fragment.effect_nodes["constant_\"test\""].effect_type, "constant");
        assert_eq!(fragment.effect_nodes["constant_\"test\""].parameters["value"], ParameterValue::String("test".to_string()));
    }
    
    #[test]
    fn test_composition_combinator() {
        let composition = CoreCombinator::Composition {
            f_id: "f".to_string(),
            g_id: "g".to_string(),
        };
        
        let fragment = composition.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 3);
        assert!(fragment.effect_nodes.contains_key("ref_f"));
        assert!(fragment.effect_nodes.contains_key("ref_g"));
        assert!(fragment.effect_nodes.contains_key("B_f_g"));
        
        assert_eq!(fragment.effect_nodes["ref_f"].effect_type, "reference");
        assert_eq!(fragment.effect_nodes["ref_f"].parameters["name"], ParameterValue::String("f".to_string()));
        
        assert_eq!(fragment.effect_nodes["ref_g"].effect_type, "reference");
        assert_eq!(fragment.effect_nodes["ref_g"].parameters["name"], ParameterValue::String("g".to_string()));
        
        assert_eq!(fragment.effect_nodes["B_f_g"].effect_type, "composition");
        assert_eq!(fragment.effect_nodes["B_f_g"].parameters["f_id"], ParameterValue::String("f".to_string()));
        assert_eq!(fragment.effect_nodes["B_f_g"].parameters["g_id"], ParameterValue::String("g".to_string()));
    }
    
    #[test]
    fn test_application_combinator() {
        let application = ApplicationCombinator {
            function_id: "f".to_string(),
            argument_id: "x".to_string(),
        };
        
        let fragment = application.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert!(fragment.effect_nodes.contains_key("apply_f_x"));
        assert_eq!(fragment.effect_nodes["apply_f_x"].effect_type, "apply");
        assert_eq!(fragment.effect_nodes["apply_f_x"].parameters["function_id"], ParameterValue::String("f".to_string()));
        assert_eq!(fragment.effect_nodes["apply_f_x"].parameters["argument_id"], ParameterValue::String("x".to_string()));
    }
    
    #[test]
    fn test_literal_combinator() {
        let literal = LiteralCombinator {
            value: json!(42),
        };
        
        let fragment = literal.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert!(fragment.effect_nodes.contains_key("literal_42"));
        assert_eq!(fragment.effect_nodes["literal_42"].effect_type, "literal");
        assert_eq!(fragment.effect_nodes["literal_42"].parameters["value"], ParameterValue::Integer(42));
    }
    
    #[test]
    fn test_reference_combinator() {
        let reference = ReferenceCombinator {
            name: "x".to_string(),
        };
        
        let fragment = reference.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert!(fragment.effect_nodes.contains_key("ref_x"));
        assert_eq!(fragment.effect_nodes["ref_x"].effect_type, "reference");
        assert_eq!(fragment.effect_nodes["ref_x"].parameters["name"], ParameterValue::String("x".to_string()));
    }
    
    #[test]
    fn test_effect_combinator() {
        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), json!("value1"));
        
        let effect = EffectCombinator {
            effect_name: "test_effect".to_string(),
            domain_id: "test_domain".to_string(),
            parameters,
            required_capabilities: vec!["test_capability".to_string()],
        };
        
        let fragment = effect.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert!(fragment.effect_nodes.contains_key("effect_test_effect"));
        
        let effect_node = &fragment.effect_nodes["effect_test_effect"];
        assert_eq!(effect_node.effect_type, "effect_test_effect");
        assert_eq!(effect_node.domain_id, "test_domain");
        assert_eq!(effect_node.parameters["param1"], ParameterValue::String("value1".to_string()));
        assert_eq!(effect_node.required_capabilities, vec!["test_capability"]);
    }
    
    #[test]
    fn test_state_transition_combinator() {
        let from_state = json!("active");
        let to_state = json!("frozen");
        
        let transition = StateTransitionCombinator {
            resource_id: "resource1".to_string(),
            from_state: from_state.clone(),
            to_state: to_state.clone(),
            domain_id: "test_domain".to_string(),
        };
        
        let fragment = transition.to_teg_fragment().unwrap();
        
        // Check that we have the right nodes
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.resource_nodes.len(), 2);
        
        // Check resources
        assert!(fragment.resource_nodes.contains_key("resource1_from"));
        assert!(fragment.resource_nodes.contains_key("resource1_to"));
        
        let from_resource = &fragment.resource_nodes["resource1_from"];
        assert_eq!(from_resource.state, ResourceState::Active);
        
        let to_resource = &fragment.resource_nodes["resource1_to"];
        assert_eq!(to_resource.state, ResourceState::Frozen);
        
        // Check effect
        let effect_id = format!("transition_resource1_{}_{}",ResourceState::Active, ResourceState::Frozen);
        assert!(fragment.effect_nodes.contains_key(&effect_id));
        
        let effect = &fragment.effect_nodes[&effect_id];
        assert_eq!(effect.effect_type, "state_transition");
        assert_eq!(effect.parameters["resource_id"], ParameterValue::String("resource1".to_string()));
        
        // Check resource relationship
        assert!(fragment.resource_relationships.contains_key("resource1_from"));
        assert_eq!(fragment.resource_relationships["resource1_from"].len(), 1);
        assert_eq!(fragment.resource_relationships["resource1_from"][0].0, "resource1_to");
        assert_eq!(fragment.resource_relationships["resource1_from"][0].1, RelationshipType::StateTransition);
    }
    
    #[test]
    fn test_content_addressing_combinator() {
        let content = json!("test_content");
        
        let content_addressing = ContentAddressingCombinator {
            content: content.clone(),
            method: "sha256".to_string(),
        };
        
        let fragment = content_addressing.to_teg_fragment().unwrap();
        
        assert_eq!(fragment.effect_nodes.len(), 1);
        
        let effect_id = "content_address_sha256_\"test_content\"";
        assert!(fragment.effect_nodes.contains_key(effect_id));
        
        let effect = &fragment.effect_nodes[effect_id];
        assert_eq!(effect.effect_type, "content_addressing");
        assert_eq!(effect.parameters["content"], ParameterValue::String("test_content".to_string()));
        assert_eq!(effect.parameters["method"], ParameterValue::String("sha256".to_string()));
        assert_eq!(effect.domain_id, "cryptography");
        assert!(effect.required_capabilities.contains(&"content_addressing".to_string()));
    }
    
    #[test]
    fn test_resource_combinator() {
        let initial_state = json!({});
        let mut metadata = HashMap::new();
        metadata.insert("owner".to_string(), "test_user".to_string());
        
        let resource = ResourceCombinator {
            resource_id: "resource1".to_string(),
            resource_type: "test_resource".to_string(),
            initial_state: initial_state.clone(),
            domain_id: "test_domain".to_string(),
            metadata: metadata.clone(),
        };
        
        let fragment = resource.to_teg_fragment().unwrap();
        
        // Check that we have the right nodes
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert_eq!(fragment.resource_nodes.len(), 1);
        
        // Check resource
        assert!(fragment.resource_nodes.contains_key("resource1"));
        let resource_node = &fragment.resource_nodes["resource1"];
        assert_eq!(resource_node.resource_type, "test_resource");
        assert_eq!(resource_node.domain_id, "test_domain");
        assert_eq!(resource_node.metadata["owner"], ParameterValue::String("test_user".to_string()));
        
        // Check effect
        assert!(fragment.effect_nodes.contains_key("create_resource_resource1"));
        let effect = &fragment.effect_nodes["create_resource_resource1"];
        assert_eq!(effect.effect_type, "resource_creation");
        assert_eq!(effect.parameters["resource_id"], ParameterValue::String("resource1".to_string()));
        assert_eq!(effect.parameters["resource_type"], ParameterValue::String("test_resource".to_string()));
        assert_eq!(effect.resources_accessed, vec!["resource1"]);
    }
    
    #[test]
    fn test_query_combinator() {
        let mut parameters = HashMap::new();
        parameters.insert("filter".to_string(), json!("field=value"));
        
        let query = QueryCombinator {
            resource_id: "resource1".to_string(),
            query_type: "filter".to_string(),
            parameters: parameters.clone(),
            domain_id: "test_domain".to_string(),
        };
        
        let fragment = query.to_teg_fragment().unwrap();
        
        // Check effect
        assert_eq!(fragment.effect_nodes.len(), 1);
        assert!(fragment.effect_nodes.contains_key("query_resource1_filter"));
        
        let effect = &fragment.effect_nodes["query_resource1_filter"];
        assert_eq!(effect.effect_type, "resource_query");
        assert_eq!(effect.parameters["resource_id"], ParameterValue::String("resource1".to_string()));
        assert_eq!(effect.parameters["query_type"], ParameterValue::String("filter".to_string()));
        assert_eq!(effect.parameters["filter"], ParameterValue::String("field=value".to_string()));
        assert_eq!(effect.resources_accessed, vec!["resource1"]);
        assert_eq!(effect.required_capabilities, vec!["resource_query_test_domain"]);
    }
}
