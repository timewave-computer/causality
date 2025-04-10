//! CosmWasm Code Generator for TEG
//!
//! This module provides a code generator that translates a Temporal Effect Graph
//! into CosmWasm smart contract code.

use std::collections::HashMap;
use anyhow::{Result, anyhow};

use crate::TemporalEffectGraph;
use crate::EffectNode;
use crate::ResourceNode;
use super::{CodeGenConfig, GeneratedCode, CodeGenerator};
use super::target::{Target, BaseTarget, TargetCapabilities};

/// CosmWasm code generator for TEG
pub struct CosmWasmGenerator {
    /// Base target implementation
    base_target: BaseTarget,
}

impl CosmWasmGenerator {
    /// Create a new CosmWasm code generator
    pub fn new() -> Self {
        Self {
            base_target: BaseTarget::new(
                "cosmwasm",
                "CosmWasm smart contract generator for TEG",
                TargetCapabilities {
                    supports_concurrency: false,
                    supports_algebraic_effects: false,
                    supports_capabilities: true,
                    supported_resource_types: vec![
                        "basic".to_string(),
                        "token".to_string(),
                        "account".to_string(),
                        "state".to_string(),
                    ],
                    supported_effect_types: vec![
                        "basic".to_string(),
                        "state".to_string(),
                        "query".to_string(),
                        "bank".to_string(),
                    ],
                    additional_capabilities: {
                        let mut caps = HashMap::new();
                        caps.insert("ibc".to_string(), true);
                        caps.insert("cosmwasm_std".to_string(), true);
                        caps
                    },
                },
            ),
        }
    }
    
    /// Generate the Cargo.toml file for the CosmWasm project
    fn generate_cargo_toml(&self) -> String {
        r#"[package]
name = "cosmwasm-teg-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cosmwasm-std = "1.1.0"
cosmwasm-storage = "1.1.0"
cw-storage-plus = "1.0.0"
schemars = "0.8.10"
serde = { version = "1.0.145", default-features = false, features = ["derive"] }
thiserror = "1.0.31"

[dev-dependencies]
cosmwasm-schema = "1.1.0"
cw-multi-test = "0.16.0"
"#.to_string()
    }
    
    /// Generate the basic contract structure
    fn generate_contract_structure(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // Add module header
        code.push_str("#[cfg(not(feature = \"library\"))]\n");
        code.push_str("use cosmwasm_std::entry_point;\n");
        code.push_str("use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};\n");
        code.push_str("use cosmwasm_std::{Event, Coin, Uint128, StdError, Addr};\n");
        code.push_str("use serde::{Deserialize, Serialize};\n\n");
        
        // Contract errors
        code.push_str("use thiserror::Error;\n\n");
        code.push_str("#[derive(Error, Debug, PartialEq)]\n");
        code.push_str("pub enum ContractError {\n");
        code.push_str("    #[error(\"{0}\")]\n");
        code.push_str("    Std(#[from] StdError),\n\n");
        code.push_str("    #[error(\"Unauthorized\")]\n");
        code.push_str("    Unauthorized {},\n\n");
        code.push_str("    #[error(\"Resource not found: {id}\")]\n");
        code.push_str("    ResourceNotFound { id: String },\n\n");
        code.push_str("    #[error(\"Invalid operation: {msg}\")]\n");
        code.push_str("    InvalidOperation { msg: String },\n");
        code.push_str("}\n\n");
        
        // Contract state
        code.push_str("use cw_storage_plus::{Item, Map};\n\n");
        code.push_str("pub struct Contract {\n");
        
        // Add storage for each resource type
        let mut resource_types = std::collections::HashSet::new();
        for (_, resource) in teg.resources() {
            resource_types.insert(resource.resource_type.clone());
        }
        
        for resource_type in resource_types {
            let storage_name = format!("{}_storage", resource_type.to_lowercase());
            code.push_str(&format!("    pub {}: Map<String, {}>,\n", 
                storage_name, self.to_pascal_case(&resource_type)));
        }
        
        code.push_str("}\n\n");
        
        // Contract implementation
        code.push_str("impl Default for Contract {\n");
        code.push_str("    fn default() -> Self {\n");
        code.push_str("        Self {\n");
        
        for resource_type in resource_types {
            let storage_name = format!("{}_storage", resource_type.to_lowercase());
            code.push_str(&format!("            {}: Map::new(\"{}\"),\n", 
                storage_name, storage_name));
        }
        
        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");
        
        code
    }
    
