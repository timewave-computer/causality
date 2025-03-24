// Ethereum Virtual Machine ZK Effects
//
// This module implements ZK operations specific to the Ethereum Virtual Machine (EVM),
// allowing for the generation and verification of zero-knowledge proofs within
// Ethereum smart contracts.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use ethers::types::Address;
use borsh::{BorshSerialize, BorshDeserialize};
use std::any::Any;

use crate::domain::DomainId;
use crate::resource::ContentId;
use crate::effect::{
    Effect, EffectContext, EffectResult, EffectError, EffectOutcome,
    FactDependency, ExecutionBoundary, EffectId
};
use crate::fact::{Fact, FactId, FactSnapshot};
use crate::crypto::hash::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// Represents a RISC-V program for ZK operations
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RiscVProgram {
    /// Name of the program
    pub name: String,
    /// Target architecture
    pub target: String,
    /// Compiled bytecode
    pub code: Vec<u8>,
}

impl ContentAddressed for RiscVProgram {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Represents a witness for a ZK proof
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Witness {
    /// Program this witness is for
    pub program_name: String,
    /// Witness data
    pub data: Vec<u8>,
}

impl ContentAddressed for Witness {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Represents a generated ZK proof
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Proof {
    /// Program this proof is for
    pub program_name: String,
    /// Proof data
    pub data: Vec<u8>,
}

impl ContentAddressed for Proof {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Uniquely identifies an effect
#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct EvmZkCompileEffect {
    /// Unique identifier for this effect
    id: EffectId,
    /// The resource register ID
    resource_id: ContentId,
    /// Name of the program to compile
    pub name: String,
    /// Source code to compile
    pub code: Vec<u8>,
    /// Target architecture
    pub target: String,
    /// The domain ID
    pub domain_id: DomainId,
    /// The invoker address
    pub invoker_address: String,
    /// The contract address
    pub contract_address: Address,
    /// Fact dependencies
    fact_dependencies: Vec<FactDependency>,
    /// Fact snapshot
    fact_snapshot: Option<FactSnapshot>,
}

impl ContentAddressed for EvmZkCompileEffect {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl EvmZkCompileEffect {
    /// Create a new EVM ZK compilation effect
    pub fn new(
        resource_id: ContentId,
        name: String,
        code: Vec<u8>,
        target: String,
        domain_id: DomainId,
        invoker_address: String,
        contract_address: Address,
    ) -> Self {
        let mut effect = Self {
            id: EffectId::new_unique(), // Use EffectId's new_unique method
            resource_id,
            name,
            code,
            target,
            domain_id,
            invoker_address,
            contract_address,
            fact_dependencies: Vec::new(),
            fact_snapshot: None,
        };
        
        effect
    }
    
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(mut self, dependency: FactDependency) -> Self {
        self.fact_dependencies.push(dependency);
        self
    }
    
    /// Set the fact snapshot for this effect
    pub fn with_fact_snapshot(mut self, snapshot: FactSnapshot) -> Self {
        self.fact_snapshot = Some(snapshot);
        self
    }
}

impl Effect for EvmZkCompileEffect {
    fn id(&self) -> EffectId {
        self.id.clone()
    }
    
    fn name(&self) -> &str {
        "evm_zk_compile"
    }
    
    fn display_name(&self) -> String {
        "EVM ZK Program Compilation".to_string()
    }
    
    fn description(&self) -> String {
        "Compiles RISC-V program for ZK circuit generation on Ethereum".to_string()
    }
    
    fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        Err(EffectError::SynchronousExecutionNotSupported(self.name().to_string()))
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would compile RISC-V code for ZK circuit generation
        // For now, we return a simulated success outcome
        let mut outcome = EffectOutcome::success(self.id.clone());
        
        // Add data to the outcome
        outcome.add_data("program_name", self.name.clone());
        outcome.add_data("target", self.target.clone());
        outcome.add_data("contract_address", format!("{:?}", self.contract_address));
        outcome.add_data("domain_id", self.domain_id.to_string());
        
        // Add metadata
        outcome.add_metadata("code_size", format!("{} bytes", self.code.len()));
        outcome.add_metadata("invoker", self.invoker_address.clone());
        
        // Set fact snapshot if available
        if let Some(snapshot) = &self.fact_snapshot {
            outcome.set_fact_snapshot(Some(snapshot.clone()));
        }
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == ExecutionBoundary::External
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.fact_dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.fact_snapshot.clone()
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), self.name.clone());
        params.insert("target".to_string(), self.target.clone());
        params.insert("code_size".to_string(), format!("{} bytes", self.code.len()));
        params.insert("contract_address".to_string(), format!("{:?}", self.contract_address));
        params.insert("domain_id".to_string(), self.domain_id.to_string());
        params
    }
    
