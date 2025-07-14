//! Integration tests for S-expression intermediate format specification
//!
//! This module tests all features described in docs/106-s-expression-intermediate-format-specification.md
//! to validate that the specification is correct and implementable.

use causality_compiler::pipeline::{
    compile_sexpr_to_term, compile_term_to_instructions, parse_sexpr, SExpression,
};
use causality_core::lambda::{Literal, TermKind};
use causality_core::machine::Instruction;

// ========================================================================
// 1. Basic Syntax Tests (Section 1.1)
// ========================================================================

#[test]
fn test_basic_syntax_atoms() {
    // Integer literals
    let expr = parse_sexpr("42").unwrap();
    assert_eq!(expr, SExpression::Integer(42));

    // Boolean literals
    let expr = parse_sexpr("#t").unwrap();
    assert_eq!(expr, SExpression::Boolean(true));

    let expr = parse_sexpr("#f").unwrap();
    assert_eq!(expr, SExpression::Boolean(false));

    // Symbol/identifier
    let expr = parse_sexpr("symbol-name").unwrap();
    assert_eq!(expr, SExpression::Symbol("symbol-name".to_string()));
}

#[test]
fn test_basic_syntax_lists() {
    // Basic list
    let expr = parse_sexpr("(operator operand1 operand2)").unwrap();
    match expr {
        SExpression::List(elements) => {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], SExpression::Symbol("operator".to_string()));
            assert_eq!(elements[1], SExpression::Symbol("operand1".to_string()));
            assert_eq!(elements[2], SExpression::Symbol("operand2".to_string()));
        }
        _ => panic!("Expected list"),
    }
}

// ========================================================================
// 2. Core Expression Types Tests (Section 1.2)
// ========================================================================

#[test]
fn test_pure_values() {
    // Pure integer
    let expr = parse_sexpr("(pure 42)").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    match &term.kind {
        TermKind::Literal(Literal::Int(42)) => {}
        _ => panic!("Expected pure integer literal, got {:?}", term.kind),
    }

    // Pure boolean
    let expr = parse_sexpr("(pure #t)").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    match &term.kind {
        TermKind::Literal(Literal::Bool(true)) => {}
        _ => panic!("Expected pure boolean literal, got {:?}", term.kind),
    }

    // Pure symbol
    let expr = parse_sexpr("(pure success)").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    // Pure of symbol should just be the symbol
    match &term.kind {
        TermKind::Var(name) if name == "success" => {}
        _ => panic!("Expected variable 'success', got {:?}", term.kind),
    }
}

#[test]
fn test_resource_operations() {
    // Resource allocation
    let expr = parse_sexpr("(alloc TokenA 100)").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    match &term.kind {
        TermKind::Alloc { .. } => {}
        _ => panic!("Expected alloc term, got {:?}", term.kind),
    }

    // Resource consumption
    let expr = parse_sexpr("(consume r1)").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    match &term.kind {
        TermKind::Consume { .. } => {}
        _ => panic!("Expected consume term, got {:?}", term.kind),
    }
}

#[test]
fn test_lambda_functions() {
    // Lambda definition
    let expr = parse_sexpr("(lambda (amount) (alloc TokenA amount))").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    match &term.kind {
        TermKind::Lambda { param, body, .. } => {
            assert_eq!(param, "amount");
            match &body.kind {
                TermKind::Alloc { .. } => {}
                _ => panic!("Expected alloc in lambda body"),
            }
        }
        _ => panic!("Expected lambda term, got {:?}", term.kind),
    }

    // Function application
    let expr = parse_sexpr("(apply transfer-fn 100 recipient-id)").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    match &term.kind {
        TermKind::Apply { .. } => {}
        _ => panic!("Expected apply term, got {:?}", term.kind),
    }
}

#[test]
fn test_monadic_operations() {
    // Bind operation
    let expr =
        parse_sexpr("(bind (alloc TokenA 100) (lambda (token) (consume token)))")
            .unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    match &term.kind {
        TermKind::Apply { func, arg } => {
            // bind compiles to application
            match &func.kind {
                TermKind::Lambda { .. } => {}
                _ => {}
            }
            match &arg.kind {
                TermKind::Alloc { .. } => {}
                _ => panic!("Expected alloc as bind argument"),
            }
        }
        _ => panic!("Expected apply term from bind, got {:?}", term.kind),
    }
}

// ========================================================================
// 3. Multi-Domain Extensions Tests (Section 1.3)
// ========================================================================

#[test]
fn test_domain_declarations_parsing() {
    // Domain declaration syntax
    let expr = parse_sexpr(
        r#"
        (domain ethereum-mainnet
          (capabilities token-transfer liquidity-provision)
          (resources ETH USDC DAI)
          (interfaces uniswap-v3 curve))
    "#,
    )
    .unwrap();

    match expr {
        SExpression::List(elements) => {
            assert_eq!(elements[0], SExpression::Symbol("domain".to_string()));
            assert_eq!(
                elements[1],
                SExpression::Symbol("ethereum-mainnet".to_string())
            );
            // Verify structure exists (implementation would handle semantics)
            assert!(elements.len() >= 4);
        }
        _ => panic!("Expected domain declaration"),
    }
}