    /// Generate message types
    fn generate_message_types(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // InstantiateMsg
        code.push_str("#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]\n");
        code.push_str("pub struct InstantiateMsg {\n");
        code.push_str("    pub admin: Option<String>,\n");
        code.push_str("}\n\n");
        
        // ExecuteMsg
        code.push_str("#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]\n");
        code.push_str("pub enum ExecuteMsg {\n");
        
        // Add variants for each effect
        for (_, effect) in teg.effects() {
            if effect.effect_type.starts_with("execute_") {
                let effect_name = effect.effect_type.replace("execute_", "");
                let pascal_name = self.to_pascal_case(&effect_name);
                
                code.push_str(&format!("    {} {{\n", pascal_name));
                
                // Add fields for each parameter
                for (param_name, param_value) in &effect.parameters {
                    let param_type = self.parameter_type_to_rust_type(param_value);
                    code.push_str(&format!("        {}: {},\n", param_name, param_type));
                }
                
                code.push_str("    },\n");
            }
        }
        
        code.push_str("}\n\n");
        
        // QueryMsg
        code.push_str("#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]\n");
        code.push_str("pub enum QueryMsg {\n");
        
        // Add variants for each effect that is a query
        for (_, effect) in teg.effects() {
            if effect.effect_type.starts_with("query_") {
                let effect_name = effect.effect_type.replace("query_", "");
                let pascal_name = self.to_pascal_case(&effect_name);
                
                code.push_str(&format!("    {} {{\n", pascal_name));
                
                // Add fields for each parameter
                for (param_name, param_value) in &effect.parameters {
                    let param_type = self.parameter_type_to_rust_type(param_value);
                    code.push_str(&format!("        {}: {},\n", param_name, param_type));
                }
                
                code.push_str("    },\n");
            }
        }
        
        code.push_str("}\n\n");
        
        // Response types for queries
        for (_, effect) in teg.effects() {
            if effect.effect_type.starts_with("query_") {
                let effect_name = effect.effect_type.replace("query_", "");
                let pascal_name = self.to_pascal_case(&effect_name);
                
                code.push_str(&format!("#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]\n"));
                code.push_str(&format!("pub struct {}Response {{\n", pascal_name));
                
                // Add fields based on expected return type
                code.push_str("    pub result: String, // Placeholder, adjust based on actual return type\n");
                
                code.push_str("}\n\n");
            }
        }
        
        code
    }
    
    /// Generate resource struct definitions
    fn generate_resource_structs(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // Collect unique resource types
        let mut resource_types = std::collections::HashMap::new();
        for (_, resource) in teg.resources() {
            resource_types.insert(resource.resource_type.clone(), resource.clone());
        }
        
        // Generate struct for each resource type
        for (type_name, resource) in resource_types {
            code.push_str(&format!("#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]\n"));
            code.push_str(&format!("pub struct {} {{\n", self.to_pascal_case(&type_name)));
            code.push_str("    pub id: String,\n");
            
            // Add fields based on sample resource state
            if let Ok(sample_state) = resource.state() {
                if let Some(obj) = sample_state.as_object() {
                    for (key, value) in obj {
                        if key != "id" {  // Skip id as it's already included
                            let field_type = self.json_value_to_rust_type(value);
                            code.push_str(&format!("    pub {}: {},\n", key, field_type));
                        }
                    }
                }
            }
            
            code.push_str("}\n\n");
        }
        
        code
    }
    