    fn validate_fact_dependencies(&self, facts: &HashMap<FactId, Arc<dyn Fact>>) -> EffectResult<()> {
        if self.fact_dependencies.is_empty() {
            return Ok(());
        }
        
        // Simple validation: ensure all required facts are present
        for dependency in &self.fact_dependencies {
            let found = facts.values().any(|fact| {
                fact.fact_type() == dependency.fact_type && 
                fact.parameters() == &dependency.parameters
            });
            
            if !found {
                return Err(EffectError::MissingFactDependency(dependency.fact_type.clone()));
            }
        }
        
        Ok(())
    }
}

/// EVM ZK witness generation effect
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct EvmZkWitnessEffect {
    /// Unique identifier for this effect
    id: EffectId,
    /// The resource register ID
    resource_id: ContentId,
    /// The compiled program to generate a witness for
    pub program: RiscVProgram,
    /// Public inputs to the program
    pub public_inputs: Vec<String>,
    /// Private inputs to the program
    pub private_inputs: Vec<String>,
    /// The domain ID
    pub domain_id: DomainId,
    /// The invoker address
    pub invoker_address: String,
    /// The contract address
    pub contract_address: Address,
    /// Fact dependencies
    fact_dependencies: Vec<FactDependency>,
    /// Fact snapshot
    fact_snapshot: Option<FactSnapshot>,
}

impl ContentAddressed for EvmZkWitnessEffect {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl EvmZkWitnessEffect {
    /// Create a new EVM ZK witness generation effect
    pub fn new(
        resource_id: ContentId,
        program: RiscVProgram,
        public_inputs: Vec<String>,
        private_inputs: Vec<String>,
        domain_id: DomainId,
        invoker_address: String,
        contract_address: Address,
    ) -> Self {
        let mut effect = Self {
            id: EffectId::new_unique(),
            resource_id,
            program,
            public_inputs,
            private_inputs,
            domain_id,
            invoker_address,
            contract_address,
            fact_dependencies: Vec::new(),
            fact_snapshot: None,
        };
        
        effect
    }
    
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(mut self, dependency: FactDependency) -> Self {
        self.fact_dependencies.push(dependency);
        self
    }
    
    /// Set the fact snapshot for this effect
    pub fn with_fact_snapshot(mut self, snapshot: FactSnapshot) -> Self {
        self.fact_snapshot = Some(snapshot);
        self
    }
}

#[async_trait]
impl Effect for EvmZkWitnessEffect {
    fn id(&self) -> EffectId {
        self.id.clone()
    }
    
    fn name(&self) -> &str {
        "evm_zk_witness"
    }
    
    fn display_name(&self) -> &str {
        "EVM ZK Witness Generation"
    }
    
    fn description(&self) -> &str {
        "Generates a witness for a ZK circuit on EVM"
    }
    
    fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        Err(EffectError::SynchronousExecutionNotSupported(self.name().to_string()))
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would interact with ZK tooling
        // For now, we return a simulated success outcome
        let outcome = EffectOutcome::success(self.id.clone())
            .with_data("program", self.program.name.clone())
            .with_data("public_inputs_count", self.public_inputs.len().to_string())
            .with_data("private_inputs_count", self.private_inputs.len().to_string())
            .with_data("contract_address", format!("{:?}", self.contract_address))
            .with_data("domain_id", self.domain_id.to_string());
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        matches!(boundary, ExecutionBoundary::EvmContract)
    }
    
    fn fact_dependencies(&self) -> &[FactDependency] {
        &self.fact_dependencies
    }
    
    fn fact_snapshot(&self) -> Option<&FactSnapshot> {
        self.fact_snapshot.as_ref()
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("program_name".to_string(), self.program.name.clone());
        params.insert("public_inputs".to_string(), format!("{} inputs", self.public_inputs.len()));
        params.insert("private_inputs".to_string(), format!("{} inputs", self.private_inputs.len()));
        params.insert("contract_address".to_string(), format!("{:?}", self.contract_address));
        params
    }
    
    fn validate_fact_dependencies(&self, facts: &HashMap<FactId, Arc<dyn Fact>>) -> EffectResult<()> {
        // Validate that all required facts are present
        for dependency in &self.fact_dependencies {
            if !facts.contains_key(&dependency.fact_id) {
                return Err(EffectError::MissingFactDependency(dependency.fact_id.clone()));
            }
        }
        
        Ok(())
    }
}

/// EVM ZK proof generation effect
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct EvmZkProveEffect {
    /// Unique identifier for this effect
    id: EffectId,
    /// The resource register ID
    resource_id: ContentId,
    /// The witness to generate a proof from
    pub witness: Witness,
    /// The domain ID
    pub domain_id: DomainId,
    /// The invoker address
    pub invoker_address: String,
    /// The contract address
    pub contract_address: Address,
    /// Fact dependencies
    fact_dependencies: Vec<FactDependency>,
    /// Fact snapshot
    fact_snapshot: Option<FactSnapshot>,
}

