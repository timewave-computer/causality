//! Query capabilities for the Temporal Effect Graph (TEG)
//!
//! This module provides advanced query capabilities for extracting information
//! from a TEG, including filtering, pagination, and pattern matching.

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::{TemporalEffectGraph, EffectId, ResourceId, DomainId};
use crate::effect_node::ParameterValue;

/// Query parameters for fetching effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectQuery {
    /// Filter by effect type
    pub effect_type: Option<String>,
    
    /// Filter by domain
    pub domain: Option<DomainId>,
    
    /// Filter by parameter existence
    pub has_parameter: Option<String>,
    
    /// Filter by parameter value
    pub parameter_value: Option<HashMap<String, ParameterValue>>,
    
    /// Filter by resource access
    pub accesses_resource: Option<ResourceId>,
    
    /// Filter by capability
    pub requires_capability: Option<String>,
    
    /// Pagination offset
    pub offset: Option<usize>,
    
    /// Pagination limit
    pub limit: Option<usize>,
}

/// Query parameters for fetching resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuery {
    /// Filter by resource type
    pub resource_type: Option<String>,
    
    /// Filter by domain
    pub domain: Option<DomainId>,
    
    /// Filter by effect access
    pub accessed_by_effect: Option<EffectId>,
    
    /// Pagination offset
    pub offset: Option<usize>,
    
    /// Pagination limit
    pub limit: Option<usize>,
}

/// Query response for effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectQueryResponse {
    /// Matching effects
    pub effects: Vec<EffectId>,
    
    /// Total count of matching effects (for pagination)
    pub total_count: usize,
    
    /// Whether there are more results available
    pub has_more: bool,
}

/// Query response for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQueryResponse {
    /// Matching resources
    pub resources: Vec<ResourceId>,
    
    /// Total count of matching resources (for pagination)
    pub total_count: usize,
    
    /// Whether there are more results available
    pub has_more: bool,
}

/// Pattern matching query for finding subgraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternQuery {
    /// Pattern description
    pub pattern: GraphPattern,
    
    /// Maximum results to return
    pub max_results: Option<usize>,
}

/// A graph pattern for matching subgraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPattern {
    /// Effect node patterns
    pub effect_patterns: Vec<EffectPattern>,
    
    /// Resource node patterns
    pub resource_patterns: Vec<ResourcePattern>,
    
    /// Edge patterns connecting nodes
    pub edge_patterns: Vec<EdgePattern>,
}

/// Pattern for matching effect nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectPattern {
    /// Identifier for this pattern node
    pub pattern_id: String,
    
    /// Effect type pattern (supports wildcards with * and ?)
    pub effect_type_pattern: Option<String>,
    
    /// Domain pattern
    pub domain_pattern: Option<String>,
    
    /// Parameter patterns (name and optional value)
    pub parameter_patterns: HashMap<String, Option<ParameterValue>>,
}

/// Pattern for matching resource nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePattern {
    /// Identifier for this pattern node
    pub pattern_id: String,
    
    /// Resource type pattern (supports wildcards with * and ?)
    pub resource_type_pattern: Option<String>,
    
    /// Domain pattern
    pub domain_pattern: Option<String>,
}

/// Pattern for edges between pattern nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePattern {
    /// Source pattern node ID
    pub from_pattern_id: String,
    
    /// Target pattern node ID
    pub to_pattern_id: String,
    
    /// Type of edge
    pub edge_type: EdgePatternType,
}

/// Types of edge patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgePatternType {
    /// Continuation edge
    Continuation,
    
    /// Dependency edge
    Dependency,
    
    /// Resource access edge
    ResourceAccess,
    
    /// Any edge type
    Any,
}

/// Pattern matching response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatchResponse {
    /// List of matching subgraphs
    pub matches: Vec<PatternMatch>,
    
    /// Whether there are more matches available
    pub has_more: bool,
}

/// A single pattern match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Mapping from pattern IDs to actual effect IDs
    pub effect_matches: HashMap<String, EffectId>,
    
    /// Mapping from pattern IDs to actual resource IDs
    pub resource_matches: HashMap<String, ResourceId>,
}

/// Query executor for TEG
pub struct QueryExecutor<'a> {
    /// The TEG to query
    teg: &'a TemporalEffectGraph,
}

impl<'a> QueryExecutor<'a> {
    /// Create a new query executor
    pub fn new(teg: &'a TemporalEffectGraph) -> Self {
        Self { teg }
    }
    
