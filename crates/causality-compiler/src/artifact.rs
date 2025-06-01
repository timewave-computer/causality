//! Content-addressed compilation artifacts
//!
//! This module provides content-addressable storage for compilation results,
//! enabling caching and reproducible builds.

use crate::pipeline::{SExpression, CompiledArtifact};
use crate::error::{CompileError, CompileResult};
use causality_core::lambda::Term;
use causality_core::machine::Instruction;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Content hash for compilation artifacts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(pub u64);

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Content-addressable compilation artifact
/// 
/// This extends the basic CompiledArtifact with content addressing
/// for caching and reproducible builds.
#[derive(Debug, Clone)]
pub struct ContentAddressedArtifact {
    /// Content hash of the artifact
    pub hash: ContentHash,
    /// The compiled artifact
    pub artifact: CompiledArtifact,
}

impl ContentAddressedArtifact {
    /// Create a new content-addressed artifact
    pub fn new(artifact: CompiledArtifact) -> Self {
        let hash = compute_content_hash(&artifact);
        Self { hash, artifact }
    }
    
    /// Get the content hash
    pub fn hash(&self) -> &ContentHash {
        &self.hash
    }
    
    /// Get the source code
    pub fn source(&self) -> &str {
        &self.artifact.source
    }
    
    /// Get the S-expression
    pub fn sexpr(&self) -> &SExpression {
        &self.artifact.sexpr
    }
    
    /// Get the compiled term
    pub fn term(&self) -> &Term {
        &self.artifact.term
    }
    
    /// Get the compiled instructions
    pub fn instructions(&self) -> &[Instruction] {
        &self.artifact.instructions
    }
}

/// Compute content hash for a compilation artifact
/// 
/// This creates a deterministic hash based on the source code,
/// ensuring identical source produces identical hashes.
fn compute_content_hash(artifact: &CompiledArtifact) -> ContentHash {
    let mut hasher = DefaultHasher::new();
    
    // Hash the source code (this is the primary input)
    artifact.source.hash(&mut hasher);
    
    // Note: We only hash the source, not the compiled outputs,
    // because those should be deterministic given the source.
    // This ensures that the same source always produces the same hash.
    
    ContentHash(hasher.finish())
}

/// Simple artifact cache for development
/// 
/// In production, this would be replaced with a more sophisticated
/// content-addressable storage system.
#[derive(Debug, Default)]
pub struct ArtifactCache {
    artifacts: std::collections::HashMap<ContentHash, ContentAddressedArtifact>,
}

impl ArtifactCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Insert an artifact into the cache
    pub fn insert(&mut self, artifact: ContentAddressedArtifact) {
        self.artifacts.insert(artifact.hash().clone(), artifact);
    }
    
    /// Retrieve an artifact by hash
    pub fn get(&self, hash: &ContentHash) -> Option<&ContentAddressedArtifact> {
        self.artifacts.get(hash)
    }
    
    /// Check if an artifact exists in the cache
    pub fn contains(&self, hash: &ContentHash) -> bool {
        self.artifacts.contains_key(hash)
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.artifacts.len(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
}

/// Build a content-addressed artifact from source
pub fn build_artifact(source: &str) -> CompileResult<ContentAddressedArtifact> {
    let artifact = crate::pipeline::compile(source)?;
    Ok(ContentAddressedArtifact::new(artifact))
}

/// Verify that an artifact's hash matches its content
pub fn verify_artifact(artifact: &ContentAddressedArtifact) -> bool {
    let expected_hash = compute_content_hash(&artifact.artifact);
    artifact.hash == expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_hash_deterministic() {
        let source = "(pure 42)";
        
        let artifact1 = build_artifact(source).unwrap();
        let artifact2 = build_artifact(source).unwrap();
        
        // Same source should produce same hash
        assert_eq!(artifact1.hash(), artifact2.hash());
    }
    
    #[test]
    fn test_content_hash_different_sources() {
        let artifact1 = build_artifact("(pure 42)").unwrap();
        let artifact2 = build_artifact("(pure 43)").unwrap();
        
        // Different sources should produce different hashes
        assert_ne!(artifact1.hash(), artifact2.hash());
    }
    
    #[test]
    fn test_artifact_cache() {
        let mut cache = ArtifactCache::new();
        let artifact = build_artifact("(pure 42)").unwrap();
        let hash = artifact.hash().clone();
        
        // Insert and retrieve
        cache.insert(artifact);
        assert!(cache.contains(&hash));
        
        let retrieved = cache.get(&hash).unwrap();
        assert_eq!(retrieved.source(), "(pure 42)");
    }
    
    #[test]
    fn test_verify_artifact() {
        let artifact = build_artifact("(pure 42)").unwrap();
        assert!(verify_artifact(&artifact));
    }
    
    #[test]
    fn test_content_hash_display() {
        let artifact = build_artifact("(pure 42)").unwrap();
        let hash_str = format!("{}", artifact.hash());
        
        // Should be a 16-character hex string
        assert_eq!(hash_str.len(), 16);
        assert!(hash_str.chars().all(|c| c.is_ascii_hexdigit()));
    }
} 