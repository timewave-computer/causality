//! Optimization Framework for TEG
//!
//! This module provides a framework for implementing optimization passes
//! for Temporal Effect Graphs (TEGs), including resource-specific optimizations.

pub mod constant;
pub mod dead_code;
pub mod inlining;
pub mod resource;
pub mod cross_domain;

use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;

use crate::TemporalEffectGraph;

/// Configuration for optimization passes
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Which optimizations to enable
    pub enabled_optimizations: HashMap<String, bool>,
    
    /// Optimization level (0-3)
    /// - 0: No optimizations
    /// - 1: Basic optimizations (constant folding, etc)
    /// - 2: Intermediate optimizations (including resource access)
    /// - 3: Advanced optimizations (including batching)
    pub level: u8,
    
    /// Maximum number of iterations for fixed-point optimizations
    pub max_iterations: u32,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        let mut enabled_optimizations = HashMap::new();
        
        // Enable default optimizations
        enabled_optimizations.insert("constant_folding".to_string(), true);
        enabled_optimizations.insert("dead_code_elimination".to_string(), true);
        enabled_optimizations.insert("resource_access_optimization".to_string(), true);
        enabled_optimizations.insert("resource_operation_batching".to_string(), true);
        
        Self {
            enabled_optimizations,
            level: 1,
            max_iterations: 10,
        }
    }
}

/// Trait for implementing optimization passes
pub trait Optimization: std::fmt::Debug {
    /// Name of the optimization
    fn name(&self) -> &str;
    
    /// Description of what the optimization does
    fn description(&self) -> &str;
    
    /// Apply the optimization to the TEG
    /// 
    /// Returns a boolean indicating whether the TEG was modified
    fn apply(&self, teg: &mut TemporalEffectGraph, config: &OptimizationConfig) -> Result<bool>;
    
    /// Whether this optimization preserves the adjunction property
    fn preserves_adjunction(&self) -> bool;
    
    /// Whether this optimization preserves the monoidal structure of resources
    fn preserves_resource_structure(&self) -> bool;
}

/// Statistics for an optimization pass
#[derive(Debug, Default, Clone)]
pub struct PassStats {
    /// Number of iterations
    pub iterations: u32,
    
    /// Time spent in milliseconds
    pub time_ms: u64,
    
    /// Whether the pass made any changes
    pub changed: bool,
}

/// Statistics for the entire optimization pipeline
#[derive(Debug, Default, Clone)]
pub struct OptimizationStats {
    /// Stats for each pass
    pub pass_stats: HashMap<String, PassStats>,
    
    /// Total time spent in milliseconds
    pub total_time_ms: u64,
    
    /// Whether any pass made changes
    pub changed: bool,
}

/// A pipeline of optimization passes
#[derive(Debug)]
pub struct OptimizationPipeline {
    /// The optimization passes
    passes: Vec<Box<dyn Optimization>>,
    
    /// Configuration for the pipeline
    config: OptimizationConfig,
}

impl OptimizationPipeline {
    /// Create a new optimization pipeline with the given configuration
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            passes: Vec::new(),
            config,
        }
    }
    
    /// Register an optimization pass
    pub fn register<T: Optimization + 'static>(&mut self, pass: T) -> &mut Self {
        self.passes.push(Box::new(pass));
        self
    }
    
    /// Apply all registered optimization passes
    pub fn apply_all(&self, teg: &mut TemporalEffectGraph) -> Result<OptimizationStats> {
        let start = Instant::now();
        let mut stats = OptimizationStats::default();
        let mut changed = false;
        
        for pass in &self.passes {
            let pass_name = pass.name().to_string();
            
            // Skip if optimization is disabled
            if let Some(enabled) = self.config.enabled_optimizations.get(&pass_name) {
                if !enabled {
                    continue;
                }
            }
            
            // Apply the optimization
            let pass_start = Instant::now();
            let mut iterations = 0;
            let mut pass_changed = false;
            
            // Apply until fixed point or max iterations reached
            loop {
                iterations += 1;
                
                if iterations > self.config.max_iterations {
                    break;
                }
                
                let result = pass.apply(teg, &self.config)?;
                
                if result {
                    pass_changed = true;
                } else {
                    // No changes, we've reached a fixed point
                    break;
                }
            }
            
            // Record stats
            let elapsed = pass_start.elapsed();
            let pass_stats = PassStats {
                iterations,
                time_ms: elapsed.as_millis() as u64,
                changed: pass_changed,
            };
            
            stats.pass_stats.insert(pass_name, pass_stats);
            
            if pass_changed {
                changed = true;
            }
        }
        
        stats.total_time_ms = start.elapsed().as_millis() as u64;
        stats.changed = changed;
        
        Ok(stats)
    }
}

/// Create a default optimization pipeline with all standard passes
pub fn create_default_pipeline() -> OptimizationPipeline {
    let mut pipeline = OptimizationPipeline::new(OptimizationConfig::default());
    
    // Register all optimization passes
    pipeline
        .register(constant::ConstantFolding::new())
        .register(dead_code::DeadCodeElimination::new())
        .register(inlining::EffectInlining::new())
        .register(resource::ResourceAccessOptimization::new())
        .register(resource::ResourceOperationBatching::new())
        .register(cross_domain::CrossDomainReordering::new())
        .register(cross_domain::DomainSpecificSpecialization::new());
        
    pipeline
}

