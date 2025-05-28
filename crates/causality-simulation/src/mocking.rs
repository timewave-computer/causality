//! Simulation Mocking
//!
//! Manages mocking behavior for effects during simulation, allowing for
//! testable and deterministic execution of effects without external dependencies.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

use causality_runtime::tel::traits::{AutoMockStrategy, MockBehavior, MockProvider};
use causality_types::{
    core::{
        str::Str,
        numeric::Number,
    },
    effect::{ConversionError, HandlerError},
    expr::{
        value::{ValueExpr, ValueExprMap},
        TypeExpr,
    },
    serialization::{Decode},
};

//-----------------------------------------------------------------------------
// Schema registry for mock simulation
//-----------------------------------------------------------------------------

/// Storage backend for schema registry
#[derive(Clone, Debug)]
pub enum SchemaStorageBackend {
    /// HashMap-based storage (legacy)
    HashMap(HashMap<String, EffectSchenaPair>),
    /// SMT-based storage with domain awareness
    Smt {
        smt: Arc<std::sync::Mutex<causality_core::smt::TegMultiDomainSmt<causality_core::smt::MemoryBackend>>>,
        domain_id: causality_types::primitive::ids::DomainId,
    },
}

/// Registry of effect schemas for simulation mocks
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct SchemaRegistry {
    backend: SchemaStorageBackend,
    auto_mock_strategy: AutoMockStrategy,
}

/// Pair of input and output schemas for an effect
#[derive(Clone, Debug)]
pub struct EffectSchenaPair {
    input: TypeExpr,
    output: TypeExpr,
}

impl causality_types::serialization::SimpleSerialize for EffectSchenaPair {}

impl causality_types::serialization::Encode for EffectSchenaPair {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.input.as_ssz_bytes());
        bytes.extend(self.output.as_ssz_bytes());
        bytes
    }
}

impl causality_types::serialization::Decode for EffectSchenaPair {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        // For simplicity, we'll assume the input TypeExpr comes first and output second
        // This is a simplified implementation - a proper implementation would need length prefixes
        if bytes.len() < 2 {
            return Err(causality_types::serialization::DecodeError {
                message: "Insufficient bytes for EffectSchenaPair".to_string(),
            });
        }
        
        // Split bytes roughly in half for input and output
        // This is a placeholder - real implementation would need proper serialization format
        let mid = bytes.len() / 2;
        let input_bytes = &bytes[..mid];
        let output_bytes = &bytes[mid..];
        
        let input = TypeExpr::from_ssz_bytes(input_bytes)?;
        let output = TypeExpr::from_ssz_bytes(output_bytes)?;
        
