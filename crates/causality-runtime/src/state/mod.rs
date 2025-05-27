// Re-export essential traits and types for state operations
pub use causality_types::resource::state::ResourceState;
// Note: AsResource and AsValue traits don't exist in causality_types::state

// Export state proof verification
pub mod state_proof; 