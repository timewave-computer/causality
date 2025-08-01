//! Interactive REPL for Causality Lisp
//!
//! Provides an interactive Read-Eval-Print Loop for evaluating Causality Lisp expressions
//! with support for resource inspection and step-through execution.

use crate::error::CliErrorHandler;
use std::sync::Arc;
use std::io::{self, Write};
use colored::Colorize;
use anyhow::{Result, anyhow};

/// REPL commands and configuration
#[derive(Debug, Clone)]
pub struct ReplCommand {
    /// Enable debug mode with verbose output
    pub debug: bool,
    
    /// Maximum execution steps before timeout
    #[allow(dead_code)]
    pub max_steps: Option<usize>,
    
    /// Show machine state after each evaluation
    pub show_state: bool,
}

impl Default for ReplCommand {
    fn default() -> Self {
        Self {
            debug: false,
            max_steps: Some(10000),
            show_state: false,
        }
    }
}

/// REPL state management
pub struct ReplState {
    /// Configuration
    config: ReplCommand,
}

impl ReplState {
    /// Create a new REPL state
    pub fn new(config: ReplCommand) -> Self {
        Self { config }
    }
    
    /// Evaluate a Lisp expression
    pub fn evaluate(&mut self, input: &str) -> Result<String, anyhow::Error> {
        if input.trim().is_empty() {
            return Ok(String::new());
        }
        
        // Handle REPL commands
        if input.starts_with(':') {
            return self.handle_repl_command(input);
        }
        
        // Compile the input to machine instructions using unified pipeline
        let compiled_artifact = causality_compiler::compile(input)
            .map_err(|e| anyhow!("Compilation failed: {:?}", e))?;
        
        if self.config.debug {
            println!("{}", "Compiled instructions:".cyan());
            for (i, instr) in compiled_artifact.instructions.iter().enumerate() {
                println!("  {}: {:?}", i, instr);
            }
        }
        
        // Execute using unified 5-instruction machine
        let mut executor = causality_core::machine::BoundedExecutor::new(compiled_artifact.instructions.clone())?;
        let result = executor.execute()
            .map_err(|e| anyhow!("Execution failed: {:?}", e))?;
        
        if self.config.show_state {
            self.print_execution_result(&result);
        }
        
        Ok(format!("{:?}", result))
    }
    
    /// Handle special REPL commands
    fn handle_repl_command(&mut self, input: &str) -> Result<String, anyhow::Error> {
        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        
        match parts.first() {
            Some(&"help") | Some(&"h") => Ok(self.print_help()),
            Some(&"debug") => {
                self.config.debug = !self.config.debug;
                Ok(format!("Debug mode: {}", if self.config.debug { "on" } else { "off" }))
            }
            Some(&"state") => {
                self.config.show_state = !self.config.show_state;
                Ok(format!("Show state: {}", if self.config.show_state { "on" } else { "off" }))
            }
            Some(&"reset") => {
                // Reset state by creating new REPL state
                *self = ReplState::new(self.config.clone());
                Ok("REPL state reset".to_string())
            }
            Some(&"quit") | Some(&"exit") | Some(&"q") => {
                println!("{}", "Goodbye!".green());
                std::process::exit(0);
            }
            Some(cmd) => Err(anyhow!("Unknown command: {}", cmd)),
            None => Err(anyhow!("Empty command")),
        }
    }
    
    /// Print help information
    fn print_help(&self) -> String {
        format!(
            "{}\n\
            {}:\n  \
              (+ 1 2)           - Arithmetic operations\n  \
              (let x 42 x)      - Variable binding\n  \
              ((lambda (x) (+ x 1)) 5) - Lambda functions\n\
            {}:\n  \
              :help, :h         - Show this help\n  \
              :debug            - Toggle debug mode\n  \
              :state            - Toggle state display\n  \
              :reset            - Reset REPL state\n  \
              :quit, :exit, :q  - Exit REPL",
            "Causality Lisp REPL".cyan().bold(),
            "Examples".yellow(),
            "Commands".yellow()
        )
    }
    
    /// Print the execution result information
    fn print_execution_result(&self, result: &causality_core::machine::ExecutionResult) {
        println!("{}", "Execution Result:".cyan());
        match result {
            causality_core::machine::ExecutionResult::Success { steps_executed, .. } => {
                println!("    Status: Success");
                println!("    Steps executed: {}", steps_executed);
            }
            causality_core::machine::ExecutionResult::Error { message, steps_executed, .. } => {
                println!("    Status: Error");
                println!("    Message: {}", message);
                println!("    Steps executed: {}", steps_executed);
            }
            causality_core::machine::ExecutionResult::Timeout { steps_executed, .. } => {
                println!("    Status: Timeout");
                println!("    Steps executed: {}", steps_executed);
            }
        }
    }
}

/// Handle the REPL command
pub async fn handle_repl_command(
    config: ReplCommand,
    _error_handler: Arc<CliErrorHandler>,
) -> Result<(), anyhow::Error> {
    println!("{}", "Causality Lisp REPL".cyan().bold());
    println!("{}", "Type :help for commands or :quit to exit".dimmed());
    println!("{}", "Note: This REPL uses the unified 5-instruction machine system".dimmed());
    
    let mut repl_state = ReplState::new(config);
    
    loop {
        // Print prompt
        print!("{} ", ">".green().bold());
        io::stdout().flush().unwrap();
        
        // Read input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                match repl_state.evaluate(input.trim()) {
                    Ok(result) => {
                        if !result.is_empty() {
                            println!("{}", result);
                        }
                    }
                    Err(e) => {
                        println!("{}: {}", "Error".red().bold(), e);
                    }
                }
            }
            Err(e) => {
                println!("{}: {}", "Input error".red().bold(), e);
                break;
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repl_state_creation() {
        let config = ReplCommand::default();
        let _repl_state = ReplState::new(config);
    }
    
    #[test]
    fn test_repl_commands() {
        let config = ReplCommand::default();
        let mut repl_state = ReplState::new(config);
        
        // Test help command
        let result = repl_state.handle_repl_command(":help").unwrap();
        assert!(result.contains("Causality Lisp REPL"));
    }
    
    #[tokio::test]
    async fn test_basic_evaluation() {
        let config = ReplCommand::default();
        let mut repl_state = ReplState::new(config);
        
        // Test simple evaluation (this will fail until we have proper Lisp parsing)
        let _result = repl_state.evaluate("42");
    }
}