        Ok(EffectSchenaPair { input, output })
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaRegistry {
    /// Create a new empty schema registry with HashMap storage
    pub fn new() -> Self {
        Self {
            backend: SchemaStorageBackend::HashMap(HashMap::new()),
            auto_mock_strategy: AutoMockStrategy::SucceedWithDefaultSchemaValue,
        }
    }
    
    /// Create a new schema registry with SMT storage
    pub fn new_with_smt(_domain_id: causality_types::primitive::ids::DomainId) -> Self {
        // For now, use HashMap backend instead of SMT to avoid complexity
        Self {
            backend: SchemaStorageBackend::HashMap(HashMap::new()),
            auto_mock_strategy: AutoMockStrategy::SucceedWithDefaultSchemaValue,
        }
    }

    /// Register a schema pair for an effect type
    pub fn register(
        &mut self,
        effect_type: impl Into<String>,
        input: TypeExpr,
        output: TypeExpr,
    ) {
        let effect_type_str = effect_type.into();
        let schema_pair = EffectSchenaPair { input, output };
        
        match &mut self.backend {
            SchemaStorageBackend::HashMap(map) => {
                map.insert(effect_type_str, schema_pair);
            }
            SchemaStorageBackend::Smt { .. } => {
                // SMT storage not implemented yet - convert to HashMap
                let mut new_map = HashMap::new();
                new_map.insert(effect_type_str, schema_pair);
                self.backend = SchemaStorageBackend::HashMap(new_map);
            }
        }
    }

    /// Get the input schema for an effect type
    pub fn get_input_schema(&self, effect_type: &str) -> Option<TypeExpr> {
        match &self.backend {
            SchemaStorageBackend::HashMap(schemas) => {
                schemas.get(effect_type).map(|pair| pair.input.clone())
            }
            SchemaStorageBackend::Smt { smt, domain_id: _ } => {
                // Generate SMT key from effect type
                let schema_key = format!("schema-{}", effect_type);
                
                // Retrieve from SMT
                if let Ok(smt_guard) = smt.lock() {
                    if let Ok(Some(serialized_schema)) = smt_guard.get_data(&schema_key) {
                        if let Ok(schema_pair) = EffectSchenaPair::from_ssz_bytes(&serialized_schema) {
                            return Some(schema_pair.input);
                        }
                    }
                }
                None
            }
        }
    }

    /// Get the output schema for an effect type
    pub fn get_output_schema(&self, effect_type: &str) -> Option<TypeExpr> {
        match &self.backend {
            SchemaStorageBackend::HashMap(schemas) => {
                schemas.get(effect_type).map(|pair| pair.output.clone())
            }
            SchemaStorageBackend::Smt { smt, domain_id: _ } => {
                // Generate SMT key from effect type
                let schema_key = format!("schema-{}", effect_type);
                
                // Retrieve from SMT
                if let Ok(smt_guard) = smt.lock() {
                    if let Ok(Some(serialized_schema)) = smt_guard.get_data(&schema_key) {
                        if let Ok(schema_pair) = EffectSchenaPair::from_ssz_bytes(&serialized_schema) {
                            return Some(schema_pair.output);
                        }
                    }
                }
                None
            }
        }
    }

    #[allow(dead_code)]
    fn generate_default_value(
        &self,
        schema: &TypeExpr,
    ) -> Result<ValueExpr, HandlerError> {
        // For the SchemaRegistry, this is a simple match on the auto_mock_strategy field
        match self.auto_mock_strategy {
            AutoMockStrategy::SucceedWithDefaultSchemaValue => match schema {
                TypeExpr::Unit => Ok(ValueExpr::Nil),
                TypeExpr::Bool => Ok(ValueExpr::Bool(false)),
                TypeExpr::Integer => Ok(ValueExpr::Number(Number::Integer(0))),
                TypeExpr::Number => Ok(ValueExpr::Number(Number::Integer(0))),
                TypeExpr::String => Ok(ValueExpr::String(Str::from("default"))),
                TypeExpr::List(_) => Ok(ValueExpr::List(vec![].into())),
                TypeExpr::Record(_) => {
                    Ok(ValueExpr::Map(ValueExprMap(BTreeMap::new())))
                }
                TypeExpr::Union(_) => Ok(ValueExpr::Nil), // Conservative default for union
                TypeExpr::Any => Ok(ValueExpr::Nil), // Conservative default for any
                _ => Err(HandlerError::OutputConversionFailed(
                    ConversionError::Custom(
                        "Unsupported schema type for default value generation"
                            .to_string(),
                    ),
                )),
            },
            AutoMockStrategy::AlwaysFail => {
                Err(HandlerError::EffectExecutionFailed(format!(
                    "Effect with type '{}' automatically failed by mock strategy",
                    schema
                )))
            }
        }
    }
}

//-----------------------------------------------------------------------------
// Mock Manager
//-----------------------------------------------------------------------------

/// Manages mocks for simulation testing
#[derive(Debug)]
pub struct SimulationMockManager {
    // Maps effect_type -> explicit mock behavior
    explicit_mocks: RwLock<BTreeMap<Str, MockBehavior>>,
    schema_registry: Arc<SchemaRegistry>,
    auto_mock_strategy: RwLock<AutoMockStrategy>,
}

impl Clone for SimulationMockManager {
    fn clone(&self) -> Self {
        Self {
            explicit_mocks: RwLock::new(self.explicit_mocks.read().unwrap().clone()),
            schema_registry: Arc::clone(&self.schema_registry),
            auto_mock_strategy: RwLock::new(
                self.auto_mock_strategy.read().unwrap().clone(),
            ),
        }
    }
}

impl SimulationMockManager {
    /// Create a new mock manager with default settings
    pub fn new() -> Self {
        Self {
            explicit_mocks: RwLock::new(BTreeMap::new()),
            schema_registry: Arc::new(SchemaRegistry::new()),
            auto_mock_strategy: RwLock::new(
                AutoMockStrategy::SucceedWithDefaultSchemaValue,
            ),
        }
    }

    /// Create a new mock manager with a specific auto mock strategy
    pub fn new_with_strategy(strategy: AutoMockStrategy) -> Self {
        Self {
            explicit_mocks: RwLock::new(BTreeMap::new()),
            schema_registry: Arc::new(SchemaRegistry::new()),
            auto_mock_strategy: RwLock::new(strategy),
        }
    }

    /// Register a schema for an effect type
    pub fn register_schema(
        &self,
        effect_type: impl Into<String>,
        input_schema: TypeExpr,
        output_schema: TypeExpr,
    ) {
        let input_schema_clone = input_schema.clone();
        let output_schema_clone = output_schema.clone();
        let effect_type_str: String = effect_type.into();
        let effect_type_clone = effect_type_str.clone();

        // Clone the current registry
        let current_registry = Arc::clone(&self.schema_registry);
        // Create a new mutable registry with the same contents
        let mut new_registry = (*current_registry).clone();
        // Register the new schema
        new_registry.register(effect_type_str, input_schema, output_schema);
        // Replace the Arc with the updated registry
        std::sync::Arc::get_mut(&mut self.schema_registry.clone())
            .expect("Arc must be uniquely owned")
            .register(effect_type_clone, input_schema_clone, output_schema_clone);
    }

