// This module implements the transformation from TEG fragments back to TEL combinators.
// It is part of the G functor in the TEL-TEG adjunction.

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::{
    TEGFragment,
    effect_node::ParameterValue,
};
// Condition is only used in tests
#[cfg(test)]
use crate::graph::edge::Condition;

/// TEL Combinator type (matches causality-tel's Combinator enum)
#[derive(Debug, Clone)]
pub enum Combinator {
    I,
    K,
    S,
    B,
    C,
    App {
        function: Box<Combinator>,
        argument: Box<Combinator>,
    },
    Literal(Literal),
    Ref(String),
    Effect {
        effect_name: String,
        args: Vec<Combinator>,
    },
    StateTransition {
        target_state: String,
        fields: HashMap<String, Combinator>,
        resource_id: Option<String>,
    },
    ContentId(Box<Combinator>),
    Store(Box<Combinator>),
    Load(Box<Combinator>),
    Query {
        source: String,
        domain: Option<String>,
        params: HashMap<String, Combinator>,
    },
    Resource {
        operation: String,
        resource_type: String,
        resource_id: Option<String>,
        params: HashMap<String, Combinator>,
    },
}

/// Literal values for combinators
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Literal>),
    Map(HashMap<String, Literal>),
}

impl Combinator {
    /// Create an application of combinators
    pub fn app(f: Combinator, x: Combinator) -> Self {
        Combinator::App {
            function: Box::new(f),
            argument: Box::new(x),
        }
    }
    
    /// Create a literal combinator with an integer value
    pub fn int(n: i64) -> Self {
        Combinator::Literal(Literal::Int(n))
    }
    
    /// Create a literal combinator with a string value
    pub fn string(s: impl Into<String>) -> Self {
        Combinator::Literal(Literal::String(s.into()))
    }
    
    /// Create an effect combinator
    pub fn effect(name: impl Into<String>, args: Vec<Combinator>) -> Self {
        Combinator::Effect {
            effect_name: name.into(),
            args,
        }
    }
}

/// Helper function to convert serde_json::Value to Literal
fn value_to_literal(value: &Value) -> Result<Literal> {
    match value {
        Value::Null => Ok(Literal::Null),
        Value::Bool(b) => Ok(Literal::Bool(*b)),
        Value::Number(n) => {
            if n.is_i64() {
                Ok(Literal::Int(n.as_i64().unwrap()))
            } else {
                Ok(Literal::Float(n.as_f64().unwrap_or(0.0)))
            }
        },
        Value::String(s) => Ok(Literal::String(s.clone())),
        Value::Array(arr) => {
            let mut items = Vec::new();
            for item in arr {
                items.push(value_to_literal(item)?);
            }
            Ok(Literal::List(items))
        },
        Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                map.insert(key.clone(), value_to_literal(val)?);
            }
            Ok(Literal::Map(map))
        }
    }
}

/// Helper function to convert ParameterValue to Literal
fn parameter_value_to_literal(value: &ParameterValue) -> Result<Literal> {
    match value {
        ParameterValue::Boolean(b) => Ok(Literal::Bool(*b)),
        ParameterValue::Integer(n) => Ok(Literal::Int(*n)),
        ParameterValue::Float(f) => Ok(Literal::Float(*f)),
        ParameterValue::String(s) => Ok(Literal::String(s.clone())),
        ParameterValue::Array(arr) => {
            let mut items = Vec::new();
            for item in arr {
                items.push(parameter_value_to_literal(item)?);
            }
            Ok(Literal::List(items))
        },
        ParameterValue::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                map.insert(key.clone(), parameter_value_to_literal(val)?);
            }
            Ok(Literal::Map(map))
        },
        ParameterValue::Null => Ok(Literal::Null),
        ParameterValue::Bytes(b) => {
            // Convert bytes to a string representation for literals
            Ok(Literal::String(format!("{:?}", b)))
        },
    }
}

/// Convert serde_json::Value to ParameterValue
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

/// Helper function to extract a string from a ParameterValue
fn parameter_value_as_str<'a>(value: &'a ParameterValue) -> Option<&'a str> {
    if let ParameterValue::String(s) = value {
        Some(s.as_str())
    } else {
        None
    }
}

/// Trait for converting a TEG fragment to a TEL combinator.
/// 
/// This trait enables the transformation from the TEG intermediate
/// representation back to TEL combinators, completing the adjunction.
pub trait ToTELCombinator {
    /// The TEL combinator type that this fragment converts to
    type TELType;
    