    /// Generate entry points
    fn generate_entry_points(&self) -> String {
        let mut code = String::new();
        
        // Instantiate entry point
        code.push_str("#[cfg_attr(not(feature = \"library\"), entry_point)]\n");
        code.push_str("pub fn instantiate(\n");
        code.push_str("    deps: DepsMut,\n");
        code.push_str("    env: Env,\n");
        code.push_str("    info: MessageInfo,\n");
        code.push_str("    msg: InstantiateMsg,\n");
        code.push_str(") -> Result<Response, ContractError> {\n");
        code.push_str("    // Initialize contract state\n");
        code.push_str("    let contract = Contract::default();\n\n");
        code.push_str("    // Store admin if provided\n");
        code.push_str("    if let Some(admin) = msg.admin {\n");
        code.push_str("        // Store admin address\n");
        code.push_str("    }\n\n");
        code.push_str("    Ok(Response::new()\n");
        code.push_str("        .add_attribute(\"method\", \"instantiate\")\n");
        code.push_str("        .add_attribute(\"owner\", info.sender))\n");
        code.push_str("}\n\n");
        
        // Execute entry point
        code.push_str("#[cfg_attr(not(feature = \"library\"), entry_point)]\n");
        code.push_str("pub fn execute(\n");
        code.push_str("    deps: DepsMut,\n");
        code.push_str("    env: Env,\n");
        code.push_str("    info: MessageInfo,\n");
        code.push_str("    msg: ExecuteMsg,\n");
        code.push_str(") -> Result<Response, ContractError> {\n");
        code.push_str("    match msg {\n");
        code.push_str("        // Match on ExecuteMsg variants\n");
        code.push_str("        // Placeholder for generated message handlers\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");
        
        // Query entry point
        code.push_str("#[cfg_attr(not(feature = \"library\"), entry_point)]\n");
        code.push_str("pub fn query(\n");
        code.push_str("    deps: Deps,\n");
        code.push_str("    env: Env,\n");
        code.push_str("    msg: QueryMsg,\n");
        code.push_str(") -> StdResult<Binary> {\n");
        code.push_str("    match msg {\n");
        code.push_str("        // Match on QueryMsg variants\n");
        code.push_str("        // Placeholder for generated query handlers\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");
        
        code
    }
    
    /// Generate execute message handlers
    fn generate_execute_handlers(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // Find execute effects
        for (_, effect) in teg.effects() {
            if effect.effect_type.starts_with("execute_") {
                let effect_name = effect.effect_type.replace("execute_", "");
                let fn_name = self.to_snake_case(&effect_name);
                let pascal_name = self.to_pascal_case(&effect_name);
                
                code.push_str(&format!("// Handler for {} operation\n", effect_name));
                code.push_str(&format!("fn {}(\n", fn_name));
                code.push_str("    deps: DepsMut,\n");
                code.push_str("    env: Env,\n");
                code.push_str("    info: MessageInfo,\n");
                
                // Add parameters
                for (param_name, param_value) in &effect.parameters {
                    let param_type = self.parameter_type_to_rust_type(param_value);
                    code.push_str(&format!("    {}: {},\n", param_name, param_type));
                }
                
                code.push_str(") -> Result<Response, ContractError> {\n");
                
                // Implement handler based on effect type
                if effect_name.contains("transfer") {
                    code.push_str("    // Transfer implementation\n");
                    code.push_str("    // Check permissions\n");
                    code.push_str("    // Verify resources exist\n");
                    code.push_str("    // Perform transfer operation\n");
                } else if effect_name.contains("create") {
                    code.push_str("    // Create resource implementation\n");
                    code.push_str("    // Validate input\n");
                    code.push_str("    // Create new resource\n");
                    code.push_str("    // Store in state\n");
                } else if effect_name.contains("update") {
                    code.push_str("    // Update resource implementation\n");
                    code.push_str("    // Check resource exists\n");
                    code.push_str("    // Validate update\n");
                    code.push_str("    // Apply changes\n");
                } else {
                    code.push_str("    // Generic implementation\n");
                    code.push_str("    // Perform necessary operations\n");
                }
                
                code.push_str("\n    Ok(Response::new()\n");
                code.push_str(&format!("        .add_attribute(\"action\", \"{}\")\n", effect_name));
                code.push_str("        .add_attribute(\"sender\", info.sender.to_string()))\n");
                code.push_str("}\n\n");
                
                // Add the match arm to execute function
                code.push_str(&format!("// Add to execute match:\n"));
                code.push_str(&format!("ExecuteMsg::{} {{ ", pascal_name));
                
                let param_list = effect.parameters.keys()
                    .map(|k| format!("{}", k))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                code.push_str(&format!("{} }} => {}(deps, env, info, {}),\n\n", 
                    param_list, fn_name, param_list));
            }
        }
        
        code
    }
    
