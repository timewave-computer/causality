//! Program Lifecycle Management
//!
//! This module provides lifecycle status tracking for program compilation.

/// Compilation stages for a program
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationStage {
    /// Initial program registration
    Registered,

    /// Program is queued for validation
    ValidationQueued,

    /// Program is being validated
    Validating,

    /// Program has been validated
    Validated,

    /// Program is queued for compilation
    CompilationQueued,

    /// Program is being compiled
    Compiling,

    /// Program has been successfully compiled
    Compiled,

    /// Program compilation failed
    Failed,
}

impl CompilationStage {
    /// Check if the stage is terminal (compilation completed successfully or failed)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Compiled | Self::Failed)
    }

    /// Check if the stage indicates compilation is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self,
            Self::ValidationQueued
                | Self::Validating
                | Self::CompilationQueued
                | Self::Compiling
        )
    }
}
