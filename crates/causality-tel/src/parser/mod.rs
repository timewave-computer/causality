// Parser module for TEL 
pub mod common;
pub mod expression;
pub mod statement;
pub mod types;
pub mod flow;
pub mod effect;
pub mod handler;
pub mod state_machine; // Add state machine parser

use nom::IResult;
use crate::ast::Program;
use crate::parser::state_machine::parse_state_machine;

/// Parse a complete TEL program
pub fn parse_program(input: &str) -> IResult<&str, Program> {
    // TODO: Implement complete program parsing
    // For now, just return a placeholder Program 
    Ok((input, Program::default()))
}

// Export key parsing functions
pub use self::expression::parse_expression;
pub use self::statement::parse_statement;
pub use self::flow::parse_flow;
pub use self::effect::parse_effect;
pub use self::handler::parse_handler;
pub use self::state_machine::parse_state_machine; 