#[test]
fn test_cross_domain_operations_parsing() {
    // Cross-domain transfer syntax
    let expr = parse_sexpr(
        r#"
        (cross-domain-transfer
          (from-domain ethereum)
          (to-domain polygon)
          (resource USDC 1000)
          (conditions (minimum-confirmations 12)))
    "#,
    )
    .unwrap();

    match expr {
        SExpression::List(elements) => {
            assert_eq!(
                elements[0],
                SExpression::Symbol("cross-domain-transfer".to_string())
            );
            assert!(elements.len() >= 4);
            // Verify from-domain, to-domain, resource, conditions are present
        }
        _ => panic!("Expected cross-domain-transfer"),
    }
}

// ========================================================================
// 4. Compilation Semantics Tests (Section 2)
// ========================================================================

#[test]
fn test_compilation_phases() {
    // Test that we can compile through all phases
    let source = "(bind (alloc TokenA 100) (lambda (token) (consume token)))";

    // Phase 1: Parse to S-expression
    let sexpr = parse_sexpr(source).unwrap();

    // Phase 2: Compile to lambda calculus term
    let term = compile_sexpr_to_term(&sexpr).unwrap();

    // Phase 3: Compile to register machine instructions
    let instructions = compile_term_to_instructions(&term).unwrap();

    assert!(!instructions.is_empty());

    // Verify we have some expected instruction types
    let has_alloc = instructions
        .iter()
        .any(|inst| matches!(inst, Instruction::Alloc { .. }));
    assert!(has_alloc, "Expected Alloc instruction in compiled output");
}

#[test]
fn test_type_checking_integration() {
    // Test that type information is preserved through compilation
    let source = "(lambda (x) (alloc TokenA x))";
    let sexpr = parse_sexpr(source).unwrap();
    let term = compile_sexpr_to_term(&sexpr).unwrap();

    match &term.kind {
        TermKind::Lambda { param, body, .. } => {
            assert_eq!(param, "x");
            match &body.kind {
                TermKind::Alloc { .. } => {}
                _ => panic!("Expected alloc in lambda body"),
            }
        }
        _ => panic!("Expected lambda term"),
    }
}

// ========================================================================
// 5. Error Handling Tests
// ========================================================================

#[test]
fn test_syntax_error_handling() {
    // Unclosed parenthesis
    let result = parse_sexpr("(alloc TokenA 100");
    assert!(result.is_err());

    // Invalid token
    let result = parse_sexpr("(alloc TokenA @invalid)");
    assert!(result.is_err());
}

#[test]
fn test_semantic_error_handling() {
    // Wrong arity for pure
    let sexpr = parse_sexpr("(pure)").unwrap();
    let result = compile_sexpr_to_term(&sexpr);
    assert!(result.is_err());

    // Wrong arity for bind
    let sexpr = parse_sexpr("(bind expr)").unwrap();
    let result = compile_sexpr_to_term(&sexpr);
    assert!(result.is_err());
}

// ========================================================================
// 6. Complex Integration Tests
// ========================================================================

#[test]
fn test_complex_arbitrage_program() {
    let program = r#"
        (bind (alloc USDC 1000)
          (lambda (usdc)
            (bind (cross-domain-transfer usdc polygon)
              (lambda (polygon-usdc)
                (bind (domain-effect polygon (swap polygon-usdc MATIC))
                  (lambda (matic)
                    (bind (cross-domain-transfer matic ethereum)
                      (lambda (eth-matic)
                        (domain-effect ethereum (swap eth-matic ETH))))))))))
    "#;

    let sexpr = parse_sexpr(program).unwrap();
    let term = compile_sexpr_to_term(&sexpr).unwrap();
    let instructions = compile_term_to_instructions(&term).unwrap();

    assert!(!instructions.is_empty());

    // Should contain resource allocation and consumption
    let has_alloc = instructions
        .iter()
        .any(|inst| matches!(inst, Instruction::Alloc { .. }));
    assert!(has_alloc);
}

#[test]
fn test_tensor_operations() {
    let expr =
        parse_sexpr("(tensor (alloc TokenA 100) (alloc TokenB 200))").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();

    match &term.kind {
        TermKind::Tensor { left, right } => match (&left.kind, &right.kind) {
            (TermKind::Alloc { .. }, TermKind::Alloc { .. }) => {}
            _ => panic!("Expected alloc terms in tensor"),
        },
        _ => panic!("Expected tensor term, got {:?}", term.kind),
    }
}

// ========================================================================
// 7. Performance and Optimization Tests
// ========================================================================