    /// Convert the implementing type to a TEL combinator.
    /// 
    /// This transformation reconstructs a TEL combinator that
    /// represents the semantics of the given TEG fragment.
    fn to_tel_combinator(&self) -> Result<Self::TELType>;
}

/// Implementation of ToTELCombinator for EffectNode
impl ToTELCombinator for crate::effect_node::EffectNode {
    type TELType = Combinator;
    
    fn to_tel_combinator(&self) -> Result<Self::TELType> {
        // Handle different effect types
        match self.effect_type.as_str() {
            // Core combinators
            "identity" => Ok(Combinator::I),
            "constant" => {
                if let Some(value) = self.parameters.get("value") {
                    let literal = parameter_value_to_literal(value)?;
                    Ok(Combinator::Literal(literal))
                } else {
                    Ok(Combinator::K)
                }
            },
            
            // Application
            "apply" => {
                let function_id = self.parameters.get("function_id")
                    .and_then(|v| parameter_value_as_str(v))
                    .ok_or_else(|| anyhow!("Missing function_id parameter in apply effect"))?;
                
                let argument_id = self.parameters.get("argument_id")
                    .and_then(|v| parameter_value_as_str(v))
                    .ok_or_else(|| anyhow!("Missing argument_id parameter in apply effect"))?;
                
                Ok(Combinator::app(
                    Combinator::Ref(function_id.to_string()),
                    Combinator::Ref(argument_id.to_string()),
                ))
            },
            
            // Effect invocation
            effect_type if effect_type.starts_with("effect_") => {
                // Extract effect name (remove the "effect_" prefix)
                let effect_name = effect_type.strip_prefix("effect_").unwrap_or(effect_type);
                
                // Convert parameters to arguments
                let mut args = Vec::new();
                for (name, value) in &self.parameters {
                    if name != "effect_name" {
                        let literal = parameter_value_to_literal(value)?;
                        args.push(Combinator::Literal(literal));
                    }
                }
                
                Ok(Combinator::Effect {
                    effect_name: effect_name.to_string(),
                    args,
                })
            },
            
            // State transition
            "state_transition" => {
                let resource_id = self.parameters.get("resource_id")
                    .and_then(|v| parameter_value_as_str(v))
                    .map(|s| s.to_string());
                
                let target_state = self.parameters.get("to_state")
                    .and_then(|v| parameter_value_as_str(v))
                    .ok_or_else(|| anyhow!("Missing to_state parameter in state_transition effect"))?
                    .to_string();
                
                // Create fields map
                let mut fields = HashMap::new();
                for (name, value) in &self.parameters {
                    if name != "resource_id" && name != "from_state" && name != "to_state" {
                        let literal = parameter_value_to_literal(value)?;
                        fields.insert(name.clone(), Combinator::Literal(literal));
                    }
                }
                
                Ok(Combinator::StateTransition {
                    target_state,
                    fields,
                    resource_id,
                })
            },
            
            // Content addressing
            "content_addressing" => {
                let content = self.parameters.get("content")
                    .ok_or_else(|| anyhow!("Missing content parameter in content_addressing effect"))?;
                
                let literal = parameter_value_to_literal(content)?;
                Ok(Combinator::ContentId(Box::new(Combinator::Literal(literal))))
            },
            
            // Store operation
            "store" => {
                let content = self.parameters.get("content")
                    .ok_or_else(|| anyhow!("Missing content parameter in store effect"))?;
                
                let literal = parameter_value_to_literal(content)?;
                Ok(Combinator::Store(Box::new(Combinator::Literal(literal))))
            },
            
            // Load operation
            "load" => {
                let content_id = self.parameters.get("content_id")
                    .and_then(|v| parameter_value_as_str(v))
                    .ok_or_else(|| anyhow!("Missing content_id parameter in load effect"))?;
                
                Ok(Combinator::Load(Box::new(Combinator::string(content_id))))
            },
            
            // Resource operation
            "resource_creation" | "resource_update" | "resource_deletion" => {
                let operation = match self.effect_type.as_str() {
                    "resource_creation" => "create",
                    "resource_update" => "update",
                    "resource_deletion" => "delete",
                    _ => self.effect_type.as_str(),
                };
                
                let resource_type = self.parameters.get("resource_type")
                    .and_then(|v| parameter_value_as_str(v))
                    .ok_or_else(|| anyhow!("Missing resource_type parameter in resource operation"))?
                    .to_string();
                
                let resource_id = self.parameters.get("resource_id")
                    .and_then(|v| parameter_value_as_str(v))
                    .map(|s| s.to_string());
                
                // Create params map
                let mut params = HashMap::new();
                for (name, value) in &self.parameters {
                    if name != "resource_id" && name != "resource_type" && name != "operation" {
                        let literal = parameter_value_to_literal(value)?;
                        params.insert(name.clone(), Combinator::Literal(literal));
                    }
                }
                
                Ok(Combinator::Resource {
                    operation: operation.to_string(),
                    resource_type,
                    resource_id,
                    params,
                })
            },
            
            // Query operation
            "resource_query" => {
                let resource_id = self.parameters.get("resource_id")
                    .and_then(|v| parameter_value_as_str(v))
                    .ok_or_else(|| anyhow!("Missing resource_id parameter in resource_query effect"))?
                    .to_string();
                
                let _query_type = self.parameters.get("query_type")
                    .and_then(|v| parameter_value_as_str(v))
                    .unwrap_or("query")
                    .to_string();
                
                // Create params map
                let mut params = HashMap::new();
                for (name, value) in &self.parameters {
                    if name != "resource_id" && name != "query_type" {
                        let literal = parameter_value_to_literal(value)?;
                        params.insert(name.clone(), Combinator::Literal(literal));
                    }
                }
                
                Ok(Combinator::Query {
                    source: resource_id,
                    domain: Some(self.domain_id.clone()),
                    params,
                })
            },
            
            // Reference
            "reference" => {
                let name = self.parameters.get("name")
                    .and_then(|v| parameter_value_as_str(v))
                    .ok_or_else(|| anyhow!("Missing name parameter in reference effect"))?;
                
                Ok(Combinator::Ref(name.to_string()))
            },
            
            // Literal
            "literal" => {
                let value = self.parameters.get("value")
                    .ok_or_else(|| anyhow!("Missing value parameter in literal effect"))?;
                
                let literal = parameter_value_to_literal(value)?;
                Ok(Combinator::Literal(literal))
            },
            
            // Default - treat as generic effect
            _ => {
                // Convert parameters to arguments
                let mut args = Vec::new();
                for (_, value) in &self.parameters {
                    let literal = parameter_value_to_literal(value)?;
                    args.push(Combinator::Literal(literal));
                }
                
                Ok(Combinator::Effect {
                    effect_name: self.effect_type.clone(),
                    args,
                })
            }
        }
    }
}

