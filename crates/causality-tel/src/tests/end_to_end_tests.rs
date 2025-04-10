//! End-to-end tests for the full TEL → TEG → Execution workflow
//!
//! These tests verify the complete pipeline of parsing TEL code,
//! converting it to a Temporal Effect Graph, and executing it
//! using the engine.

#[cfg(test)]
mod tests {
    use crate::ast::{Program, Flow, Statement, Expression, Literal};
    use crate::parser::parse_program;
    use crate::ToTEG;
    use causality_ir::TemporalEffectGraph;
    
    /// Test the complete workflow from TEL parsing to TEG conversion
    #[test]
    fn test_tel_to_teg_workflow() {
        // Define a simple TEL program as a string
        let tel_code = r#"
            effect log(message: String): Unit;
            
            flow hello_world() -> String {
                perform log("Hello, TEG!");
                return "Completed";
            }
        "#;
        
        // Parse the TEL program
        let program = parse_program(tel_code).expect("Failed to parse TEL program");
        
        // Verify basic program structure
        assert!(program.flows.contains_key("hello_world"), "Program should contain the hello_world flow");
        assert_eq!(program.effect_defs.len(), 1, "Program should have one effect definition");
        assert!(program.effect_defs.contains_key("log"), "Program should define the log effect");
        
        // Convert to TEG
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Verify TEG structure
        assert!(!teg.is_empty(), "TEG should not be empty");
        
        // Check TEG serialization
        let serialized = serde_json::to_string(&teg).expect("Failed to serialize TEG");
        assert!(!serialized.is_empty(), "Serialized TEG should not be empty");
        
        // Note: Actual execution would be done with TelEffectExecutor from causality-engine
        println!("Successfully converted TEL to TEG");
    }
    
    /// Test creating and manipulating a TEG programmatically
    #[test]
    fn test_programmatic_teg_creation() {
        // Create a program manually
        let mut program = Program::new();
        
        // Add an effect definition
        program.add_effect_definition(
            "log",
            vec![("message".to_string(), crate::types::BaseType::String.into())],
            crate::types::BaseType::Unit.into()
        );
        
        // Create a flow
        let mut flow = Flow::new("test_flow");
        flow.body.push(Statement::Perform {
            effect_name: "log".to_string(),
            args: vec![
                Expression::Literal(Literal::String("Programmatically created TEG".to_string()))
            ]
        });
        
        program.flows.insert("test_flow".to_string(), flow);
        
        // Convert to TEG
        let teg = program.to_teg().expect("Failed to convert program to TEG");
        
        // Verify TEG structure
        assert!(!teg.is_empty(), "TEG should not be empty");
        
        // Test TEG manipulations
        let mut teg2 = teg.clone();
        teg2.add_metadata("version", "1.0".into()).expect("Failed to add metadata");
        
        assert!(teg2.metadata().contains_key("version"), "TEG should contain version metadata");
    }
} 