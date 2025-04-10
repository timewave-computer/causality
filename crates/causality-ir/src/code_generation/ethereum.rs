//! Ethereum Solidity Code Generator for TEG
//!
//! This module provides a code generator that translates a Temporal Effect Graph
//! into Ethereum Solidity smart contracts.

use std::collections::HashMap;
use anyhow::{Result, anyhow};

use crate::TemporalEffectGraph;
use crate::EffectNode;
use crate::ResourceNode;
use super::{CodeGenConfig, GeneratedCode, CodeGenerator};
use super::target::{Target, BaseTarget, TargetCapabilities};

/// Ethereum Solidity code generator for TEG
pub struct EthereumGenerator {
    /// Base target implementation
    base_target: BaseTarget,
}

impl EthereumGenerator {
    /// Create a new Ethereum Solidity code generator
    pub fn new() -> Self {
        Self {
            base_target: BaseTarget::new(
                "ethereum",
                "Ethereum Solidity code generator for TEG",
                TargetCapabilities {
                    supports_concurrency: false,
                    supports_algebraic_effects: false,
                    supports_capabilities: false,
                    supported_resource_types: vec![
                        "basic".to_string(),
                        "state".to_string(),
                        "token".to_string(),
                    ],
                    supported_effect_types: vec![
                        "basic".to_string(),
                        "state".to_string(),
                        "transfer".to_string(),
                        "call".to_string(),
                    ],
                    additional_capabilities: {
                        let mut caps = HashMap::new();
                        caps.insert("gas_optimization".to_string(), true);
                        caps.insert("storage_optimization".to_string(), true);
                        caps
                    },
                },
            ),
        }
    }
    
    /// Generate Solidity contract header and pragma statement
    fn generate_contract_header(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // Add pragma and SPDX license
        code.push_str("// SPDX-License-Identifier: MIT\n");
        code.push_str("pragma solidity ^0.8.0;\n\n");
        
        // Add imports
        code.push_str("// Import OpenZeppelin contracts if needed\n");
        code.push_str("import \"@openzeppelin/contracts/access/Ownable.sol\";\n");
        code.push_str("import \"@openzeppelin/contracts/utils/math/SafeMath.sol\";\n\n");
        
        // Contract comment header
        code.push_str("/**\n");
        code.push_str(" * @title Generated from Temporal Effect Graph\n");
        code.push_str(" * @dev DO NOT EDIT - This contract is automatically generated\n");
        code.push_str(" */\n");
        
        code
    }
    
    /// Generate Solidity struct for a resource node
    fn generate_resource_struct(&self, resource: &ResourceNode) -> String {
        let mut code = String::new();
        
        // Generate a struct for the resource
        code.push_str(&format!("/**\n * @dev Resource: {}\n */\n", resource.id()));
        code.push_str(&format!("struct {} {{\n", self.to_pascal_case(resource.resource_type())));
        
        // Add resource fields based on state
        if let Ok(state) = resource.state().as_object() {
            for (key, value) in state {
                // Generate field based on value type
                if value.is_string() {
                    code.push_str(&format!("    string {};\n", key));
                } else if value.is_number() {
                    if value.is_i64() {
                        code.push_str(&format!("    int256 {};\n", key));
                    } else {
                        code.push_str(&format!("    uint256 {};\n", key));
                    }
                } else if value.is_boolean() {
                    code.push_str(&format!("    bool {};\n", key));
                } else if value.is_object() {
                    // Objects are mapped as mappings when possible, otherwise left as comments
                    code.push_str(&format!("    // Complex object {} requires custom mapping\n", key));
                } else if value.is_array() {
                    code.push_str(&format!("    // Array {} requires custom implementation\n", key));
                } else {
                    code.push_str(&format!("    bytes {}; // Generic data\n", key));
                }
            }
        }
        
        // Add an id field for resource tracking
        code.push_str("    bytes32 id; // Resource identifier\n");
        code.push_str("}\n\n");
        
        code
    }
    
    /// Generate Solidity mappings and state variables for resources
    fn generate_resource_storage(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        code.push_str("    // Resource storage mappings\n");
        
        // Create a mapping for each resource type
        let mut resource_types = std::collections::HashSet::new();
        for (_, resource) in teg.resources() {
            resource_types.insert(resource.resource_type().to_string());
        }
        
        for resource_type in resource_types {
            let struct_name = self.to_pascal_case(&resource_type);
            code.push_str(&format!("    mapping(bytes32 => {}) public {}Storage;\n", 
                struct_name, resource_type));
            code.push_str(&format!("    bytes32[] public {}Ids;\n", resource_type));
        }
        
        code.push_str("\n");
        code
    }
    
