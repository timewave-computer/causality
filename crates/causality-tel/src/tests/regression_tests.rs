//! Regression tests for TEL features
//!
//! These tests ensure that features that worked in previous versions
//! of the TEL implementation continue to work after the migration to
//! the TEG-based architecture.

#[cfg(test)]
mod tests {
    use crate::ast::{Program, Flow, Statement, Expression, Literal};
    use crate::parser::{parse_program, parse_expression, parse_statement};
    use crate::combinators::{Combinator, reducer::BetaReducer};
    use crate::types::{TelType, BaseType, TypeEnvironment};
    use crate::ToTEG;
    
    /// Test that basic AST parsing continues to work
    #[test]
    fn test_ast_parsing() {
        // Parse a simple expression
        let expr_str = "1 + 2";
        let expr = parse_expression(expr_str).expect("Failed to parse expression");
        
        match expr {
            Expression::Binary { .. } => {
                // This is the expected type
            },
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
        
        // Parse a simple statement
        let stmt_str = "let x = 42;";
        let stmt = parse_statement(stmt_str).expect("Failed to parse statement");
        
        match stmt {
            Statement::Let { name, .. } => {
                assert_eq!(name, "x");
            },
            _ => panic!("Expected let statement, got {:?}", stmt),
        }
        
        // Parse a complete program
        let prog_str = r#"
            flow test() -> Int {
                let x = 42;
                return x;
            }
        "#;
        
        let prog = parse_program(prog_str).expect("Failed to parse program");
        assert!(prog.flows.contains_key("test"), "Program should contain test flow");
    }
    
    /// Test that combinatory logic evaluation still works
    #[test]
    fn test_combinator_evaluation() {
        // Create a simple identity combinator application: I x
        let i_combinator = Combinator::I;
        let x_literal = Combinator::int(42);
        let application = Combinator::app(i_combinator, x_literal.clone());
        
        // Evaluate the application
        let result = application.eval().expect("Failed to evaluate combinator");
        assert_eq!(result, x_literal, "I x should evaluate to x");
        
        // Test K combinator: K x y = x
        let k_combinator = Combinator::K;
        let k_x = Combinator::app(k_combinator, x_literal.clone());
        let y_literal = Combinator::string("hello");
        let k_x_y = Combinator::app(k_x, y_literal);
        
        // Evaluate K x y
        let result = k_x_y.eval().expect("Failed to evaluate K x y");
        
        // Since our eval only does one step at a time, we need to check the structure
        match result {
            Combinator::App { .. } => {
                // This is expected - would need multiple eval steps to get to x
            },
            _ => panic!("Unexpected result from K x y: {:?}", result),
        }
    }
    
    /// Test that type system features still work
    #[test]
    fn test_type_system() {
        // Create a simple function type
        let string_type = TelType::Base(BaseType::String);
        let int_type = TelType::Base(BaseType::Int);
        let fn_type = TelType::Function(
            Box::new(string_type.clone()),
            Box::new(int_type.clone())
        );
        
        // Test basic subtyping
        assert!(string_type.is_subtype(&string_type), "A type should be a subtype of itself");
        
        // Function subtyping is contravariant in argument and covariant in result
        let any_type = TelType::Base(BaseType::Any);
        let fn_super = TelType::Function(
            Box::new(any_type.clone()),  // more general parameter type
            Box::new(int_type.clone())   // same return type
        );
        
        assert!(fn_type.is_subtype(&fn_super), "Function types should respect parameter contravariance");
        
        // Create a type environment
        let mut env = TypeEnvironment::new();
        env.insert("x".to_string(), string_type.clone());
        env.insert("y".to_string(), int_type.clone());
        
        assert_eq!(env.get("x"), Some(&string_type), "Environment should contain x: String");
        assert_eq!(env.get("y"), Some(&int_type), "Environment should contain y: Int");
    }
    
    /// Test that content addressing still works
    #[test]
    fn test_content_addressing() {
        // Create a simple expression
        let expr = Combinator::string("hello world");
        
        // Get content ID
        let content_id = expr.content_id().expect("Failed to get content ID");
        assert!(!content_id.is_empty(), "Content ID should not be empty");
        
        // Create an identical expression
        let expr2 = Combinator::string("hello world");
        let content_id2 = expr2.content_id().expect("Failed to get content ID");
        
        // Content IDs should be identical for identical expressions
        assert_eq!(content_id, content_id2, "Content IDs should be identical for identical expressions");
        
        // Create a different expression
        let expr3 = Combinator::string("different text");
        let content_id3 = expr3.content_id().expect("Failed to get content ID");
        
        // Content IDs should be different for different expressions
        assert_ne!(content_id, content_id3, "Content IDs should be different for different expressions");
    }
    
    /// Test that conversion to TEG works for complex programs
    #[test]
    fn test_complex_program_conversion() {
        // Create a more complex program with multiple flows and effects
        let tel_code = r#"
            effect log(message: String): Unit;
            effect read(path: String): String;
            effect write(path: String, data: String): Unit;
            
            flow process_file(input_path: String, output_path: String) -> Unit {
                perform log("Processing file: " + input_path);
                let content = perform read(input_path);
                let processed = transform(content);
                perform write(output_path, processed);
                perform log("Processing complete");
            }
            
            flow transform(content: String) -> String {
                // Just a simple transformation for testing
                return content + " (transformed)";
            }
        "#;
        
        // Parse the program
        let program = parse_program(tel_code).expect("Failed to parse complex program");
        
        // Verify basic program structure
        assert_eq!(program.flows.len(), 2, "Program should have 2 flows");
        assert_eq!(program.effect_defs.len(), 3, "Program should have 3 effect definitions");
        
        // Convert to TEG
        let teg = program.to_teg().expect("Failed to convert complex program to TEG");
        
        // Verify TEG structure
        assert!(!teg.is_empty(), "TEG should not be empty");
        
        // The TEG should contain effect nodes for each effect operation
        let effect_count = teg.effect_nodes().count();
        assert!(effect_count >= 4, "TEG should have at least 4 effect nodes (log, read, transform, write, log)");
    }
} 