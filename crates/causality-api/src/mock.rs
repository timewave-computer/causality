//! Mock client implementation for testing
//!
//! This module provides a fully functional mock implementation of blockchain
//! clients, transaction handlers, and intent registries for testing without
//! actual blockchain dependencies. All implementations maintain consistent
//! interfaces with real implementations.

//-----------------------------------------------------------------------------
// Imports and Dependencie
//-----------------------------------------------------------------------------

use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
// Import Digest trait to enable Sha256::new()
use sha2::Digest;
// use chrono::{DateTime, Utc};
// use futures::stream::{self, BoxStream};
// use serde::Serialize;
// use tracing::debug;

use crate::models::{
    IntentMetadata, IntentQueryInput, IntentQueryOutput, IntentStatus,
    IntentSubmissionInput, IntentSubmissionOutput,
};
use crate::traits::{ChainConfig, Query, Transaction};
use causality_types::primitive::ids::IntentId;
use causality_types::core::Intent;
use causality_types::serialization::Encode;
// use causality_types::trace::TraceEntry;
// use causality_types::primitive::ids::ResourceId;
// use causality_types::trace::CausalityTrace;
// use causality_types::resource::Resource;

//-----------------------------------------------------------------------------
// Mock State
//-----------------------------------------------------------------------------

/// Represents blockchain state that can be queried and modified by transactions
pub trait MockState: Clone + Debug + Send + Sync + 'static {
    /// State identifier type
    type Key: Clone + Debug + Eq + std::hash::Hash + Send + Sync + 'static;

    /// State value type
    type Value: Clone + Debug + Send + Sync + 'static;

    /// Get a value from the state
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    /// Set a value in the state
    fn set(&mut self, key: Self::Key, value: Self::Value);
}

/// Simple implementation of MockState using a HashMap
#[derive(Debug, Clone, Default)]
pub struct HashMapState<K, V>
where
    K: Clone + Debug + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    pub storage: HashMap<K, V>,
}

impl<K, V> MockState for HashMapState<K, V>
where
    K: Clone + Debug + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    type Key = K;
    type Value = V;

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.storage.get(key).cloned()
    }

    fn set(&mut self, key: Self::Key, value: Self::Value) {
        self.storage.insert(key, value);
    }
}

//-----------------------------------------------------------------------------
// State Transition
//-----------------------------------------------------------------------------

/// Represents a state transition triggered by a transaction
pub trait StateTransition<S: MockState, I> {
    /// Apply the state transition
    fn apply(&self, state: &mut S, input: &I) -> Result<()>;
}

/// Simple state transition that sets values in the state
pub struct SimpleStateTransition<S: MockState, I> {
    pub key_fn: Box<dyn Fn(&I) -> S::Key + Send + Sync>,
    pub value_fn: Box<dyn Fn(&I) -> S::Value + Send + Sync>,
}

impl<S: MockState, I> SimpleStateTransition<S, I> {
    /// Create a new simple state transition with a fixed value function
    pub fn new<KF, VF>(key_fn: KF, value_fn: VF) -> Self
    where
        KF: Fn(&I) -> S::Key + Send + Sync + 'static,
        VF: Fn(&I) -> S::Value + Send + Sync + 'static,
    {
        Self {
            key_fn: Box::new(key_fn),
            value_fn: Box::new(value_fn),
        }
    }
}

impl<S: MockState, I> StateTransition<S, I> for SimpleStateTransition<S, I> {
    fn apply(&self, state: &mut S, input: &I) -> Result<()> {
        let key = (self.key_fn)(input);
        let value = (self.value_fn)(input);
        state.set(key, value);
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Mock Transaction Handler
//-----------------------------------------------------------------------------

/// Handler for mock transactions
pub struct MockTransactionHandler<S, I, O>
where
    S: MockState,
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    pub output_fn: Box<dyn Fn(&I) -> O + Send + Sync>,
    pub transitions: Vec<Box<dyn StateTransition<S, I> + Send + Sync>>,
    pub validation_fn: Option<Box<dyn Fn(&I) -> Result<()> + Send + Sync>>,
}

impl<S, I, O> MockTransactionHandler<S, I, O>
where
    S: MockState,
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    pub fn new<F>(output_fn: F) -> Self
    where
        F: Fn(&I) -> O + Send + Sync + 'static,
    {
        Self {
            output_fn: Box::new(output_fn),
            transitions: Vec::new(),
            validation_fn: None,
        }
    }