    /// Query effects using the provided parameters
    pub fn query_effects(&self, query: &EffectQuery) -> Result<EffectQueryResponse> {
        let mut effects: Vec<EffectId> = self.teg.effect_nodes.keys().cloned().collect();
        
        // Apply filters
        
        // Filter by effect type
        if let Some(effect_type) = &query.effect_type {
            effects.retain(|id| {
                if let Some(effect) = self.teg.effect_nodes.get(id) {
                    effect.effect_type == *effect_type
                } else {
                    false
                }
            });
        }
        
        // Filter by domain
        if let Some(domain) = &query.domain {
            effects.retain(|id| {
                if let Some(effect) = self.teg.effect_nodes.get(id) {
                    &effect.domain_id == domain
                } else {
                    false
                }
            });
        }
        
        // Filter by parameter existence
        if let Some(param_name) = &query.has_parameter {
            effects.retain(|id| {
                if let Some(effect) = self.teg.effect_nodes.get(id) {
                    effect.parameters.contains_key(param_name)
                } else {
                    false
                }
            });
        }
        
        // Filter by parameter value
        if let Some(param_values) = &query.parameter_value {
            effects.retain(|id| {
                if let Some(effect) = self.teg.effect_nodes.get(id) {
                    param_values.iter().all(|(name, value)| {
                        if let Some(actual_value) = effect.parameters.get(name) {
                            actual_value == value
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            });
        }
        
        // Filter by resource access
        if let Some(resource_id) = &query.accesses_resource {
            effects.retain(|id| {
                if let Some(effect) = self.teg.effect_nodes.get(id) {
                    effect.resources_accessed.contains(resource_id)
                } else {
                    false
                }
            });
        }
        
        // Filter by capability
        if let Some(capability) = &query.requires_capability {
            effects.retain(|id| {
                if let Some(effect) = self.teg.effect_nodes.get(id) {
                    effect.required_capabilities.iter().any(|cap| cap == capability)
                } else {
                    false
                }
            });
        }
        
        // Store total count for pagination
        let total_count = effects.len();
        
        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);
        
        let has_more = offset + limit < total_count;
        
        effects = effects.into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        
        Ok(EffectQueryResponse {
            effects,
            total_count,
            has_more,
        })
    }
    
    /// Query resources using the provided parameters
    pub fn query_resources(&self, query: &ResourceQuery) -> Result<ResourceQueryResponse> {
        let mut resources: Vec<ResourceId> = self.teg.resource_nodes.keys().cloned().collect();
        
        // Apply filters
        
        // Filter by resource type
        if let Some(resource_type) = &query.resource_type {
            resources.retain(|id| {
                if let Some(resource) = self.teg.resource_nodes.get(id) {
                    resource.resource_type == *resource_type
                } else {
                    false
                }
            });
        }
        
        // Filter by domain
        if let Some(domain) = &query.domain {
            resources.retain(|id| {
                if let Some(resource) = self.teg.resource_nodes.get(id) {
                    &resource.domain_id == domain
                } else {
                    false
                }
            });
        }
        
        // Filter by effect access
        if let Some(effect_id) = &query.accessed_by_effect {
            if let Some(effect) = self.teg.effect_nodes.get(effect_id) {
                resources.retain(|id| effect.resources_accessed.contains(id));
            } else {
                resources.clear(); // Effect doesn't exist, so no resources match
            }
        }
        
        // Store total count for pagination
        let total_count = resources.len();
        
        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);
        
        let has_more = offset + limit < total_count;
        
        resources = resources.into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        
        Ok(ResourceQueryResponse {
            resources,
            total_count,
            has_more,
        })
    }
    
    /// Find subgraphs matching a pattern
    pub fn find_patterns(&self, query: &PatternQuery) -> Result<PatternMatchResponse> {
        // This is a simplified implementation of subgraph isomorphism
        // A real implementation would use a more sophisticated algorithm
        
        // For now, return an empty response
        Ok(PatternMatchResponse {
            matches: Vec::new(),
            has_more: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_effect_query() {
        let mut builder = GraphBuilder::new();
        
        // Create effects
        builder.add_effect("effect1", "domain1");
        builder.add_effect("effect2", "domain1");
        builder.add_effect("effect3", "domain2");
        
        let teg = builder.build().unwrap();
        let executor = QueryExecutor::new(&teg);
        
        // Query all effects
        let query = EffectQuery {
            effect_type: None,
            domain: None,
            has_parameter: None,
            parameter_value: None,
            accesses_resource: None,
            requires_capability: None,
            offset: None,
            limit: None,
        };
        
        let response = executor.query_effects(&query).unwrap();
        assert_eq!(response.total_count, 3);
        assert_eq!(response.effects.len(), 3);
        
        // Query by domain
        let query = EffectQuery {
            effect_type: None,
            domain: Some("domain1".to_string()),
            has_parameter: None,
            parameter_value: None,
            accesses_resource: None,
            requires_capability: None,
            offset: None,
            limit: None,
        };
        
        let response = executor.query_effects(&query).unwrap();
        assert_eq!(response.total_count, 2);
        assert_eq!(response.effects.len(), 2);
    }
    
    #[test]
    fn test_resource_query() {
        let mut builder = GraphBuilder::new();
        
        // Create resources
        builder.add_resource("resource1", "type1");
        builder.add_resource("resource2", "type1");
        builder.add_resource("resource3", "type2");
        
        let teg = builder.build().unwrap();
        let executor = QueryExecutor::new(&teg);
        
        // Query all resources
        let query = ResourceQuery {
            resource_type: None,
            domain: None,
            accessed_by_effect: None,
            offset: None,
            limit: None,
        };
        
        let response = executor.query_resources(&query).unwrap();
        assert_eq!(response.total_count, 3);
        assert_eq!(response.resources.len(), 3);
        
        // Query by type
        let query = ResourceQuery {
            resource_type: Some("type1".to_string()),
            domain: None,
            accessed_by_effect: None,
            offset: None,
            limit: None,
        };
        
        let response = executor.query_resources(&query).unwrap();
        assert_eq!(response.total_count, 2);
        assert_eq!(response.resources.len(), 2);
    }
} 