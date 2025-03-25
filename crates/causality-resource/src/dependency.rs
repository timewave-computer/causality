// Resource dependency management (LEGACY VERSION)
//
// This module contains the deprecated implementation of resource dependency
// management. Use the ResourceDependency trait implementations in
// causality-effects::resource::dependency instead.

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use causality_common::identity::ContentId;
use thiserror::Error;

use crate::interface::deprecation::messages;
use crate::deprecated_warning;
use crate::deprecated_error;

/// Types of dependencies between resources
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::DEPENDENCY_DEPRECATED
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// Strong dependency (source depends on target to function)
    Strong,
    
    /// Weak dependency (source can function without target but is enhanced by it)
    Weak,
    
    /// Reference dependency (source references target but doesn't depend on it)
    Reference,
}

/// Information about a dependency
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::DEPENDENCY_DEPRECATED
)]
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// Source resource ID
    pub source_id: ContentId,
    
    /// Target resource ID
    pub target_id: ContentId,
    
    /// Type of dependency
    pub dependency_type: DependencyType,
    
    /// Optional domain ID for cross-domain dependencies
    pub domain_id: Option<ContentId>,
    
    /// Optional effect ID that created the dependency
    pub creator_effect_id: Option<ContentId>,
    
    /// Optional metadata
    pub metadata: Option<String>,
}

/// Errors that can occur during dependency operations
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::DEPENDENCY_DEPRECATED
)]
#[derive(Debug, Error)]
pub enum DependencyError {
    /// Resource does not exist
    #[error("Resource {0} does not exist")]
    ResourceNotFound(ContentId),
    
    /// Dependency already exists
    #[error("Dependency from {0} to {1} already exists")]
    DependencyExists(ContentId, ContentId),
    
    /// Dependency does not exist
    #[error("Dependency from {0} to {1} does not exist")]
    DependencyNotFound(ContentId, ContentId),
    
    /// Circular dependency detected
    #[error("Adding dependency from {0} to {1} would create a circular dependency")]
    CircularDependency(ContentId, ContentId),
    
    /// Generic dependency error
    #[error("Dependency error: {0}")]
    Other(String),
}

/// Result type for dependency operations
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::DEPENDENCY_DEPRECATED
)]
pub type DependencyResult<T> = Result<T, DependencyError>;

/// A structure that encapsulates details about a dependency
#[deprecated_warning(
    since = messages::SINCE_VERSION,
    note = messages::DEPENDENCY_DEPRECATED
)]
#[derive(Debug, Clone)]
pub struct ResourceDependency {
    /// The source resource ID (the dependent resource)
    pub source_id: ContentId,
    
    /// The target resource ID (the resource being depended on)
    pub target_id: ContentId,
    
    /// The type of dependency
    pub dependency_type: DependencyType,
    
    /// Optional domain IDs for cross-domain dependencies
    pub domain_ids: Option<(ContentId, ContentId)>,
    
    /// Optional ID of the effect that created this dependency
    pub creator_effect_id: Option<ContentId>,
    
    /// Additional metadata about the dependency
    pub metadata: HashMap<String, String>,
}

/// Legacy resource dependency manager
#[deprecated_error(
    since = messages::SINCE_VERSION,
    note = messages::DEPENDENCY_DEPRECATED
)]
pub struct ResourceDependencyManager {
    /// Map of source resource ID to dependencies (outgoing dependencies)
    dependencies_by_source: RwLock<HashMap<ContentId, HashSet<ResourceDependency>>>,
    
    /// Map of target resource ID to dependent resources (incoming dependencies)
    dependencies_by_target: RwLock<HashMap<ContentId, HashSet<ResourceDependency>>>,
}

impl ResourceDependencyManager {
    /// Create a new resource dependency manager
    pub fn new() -> Self {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::new",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        Self {
            dependencies_by_source: RwLock::new(HashMap::new()),
            dependencies_by_target: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a dependency between resources
    pub fn add_dependency(&self, dependency: ResourceDependency) -> DependencyResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::add_dependency",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        // Check for circular dependencies
        if self.would_create_cycle(&dependency.source_id, &dependency.target_id) {
            return Err(DependencyError::CircularDependency(
                dependency.source_id.clone(),
                dependency.target_id.clone(),
            ));
        }
        
        // Add to source map
        {
            let mut source_map = self.dependencies_by_source.write().unwrap();
            let source_deps = source_map
                .entry(dependency.source_id.clone())
                .or_insert_with(HashSet::new);
            
            // Check if dependency already exists
            for existing in source_deps.iter() {
                if existing.target_id == dependency.target_id 
                   && existing.dependency_type == dependency.dependency_type {
                    return Err(DependencyError::DependencyExists(
                        dependency.source_id.clone(),
                        dependency.target_id.clone(),
                    ));
                }
            }
            
            source_deps.insert(dependency.clone());
        }
        
        // Add to target map
        {
            let mut target_map = self.dependencies_by_target.write().unwrap();
            let target_deps = target_map
                .entry(dependency.target_id.clone())
                .or_insert_with(HashSet::new);
            
            target_deps.insert(dependency);
        }
        
        Ok(())
    }
    