    /// Generate query handlers
    fn generate_query_handlers(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // Find query effects
        for (_, effect) in teg.effects() {
            if effect.effect_type.starts_with("query_") {
                let effect_name = effect.effect_type.replace("query_", "");
                let fn_name = self.to_snake_case(&effect_name);
                let pascal_name = self.to_pascal_case(&effect_name);
                
                code.push_str(&format!("// Query handler for {}\n", effect_name));
                code.push_str(&format!("fn {}(\n", fn_name));
                code.push_str("    deps: Deps,\n");
                code.push_str("    env: Env,\n");
                
                // Add parameters
                for (param_name, param_value) in &effect.parameters {
                    let param_type = self.parameter_type_to_rust_type(param_value);
                    code.push_str(&format!("    {}: {},\n", param_name, param_type));
                }
                
                code.push_str(&format!(") -> StdResult<{}Response> {{\n", pascal_name));
                
                // Implement query based on effect name
                if effect_name.contains("get") {
                    code.push_str("    // Retrieve resource implementation\n");
                    code.push_str("    // Check if resource exists\n");
                    code.push_str("    // Return resource data\n");
                } else if effect_name.contains("list") {
                    code.push_str("    // List resources implementation\n");
                    code.push_str("    // Query state for resources\n");
                    code.push_str("    // Return list of resources\n");
                } else {
                    code.push_str("    // Generic query implementation\n");
                    code.push_str("    // Perform necessary query operations\n");
                }
                
                code.push_str("\n    Ok(");
                code.push_str(&format!("{}Response {{\n", pascal_name));
                code.push_str("        result: \"query result\".to_string(), // Replace with actual result\n");
                code.push_str("    })");
                code.push_str(")\n");
                code.push_str("}\n\n");
                
                // Add the match arm to query function
                code.push_str(&format!("// Add to query match:\n"));
                code.push_str(&format!("QueryMsg::{} {{ ", pascal_name ));
                
                let param_list = effect.parameters.keys()
                    .map(|k| format!("{}", k))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                code.push_str(&format!("{} }} => to_binary(&{}(deps, env, {})?),\n\n", 
                    param_list, fn_name, param_list));
            }
        }
        
        code
    }
    
    /// Convert a TEG parameter value to a Rust type
    fn parameter_type_to_rust_type(&self, _value: &crate::effect_node::ParameterValue) -> String {
        // For now, a very simplified conversion
        "String".to_string()
    }
    
    /// Convert a JSON value to a Rust type
    fn json_value_to_rust_type(&self, _value: &serde_json::Value) -> String {
        // For now, a very simplified conversion
        "String".to_string()
    }
    
    /// Generate test file
    fn generate_test_file(&self) -> String {
        r#"#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { admin: None };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    // Add more tests for your contract
}
"#.to_string()
    }
    
    /// Generate schema.rs file
    fn generate_schema_file(&self) -> String {
        r#"use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_schema::write_api;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
"#.to_string()
    }
    
    /// Helper function to convert string to PascalCase
    fn to_pascal_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut capitalize = true;
        
        for c in s.chars() {
            if c == '_' || c == '-' || c == ' ' {
                capitalize = true;
            } else if capitalize {
                result.push(c.to_ascii_uppercase());
                capitalize = false;
            } else {
                result.push(c);
            }
        }
        
        result
    }
    
    /// Helper function to convert string to snake_case
    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut prev_lower = false;
        
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 && prev_lower {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
                prev_lower = false;
            } else if c == ' ' || c == '-' {
                result.push('_');
                prev_lower = false;
            } else {
                result.push(c);
                prev_lower = c.is_lowercase();
            }
        }
        
        result
    }
}