/// Implementation of ToTELCombinator for ResourceNode
impl ToTELCombinator for crate::resource_node::ResourceNode {
    type TELType = Combinator;
    
    fn to_tel_combinator(&self) -> Result<Self::TELType> {
        // Create a resource combinator based on resource node
        let mut params = HashMap::new();
        
        // Add state based on the ResourceState enum
        let state_str = match &self.state {
            crate::resource_node::ResourceState::Active => "active",
            crate::resource_node::ResourceState::Frozen => "frozen",
            crate::resource_node::ResourceState::Locked => "locked",
            crate::resource_node::ResourceState::Inactive => "inactive",
            crate::resource_node::ResourceState::Custom(s) => s.as_str(),
        };
        params.insert("state".to_string(), Combinator::string(state_str));
        
        // Add metadata
        for (key, value) in &self.metadata {
            let value_literal = parameter_value_to_literal(value)?;
            params.insert(format!("metadata_{}", key), Combinator::Literal(value_literal));
        }
        
        Ok(Combinator::Resource {
            operation: "define".to_string(),
            resource_type: self.resource_type.clone(),
            resource_id: Some(self.id.clone()),
            params,
        })
    }
}

/// Implementation of ToTELCombinator for TEGFragment
impl ToTELCombinator for TEGFragment {
    type TELType = Combinator;
    
