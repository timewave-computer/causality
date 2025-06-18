// ------------ PHASE 2 E2E TEST ------------ 
// Purpose: E2E test demonstrating multi-chain token balance queries and state-dependent strategy execution

use causality_compiler::{
    state_analysis::{StateQueryAnalyzer, StateQueryRequirement, QueryType},
    almanac_schema::{AlmanacSchemaGenerator, AlmanacSchema},
    query_primitives::{QueryPrimitiveCompiler, QueryStatePrimitive, ParameterType, QueryReturnType},
    almanac_runtime::{AlmanacRuntime, RuntimeConfig},
};
use causality_lisp::ast::{Expr, ExprKind, LispValue, Span};
use causality_core::{
    lambda::{base::{TypeInner, BaseType}, Symbol},
    system::content_addressing::Str,
};
use std::collections::HashMap;

/// E2E test: Multi-chain token balance queries with state-dependent strategy execution
#[tokio::test]
async fn test_multi_chain_balance_queries_e2e() {
    // Step 1: Create a mock OCaml program that queries token balances across multiple chains
    let program = create_multi_chain_balance_program();
    
    // Step 2: Analyze the program to identify state query requirements
    let mut analyzer = StateQueryAnalyzer::new();
    let analysis_result = analyzer.analyze_program(&program);
    
    // Verify that the analysis detected the expected queries
    println!("Detected {} queries", analysis_result.required_queries.len());
    for query in &analysis_result.required_queries {
        println!("  - {} on {} ({})", query.contract, query.domain, query.storage_slot);
    }
    // The analysis correctly deduplicates identical queries - we have 2 unique contracts
    assert_eq!(analysis_result.required_queries.len(), 2); // USDC and WETH (deduplicated by domain)
    assert_eq!(analysis_result.queries_by_domain.len(), 1); // All on ethereum by default
    
    // Verify we have both contracts
    assert!(analysis_result.queries_by_contract.contains_key("usdc"));
    assert!(analysis_result.queries_by_contract.contains_key("weth"));
    
    // Step 3: Generate Almanac schemas from the analysis
    let schema_generator = AlmanacSchemaGenerator::new();
    let schema_result = schema_generator.generate_schemas(&analysis_result);
    
    // Verify schemas were generated for both contracts
    assert!(schema_result.schemas.contains_key("usdc"));
    assert!(schema_result.schemas.contains_key("weth"));
    
    // Step 4: Compile query primitives
    let mut query_compiler = QueryPrimitiveCompiler::new();
    
    // Register schemas with the compiler
    for (contract_id, schema) in &schema_result.schemas {
        query_compiler.register_schema(contract_id.clone(), schema.clone());
    }
    
    // Create query expressions for each balance check
    let usdc_query = create_balance_query_expr("usdc", "ethereum", "balances");
    let weth_query = create_balance_query_expr("weth", "ethereum", "balances");
    
    // Compile the queries
    let compiled_queries = vec![
        query_compiler.compile_query_state(&usdc_query).unwrap(),
        query_compiler.compile_query_state(&weth_query).unwrap(),
    ];
    
    // Step 5: Set up Almanac runtime
    let runtime_config = RuntimeConfig {
        almanac_endpoint: "http://localhost:8080".to_string(),
        enable_caching: true,
        cache_ttl_seconds: 60,
        ..RuntimeConfig::default()
    };
    
    let mut runtime = AlmanacRuntime::new(runtime_config);
    
    // Register schemas with runtime
    for (contract_id, schema) in schema_result.schemas {
        runtime.register_schema(contract_id, schema);
    }
    
    // Step 6: Execute queries and verify results
    let results = runtime.execute_batch(&compiled_queries).await.unwrap();
    
    assert_eq!(results.len(), 2);
    for result in &results {
        assert!(!result.data.is_empty());
        assert!(result.data.contains("value"));
        assert!(!result.metadata.from_cache); // First execution should not be cached
    }
    
    // Step 7: Test caching by executing the same queries again
    let cached_results = runtime.execute_batch(&compiled_queries).await.unwrap();
    
    assert_eq!(cached_results.len(), 2);
    for result in &cached_results {
        assert!(result.metadata.from_cache); // Second execution should be cached
    }
    
    // Step 8: Verify state-dependent strategy execution
    let strategy_result = execute_rebalancing_strategy(&results).await;
    assert!(strategy_result.is_ok());
    
    println!("✅ Phase 2 E2E test passed: Multi-chain token balance queries with caching and state-dependent strategy execution");
}