    pub fn with_transition<T: StateTransition<S, I> + Send + Sync + 'static>(
        mut self,
        transition: T,
    ) -> Self {
        self.transitions.push(Box::new(transition));
        self
    }

    pub fn with_validation<F>(mut self, validation_fn: F) -> Self
    where
        F: Fn(&I) -> Result<()> + Send + Sync + 'static,
    {
        self.validation_fn = Some(Box::new(validation_fn));
        self
    }

    pub fn handle(&self, state: &mut S, input: &I) -> Result<O> {
        // Run validation if provided
        if let Some(validation) = &self.validation_fn {
            validation(input)?;
        }

        // Apply all state transitions
        for transition in &self.transitions {
            transition.apply(state, input)?;
        }

        // Generate output
        Ok((self.output_fn)(input))
    }
}

//-----------------------------------------------------------------------------
// Mock Query Handler
//-----------------------------------------------------------------------------

/// Handler for mock queries
pub struct MockQueryHandler<S, I, O>
where
    S: MockState,
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    pub handler_fn: Box<dyn Fn(&S, &I) -> Result<O> + Send + Sync>,
}

impl<S, I, O> MockQueryHandler<S, I, O>
where
    S: MockState,
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    pub fn new<F>(handler_fn: F) -> Self
    where
        F: Fn(&S, &I) -> Result<O> + Send + Sync + 'static,
    {
        Self {
            handler_fn: Box::new(handler_fn),
        }
    }

    pub fn handle(&self, state: &S, input: &I) -> Result<O> {
        (self.handler_fn)(state, input)
    }
}

//-----------------------------------------------------------------------------
// Mock Client
//-----------------------------------------------------------------------------

/// Configuration for mock client
#[derive(Debug, Clone)]
pub struct MockConfig {
    pub chain_name: String,
    pub chain_id: String,
    pub chain_type: String,
}

impl ChainConfig for MockConfig {
    const CHAIN_NAME: &'static str = "mock";
    const CHAIN_ID: &'static str = "mock-1";
    const DEFAULT_RPC_PORT: &'static str = "8888";
    const CHAIN_TYPE: &'static str = "mock";
}

/// A client that mocks blockchain interactions
#[derive(Clone)]
pub struct MockClient<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    pub config: MockConfig,
    pub state: Arc<Mutex<S>>,
    pub tx_handlers: Arc<
        Mutex<
            HashMap<String, Box<dyn Fn(&S, &TxIn) -> Result<TxOut> + Send + Sync>>,
        >,
    >,
    pub query_handlers: Arc<
        Mutex<HashMap<String, Box<dyn Fn(&S, &QIn) -> Result<QOut> + Send + Sync>>>,
    >,
    pub intent_registry: Arc<MockIntentRegistry>,
    _marker: PhantomData<(TxIn, TxOut, QIn, QOut)>,
}