    fn to_tel_combinator(&self) -> Result<Self::TELType> {
        // Special case: empty fragment is identity
        if self.effect_nodes.is_empty() && self.resource_nodes.is_empty() {
            return Ok(Combinator::I);
        }
        
        // Special case: single effect node
        if self.effect_nodes.len() == 1 && self.resource_nodes.is_empty() {
            let effect_id = self.effect_nodes.keys().next().unwrap();
            let effect = &self.effect_nodes[effect_id];
            return effect.to_tel_combinator();
        }
        
        // Special case: single resource node
        if self.effect_nodes.is_empty() && self.resource_nodes.len() == 1 {
            let resource_id = self.resource_nodes.keys().next().unwrap();
            let resource = &self.resource_nodes[resource_id];
            return resource.to_tel_combinator();
        }
        
        // For more complex fragments, we need to build a composition of combinators
        
        // First, convert all resources to combinators
        let mut resource_combinators = HashMap::new();
        for (id, resource) in &self.resource_nodes {
            resource_combinators.insert(id.clone(), resource.to_tel_combinator()?);
        }
        
        // Next, convert all effects to combinators
        let mut effect_combinators = HashMap::new();
        for (id, effect) in &self.effect_nodes {
            effect_combinators.insert(id.clone(), effect.to_tel_combinator()?);
        }
        
        // Create a sequence if there are multiple entry points
        if !self.entry_points.is_empty() {
            // If we have continuations, we need to create a composition
            if !self.effect_continuations.is_empty() {
                // Find the entry and exit points
                let entry_id = self.entry_points.first()
                    .ok_or_else(|| anyhow!("Fragment has no entry points"))?;
                
                let exit_id = self.exit_points.first()
                    .ok_or_else(|| anyhow!("Fragment has no exit points"))?;
                
                // If entry and exit are the same, just return that combinator
                if entry_id == exit_id {
                    return Ok(effect_combinators.get(entry_id)
                        .ok_or_else(|| anyhow!("Entry point not found: {}", entry_id))?
                        .clone());
                }
                
                // Otherwise, build a path from entry to exit
                let mut current_id = entry_id;
                let mut result = effect_combinators.get(current_id)
                    .ok_or_else(|| anyhow!("Entry point not found: {}", current_id))?
                    .clone();
                
                // Follow the continuations until we reach the exit point
                while current_id != exit_id {
                    if let Some(continuations) = self.effect_continuations.get(current_id) {
                        let (next_id, _) = &continuations[0]; // Take the first continuation for now
                        current_id = next_id;
                        
                        // Compose with the next combinator
                        let next_combinator = effect_combinators.get(current_id)
                            .ok_or_else(|| anyhow!("Continuation point not found: {}", current_id))?
                            .clone();
                        
                        // Use B to compose: B f g x = f (g x)
                        result = Combinator::app(
                            Combinator::app(Combinator::B, next_combinator),
                            result
                        );
                    } else {
                        return Err(anyhow!("No continuation found from {} to exit", current_id));
                    }
                }
                
                Ok(result)
            } else {
                // Without continuations, create a sequence of combinators
                let mut combinators = Vec::new();
                
                for entry_id in &self.entry_points {
                    if let Some(combinator) = effect_combinators.get(entry_id) {
                        combinators.push(combinator.clone());
                    }
                }
                
                // If only one combinator, return it directly
                if combinators.len() == 1 {
                    Ok(combinators.pop().unwrap())
                } else {
                    // Otherwise compose them with S combinator
                    let mut result = combinators.pop()
                        .ok_or_else(|| anyhow!("No valid entry points found"))?;
                    
                    for combinator in combinators.into_iter().rev() {
                        // Use S to combine functions: S f g x = f x (g x)
                        result = Combinator::app(
                            Combinator::app(Combinator::S, combinator),
                            result
                        );
                    }
                    
                    Ok(result)
                }
            }
        } else {
            // If no entry points, default to identity combinator
            Ok(Combinator::I)
        }
    }
}

/// Convert a TEG fragment to a TEL combinator of the specified type.
/// 
/// This is a convenience function that wraps the trait method.
pub fn to_tel_combinator<T, F: ToTELCombinator<TELType = T>>(teg_fragment: &F) -> Result<T> {
    teg_fragment.to_tel_combinator()
}