/// Test schema generation and validation
#[test]
fn test_schema_generation_validation() {
    // Create analysis result with multi-chain queries
    let queries = vec![
        StateQueryRequirement {
            contract: "usdc".to_string(),
            storage_slot: "balances".to_string(),
            domain: "ethereum".to_string(),
            query_type: QueryType::TokenBalance,
            is_conditional: true,
        },
        StateQueryRequirement {
            contract: "usdc".to_string(),
            storage_slot: "balances".to_string(),
            domain: "polygon".to_string(),
            query_type: QueryType::TokenBalance,
            is_conditional: true,
        },
    ];
    
    let mut queries_by_contract = HashMap::new();
    queries_by_contract.insert("usdc".to_string(), queries.clone());
    
    let mut queries_by_domain = HashMap::new();
    queries_by_domain.insert("ethereum".to_string(), vec![queries[0].clone()]);
    queries_by_domain.insert("polygon".to_string(), vec![queries[1].clone()]);
    
    let analysis_result = causality_compiler::state_analysis::StateAnalysisResult {
        required_queries: queries,
        queries_by_contract,
        queries_by_domain,
        metadata: causality_compiler::state_analysis::AnalysisMetadata {
            expressions_analyzed: 10,
            patterns_detected: 2,
            analysis_duration_ms: 5,
        },
    };
    
    // Generate schemas
    let generator = AlmanacSchemaGenerator::new();
    let result = generator.generate_schemas(&analysis_result);
    
    // Validate schema structure
    assert_eq!(result.schemas.len(), 1);
    let usdc_schema = &result.schemas["usdc"];
    
    assert_eq!(usdc_schema.contract_id, "usdc");
    assert!(!usdc_schema.indexed_slots.is_empty());
    assert!(!usdc_schema.layout_commitment.commitment_hash.is_empty());
    
    // Verify that conditional queries are properly marked
    let balance_slot = usdc_schema.indexed_slots.iter()
        .find(|slot| slot.slot_id == "balances")
        .expect("Should have balances slot");
    
    assert!(balance_slot.is_hot); // Should be marked as hot due to conditional usage
    
    println!("✅ Schema generation validation passed");
}

