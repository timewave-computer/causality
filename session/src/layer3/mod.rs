// Layer 3: Agent Orchestration
// Multi-party choreographies with capability-based access control

pub mod agent;
pub mod choreography;
pub mod capability;
pub mod compiler;

// Re-export core types
pub use agent::{Agent, AgentId, AgentRegistry, AgentState, AgentStatus};
pub use choreography::{Choreography, ChoreographyStep, Message};
pub use capability::{Capability, CapabilityHandler};
pub use compiler::{compile_choreography, CompilerError};