    /// Generate Solidity function for an effect node
    fn generate_effect_function(&self, effect: &EffectNode) -> String {
        let mut code = String::new();
        
        // Function visibility - determine if it should be external or internal
        let visibility = if effect.is_public() { "external" } else { "internal" };
        
        // Generate a function for the effect
        code.push_str(&format!("    /**\n     * @dev Effect: {}\n", effect.name()));
        if let Some(op_type) = effect.operation_type() {
            code.push_str(&format!("     * @dev Type: {}\n", op_type));
        }
        code.push_str("     */\n");
        
        // Function signature
        let function_name = self.to_camel_case(effect.name());
        code.push_str(&format!("    function {}(", function_name));
        
        // Add parameters based on the effect's parameters
        let params = effect.parameters();
        let mut param_list = Vec::new();
        for (name, param_type) in params {
            // Convert parameter type to Solidity type
            let solidity_type = self.tel_type_to_solidity(param_type);
            param_list.push(format!("{} {}", solidity_type, name));
        }
        code.push_str(&param_list.join(", "));
        
        // Return type based on effect's return type
        let return_type = if let Some(ret_type) = effect.return_type() {
            self.tel_type_to_solidity(ret_type)
        } else {
            "bool".to_string()
        };
        
        code.push_str(&format!(") {} returns ({}) {{\n", visibility, return_type));
        
        // Generate function body based on effect type
        if let Some(op_type) = effect.operation_type() {
            match op_type {
                "read" => {
                    // Read operation for a resource
                    let resource_param = params.iter().find(|(name, _)| *name == "resourceId");
                    if let Some((name, _)) = resource_param {
                        code.push_str(&format!("        // Read operation for resource\n"));
                        code.push_str(&format!("        bytes32 resourceId = {};\n", name));
                        code.push_str(&format!("        require(resourceExists(resourceId), \"Resource does not exist\");\n"));
                        
                        // If we know what resource type is being read, we can be more specific
                        if let Some(resource_type) = effect.metadata().get("resource_type") {
                            code.push_str(&format!("        return get{}(resourceId);\n", 
                                self.to_pascal_case(resource_type)));
                        } else {
                            code.push_str("        // Generic read operation\n");
                            code.push_str("        return true;\n");
                        }
                    } else {
                        code.push_str("        // Read operation with unknown resource\n");
                        code.push_str("        return true;\n");
                    }
                },
                "write" => {
                    // Write operation for a resource
                    let resource_param = params.iter().find(|(name, _)| *name == "resourceId");
                    if let Some((name, _)) = resource_param {
                        code.push_str(&format!("        // Write operation for resource\n"));
                        code.push_str(&format!("        bytes32 resourceId = {};\n", name));
                        
                        // If we know what resource type is being written, we can be more specific
                        if let Some(resource_type) = effect.metadata().get("resource_type") {
                            code.push_str(&format!("        return update{}(resourceId, {});\n", 
                                self.to_pascal_case(resource_type), 
                                params.iter().filter(|(param_name, _)| *param_name != "resourceId")
                                    .map(|(param_name, _)| param_name.clone())
                                    .collect::<Vec<_>>().join(", ")));
                        } else {
                            code.push_str("        // Generic write operation\n");
                            code.push_str("        return true;\n");
                        }
                    } else {
                        code.push_str("        // Write operation with unknown resource\n");
                        code.push_str("        return true;\n");
                    }
                },
                "transfer" => {
                    // Token transfer effect
                    code.push_str("        // Token transfer operation\n");
                    code.push_str("        address to;\n");
                    code.push_str("        uint256 amount;\n");
                    
                    // Get 'to' and 'amount' parameters if they exist
                    let to_param = params.iter().find(|(name, _)| *name == "to" || *name == "recipient");
                    let amount_param = params.iter().find(|(name, _)| *name == "amount" || *name == "value");
                    
                    if let Some((to_name, _)) = to_param {
                        code.push_str(&format!("        to = {};\n", to_name));
                    } else {
                        code.push_str("        // 'to' parameter not found\n");
                        code.push_str("        revert(\"Missing recipient parameter\");\n");
                    }
                    
                    if let Some((amount_name, _)) = amount_param {
                        code.push_str(&format!("        amount = {};\n", amount_name));
                    } else {
                        code.push_str("        // 'amount' parameter not found\n");
                        code.push_str("        revert(\"Missing amount parameter\");\n");
                    }
                    
                    code.push_str("        return executeTransfer(to, amount);\n");
                },
                "call" => {
                    // External contract call
                    code.push_str("        // External contract call\n");
                    code.push_str("        address target;\n");
                    code.push_str("        bytes memory data;\n");
                    
                    // Get 'target' and 'data' parameters if they exist
                    let target_param = params.iter().find(|(name, _)| *name == "target" || *name == "contract");
                    let data_param = params.iter().find(|(name, _)| *name == "data" || *name == "calldata");
                    
                    if let Some((target_name, _)) = target_param {
                        code.push_str(&format!("        target = {};\n", target_name));
                    } else {
                        code.push_str("        // 'target' parameter not found\n");
                        code.push_str("        revert(\"Missing target parameter\");\n");
                    }
                    
                    if let Some((data_name, _)) = data_param {
                        code.push_str(&format!("        data = {};\n", data_name));
                    } else {
                        code.push_str("        // 'data' parameter not found\n");
                        code.push_str("        revert(\"Missing calldata parameter\");\n");
                    }
                    
                    code.push_str("        return executeCall(target, data);\n");
                },
                _ => {
                    // Generic operation
                    code.push_str(&format!("        // Generic {} operation\n", op_type));
                    code.push_str("        // Implementation placeholder\n");
                    
                    // Return default value based on return type
                    match return_type.as_str() {
                        "bool" => code.push_str("        return true;\n"),
                        "uint256" | "int256" => code.push_str("        return 0;\n"),
                        "string memory" => code.push_str("        return \"\";\n"),
                        "address" => code.push_str("        return address(0);\n"),
                        "bytes memory" => code.push_str("        return \"\";\n"),
                        _ => code.push_str("        // Return value not implemented\n"),
                    }
                }
            }
        } else {
            // No specific operation type
            code.push_str("        // Generic effect with no specific operation type\n");
            
            // Return default value based on return type
            match return_type.as_str() {
                "bool" => code.push_str("        return true;\n"),
                "uint256" | "int256" => code.push_str("        return 0;\n"),
                "string memory" => code.push_str("        return \"\";\n"),
                "address" => code.push_str("        return address(0);\n"),
                "bytes memory" => code.push_str("        return \"\";\n"),
                _ => code.push_str("        // Return value not implemented\n"),
            }
        }
        
        code.push_str("    }\n\n");
        
        code
    }
    