impl<S, TxIn, TxOut, QIn, QOut> MockClient<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    pub fn new(config: MockConfig, initial_state: S) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(initial_state)),
            tx_handlers: Arc::new(Mutex::new(HashMap::new())),
            query_handlers: Arc::new(Mutex::new(HashMap::new())),
            intent_registry: Arc::new(MockIntentRegistry::new()),
            _marker: PhantomData,
        }
    }

    pub fn register_tx_handler(
        &mut self,
        name: &str,
        handler: Box<dyn Fn(&S, &TxIn) -> Result<TxOut> + Send + Sync>,
    ) {
        self.tx_handlers
            .lock()
            .unwrap()
            .insert(name.to_string(), handler);
    }

    pub fn register_query_handler(
        &mut self,
        name: &str,
        handler: Box<dyn Fn(&S, &QIn) -> Result<QOut> + Send + Sync>,
    ) {
        self.query_handlers
            .lock()
            .unwrap()
            .insert(name.to_string(), handler);
    }

    /// Get a reference to the intent registry
    pub fn intent_registry(&self) -> Arc<MockIntentRegistry> {
        self.intent_registry.clone()
    }

    pub fn chain_name(&self) -> &str {
        &self.config.chain_name
    }

    pub fn chain_id(&self) -> &str {
        &self.config.chain_id
    }

    pub fn chain_type(&self) -> &str {
        &self.config.chain_type
    }

    /// Execute a query using the registered query handlers
    /// This is a convenience method for tests that maintains compatibility
    /// with the previous API
    pub async fn query(&self, query: (String, QIn)) -> Result<QOut> {
        self.execute_query(query).await
    }
}

#[async_trait]
impl<S, TxIn, TxOut, QIn, QOut> Transaction for MockClient<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    type Input = (String, TxIn); // (handler_name, transaction_input)
    type Output = TxOut;

    async fn submit_transaction(
        &self,
        tx: <Self as Transaction>::Input,
    ) -> Result<<Self as Transaction>::Output> {
        let (handler_name, tx_input) = tx;

        // Lock the handlers mutex and find the handler
        let handlers = self.tx_handlers.lock().unwrap();
        let handler = handlers.get(&handler_name).ok_or_else(|| {
            anyhow!("No transaction handler found for name: {}", handler_name)
        })?;

        // Execute the transaction against the current state
        let state = self.state.lock().unwrap();
        handler(&state, &tx_input)
    }
}

#[async_trait]
impl<S, TxIn, TxOut, QIn, QOut> Query for MockClient<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    type Input = (String, QIn);
    type Output = QOut;

    async fn execute_query(
        &self,
        query: <Self as Query>::Input,
    ) -> Result<<Self as Query>::Output> {
        // Extract the handler name and query input from the tuple
        let (handler_name, query_input) = query;

        // Look up the handler
        let handlers = self.query_handlers.lock().unwrap();
        let handler = handlers.get(&handler_name).ok_or_else(|| {
            anyhow!("No query handler found for name: {}", handler_name)
        })?;

        // Execute the query against the current state
        let state = self.state.lock().unwrap();
        handler(&state, &query_input)
    }
}

#[async_trait]
impl<S, TxIn, TxOut, QIn, QOut> IntentQuery for MockClient<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    async fn query_intent(
        &self,
        input: IntentQueryInput,
    ) -> Result<IntentQueryOutput> {
        // Delegate to the intent registry
        self.intent_registry.query_intent(input).await
    }
}

#[async_trait]
impl<S, TxIn, TxOut, QIn, QOut> IntentSubmission
    for MockClient<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    async fn submit_intent(
        &self,
        input: IntentSubmissionInput,
    ) -> Result<IntentSubmissionOutput> {
        // Delegate to the intent registry
        self.intent_registry.submit_intent(input).await
    }
}

//-----------------------------------------------------------------------------
// Intent Mock Implementation
//-----------------------------------------------------------------------------

use crate::traits::{IntentQuery, IntentSubmission};

/// Registry for storing and retrieving intents in a mock blockchain

#[derive(Debug)]
pub struct MockIntentRegistry {
    intents: Arc<Mutex<HashMap<IntentId, (Intent, IntentMetadata)>>>,
    current_block: Arc<Mutex<u64>>,
    current_timestamp: Arc<Mutex<u64>>,
}