    /// Remove a dependency between resources
    pub fn remove_dependency(
        &self,
        source_id: &ContentId,
        target_id: &ContentId,
        dependency_type: DependencyType,
    ) -> DependencyResult<()> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::remove_dependency",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        let dependency_found = {
            let mut source_map = self.dependencies_by_source.write().unwrap();
            if let Some(source_deps) = source_map.get_mut(source_id) {
                let before_len = source_deps.len();
                source_deps.retain(|dep| {
                    !(dep.target_id == *target_id && dep.dependency_type == dependency_type)
                });
                
                // Remove the entry if no more dependencies
                if source_deps.is_empty() {
                    source_map.remove(source_id);
                }
                
                before_len > source_deps.len()
            } else {
                false
            }
        };
        
        if !dependency_found {
            return Err(DependencyError::DependencyNotFound(
                source_id.clone(),
                target_id.clone(),
            ));
        }
        
        // Update target map
        {
            let mut target_map = self.dependencies_by_target.write().unwrap();
            if let Some(target_deps) = target_map.get_mut(target_id) {
                target_deps.retain(|dep| {
                    !(dep.source_id == *source_id && dep.dependency_type == dependency_type)
                });
                
                // Remove the entry if no more dependencies
                if target_deps.is_empty() {
                    target_map.remove(target_id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get dependencies for a source resource
    pub fn get_dependencies_for_source(&self, source_id: &ContentId) -> Vec<ResourceDependency> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::get_dependencies_for_source",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        let source_map = self.dependencies_by_source.read().unwrap();
        source_map
            .get(source_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get dependencies for a target resource
    pub fn get_dependencies_for_target(&self, target_id: &ContentId) -> Vec<ResourceDependency> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::get_dependencies_for_target",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        let target_map = self.dependencies_by_target.read().unwrap();
        target_map
            .get(target_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Check if a resource has any dependencies
    pub fn has_dependencies(&self, source_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::has_dependencies",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        let source_map = self.dependencies_by_source.read().unwrap();
        source_map.contains_key(source_id)
    }
    
    /// Check if a resource has any dependents
    pub fn has_dependents(&self, target_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::has_dependents",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        let target_map = self.dependencies_by_target.read().unwrap();
        target_map.contains_key(target_id)
    }
    
    /// Check if there's a dependency between two resources
    pub fn has_dependency(&self, source_id: &ContentId, target_id: &ContentId) -> bool {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::has_dependency",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        let source_map = self.dependencies_by_source.read().unwrap();
        if let Some(deps) = source_map.get(source_id) {
            deps.iter().any(|dep| dep.target_id == *target_id)
        } else {
            false
        }
    }
    
    /// Get all resources that depend on a target resource
    pub fn get_dependent_resources(&self, target_id: &ContentId) -> Vec<ContentId> {
        crate::interface::deprecation::emit_deprecation_warning(
            "ResourceDependencyManager::get_dependent_resources",
            messages::SINCE_VERSION,
            messages::DEPENDENCY_DEPRECATED
        );
        
        let target_map = self.dependencies_by_target.read().unwrap();
        target_map
            .get(target_id)
            .map(|deps| deps.iter().map(|dep| dep.source_id.clone()).collect())
            .unwrap_or_default()
    }
    
    // Helper method to check for circular dependencies
    fn would_create_cycle(&self, source_id: &ContentId, target_id: &ContentId) -> bool {
        // Quick check: direct cycle
        if source_id == target_id {
            return true;
        }
        
        // Check if target depends on source (which would create a cycle)
        let mut visited = HashSet::new();
        let mut stack = vec![target_id.clone()];
        
        while let Some(current) = stack.pop() {
            if &current == source_id {
                return true;
            }
            
            if visited.insert(current.clone()) {
                // Get dependencies of current
                let deps = self.get_dependencies_for_source(&current);
                for dep in deps {
                    stack.push(dep.target_id);
                }
            }
        }
        
        false
    }
}

impl Default for ResourceDependencyManager {
    fn default() -> Self {
        Self::new()
    }
} 