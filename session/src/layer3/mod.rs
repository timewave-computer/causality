// Layer 3: Agent Orchestration
// Provides high-level abstractions for multi-party interactions

pub mod agent;
pub mod choreography;
pub mod capability;
pub mod compiler;

// Re-export key types
pub use agent::{Agent, AgentId, AgentState, AgentStatus};
pub use choreography::{Choreography, ChoreographyStep};
pub use capability::Capability;
pub use compiler::{compile_choreography, CompileError};