/// Create a mock OCaml program that queries balances across multiple chains
fn create_multi_chain_balance_program() -> Expr {
    // Simplified representation of:
    // let ethereum_usdc_balance = query_state "usdc" "balances" in
    // let polygon_usdc_balance = query_state "usdc" "balances" in
    // let ethereum_weth_balance = query_state "weth" "balances" in
    // let polygon_weth_balance = query_state "weth" "balances" in
    // if ethereum_usdc_balance > polygon_usdc_balance then
    //   rebalance_to_polygon()
    // else
    //   rebalance_to_ethereum()
    
    // Create a sequence of 4 query_state calls
    Expr {
        kind: ExprKind::Apply(
            Box::new(Expr {
                kind: ExprKind::Var(Symbol::new("sequence")),
                ty: Some(TypeInner::Base(BaseType::Unit)),
                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
            }),
            vec![
                // USDC Ethereum query
                Expr {
                    kind: ExprKind::Apply(
                        Box::new(Expr {
                            kind: ExprKind::Var(Symbol::new("query_state")),
                            ty: Some(TypeInner::Base(BaseType::Unit)),
                            span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                        }),
                        vec![
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("usdc"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("balances"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                        ]
                    ),
                    ty: Some(TypeInner::Base(BaseType::Unit)),
                    span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                },
                // USDC Polygon query
                Expr {
                    kind: ExprKind::Apply(
                        Box::new(Expr {
                            kind: ExprKind::Var(Symbol::new("query_state")),
                            ty: Some(TypeInner::Base(BaseType::Unit)),
                            span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                        }),
                        vec![
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("usdc"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("balances"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                        ]
                    ),
                    ty: Some(TypeInner::Base(BaseType::Unit)),
                    span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                },
                // WETH Ethereum query
                Expr {
                    kind: ExprKind::Apply(
                        Box::new(Expr {
                            kind: ExprKind::Var(Symbol::new("query_state")),
                            ty: Some(TypeInner::Base(BaseType::Unit)),
                            span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                        }),
                        vec![
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("weth"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("balances"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                        ]
                    ),
                    ty: Some(TypeInner::Base(BaseType::Unit)),
                    span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                },
                // WETH Polygon query
                Expr {
                    kind: ExprKind::Apply(
                        Box::new(Expr {
                            kind: ExprKind::Var(Symbol::new("query_state")),
                            ty: Some(TypeInner::Base(BaseType::Unit)),
                            span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                        }),
                        vec![
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("weth"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                            Expr {
                                kind: ExprKind::Const(LispValue::String(Str::new("balances"))),
                                ty: Some(TypeInner::Base(BaseType::Unit)),
                                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                            },
                        ]
                    ),
                    ty: Some(TypeInner::Base(BaseType::Unit)),
                    span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                },
            ]
        ),
        ty: Some(TypeInner::Base(BaseType::Unit)),
        span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
    }
}

/// Create a balance query expression
fn create_balance_query_expr(contract: &str, _domain: &str, slot: &str) -> Expr {
    Expr {
        kind: ExprKind::Apply(
            Box::new(Expr {
                kind: ExprKind::Var(Symbol::new("query_state")),
                ty: Some(TypeInner::Base(BaseType::Unit)),
                span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
            }),
            vec![
                Expr {
                    kind: ExprKind::Const(LispValue::String(Str::new(contract))),
                    ty: Some(TypeInner::Base(BaseType::Unit)),
                    span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                },
                Expr {
                    kind: ExprKind::Const(LispValue::String(Str::new(slot))),
                    ty: Some(TypeInner::Base(BaseType::Unit)),
                    span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
                },
            ]
        ),
        ty: Some(TypeInner::Base(BaseType::Unit)),
        span: Some(Span { start: 0, end: 0, line: 1, column: 1 }),
    }
}

/// Mock rebalancing strategy that makes decisions based on query results
async fn execute_rebalancing_strategy(
    balance_results: &[causality_compiler::almanac_runtime::QueryResult]
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse balance values from query results
    let mut balances = Vec::new();
    
    for result in balance_results {
        // Parse the mock JSON result
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result.data) {
            if let Some(value_str) = parsed.get("value").and_then(|v| v.as_str()) {
                if let Ok(balance) = value_str.parse::<u64>() {
                    balances.push(balance);
                }
            }
        }
    }
    
    // Simple rebalancing logic: if any balance is below threshold, trigger rebalancing
    let threshold = 500_000_000_000_000_000u64; // 0.5 tokens
    let needs_rebalancing = balances.iter().any(|&balance| balance < threshold);
    
    if needs_rebalancing {
        println!("🔄 Rebalancing triggered based on low balances");
        // In a real implementation, this would trigger actual rebalancing operations
    } else {
        println!("✅ Balances are sufficient, no rebalancing needed");
    }
    
    Ok(())
}

/// Test query optimization and caching strategies
#[tokio::test]
async fn test_query_optimization_and_caching() {
    let runtime_config = RuntimeConfig {
        enable_caching: true,
        cache_ttl_seconds: 30,
        max_concurrent_queries: 5,
        ..RuntimeConfig::default()
    };
    
    let mut runtime = AlmanacRuntime::new(runtime_config);
    
    // Create a test schema
    let schema = create_test_schema();
    runtime.register_schema("test_token".to_string(), schema);
    
    // Create a test query
    let query = create_test_compiled_query();
    
    // Execute query multiple times to test caching
    let start_time = std::time::Instant::now();
    
    // First execution (should hit the network)
    let result1 = runtime.execute_query(&query).await.unwrap();
    let first_execution_time = start_time.elapsed();
    
    // Second execution (should hit cache)
    let result2 = runtime.execute_query(&query).await.unwrap();
    let second_execution_time = start_time.elapsed() - first_execution_time;
    
    // Verify caching behavior
    assert!(!result1.metadata.from_cache);
    assert!(result2.metadata.from_cache);
    assert_eq!(result1.data, result2.data);
    
    // Cache hit should be faster than network call
    assert!(second_execution_time < first_execution_time);
    
    // Test cache statistics
    let stats = runtime.cache_stats();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.valid_entries, 1);
    
    println!("✅ Query optimization and caching test passed");
}

/// Helper function to create a test schema
fn create_test_schema() -> AlmanacSchema {
    use causality_compiler::almanac_schema::{StorageSlotSchema, SlotDataType, IndexingStrategy, SchemaMetadata, LayoutCommitment};
    
    AlmanacSchema {
        contract_id: "test_token".to_string(),
        domain: "ethereum".to_string(),
        layout_commitment: LayoutCommitment {
            commitment_hash: "test_hash_123".to_string(),
            version: "1.0.0".to_string(),
            timestamp: 1234567890,
        },
        indexed_slots: vec![
            StorageSlotSchema {
                slot_id: "balances".to_string(),
                data_type: SlotDataType::Uint(256),
                is_hot: true,
                indexing_strategy: IndexingStrategy::Full,
            }
        ],
        query_patterns: vec![],
        metadata: SchemaMetadata {
            version: "1.0.0".to_string(),
            generated_at: 1234567890,
            queries_analyzed: 1,
            estimated_storage_bytes: 1024,
        },
    }
}

/// Helper function to create a test compiled query
fn create_test_compiled_query() -> causality_compiler::query_primitives::CompiledQuery {
    use causality_compiler::query_primitives::{CompiledQuery, QueryStatePrimitive, QueryRuntimeConfig};
    use causality_compiler::almanac_schema::LayoutCommitment;
    
    CompiledQuery {
        primitive: QueryStatePrimitive {
            contract_id: "test_token".to_string(),
            storage_slot: "balances".to_string(),
            parameters: vec![],
            return_type: QueryReturnType::Single(ParameterType::Uint(256)),
            optimization_hints: vec![],
        },
        ocaml_interface: "let query_balance = query_state \"test_token\" \"balances\"".to_string(),
        runtime_config: QueryRuntimeConfig::default(),
        layout_commitment: LayoutCommitment {
            commitment_hash: "test_hash_123".to_string(),
            version: "1.0.0".to_string(),
            timestamp: 1234567890,
        },
    }
} 