use serde_json;

use crate::effect::{Effect, EffectResult};
use crate::error::{Error, Result};
use crate::vm::zk_integration::{Proof, PublicInputs};
use super::types::{CosmWasmCallData, CosmWasmPublicInputs, Coin};

/// Effect for compiling Rust code to CosmWasm WASM with ZK verification
pub struct CompileEffect {
    /// Source code to compile
    pub source: String,
    
    /// Program identifier
    pub program_id: String,
}

impl CompileEffect {
    /// Create a new compile effect
    pub fn new(source: String, program_id: String) -> Self {
        Self {
            source,
            program_id,
        }
    }
    
    /// Create a compile effect from a generic effect
    pub fn from_effect(effect: &Effect) -> Result<Self> {
        if effect.name != "compile" {
            return Err(Error::InvalidInput(
                format!("Expected 'compile' effect, got '{}'", effect.name)
            ));
        }
        
        let source = effect.get_param_as_string("source")?;
        let program_id = effect.get_param_as_string("program_id")?;
        
        Ok(Self::new(source, program_id))
    }
    
    /// Convert to a generic effect
    pub fn to_effect(&self) -> Effect {
        let mut effect = Effect::new("compile");
        effect.add_param("source", self.source.clone());
        effect.add_param("program_id", self.program_id.clone());
        effect
    }
}

/// Effect for executing a CosmWasm contract
pub struct ExecuteContractEffect {
    /// Call data for the execution
    pub call_data: CosmWasmCallData,
}

impl ExecuteContractEffect {
    /// Create a new execute contract effect
    pub fn new(
        contract_address: String,
        method: String,
        inputs: String,
        funds: Option<Vec<Coin>>,
    ) -> Self {
        let call_data = CosmWasmCallData {
            contract_address,
            method,
            inputs,
            funds,
            options: Default::default(),
        };
        
        Self {
            call_data,
        }
    }
    
    /// Create an execute contract effect from a generic effect
    pub fn from_effect(effect: &Effect) -> Result<Self> {
        if effect.name != "execute_contract" {
            return Err(Error::InvalidInput(
                format!("Expected 'execute_contract' effect, got '{}'", effect.name)
            ));
        }
        
        let contract_address = effect.get_param_as_string("contract_address")?;
        let method = effect.get_param_as_string("method")?;
        let inputs = effect.get_param_as_string("inputs")?;
        
        // Parse funds if present
        let funds = if let Some(funds_str) = effect.get_param_as_string_option("funds") {
            let funds: Vec<Coin> = serde_json::from_str(&funds_str)
                .map_err(|e| Error::InvalidInput(
                    format!("Failed to parse funds: {}", e)
                ))?;
            Some(funds)
        } else {
            None
        };
        
        Ok(Self::new(contract_address, method, inputs, funds))
    }
    
    /// Convert to a generic effect
    pub fn to_effect(&self) -> Effect {
        let mut effect = Effect::new("execute_contract");
        effect.add_param("contract_address", self.call_data.contract_address.clone());
        effect.add_param("method", self.call_data.method.clone());
        effect.add_param("inputs", self.call_data.inputs.clone());
        
        if let Some(funds) = &self.call_data.funds {
            if let Ok(funds_str) = serde_json::to_string(funds) {
                effect.add_param("funds", funds_str);
            }
        }
        
        effect
    }
}

/// Effect for generating a ZK proof of correct contract execution
pub struct ProveEffect {
    /// Call data for the execution to prove
    pub call_data: CosmWasmCallData,
    
    /// Expected output of the execution (optional)
    pub expected_output: Option<String>,
}

impl ProveEffect {
    /// Create a new prove effect
    pub fn new(
        contract_address: String,
        method: String,
        inputs: String,
        expected_output: Option<String>,
        funds: Option<Vec<Coin>>,
    ) -> Self {
        let call_data = CosmWasmCallData {
            contract_address,
            method,
            inputs,
            funds,
            options: Default::default(),
        };
        
        Self {
            call_data,
            expected_output,
        }
    }
    