impl Default for MockIntentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MockIntentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            intents: Arc::new(Mutex::new(HashMap::new())),
            current_block: Arc::new(Mutex::new(1)),
            current_timestamp: Arc::new(Mutex::new(1683837645)), // Fixed timestamp for determinism
        }
    }

    /// Register a new intent
    pub fn register_intent(&self, intent: Intent) -> Result<IntentSubmissionOutput> {
        // In a real implementation, we would compute the ID properly
        // For now, we'll use a simple hash of the serialized intent
        let serialized = intent.as_ssz_bytes();

        // Create a deterministic ID from the serialized bytes
        let mut hasher = sha2::Sha256::new();
        hasher.update(&serialized);
        let hash = hasher.finalize();
        // Create an IntentId from the hash
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(hash.as_slice());
        let intent_id = IntentId::new(id_bytes);

        let block = {
            let mut block_guard = self.current_block.lock().unwrap();
            *block_guard += 1;
            *block_guard
        };

        let timestamp = {
            let mut ts_guard = self.current_timestamp.lock().unwrap();
            *ts_guard += 15; // 15 seconds per block
            *ts_guard
        };

        let metadata = IntentMetadata {
            block_height: block,
            timestamp,
            tx_hash: format!(
                "mock_tx_{:x}",
                hash[0..8]
                    .iter()
                    .fold(0u64, |acc, &x| (acc << 8) | x as u64)
            ),
            status: IntentStatus::Pending,
        };

        let output = IntentSubmissionOutput {
            _intent_id: intent_id,
            tx_hash: metadata.tx_hash.clone(),
            block_height: metadata.block_height,
            fees_paid: None,
            status: metadata.status.clone(),
        };

        self.intents
            .lock()
            .unwrap()
            .insert(intent_id, (intent, metadata));

        Ok(output)
    }

    /// Get an intent by ID
    pub fn get_intent(
        &self,
        intent_id: &IntentId,
    ) -> Option<(Intent, IntentMetadata)> {
        self.intents.lock().unwrap().get(intent_id).cloned()
    }

    /// Update the status of an intent
    pub fn update_intent_status(
        &self,
        intent_id: &IntentId,
        status: IntentStatus,
    ) -> Result<()> {
        let mut intents = self.intents.lock().unwrap();

        if let Some((_, metadata)) = intents.get_mut(intent_id) {
            metadata.status = status;
            Ok(())
        } else {
            Err(anyhow!("Intent not found: {:?}", intent_id))
        }
    }

    /// Get the current block height
    pub fn current_block(&self) -> u64 {
        *self.current_block.lock().unwrap()
    }

    /// Advance the block height
    pub fn advance_block(&self, blocks: u64) {
        let mut block_guard = self.current_block.lock().unwrap();
        *block_guard += blocks;

        let mut ts_guard = self.current_timestamp.lock().unwrap();
        *ts_guard += blocks * 15; // 15 seconds per block
    }
}

#[async_trait]
impl IntentQuery for MockIntentRegistry {
    async fn query_intent(
        &self,
        input: IntentQueryInput,
    ) -> Result<IntentQueryOutput> {
        match self.get_intent(&input._intent_id) {
            Some((intent, metadata)) => Ok(IntentQueryOutput {
                intent: Some(intent),
                metadata,
            }),
            None => Ok(IntentQueryOutput {
                intent: None,
                metadata: IntentMetadata {
                    block_height: self.current_block(),
                    timestamp: *self.current_timestamp.lock().unwrap(),
                    tx_hash: String::new(),
                    status: IntentStatus::Rejected,
                },
            }),
        }
    }
}

#[async_trait]
impl IntentSubmission for MockIntentRegistry {
    async fn submit_intent(
        &self,
        input: IntentSubmissionInput,
    ) -> Result<IntentSubmissionOutput> {
        self.register_intent(input.intent)
    }
}

//-----------------------------------------------------------------------------
// Mock Client Builder
//-----------------------------------------------------------------------------

/// Builder for mock clients