impl Target for CosmWasmGenerator {
    fn name(&self) -> &str {
        "cosmwasm"
    }
    
    fn capabilities(&self) -> &TargetCapabilities {
        &self.base_target.capabilities
    }
    
    fn is_compatible(&self, teg: &TemporalEffectGraph) -> bool {
        // Check if all effect types are supported
        for (_, effect) in teg.effects() {
            // Simplistic check - in a real implementation, we'd have more sophisticated compatibility logic
            if effect.effect_type.contains("async") || effect.effect_type.contains("concurrent") {
                return false;
            }
        }
        
        true
    }
    
    fn preprocess(&self, _teg: &mut TemporalEffectGraph, _config: &CodeGenConfig) -> Result<()> {
        // No preprocessing needed for CosmWasm
        Ok(())
    }
    
    fn generate_code(&self, teg: &TemporalEffectGraph, _config: &CodeGenConfig) -> Result<GeneratedCode> {
        let mut main_output = String::new();
        
        // Generate contract structure
        main_output.push_str(&self.generate_contract_structure(teg));
        
        // Generate message types
        main_output.push_str(&self.generate_message_types(teg));
        
        // Generate resource structs
        main_output.push_str(&self.generate_resource_structs(teg));
        
        // Generate entry points
        main_output.push_str(&self.generate_entry_points());
        
        // Create additional files for a complete project
        let mut additional_files = HashMap::new();
        
        // Cargo.toml
        additional_files.insert("Cargo.toml".to_string(), self.generate_cargo_toml());
        
        // Tests
        additional_files.insert("src/tests.rs".to_string(), self.generate_test_file());
        
        // Schema generation
        additional_files.insert("src/schema.rs".to_string(), self.generate_schema_file());
        
        // Execute handlers
        let execute_handlers = self.generate_execute_handlers(teg);
        additional_files.insert("execute_handlers.rs".to_string(), execute_handlers);
        
        // Query handlers
        let query_handlers = self.generate_query_handlers(teg);
        additional_files.insert("query_handlers.rs".to_string(), query_handlers);
        
        // README
        let readme = format!(
            "# CosmWasm TEG Contract\n\nThis CosmWasm smart contract was automatically generated from a Temporal Effect Graph.\n\n## Building\n\n```\ncargo wasm\n```\n"
        );
        additional_files.insert("README.md".to_string(), readme);
        
        // Metadata
        let mut metadata = HashMap::new();
        metadata.insert("target".to_string(), "cosmwasm".to_string());
        metadata.insert("effect_count".to_string(), format!("{}", teg.effects().count()));
        metadata.insert("resource_count".to_string(), format!("{}", teg.resources().count()));
        
        Ok(GeneratedCode {
            main_output,
            language: "rust".to_string(),
            additional_files,
            metadata,
        })
    }
}

impl CodeGenerator for CosmWasmGenerator {
    fn name(&self) -> &str {
        "cosmwasm"
    }
    
    fn description(&self) -> &str {
        "Generates CosmWasm smart contracts from TEG"
    }
    
    fn generate(&self, teg: &TemporalEffectGraph, config: &CodeGenConfig) -> Result<GeneratedCode> {
        // Check if the TEG is compatible with CosmWasm
        if !self.is_compatible(teg) {
            return Err(anyhow!("TEG contains effects not compatible with CosmWasm"));
        }
        
        // Clone the TEG for preprocessing
        let mut teg_clone = teg.clone();
        
        // Preprocess
        self.preprocess(&mut teg_clone, config)?;
        
        // Generate code
        self.generate_code(&teg_clone, config)
    }
} 