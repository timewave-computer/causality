// Layer 1: Linear Session Calculus
// Adds types and structured communication to Layer 0's raw messages

pub mod types;
pub mod linear;
pub mod session;
pub mod compiler;
pub mod typechecker;

// Re-export key types
pub use types::{Type, SessionType, RowType};
pub use linear::{Variable, LinearContext, LinearityError};
pub use session::{Term, Value};
pub use typechecker::{typecheck, TypeContext, TypeError};