impl ContentAddressed for EvmZkProveEffect {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl EvmZkProveEffect {
    /// Create a new EVM ZK prove effect
    pub fn new(
        resource_id: ContentId,
        witness: Witness,
        domain_id: DomainId,
        invoker_address: String,
        contract_address: Address,
    ) -> Self {
        let mut effect = Self {
            id: EffectId::new_unique(),
            resource_id,
            witness,
            domain_id,
            invoker_address,
            contract_address,
            fact_dependencies: Vec::new(),
            fact_snapshot: None,
        };
        
        effect
    }
    
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(mut self, dependency: FactDependency) -> Self {
        self.fact_dependencies.push(dependency);
        self
    }
    
    /// Set the fact snapshot for this effect
    pub fn with_fact_snapshot(mut self, snapshot: FactSnapshot) -> Self {
        self.fact_snapshot = Some(snapshot);
        self
    }
}

#[async_trait]
impl Effect for EvmZkProveEffect {
    fn id(&self) -> EffectId {
        self.id.clone()
    }
    
    fn name(&self) -> &str {
        "evm_zk_prove"
    }
    
    fn display_name(&self) -> String {
        "EVM ZK Proof Generation".to_string()
    }
    
    fn description(&self) -> String {
        "Generates a zero-knowledge proof on Ethereum".to_string()
    }
    
    fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        Err(EffectError::SynchronousExecutionNotSupported(self.name().to_string()))
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would interact with ZK proving systems
        // For now, we return a simulated success outcome
        let mut outcome = EffectOutcome::success(self.id.clone());
        
        // Add data to the outcome
        outcome.add_data("program_name", self.witness.program_name.clone());
        outcome.add_data("contract_address", format!("{:?}", self.contract_address));
        outcome.add_data("domain_id", self.domain_id.to_string());
        
        // Add metadata
        outcome.add_metadata("witness_size", format!("{} bytes", self.witness.data.len()));
        outcome.add_metadata("invoker", self.invoker_address.clone());
        
        // Set fact snapshot if available
        if let Some(snapshot) = &self.fact_snapshot {
            outcome.set_fact_snapshot(Some(snapshot.clone()));
        }
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == ExecutionBoundary::External
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.fact_dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.fact_snapshot.clone()
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("program_name".to_string(), self.witness.program_name.clone());
        params.insert("contract_address".to_string(), format!("{:?}", self.contract_address));
        params.insert("domain_id".to_string(), self.domain_id.to_string());
        params
    }
    
    fn validate_fact_dependencies(&self, facts: &HashMap<FactId, Arc<dyn Fact>>) -> EffectResult<()> {
        if self.fact_dependencies.is_empty() {
            return Ok(());
        }
        
        // Simple validation: ensure all required facts are present
        for dependency in &self.fact_dependencies {
            let found = facts.values().any(|fact| {
                fact.fact_type() == dependency.fact_type && 
                fact.parameters() == &dependency.parameters
            });
            
            if !found {
                return Err(EffectError::MissingFactDependency(dependency.fact_type.clone()));
            }
        }
        
        Ok(())
    }
}

/// EVM ZK proof verification effect
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct EvmZkVerifyEffect {
    /// Unique identifier for this effect
    id: EffectId,
    /// The resource register ID
    resource_id: ContentId,
    /// The proof to verify
    pub proof: Proof,
    /// Public inputs to the verification
    pub public_inputs: Vec<String>,
    /// The domain ID
    pub domain_id: DomainId,
    /// The invoker address
    pub invoker_address: String,
    /// The contract address
    pub contract_address: Address,
    /// Fact dependencies
    fact_dependencies: Vec<FactDependency>,
    /// Fact snapshot
    fact_snapshot: Option<FactSnapshot>,
}

impl ContentAddressed for EvmZkVerifyEffect {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl EvmZkVerifyEffect {
    /// Create a new EVM ZK verify effect
    pub fn new(
        resource_id: ContentId,
        proof: Proof,
        public_inputs: Vec<String>,
        domain_id: DomainId,
        invoker_address: String,
        contract_address: Address,
    ) -> Self {
        let mut effect = Self {
            id: EffectId::new_unique(),
            resource_id,
            proof,
            public_inputs,
            domain_id,
            invoker_address,
            contract_address,
            fact_dependencies: Vec::new(),
            fact_snapshot: None,
        };
        
        effect
    }
    
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(mut self, dependency: FactDependency) -> Self {
        self.fact_dependencies.push(dependency);
        self
    }
    
