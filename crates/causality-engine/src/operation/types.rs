use std::fmt::Debug;

use causality_types::DomainId;

use super::context::{AbstractContext, RegisterContext, PhysicalContext, ZkContext, ExecutionEnvironment, ExecutionPhase, ExecutionContext};

/// All possible execution context types
#[derive(Debug, Clone)]
pub enum Context {
    /// Abstract execution context
    Abstract(AbstractContext),
    /// Register-based execution context
    Register(RegisterContext),
    /// Physical execution context
    Physical(PhysicalContext),
    /// ZK execution context
    Zk(ZkContext),
}

impl Context {
    /// Get the environment for this context
    pub fn environment(&self) -> ExecutionEnvironment {
        match self {
            Context::Abstract(ctx) => ctx.environment(),
            Context::Register(ctx) => ctx.environment(),
            Context::Physical(ctx) => ctx.environment(),
            Context::Zk(ctx) => ctx.environment(),
        }
    }

    /// Get the domain for this context
    pub fn domain(&self) -> Option<DomainId> {
        match self {
            Context::Abstract(ctx) => ctx.domain(),
            Context::Register(ctx) => ctx.domain(),
            Context::Physical(ctx) => ctx.domain(),
            Context::Zk(ctx) => ctx.domain(),
        }
    }

    /// Get the execution phase for this context
    pub fn phase(&self) -> ExecutionPhase {
        match self {
            Context::Abstract(ctx) => ctx.phase().clone(),
            Context::Register(ctx) => ctx.phase().clone(),
            Context::Physical(ctx) => ctx.phase().clone(),
            Context::Zk(ctx) => ctx.phase().clone(),
        }
    }

    /// Check if this context requires a proof
    pub fn requires_proof(&self) -> bool {
        match self {
            Context::Abstract(ctx) => ctx.proof_required(),
            Context::Register(ctx) => ctx.proof_required(),
            Context::Physical(ctx) => ctx.proof_required(),
            Context::Zk(ctx) => ctx.proof_required(),
        }
    }

    /// Get the required capabilities for this context
    pub fn required_capabilities(&self) -> Vec<String> {
        match self {
            Context::Abstract(ctx) => ctx.required_capabilities(),
            Context::Register(ctx) => ctx.required_capabilities(),
            Context::Physical(ctx) => ctx.required_capabilities(),
            Context::Zk(ctx) => ctx.required_capabilities(),
        }
    }
} 