    /// Create a prove effect from a generic effect
    pub fn from_effect(effect: &Effect) -> Result<Self> {
        if effect.name != "prove" {
            return Err(Error::InvalidInput(
                format!("Expected 'prove' effect, got '{}'", effect.name)
            ));
        }
        
        let contract_address = effect.get_param_as_string("contract_address")?;
        let method = effect.get_param_as_string("method")?;
        let inputs = effect.get_param_as_string("inputs")?;
        let expected_output = effect.get_param_as_string_option("expected_output");
        
        // Parse funds if present
        let funds = if let Some(funds_str) = effect.get_param_as_string_option("funds") {
            let funds: Vec<Coin> = serde_json::from_str(&funds_str)
                .map_err(|e| Error::InvalidInput(
                    format!("Failed to parse funds: {}", e)
                ))?;
            Some(funds)
        } else {
            None
        };
        
        Ok(Self::new(contract_address, method, inputs, expected_output, funds))
    }
    
    /// Convert to a generic effect
    pub fn to_effect(&self) -> Effect {
        let mut effect = Effect::new("prove");
        effect.add_param("contract_address", self.call_data.contract_address.clone());
        effect.add_param("method", self.call_data.method.clone());
        effect.add_param("inputs", self.call_data.inputs.clone());
        
        if let Some(output) = &self.expected_output {
            effect.add_param("expected_output", output.clone());
        }
        
        if let Some(funds) = &self.call_data.funds {
            if let Ok(funds_str) = serde_json::to_string(funds) {
                effect.add_param("funds", funds_str);
            }
        }
        
        effect
    }
}

/// Effect for verifying a ZK proof of correct contract execution
pub struct VerifyEffect {
    /// The proof to verify
    pub proof: Proof,
    
    /// Public inputs for verification
    pub public_inputs: CosmWasmPublicInputs,
}

impl VerifyEffect {
    /// Create a new verify effect
    pub fn new(
        proof: Proof,
        public_inputs: CosmWasmPublicInputs,
    ) -> Self {
        Self {
            proof,
            public_inputs,
        }
    }
    
    /// Create a verify effect from a generic effect
    pub fn from_effect(effect: &Effect) -> Result<Self> {
        if effect.name != "verify" {
            return Err(Error::InvalidInput(
                format!("Expected 'verify' effect, got '{}'", effect.name)
            ));
        }
        
        // In a real implementation, we would deserialize the proof and public inputs
        // from the effect parameters
        
        // For now, we'll create mock objects
        let proof = Proof {
            data: vec![1, 2, 3, 4, 5],
            verification_key: vec![0, 1, 2, 3, 4, 5],
        };
        
        let contract_address = effect.get_param_as_string("contract_address")?;
        let method = effect.get_param_as_string("method")?;
        let chain_id = effect.get_param_as_string_option("chain_id")
            .unwrap_or_else(|| "default_chain".to_string());
        let inputs = effect.get_param_as_string("inputs")?;
        let expected_output = effect.get_param_as_string_option("expected_output");
        
        let mut public_inputs = CosmWasmPublicInputs::new(
            contract_address,
            method,
            chain_id,
            inputs,
        );
        
        if let Some(output) = expected_output {
            public_inputs = public_inputs.with_expected_output(output);
        }
        
        Ok(Self::new(proof, public_inputs))
    }
    
    /// Convert to a generic effect
    pub fn to_effect(&self) -> Effect {
        let mut effect = Effect::new("verify");
        
        // In a real implementation, we would serialize the proof and public inputs
        // to the effect parameters
        
        effect.add_param("contract_address", self.public_inputs.contract_address.clone());
        effect.add_param("method", self.public_inputs.method.clone());
        effect.add_param("chain_id", self.public_inputs.chain_id.clone());
        effect.add_param("inputs", self.public_inputs.inputs.clone());
        
        if let Some(output) = &self.public_inputs.expected_output {
            effect.add_param("expected_output", output.clone());
        }
        
        // Add additional data
        for (key, value) in &self.public_inputs.additional_data {
            effect.add_param(format!("additional_{}", key), value.clone());
        }
        
        effect
    }
} 