    /// Set the fact snapshot for this effect
    pub fn with_fact_snapshot(mut self, snapshot: FactSnapshot) -> Self {
        self.fact_snapshot = Some(snapshot);
        self
    }
}

#[async_trait]
impl Effect for EvmZkVerifyEffect {
    fn id(&self) -> EffectId {
        self.id.clone()
    }
    
    fn name(&self) -> &str {
        "evm_zk_verify"
    }
    
    fn display_name(&self) -> String {
        "EVM ZK Proof Verification".to_string()
    }
    
    fn description(&self) -> String {
        "Verifies a zero-knowledge proof on Ethereum".to_string()
    }
    
    fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        Err(EffectError::SynchronousExecutionNotSupported(self.name().to_string()))
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would interact with ZK verification systems on Ethereum
        // For now, we return a simulated success outcome
        let mut outcome = EffectOutcome::success(self.id.clone());
        
        // Add data to the outcome
        outcome.add_data("program_name", self.proof.program_name.clone());
        outcome.add_data("contract_address", format!("{:?}", self.contract_address));
        outcome.add_data("domain_id", self.domain_id.to_string());
        outcome.add_data("verification_result", "valid");
        
        // Add metadata
        outcome.add_metadata("proof_size", format!("{} bytes", self.proof.data.len()));
        outcome.add_metadata("invoker", self.invoker_address.clone());
        outcome.add_metadata("public_inputs_count", self.public_inputs.len().to_string());
        
        // Set fact snapshot if available
        if let Some(snapshot) = &self.fact_snapshot {
            outcome.set_fact_snapshot(Some(snapshot.clone()));
        }
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == ExecutionBoundary::External
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.fact_dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.fact_snapshot.clone()
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("program_name".to_string(), self.proof.program_name.clone());
        params.insert("contract_address".to_string(), format!("{:?}", self.contract_address));
        params.insert("domain_id".to_string(), self.domain_id.to_string());
        params.insert("public_inputs".to_string(), format!("{} inputs", self.public_inputs.len()));
        params
    }
    
    fn validate_fact_dependencies(&self, facts: &HashMap<FactId, Arc<dyn Fact>>) -> EffectResult<()> {
        if self.fact_dependencies.is_empty() {
            return Ok(());
        }
        
        // Simple validation: ensure all required facts are present
        for dependency in &self.fact_dependencies {
            let found = facts.values().any(|fact| {
                fact.fact_type() == dependency.fact_type && 
                fact.parameters() == &dependency.parameters
            });
            
            if !found {
                return Err(EffectError::MissingFactDependency(dependency.fact_type.clone()));
            }
        }
        
        Ok(())
    }
}

/// Factory for creating EVM ZK effects
pub struct EvmZkEffectFactory {
    /// Domain ID for the effects
    domain_id: DomainId,
    /// Contract address for the effects
    contract_address: Address,
    /// Invoker address for the effects
    invoker_address: String,
}

impl EvmZkEffectFactory {
    /// Create a new EVM ZK effect factory
    pub fn new(domain_id: DomainId, contract_address: Address, invoker_address: String) -> Self {
        Self {
            domain_id,
            contract_address,
            invoker_address,
        }
    }
    
    /// Create a new EVM ZK compile effect
    pub fn create_compile_effect(
        &self,
        resource_id: ContentId,
        name: String,
        code: Vec<u8>,
        target: String,
    ) -> EvmZkCompileEffect {
        EvmZkCompileEffect::new(
            resource_id,
            name,
            code,
            target,
            self.domain_id.clone(),
            self.invoker_address.clone(),
            self.contract_address,
        )
    }
    
    /// Create a new EVM ZK witness generation effect
    pub fn create_witness_effect(
        &self,
        resource_id: ContentId,
        program: RiscVProgram,
        public_inputs: Vec<String>,
        private_inputs: Vec<String>,
    ) -> EvmZkWitnessEffect {
        EvmZkWitnessEffect::new(
            resource_id,
            program,
            public_inputs,
            private_inputs,
            self.domain_id.clone(),
            self.invoker_address.clone(),
            self.contract_address,
        )
    }
    
    /// Create a new EVM ZK proof generation effect
    pub fn create_prove_effect(
        &self,
        resource_id: ContentId,
        witness: Witness,
    ) -> EvmZkProveEffect {
        EvmZkProveEffect::new(
            resource_id,
            witness,
            self.domain_id.clone(),
            self.invoker_address.clone(),
            self.contract_address,
        )
    }
    
    /// Create a new EVM ZK proof verification effect
    pub fn create_verify_effect(
        &self,
        resource_id: ContentId,
        proof: Proof,
        public_inputs: Vec<String>,
    ) -> EvmZkVerifyEffect {
        EvmZkVerifyEffect::new(
            resource_id,
            proof,
            public_inputs,
            self.domain_id.clone(),
            self.invoker_address.clone(),
            self.contract_address,
        )
    }
} 