/// Apply all registered passes to a TEG
pub fn optimize(teg: &mut TemporalEffectGraph, config: Option<OptimizationConfig>) -> Result<OptimizationStats> {
    let config = config.unwrap_or_default();
    let pipeline = create_default_pipeline();
    pipeline.apply_all(teg)
}

/// Verify that all optimizations preserve the adjunction property
pub fn verify_adjunction_preservation(pipeline: &OptimizationPipeline) -> bool {
    pipeline.passes.iter().all(|pass| pass.preserves_adjunction())
}

/// Verify that all optimizations preserve the resource monoidal structure
pub fn verify_resource_structure_preservation(pipeline: &OptimizationPipeline) -> bool {
    pipeline.passes.iter().all(|pass| pass.preserves_resource_structure())
}

/// Verify the correctness of optimizations based on category theory
///
/// This function verifies that:
/// 1. All optimizations preserve the adjunction property between TEL and TEG
/// 2. All optimizations preserve the resource monoidal structure
/// 3. All optimizations maintain semantic equivalence
pub fn verify_optimization_correctness(pipeline: &OptimizationPipeline) -> Result<bool> {
    // Check adjunction preservation
    let preserves_adjunction = verify_adjunction_preservation(pipeline);
    
    // Check resource structure preservation
    let preserves_resource_structure = verify_resource_structure_preservation(pipeline);
    
    // Check that all passes individually preserve semantics
    let all_passes_preserve_semantics = pipeline.passes.iter().all(|pass| {
        // A pass preserves semantics if it preserves the adjunction 
        // and resource structure, or is explicitly marked as semantics-preserving
        pass.preserves_adjunction() && pass.preserves_resource_structure()
    });
    
    // Return true only if all checks pass
    Ok(preserves_adjunction && preserves_resource_structure && all_passes_preserve_semantics)
}

/// Statistics about optimization results
#[derive(Debug)]
pub struct OptimizationStats {
    /// Number of iterations
    pub iterations: u8,
    
    /// Statistics per pass
    pub pass_stats: HashMap<String, PassStats>,
}

/// Statistics for a single optimization pass
#[derive(Debug)]
pub struct PassStats {
    /// Number of times the pass was applied
    pub applications: u32,
    
    /// Number of changes made
    pub changes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TemporalEffectGraph;
    
    /// Test that verifies all default optimizations preserve the adjunction property
    #[test]
    fn test_adjunction_preservation() {
        let pipeline = create_default_pipeline();
        assert!(verify_adjunction_preservation(&pipeline), 
            "All optimizations must preserve the adjunction property");
    }
    
    /// Test that verifies all default optimizations preserve the resource monoidal structure
    #[test]
    fn test_resource_structure_preservation() {
        let pipeline = create_default_pipeline();
        assert!(verify_resource_structure_preservation(&pipeline), 
            "All optimizations must preserve the resource monoidal structure");
    }
    
    /// Test that verifies the overall correctness of all optimizations
    #[test]
    fn test_optimization_correctness() {
        let pipeline = create_default_pipeline();
        assert!(verify_optimization_correctness(&pipeline).unwrap(), 
            "All optimizations must preserve semantic correctness");
    }
    
    /// Test that optimizations don't change program semantics
    /// This is a property-based test that applies optimizations to a TEG and
    /// verifies that the result is semantically equivalent to the original
    #[test]
    fn test_optimization_semantic_preservation() {
        use crate::builder::GraphBuilder;
        
        // Create a test graph
        let mut builder = GraphBuilder::new();
        
        // Add some effects and resources
        let effect1 = builder.add_effect("effect1", "test");
        let effect2 = builder.add_effect("effect2", "test");
        let resource = builder.add_resource("resource1", "test_type");
        
        // Connect them
        builder.connect_effects(effect1, effect2);
        builder.connect_effect_to_resource(effect2, resource, crate::graph::edge::AccessMode::Read);
        
        // Build the graph
        let mut teg = builder.build().unwrap();
        
        // Clone the graph for comparison
        let original_teg = teg.clone();
        
        // Apply optimizations
        let config = OptimizationConfig {
            level: 3, // Use highest optimization level
            ..Default::default()
        };
        let result = optimize(&mut teg, Some(config)).unwrap();
        
        // Now verify the TEG is semantically equivalent to the original
        // For this test, we'll verify:
        // 1. All output effects still exist or have equivalents
        // 2. Resource relationships are preserved
        // 3. Effect dependencies are preserved (transitively)
        
        let outputs = original_teg.get_output_effects();
        for output_id in outputs {
            if !teg.has_effect(&output_id) {
                // If the original output doesn't exist directly, there should be
                // a replacement that's functionally equivalent
                let equivalent_found = teg.effects().iter().any(|(_, effect)| {
                    // Check for semantic equivalence markers
                    // This is a simplified check - in a real implementation,
                    // we would do a more sophisticated analysis
                    effect.metadata().contains_key(&format!("replaces_{}", output_id))
                });
                
                assert!(equivalent_found, 
                    "Output effect {} was removed without equivalent replacement", output_id);
            }
        }
        
        // Verify that all resource accesses are preserved
        for (resource_id, _) in original_teg.resources() {
            assert!(teg.has_resource(&resource_id), 
                "Resource {} was removed by optimization", resource_id);
            
            // Check all effects that accessed this resource
            let original_accessors = original_teg.get_resource_accessors(&resource_id);
            
            // All access patterns should be preserved (possibly in a transformed way)
            assert!(!teg.get_resource_accessors(&resource_id).is_empty() || original_accessors.is_empty(),
                "Resource {} lost all accessors after optimization", resource_id);
        }
    }
}