    /// Generate utility functions for resource management
    fn generate_utility_functions(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // Resource existence check function
        code.push_str("    /**\n     * @dev Check if a resource exists\n     */\n");
        code.push_str("    function resourceExists(bytes32 resourceId) internal view returns (bool) {\n");
        
        // Check each resource type
        let mut resource_types = std::collections::HashSet::new();
        for (_, resource) in teg.resources() {
            resource_types.insert(resource.resource_type().to_string());
        }
        
        if resource_types.is_empty() {
            code.push_str("        return false; // No resources defined\n");
        } else {
            for resource_type in resource_types {
                code.push_str(&format!("        // Check {} resources\n", resource_type));
                code.push_str(&format!("        if ({}Storage[resourceId].id == resourceId) {{\n", resource_type));
                code.push_str("            return true;\n");
                code.push_str("        }\n");
            }
            code.push_str("        return false;\n");
        }
        
        code.push_str("    }\n\n");
        
        // Add getter and setter functions for each resource type
        for resource_type in resource_types {
            let pascal_type = self.to_pascal_case(&resource_type);
            
            // Getter function
            code.push_str(&format!("    /**\n     * @dev Get a {} resource\n     */\n", resource_type));
            code.push_str(&format!("    function get{}(bytes32 resourceId) internal view returns ({} memory) {{\n", 
                pascal_type, pascal_type));
            code.push_str(&format!("        require({}Storage[resourceId].id == resourceId, \"Resource not found\");\n", 
                resource_type));
            code.push_str(&format!("        return {}Storage[resourceId];\n", resource_type));
            code.push_str("    }\n\n");
            
            // Setter function
            code.push_str(&format!("    /**\n     * @dev Update a {} resource\n     */\n", resource_type));
            code.push_str(&format!("    function update{}(bytes32 resourceId, {} memory data) internal returns (bool) {{\n", 
                pascal_type, pascal_type));
            code.push_str("        bool isNew = false;\n");
            code.push_str(&format!("        if ({}Storage[resourceId].id != resourceId) {{\n", resource_type));
            code.push_str("            isNew = true;\n");
            code.push_str(&format!("            {}Ids.push(resourceId);\n", resource_type));
            code.push_str("        }\n");
            code.push_str("        data.id = resourceId;\n");
            code.push_str(&format!("        {}Storage[resourceId] = data;\n", resource_type));
            code.push_str("        return true;\n");
            code.push_str("    }\n\n");
        }
        
        // Transfer function for token operations
        code.push_str("    /**\n     * @dev Execute a token transfer\n     */\n");
        code.push_str("    function executeTransfer(address to, uint256 amount) internal returns (bool) {\n");
        code.push_str("        // Implementation would depend on the token standard being used\n");
        code.push_str("        // For example, with ERC20:\n");
        code.push_str("        // return token.transfer(to, amount);\n");
        code.push_str("        return true;\n");
        code.push_str("    }\n\n");
        
        // External call function
        code.push_str("    /**\n     * @dev Execute an external call\n     */\n");
        code.push_str("    function executeCall(address target, bytes memory data) internal returns (bool) {\n");
        code.push_str("        // Low-level call to an external contract\n");
        code.push_str("        (bool success, ) = target.call(data);\n");
        code.push_str("        return success;\n");
        code.push_str("    }\n\n");
        
        code
    }
    