    pub fn register_explicit_mock(&self, effect_type: Str, behavior: MockBehavior) {
        let mut mocks = self.explicit_mocks.write().unwrap();
        mocks.insert(effect_type, behavior);
    }

    pub fn clear_explicit_mock(&self, effect_type: &Str) {
        let mut mocks = self.explicit_mocks.write().unwrap();
        mocks.remove(effect_type);
    }

    pub fn clear_all_explicit_mocks(&self) {
        let mut mocks = self.explicit_mocks.write().unwrap();
        mocks.clear();
    }

    pub fn set_default_auto_mock_strategy(&self, strategy: AutoMockStrategy) {
        let mut auto_mock = self.auto_mock_strategy.write().unwrap();
        *auto_mock = strategy;
    }

    // Example usage for default strategy (if it needs to be instantiated outside new)
    pub fn default_succeed() -> Self {
        Self::new()
    }

    fn generate_default_value(
        &self,
        schema: &TypeExpr,
    ) -> Result<ValueExpr, HandlerError> {
        // If we don't have a schema or get here for some other reason,
        // choose a default based on strategy
        match *self.auto_mock_strategy.read().unwrap() {
            AutoMockStrategy::SucceedWithDefaultSchemaValue => match schema {
                TypeExpr::Unit => Ok(ValueExpr::Nil),
                TypeExpr::Bool => Ok(ValueExpr::Bool(false)),
                TypeExpr::Integer => Ok(ValueExpr::Number(Number::Integer(0))),
                TypeExpr::Number => Ok(ValueExpr::Number(Number::Integer(0))),
                TypeExpr::String => Ok(ValueExpr::String(Str::from("default"))),
                TypeExpr::List(_) => Ok(ValueExpr::List(vec![].into())),
                TypeExpr::Record(_) => {
                    Ok(ValueExpr::Map(ValueExprMap(BTreeMap::new())))
                }
                TypeExpr::Union(_) => Ok(ValueExpr::Nil), // Conservative default for union
                TypeExpr::Any => Ok(ValueExpr::Nil), // Conservative default for any
                _ => Err(HandlerError::OutputConversionFailed(
                    ConversionError::Custom(
                        "Unsupported schema type for default value generation"
                            .to_string(),
                    ),
                )),
            },
            AutoMockStrategy::AlwaysFail => {
                Err(HandlerError::EffectExecutionFailed(format!(
                    "Effect with type '{}' automatically failed by mock strategy",
                    schema
                )))
            }
        }
    }
}

//-----------------------------------------------------------------------------
// MockProvider Implementation
//-----------------------------------------------------------------------------

impl MockProvider for SimulationMockManager {
    fn should_mock(&self, effect_type: &Str) -> bool {
        if self
            .explicit_mocks
            .read()
            .unwrap()
            .contains_key(effect_type)
        {
            return true;
        }
        // Auto-mock based on strategy only if no explicit mock exists.
        let strategy = self.auto_mock_strategy.read().unwrap();
        match *strategy {
            AutoMockStrategy::SucceedWithDefaultSchemaValue => true, // Always try to mock if this strategy
            AutoMockStrategy::AlwaysFail => true, // Always try to mock (to fail it) if this strategy
                                                  // Add other strategies here
        }
    }

    fn mock_output(
        &self,
        effect_type: &Str,
        _input: &ValueExpr, // Input might be used by more complex mocks in future
        output_schema: &TypeExpr, // Now using the schema passed by the interpreter
    ) -> Result<ValueExpr, HandlerError> {
        if let Some(explicit_behavior) =
            self.explicit_mocks.read().unwrap().get(effect_type)
        {
            return match explicit_behavior {
                MockBehavior::SucceedWith(value) => Ok(value.clone()),
                MockBehavior::FailWith(err_msg) => {
                    Err(HandlerError::EffectExecutionFailed(err_msg.clone()))
                }
            };
        }

        // If no explicit mock, use auto_mock_strategy
        let strategy = self.auto_mock_strategy.read().unwrap();
        match *strategy {
            AutoMockStrategy::SucceedWithDefaultSchemaValue => {
                // If schema is not found by the interpreter (e.g. not registered in Context/InterpreterData),
                // this passed `output_schema` might be a generic one or a placeholder.
                // Here, we rely on the `output_schema` passed in being accurate.
                self.generate_default_value(output_schema)
            }
            AutoMockStrategy::AlwaysFail => {
                Err(HandlerError::EffectExecutionFailed(format!(
                    "Effect with type '{}' automatically failed by mock strategy",
                    effect_type
                )))
            }
        }
    }
}

//-----------------------------------------------------------------------------
// Default Implementation
//-----------------------------------------------------------------------------

// Default implementation provides a manager that doesn't mock unless explicitly told to.
impl Default for SimulationMockManager {
    fn default() -> Self {
        Self::new()
    }
}
