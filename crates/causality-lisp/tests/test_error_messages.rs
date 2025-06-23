//! Parser Error Message Tests
//!
//! This module contains comprehensive tests for the enhanced parse error messages 
//! in Causality Lisp, ensuring that helpful, contextual error messages are provided
//! for various syntax errors.

use causality_lisp::parser::LispParser;

/// Helper function to test that parsing fails with a specific error pattern
fn assert_parse_error_contains(input: &str, expected_patterns: &[&str]) {
    let mut parser = LispParser::new();
    match parser.parse(input) {
        Ok(_) => panic!("Expected parsing to fail for input: {}", input),
        Err(error) => {
            let error_msg = format!("{}", error);
            for pattern in expected_patterns {
                assert!(
                    error_msg.contains(pattern),
                    "Error message '{}' should contain '{}'",
                    error_msg,
                    pattern
                );
            }
        }
    }
}

/// Helper function to test that parsing succeeds
fn assert_parse_success(input: &str) {
    let mut parser = LispParser::new();
    match parser.parse(input) {
        Ok(_) => {}
        Err(error) => panic!("Expected parsing to succeed for input '{}', but got error: {}", input, error),
    }
}

#[test]
fn test_basic_syntax_errors() {
    // Unexpected closing parenthesis
    assert_parse_error_contains(
        ")",
        &["unexpected closing parenthesis", "remove the extra ')'"]
    );
    
    // Unclosed function call
    assert_parse_error_contains(
        "(+ 1 2",
        &["function call", "add ')'"]
    );
    
    // Nested unclosed expressions
    assert_parse_error_contains(
        "((",
        &["list", "add ')' to close"]
    );
}

#[test]
fn test_lambda_expression_errors() {
    // Incomplete lambda - missing parameter list
    assert_parse_error_contains(
        "(lambda)",
        &["Expected '('"]
    );
    
    // Incomplete lambda - missing body
    assert_parse_error_contains(
        "(lambda ())",
        &["lambda expression", "body expression", "add an expression after the parameter list"]
    );
    
    // Unclosed lambda parameter list
    assert_parse_error_contains(
        "(lambda (x",
        &["lambda parameter list", "parameters followed by ')'", "parameter names should be symbols"]
    );
    
    // Too many expressions in lambda
    assert_parse_error_contains(
        "(lambda (x) 42 extra)",
        &["Expected ')'", "symbol 'extra'"]
    );
    
    // Non-symbol parameter in lambda
    assert_parse_error_contains(
        "(lambda (42) body)",
        &["Expected symbol for lambda parameter", "number 42"]
    );
}

#[test]
fn test_let_unit_errors() {
    // Incomplete let-unit - missing expressions
    assert_parse_error_contains(
        "(let-unit)",
        &["let-unit expression", "unit expression and body", "let-unit requires two expressions"]
    );
    
    // Incomplete let-unit - missing body
    assert_parse_error_contains(
        "(let-unit (unit))",
        &["let-unit expression", "body expression", "let-unit requires a body expression"]
    );
}

#[test]
fn test_let_tensor_errors() {
    // Incomplete let-tensor - missing all components
    assert_parse_error_contains(
        "(let-tensor)",
        &["let-tensor expression", "tensor expression, variable names, and body"]
    );
    
    // Incomplete let-tensor - missing variables and body
    assert_parse_error_contains(
        "(let-tensor (tensor 1 2))",
        &["Expected symbol for left variable in let-tensor"]
    );
    
    // Incomplete let-tensor - missing right variable and body
    assert_parse_error_contains(
        "(let-tensor (tensor 1 2) x)",
        &["Expected symbol for right variable in let-tensor"]
    );
    
    // Incomplete let-tensor - missing body
    assert_parse_error_contains(
        "(let-tensor (tensor 1 2) x y)",
        &["let-tensor expression", "body expression", "let-tensor requires a body expression"]
    );
    
    // let-tensor with non-symbol left variable
    assert_parse_error_contains(
        "(let-tensor (tensor 1 2) 123 y body)",
        &["Expected symbol for left variable in let-tensor", "number 123"]
    );
}

#[test]
fn test_case_expression_errors() {
    // Incomplete case - missing all components
    assert_parse_error_contains(
        "(case)",
        &["case expression", "sum expression and branch handlers", "case requires:"]
    );
    
    // Incomplete case - missing branches
    assert_parse_error_contains(
        "(case (inl 42))",
        &["Expected symbol for left variable in case expression"]
    );
    
    // Incomplete case - missing right branch
    assert_parse_error_contains(
        "(case (inl 42) x left-branch)",
        &["Expected symbol for right variable in case expression"]
    );
}