/// Convert a TEG fragment to a TEL combinator
pub fn teg_to_tel(fragment: &TEGFragment) -> Result<Combinator> {
    // We now delegate to the trait implementation
    fragment.to_tel_combinator()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect_node::ParameterValue;
    use std::collections::HashMap;
    
    #[test]
    fn test_parameter_value_to_literal() {
        // Test string conversion
        let string_value = ParameterValue::String("test".to_string());
        let string_literal = parameter_value_to_literal(&string_value).unwrap();
        assert!(matches!(string_literal, Literal::String(s) if s == "test"));
        
        // Test integer conversion
        let int_value = ParameterValue::Integer(42);
        let int_literal = parameter_value_to_literal(&int_value).unwrap();
        assert!(matches!(int_literal, Literal::Int(n) if n == 42));
        
        // Test boolean conversion
        let bool_value = ParameterValue::Boolean(true);
        let bool_literal = parameter_value_to_literal(&bool_value).unwrap();
        assert!(matches!(bool_literal, Literal::Bool(b) if b));
        
        // Test array conversion
        let array_value = ParameterValue::Array(vec![
            ParameterValue::Integer(1),
            ParameterValue::String("two".to_string()),
        ]);
        let array_literal = parameter_value_to_literal(&array_value).unwrap();
        assert!(matches!(array_literal, Literal::List(_)));
        
        // Test object conversion
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), ParameterValue::String("value".to_string()));
        let obj_value = ParameterValue::Object(obj);
        let obj_literal = parameter_value_to_literal(&obj_value).unwrap();
        assert!(matches!(obj_literal, Literal::Map(_)));
    }
    
    #[test]
    fn test_effect_node_to_combinator() {
        // Create a simple effect node for testing
        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), ParameterValue::String("value1".to_string()));
        
        let effect = crate::effect_node::EffectNode {
            id: "test_effect".to_string(),
            effect_type: "test_effect_type".to_string(),
            parameters,
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: "test_domain".to_string(),
            metadata: HashMap::new(),
            content_hash: causality_types::ContentHash::new("blake3", vec![0; 32]),
        };
        
        let combinator = effect.to_tel_combinator().unwrap();
        
        // Check if the combinator is an effect with the correct name
        assert!(matches!(combinator, Combinator::Effect { effect_name, .. } if effect_name == "test_effect_type"));
    }
    
    #[test]
    fn test_resource_node_to_combinator() {
        // Create a resource node for testing
        let resource = crate::resource_node::ResourceNode {
            id: "test_resource".to_string(),
            resource_type: "test_resource_type".to_string(),
            state: crate::resource_node::ResourceState::Active,
            metadata: HashMap::new(),
            content_hash: causality_types::ContentHash::new("blake3", vec![0; 32]),
            domain_id: "test_domain".to_string(),
        };
        
        let combinator = resource.to_tel_combinator().unwrap();
        
        // Check if the combinator is a resource with the correct type
        assert!(matches!(combinator, Combinator::Resource { resource_type, .. } if resource_type == "test_resource_type"));
    }
    
    #[test]
    fn test_literal_to_combinator() {
        // Test string literal
        let mut parameters = HashMap::new();
        parameters.insert("value".to_string(), ParameterValue::String("test".to_string()));
        
        let effect = crate::effect_node::EffectNode {
            id: "test_effect".to_string(),
            effect_type: "literal".to_string(),
            parameters,
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: "test_domain".to_string(),
            metadata: HashMap::new(),
            content_hash: causality_types::ContentHash::new("blake3", vec![0; 32]),
        };
        
        let combinator = effect.to_tel_combinator().unwrap();
        
        // Check if the combinator is a literal with the correct value
        assert!(matches!(combinator, Combinator::Literal(Literal::String(s)) if s == "test"));
    }
    
    #[test]
    fn test_fragment_to_combinator() {
        // Create a simple fragment with two effects
        let mut fragment = TEGFragment::new();
        
        let effect1 = crate::effect_node::EffectNode {
            id: "effect1".to_string(),
            effect_type: "reference".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("name".to_string(), ParameterValue::String("test_ref".to_string()));
                params
            },
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: "test_domain".to_string(),
            metadata: HashMap::new(),
            content_hash: causality_types::ContentHash::new("blake3", vec![0; 32]),
        };
        
        let effect2 = crate::effect_node::EffectNode {
            id: "effect2".to_string(),
            effect_type: "identity".to_string(),
            parameters: HashMap::new(),
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: "test_domain".to_string(),
            metadata: HashMap::new(),
            content_hash: causality_types::ContentHash::new("blake3", vec![0; 32]),
        };
        
        // Add effects to fragment
        fragment.effect_nodes.insert("effect1".to_string(), effect1);
        fragment.effect_nodes.insert("effect2".to_string(), effect2);
        
        // Set entry and exit points
        fragment.entry_points.push("effect1".to_string());
        fragment.exit_points.push("effect2".to_string());
        
        // Add continuation from effect1 to effect2
        fragment.effect_continuations.insert(
            "effect1".to_string(),
            vec![("effect2".to_string(), None)]
        );
        
        // Convert fragment to TEL
        let combinator = fragment.to_tel_combinator().unwrap();
        
        // We expect a composition using B combinator
        assert!(matches!(combinator, Combinator::App { .. }));
    }
}