    /// Generate main contract that ties everything together
    fn generate_main_contract(&self, teg: &TemporalEffectGraph) -> String {
        let mut code = String::new();
        
        // Start contract definition
        code.push_str("contract TemporalEffectGraph is Ownable {\n");
        code.push_str("    using SafeMath for uint256;\n\n");
        
        // Add libraries and using statements
        code.push_str("    // Events for tracking state changes\n");
        code.push_str("    event ResourceCreated(bytes32 indexed resourceId, string resourceType);\n");
        code.push_str("    event ResourceUpdated(bytes32 indexed resourceId, string resourceType);\n");
        code.push_str("    event EffectExecuted(string indexed effectName, bytes32 indexed resourceId);\n\n");
        
        // Add resource storage
        code.push_str(&self.generate_resource_storage(teg));
        
        // Add effect functions
        code.push_str("    // Effect functions\n");
        for (_, effect) in teg.effects() {
            code.push_str(&self.generate_effect_function(effect));
        }
        
        // Add utility functions
        code.push_str("    // Utility functions\n");
        code.push_str(&self.generate_utility_functions(teg));
        
        // Add constructor
        code.push_str("    /**\n     * @dev Constructor\n     */\n");
        code.push_str("    constructor() {\n");
        code.push_str("        // Initialize contract state\n");
        code.push_str("    }\n\n");
        
        // End contract definition
        code.push_str("}\n");
        
        code
    }
    
    /// Convert TEL type to Solidity type
    fn tel_type_to_solidity(&self, tel_type: &str) -> String {
        match tel_type {
            "String" => "string memory".to_string(),
            "Int" | "Integer" => "int256".to_string(),
            "UInt" | "UInteger" => "uint256".to_string(),
            "Float" | "Double" => "uint256".to_string(), // No floating point in Solidity
            "Boolean" | "Bool" => "bool".to_string(),
            "Address" => "address".to_string(),
            "Bytes" | "Binary" => "bytes memory".to_string(),
            _ if tel_type.starts_with("Array") => "bytes memory".to_string(), // Arrays need special handling
            _ if tel_type.starts_with("Map") => "bytes memory".to_string(),   // Maps need special handling
            _ => "bytes memory".to_string(), // Default for complex types
        }
    }
    