#[test]
fn test_tensor_errors() {
    // Incomplete tensor - missing expressions
    assert_parse_error_contains(
        "(tensor)",
        &["tensor expression", "two expressions to combine", "tensor requires exactly two expressions"]
    );
    
    // Incomplete tensor - missing second expression
    assert_parse_error_contains(
        "(tensor 42)",
        &["tensor expression", "second expression", "tensor requires exactly two expressions"]
    );
}

#[test]
fn test_sum_type_errors() {
    // Incomplete inl - missing value
    assert_parse_error_contains(
        "(inl)",
        &["inl expression", "value expression", "inl requires one expression"]
    );
    
    // Incomplete inr - missing value
    assert_parse_error_contains(
        "(inr)",
        &["inr expression", "value expression", "inr requires one expression"]
    );
}

#[test]
fn test_resource_errors() {
    // Incomplete alloc - missing value
    assert_parse_error_contains(
        "(alloc)",
        &["alloc expression", "value expression to allocate", "alloc requires one expression"]
    );
    
    // Incomplete consume - missing resource
    assert_parse_error_contains(
        "(consume)",
        &["consume expression", "resource expression to consume", "consume requires one expression"]
    );
}

#[test]
fn test_string_and_number_errors() {
    // Unclosed string literal
    assert_parse_error_contains(
        "\"unclosed string",
        &["Unclosed string literal"]
    );
    
    // Invalid escape sequence
    assert_parse_error_contains(
        "\"invalid \\z escape\"",
        &["Invalid escape sequence", "\\z"]
    );
    
    // Invalid number format
    assert_parse_error_contains(
        "123.45.67",
        &["Unexpected character", "."]
    );
}

#[test]
fn test_valid_expressions_parse_successfully() {
    // Valid lambda expression
    assert_parse_success("(lambda (x) x)");
    
    // Valid let-unit expression
    assert_parse_success("(let-unit (unit) 42)");
    
    // Valid let-tensor expression
    assert_parse_success("(let-tensor (tensor 1 2) x y (+ x y))");
    
    // Valid case expression
    assert_parse_success("(case (inl 42) x (+ x 1) y (- y 1))");
    
    // Valid tensor with alloc/consume
    assert_parse_success("(tensor (alloc 42) (consume resource))");
    
    // Valid arithmetic expression
    assert_parse_success("(+ (* 2 3) (/ 8 4))");
    
    // Valid string literal
    assert_parse_success("\"Hello, world!\"");
    
    // Valid float literal
    
    // Valid boolean literal
    assert_parse_success("#t");
    assert_parse_success("#f");
    
    // Valid empty list
    assert_parse_success("()");
    
    // Valid negative numbers
    assert_parse_success("-42");
    
    // Valid symbols that start with minus (not numbers)
    assert_parse_success("-");
    assert_parse_success("-symbol");
}

#[test]
fn test_error_message_contains_position_info() {
    let mut parser = LispParser::new();
    
    // Test that error messages contain line and column information
    match parser.parse("(lambda (42) body)") {
        Ok(_) => panic!("Expected parsing to fail"),
        Err(error) => {
            let error_msg = format!("{}", error);
            assert!(error_msg.contains("line 1"));
            assert!(error_msg.contains("column"));
        }
    }
}

#[test]
fn test_error_message_helpfulness() {
    // Test that error messages are educational and actionable
    let test_cases = vec![
        ("(lambda ())", "body expression"),
        ("(tensor)", "exactly two expressions"),
        ("(let-unit)", "two expressions"),
        ("(case)", "case requires:"),
        ("(", "add ')' to close"),
    ];
    
    for (input, expected_help) in test_cases {
        assert_parse_error_contains(input, &[expected_help]);
    }
}

#[test]
fn test_nested_error_contexts() {
    // Test errors in nested contexts provide appropriate information
    assert_parse_error_contains(
        "(lambda (x) (tensor",
        &["tensor expression", "two expressions"]
    );
    
    assert_parse_error_contains(
        "(case (inl (lambda",
        &["Expected '('"]
    );
}

#[test]
fn test_multiple_error_scenarios() {
    // Test various combinations of errors to ensure robust error handling
    let error_inputs = vec![
        ")",
        "(",
        "()",
        "(lambda",
        "(lambda)",
        "(lambda ()",
        "(lambda () ",
        "(tensor 1",
        "(case (inl 1) x",
        "\"unclosed",
        "#invalid",
        "123.45.67.89",
    ];
    
    for input in error_inputs {
        let mut parser = LispParser::new();
        match parser.parse(input) {
            Ok(_) => {
                // Some might actually be valid (like "()" for empty list)
                continue;
            }
            Err(error) => {
                let error_msg = format!("{}", error);
                // Just ensure we get some kind of meaningful error message
                assert!(!error_msg.is_empty(), "Error message should not be empty for input: {}", input);
                assert!(!error_msg.contains("TODO"), "Error message should not contain TODO for input: {}", input);
            }
        }
    }
} 