pub struct MockClientBuilder<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    pub initial_state: S,
    pub chain_name: String,
    pub chain_id: String,
    pub chain_type: String,
    pub tx_handlers:
        HashMap<String, Box<dyn Fn(&S, &TxIn) -> Result<TxOut> + Send + Sync>>,
    pub query_handlers:
        HashMap<String, Box<dyn Fn(&S, &QIn) -> Result<QOut> + Send + Sync>>,
}

impl<S, TxIn, TxOut, QIn, QOut> MockClientBuilder<S, TxIn, TxOut, QIn, QOut>
where
    S: MockState,
    TxIn: Clone + Send + Sync + 'static,
    TxOut: Clone + Send + Sync + 'static,
    QIn: Clone + Send + Sync + 'static,
    QOut: Clone + Send + Sync + 'static,
{
    pub fn new(initial_state: S) -> Self {
        Self {
            initial_state,
            chain_name: "mock".to_string(),
            chain_id: "mock-1".to_string(),
            chain_type: "mock".to_string(),
            tx_handlers: HashMap::new(),
            query_handlers: HashMap::new(),
        }
    }

    pub fn with_chain_name(mut self, name: &str) -> Self {
        self.chain_name = name.to_string();
        self
    }

    pub fn with_chain_id(mut self, id: &str) -> Self {
        self.chain_id = id.to_string();
        self
    }

    pub fn with_chain_type(mut self, chain_type: &str) -> Self {
        self.chain_type = chain_type.to_string();
        self
    }

    pub fn register_tx_handler(
        mut self,
        name: &str,
        handler: MockTransactionHandler<S, TxIn, TxOut>,
    ) -> Self {
        // Convert MockTransactionHandler to the simplified function signature
        let handler_fn = Box::new(move |state: &S, input: &TxIn| -> Result<TxOut> {
            // Apply validations
            if let Some(ref validation) = handler.validation_fn {
                validation(input)?
            }

            // Apply state transitions
            let mut state_clone = state.clone();
            for transition in &handler.transitions {
                transition.apply(&mut state_clone, input)?;
            }

            // Generate output
            Ok((handler.output_fn)(input))
        });

        self.tx_handlers.insert(name.to_string(), handler_fn);
        self
    }

    pub fn register_query_handler(
        mut self,
        name: &str,
        handler: MockQueryHandler<S, QIn, QOut>,
    ) -> Self {
        // Convert MockQueryHandler to the simplified function signature
        let handler_fn = Box::new(move |state: &S, input: &QIn| -> Result<QOut> {
            // Execute the query handler function
            (handler.handler_fn)(state, input)
        });

        self.query_handlers.insert(name.to_string(), handler_fn);
        self
    }

    pub fn build(self) -> MockClient<S, TxIn, TxOut, QIn, QOut> {
        // Create config from builder fields
        let config = MockConfig {
            chain_name: self.chain_name,
            chain_id: self.chain_id,
            chain_type: self.chain_type,
        };

        // Create the client with initialized handlers
        let mut tx_handlers_map = HashMap::new();
        let mut query_handlers_map = HashMap::new();

        // Transfer the handlers
        for (name, handler) in self.tx_handlers {
            tx_handlers_map.insert(name, handler);
        }

        for (name, handler) in self.query_handlers {
            query_handlers_map.insert(name, handler);
        }

        // Create the client with all the handlers already inserted
        MockClient {
            config,
            state: Arc::new(Mutex::new(self.initial_state)),
            tx_handlers: Arc::new(Mutex::new(tx_handlers_map)),
            query_handlers: Arc::new(Mutex::new(query_handlers_map)),
            intent_registry: Arc::new(MockIntentRegistry::new()),
            _marker: PhantomData,
        }
    }
}

//-----------------------------------------------------------------------------
// IntentMetadataRegistry Implementation
//-----------------------------------------------------------------------------

/// Simple type aliases for intent handler registry

pub type IntentRegistry = HashMap<IntentId, Intent>;