    /// Convert a string to PascalCase
    fn to_pascal_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }
    
    /// Convert a string to camelCase
    fn to_camel_case(&self, s: &str) -> String {
        let pascal = self.to_pascal_case(s);
        if pascal.is_empty() {
            pascal
        } else {
            let mut chars = pascal.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

impl Target for EthereumGenerator {
    fn name(&self) -> &str {
        self.base_target.name()
    }
    
    fn capabilities(&self) -> &TargetCapabilities {
        self.base_target.capabilities()
    }
    
    fn is_compatible(&self, teg: &TemporalEffectGraph) -> bool {
        // Check if all effect types in the TEG are supported by Ethereum
        for (_, effect) in teg.effects() {
            if let Some(op_type) = effect.operation_type() {
                if !self.capabilities().supported_effect_types.contains(&op_type.to_string()) {
                    return false;
                }
            }
        }
        
        // Check if all resource types in the TEG are supported by Ethereum
        for (_, resource) in teg.resources() {
            if !self.capabilities().supported_resource_types.contains(&resource.resource_type().to_string()) {
                return false;
            }
        }
        
        true
    }
    
    fn preprocess(&self, teg: &mut TemporalEffectGraph, _config: &CodeGenConfig) -> Result<()> {
        // Ethereum preprocessing may include:
        // - Converting unsupported resource types to supported ones
        // - Simplifying complex structures
        // - Adding gas estimation metadata
        Ok(())
    }
    
    fn generate_code(&self, teg: &TemporalEffectGraph, _config: &CodeGenConfig) -> Result<GeneratedCode> {
        // Generate the contract header
        let mut main_output = self.generate_contract_header(teg);
        
        // Generate resource structs
        main_output.push_str("// Resource struct definitions\n");
        for (_, resource) in teg.resources() {
            main_output.push_str(&self.generate_resource_struct(resource));
        }
        
        // Generate main contract
        main_output.push_str("// Main contract\n");
        main_output.push_str(&self.generate_main_contract(teg));
        
        // Create additional files as needed
        let mut additional_files = HashMap::new();
        
        // Add a package.json file for npm/truffle
        let package_json = r#"{
  "name": "teg-ethereum-generated",
  "version": "0.1.0",
  "description": "Generated Ethereum contracts from Temporal Effect Graph",
  "main": "truffle-config.js",
  "scripts": {
    "compile": "truffle compile",
    "test": "truffle test",
    "deploy": "truffle migrate"
  },
  "dependencies": {
    "@openzeppelin/contracts": "^4.4.0",
    "@truffle/hdwallet-provider": "^1.5.0"
  }
}"#;
        additional_files.insert("package.json".to_string(), package_json.to_string());
        
        // Add a truffle-config.js file
        let truffle_config = r#"module.exports = {
  networks: {
    development: {
      host: "127.0.0.1",
      port: 8545,
      network_id: "*"
    }
  },
  compilers: {
    solc: {
      version: "0.8.0",
      settings: {
        optimizer: {
          enabled: true,
          runs: 200
        }
      }
    }
  }
};"#;
        additional_files.insert("truffle-config.js".to_string(), truffle_config.to_string());
        
        // Add a README.md file
        let readme = r#"# Generated Ethereum Contracts

This project contains automatically generated Ethereum smart contracts from a Temporal Effect Graph.

## Setup

1. Install dependencies:
```
npm install
```

2. Compile contracts:
```
npm run compile
```

3. Run tests:
```
npm run test
```

4. Deploy contracts:
```
npm run deploy
```

## Contract Structure

The main contract implements a Temporal Effect Graph with resource management and effect execution capabilities.
"#;
        additional_files.insert("README.md".to_string(), readme.to_string());
        
        // Metadata about the generation
        let mut metadata = HashMap::new();
        metadata.insert("target".to_string(), "ethereum".to_string());
        metadata.insert("generate_time".to_string(), format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()));
        metadata.insert("effect_count".to_string(), teg.effects().len().to_string());
        metadata.insert("resource_count".to_string(), teg.resources().len().to_string());
        metadata.insert("solidity_version".to_string(), "0.8.0".to_string());
        
        Ok(GeneratedCode {
            main_output,
            language: "solidity".to_string(),
            additional_files,
            metadata,
        })
    }
}

impl CodeGenerator for EthereumGenerator {
    fn name(&self) -> &str {
        self.base_target.name()
    }
    
    fn description(&self) -> &str {
        self.base_target.description()
    }
    
    fn generate(&self, teg: &TemporalEffectGraph, config: &CodeGenConfig) -> Result<GeneratedCode> {
        // Check compatibility
        if !self.is_compatible(teg) {
            return Err(anyhow!("TEG is not compatible with the Ethereum target"));
        }
        
        // Clone the TEG for preprocessing
        let mut teg_clone = teg.clone();
        
        // Preprocess the TEG
        self.preprocess(&mut teg_clone, config)?;
        
        // Generate the code
        self.generate_code(&teg_clone, config)
    }
} 