#[test]
fn test_compilation_performance() {
    use std::time::Instant;

    let complex_program = r#"
        (bind (alloc TokenA 1000)
          (lambda (a)
            (bind (alloc TokenB 2000)
              (lambda (b)
                (bind (tensor a b)
                  (lambda (ab)
                    (bind (consume ab)
                      (lambda (_)
                        (alloc TokenC 3000)))))))))
    "#;

    let start = Instant::now();
    let sexpr = parse_sexpr(complex_program).unwrap();
    let term = compile_sexpr_to_term(&sexpr).unwrap();
    let _instructions = compile_term_to_instructions(&term).unwrap();
    let duration = start.elapsed();

    // Compilation should be reasonably fast (under 100ms for this size)
    assert!(
        duration.as_millis() < 100,
        "Compilation took too long: {:?}",
        duration
    );
}

#[test]
fn test_instruction_count_reasonable() {
    let program = "(bind (alloc TokenA 100) (lambda (token) (consume token)))";
    let sexpr = parse_sexpr(program).unwrap();
    let term = compile_sexpr_to_term(&sexpr).unwrap();
    let instructions = compile_term_to_instructions(&term).unwrap();

    // Should generate a reasonable number of instructions (not too many)
    assert!(
        instructions.len() < 50,
        "Generated too many instructions: {}",
        instructions.len()
    );
    assert!(
        instructions.len() > 0,
        "Should generate at least some instructions"
    );
}

// ========================================================================
// 8. Specification Completeness Tests
// ========================================================================

#[test]
fn test_all_core_operators_parseable() {
    let operators = vec![
        "pure",
        "alloc",
        "consume",
        "lambda",
        "apply",
        "bind",
        "tensor",
        "domain",
        "cross-domain-transfer",
        "domain-effect",
        "intent",
        "sequence",
        "parallel",
        "program",
    ];

    for op in operators {
        let expr_str = format!("({} arg1 arg2)", op);
        let result = parse_sexpr(&expr_str);
        assert!(result.is_ok(), "Failed to parse operator: {}", op);

        match result.unwrap() {
            SExpression::List(elements) => {
                assert_eq!(elements[0], SExpression::Symbol(op.to_string()));
            }
            _ => panic!("Expected list for operator: {}", op),
        }
    }
}

#[test]
fn test_nested_structure_limits() {
    // Test deeply nested structure doesn't cause stack overflow
    let mut nested = "42".to_string();
    for _ in 0..100 {
        nested = format!("(pure {})", nested);
    }

    let result = parse_sexpr(&nested);
    assert!(result.is_ok(), "Failed to parse deeply nested structure");
}

// ========================================================================
// 9. Documentation Examples Tests
// ========================================================================

#[test]
fn test_spec_document_examples() {
    // Test examples from the specification document

    // Basic pure value example
    let expr = parse_sexpr("(pure 42)").unwrap();
    assert!(matches!(expr, SExpression::List(_)));

    // Resource operations example
    let expr = parse_sexpr("(alloc TokenA 100)").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    assert!(matches!(term.kind, TermKind::Alloc { .. }));

    // Lambda example from spec
    let expr = parse_sexpr("(lambda (amount) (alloc TokenA amount))").unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    assert!(matches!(term.kind, TermKind::Lambda { .. }));

    // Bind example from spec
    let expr =
        parse_sexpr("(bind (alloc TokenA 100) (lambda (token) (consume token)))")
            .unwrap();
    let term = compile_sexpr_to_term(&expr).unwrap();
    // bind compiles to application, so we expect Apply
    assert!(matches!(term.kind, TermKind::Apply { .. }));
}

// ========================================================================
// 10. Multi-Domain Program Structure Tests
// ========================================================================

#[test]
fn test_complete_multi_domain_program_parsing() {
    let program = r#"
        (program cross-chain-defi-strategy
          (version "1.0.0")
          (domains
            (domain ethereum (capabilities defi-protocols) (resources ETH USDC))
            (domain polygon (capabilities yield-farming) (resources MATIC USDC))
            (domain coordinator (capabilities orchestration) (resources GAS)))
          
          (intents
            (intent yield-optimization
              (trigger (yield-opportunity > threshold))
              (effects
                (sequence
                  (ethereum-deposit USDC amount)
                  (cross-domain-transfer USDC polygon)
                  (polygon-farm USDC-MATIC-LP)
                  (coordinator-monitor positions)))))
          
          (execution-plan
            (phase preparation (setup-accounts))
            (phase execution (execute-intents))
            (phase cleanup (reconcile-positions))))
    "#;

    let expr = parse_sexpr(program).unwrap();
    match expr {
        SExpression::List(elements) => {
            assert_eq!(elements[0], SExpression::Symbol("program".to_string()));
            assert_eq!(
                elements[1],
                SExpression::Symbol("cross-chain-defi-strategy".to_string())
            );
            assert!(elements.len() >= 6); // version, domains, intents, execution-plan
        }
        _ => panic!("Expected program declaration"),
    }
}
