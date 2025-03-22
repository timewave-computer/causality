//! Rust Code Generator
//!
//! This module provides a code generator for Rust adapter implementations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};
#[cfg(feature = "domain")]
use crate::domain_adapters::schemas::{
    AdapterSchema, EffectDefinition, FactDefinition, ProofDefinition, TimeSyncDefinition
};
use super::{CodeGenerator, CodegenContext, CodegenTarget, GeneratedCode};
use super::rust as templates;

/// Rust code generator
pub struct RustCodeGenerator {
    /// Base output path
    base_path: PathBuf,
}

impl RustCodeGenerator {
    /// Create a new Rust code generator
    pub fn new() -> Self {
        RustCodeGenerator {
            base_path: PathBuf::from("src/generated"),
        }
    }
    
    /// Set the base output path
    pub fn with_base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = PathBuf::from(path.as_ref());
        self
    }
    
    /// Generate the adapter implementation
    fn generate_adapter_impl(&self, context: &CodegenContext) -> Result<String> {
        let schema = context.schema;
        let domain_id = schema.domain_id.as_ref();
        let domain_type = &schema.domain_type;
        
        let adapter_name = format!("{}Adapter", domain_id.to_pascal_case());
        let module_name = domain_id.to_snake_case();
        
        // Prepare template variables
        let mut vars = HashMap::new();
        vars.insert("ADAPTER_NAME".to_string(), adapter_name.clone());
        vars.insert("MODULE_NAME".to_string(), module_name);
        vars.insert("DOMAIN_ID".to_string(), domain_id.to_string());
        vars.insert("DOMAIN_TYPE".to_string(), domain_type.clone());
        vars.insert("DOMAIN_PASCALCASE".to_string(), domain_id.to_pascal_case());
        
        // Generate effect handling methods
        let mut effect_methods = String::new();
        for effect in &schema.effects {
            let effect_method = self.generate_effect_method(effect)?;
            effect_methods.push_str(&effect_method);
            effect_methods.push_str("\n\n");
        }
        vars.insert("EFFECT_METHODS".to_string(), effect_methods);
        
        // Generate fact observation methods
        let mut fact_methods = String::new();
        for fact in &schema.facts {
            let fact_method = self.generate_fact_method(fact)?;
            fact_methods.push_str(&fact_method);
            fact_methods.push_str("\n\n");
        }
        vars.insert("FACT_METHODS".to_string(), fact_methods);
        
        // Generate proof validation methods
        let mut proof_methods = String::new();
        for proof in &schema.proofs {
            let proof_method = self.generate_proof_method(proof)?;
            proof_methods.push_str(&proof_method);
            proof_methods.push_str("\n\n");
        }
        vars.insert("PROOF_METHODS".to_string(), proof_methods);
        
        // Generate RPC clients
        let mut rpc_clients = String::new();
        for rpc in &schema.rpc_interfaces {
            let rpc_client = self.generate_rpc_client(rpc)?;
            rpc_clients.push_str(&rpc_client);
            rpc_clients.push_str("\n\n");
        }
        vars.insert("RPC_CLIENTS".to_string(), rpc_clients);
        
        // Fill in the template
        let adapter_impl = templates::apply_template(templates::ADAPTER_TEMPLATE, &vars)?;
        
        Ok(adapter_impl)
    }
    
    /// Generate an effect method implementation
    fn generate_effect_method(&self, effect: &EffectDefinition) -> Result<String> {
        let effect_type = &effect.effect_type;
        let effect_name = effect_type.to_snake_case();
        let effect_pascal = effect_type.to_pascal_case();
        let rpc_call = &effect.rpc_call;
        
        // Prepare template variables
        let mut vars = HashMap::new();
        vars.insert("EFFECT_TYPE".to_string(), effect_type.clone());
        vars.insert("EFFECT_NAME".to_string(), effect_name);
        vars.insert("EFFECT_PASCAL".to_string(), effect_pascal);
        vars.insert("RPC_CALL".to_string(), rpc_call.clone());
        
        // Generate parameter validation
        let mut param_validation = String::new();
        for field in &effect.required_fields {
            let validation = format!(
                r#"        // Validate required parameter: {}
        let {} = params.params.get("{}")
            .ok_or_else(|| AdapterError::InvalidTransactionFormat(
                "Missing required parameter: {}".to_string()
            ))?;
"#,
                field, field.to_snake_case(), field, field
            );
            param_validation.push_str(&validation);
        }
        vars.insert("PARAM_VALIDATION".to_string(), param_validation);
        
        // Fill in the template
        let method_impl = templates::apply_template(templates::EFFECT_METHOD_TEMPLATE, &vars)?;
        
        Ok(method_impl)
    }
    
    /// Generate a fact method implementation
    fn generate_fact_method(&self, fact: &FactDefinition) -> Result<String> {
        let fact_type = &fact.fact_type;
        let fact_name = fact_type.to_snake_case();
        let fact_pascal = fact_type.to_pascal_case();
        let rpc_call = &fact.rpc_call;
        
        // Prepare template variables
        let mut vars = HashMap::new();
        vars.insert("FACT_TYPE".to_string(), fact_type.clone());
        vars.insert("FACT_NAME".to_string(), fact_name);
        vars.insert("FACT_PASCAL".to_string(), fact_pascal);
        vars.insert("RPC_CALL".to_string(), rpc_call.clone());
        
        // Generate parameter handling
        let mut param_handling = String::new();
        for field in &fact.required_fields {
            let handling = format!(
                r#"        // Get required parameter: {}
        let {} = params.get("{}")
            .ok_or_else(|| ObservationError::MissingFields(
                "Missing required parameter: {}".to_string()
            ))?;
"#,
                field, field.to_snake_case(), field, field
            );
            param_handling.push_str(&handling);
        }
        vars.insert("PARAM_HANDLING".to_string(), param_handling);
        
        // Fill in the template
        let method_impl = templates::apply_template(templates::FACT_METHOD_TEMPLATE, &vars)?;
        
        Ok(method_impl)
    }
    
    /// Generate a proof method implementation
    fn generate_proof_method(&self, proof: &ProofDefinition) -> Result<String> {
        let proof_type = &proof.proof_type;
        let proof_name = proof_type.to_snake_case();
        let proof_pascal = proof_type.to_pascal_case();
        let verification_method = &proof.verification_method;
        
        // Prepare template variables
        let mut vars = HashMap::new();
        vars.insert("PROOF_TYPE".to_string(), proof_type.clone());
        vars.insert("PROOF_NAME".to_string(), proof_name);
        vars.insert("PROOF_PASCAL".to_string(), proof_pascal);
        vars.insert("VERIFICATION_METHOD".to_string(), verification_method.clone());
        
        // Fill in the template
        let method_impl = templates::apply_template(templates::PROOF_METHOD_TEMPLATE, &vars)?;
        
        Ok(method_impl)
    }
    
    /// Generate an RPC client implementation
    fn generate_rpc_client(&self, rpc: &EffectDefinition) -> Result<String> {
        let rpc_name = &rpc.name;
        let rpc_variable = rpc_name.to_snake_case().replace('-', "_");
        let rpc_struct = rpc_name.to_pascal_case().replace('-', "");
        let protocol = &rpc.protocol;
        
        // Prepare template variables
        let mut vars = HashMap::new();
        vars.insert("RPC_NAME".to_string(), rpc_name.clone());
        vars.insert("RPC_VARIABLE".to_string(), rpc_variable);
        vars.insert("RPC_STRUCT".to_string(), rpc_struct);
        vars.insert("PROTOCOL".to_string(), protocol.clone());
        vars.insert("ENDPOINT_TEMPLATE".to_string(), rpc.endpoint_template.clone());
        
        // Generate method implementations
        let mut method_impls = String::new();
        for (method_name, http_method) in &rpc.methods {
            let method_impl = format!(
                r#"    /// Call the {} method
    pub async fn {}(&self, params: serde_json::Value) -> Result<serde_json::Value, reqwest::Error> {{
        self.call("{}", "{}", params).await
    }}
"#,
                method_name, method_name.to_snake_case().replace('-', "_"), 
                method_name, http_method
            );
            method_impls.push_str(&method_impl);
            method_impls.push_str("\n");
        }
        vars.insert("METHOD_IMPLS".to_string(), method_impls);
        
        // Fill in the template
        let client_impl = templates::apply_template(templates::RPC_CLIENT_TEMPLATE, &vars)?;
        
        Ok(client_impl)
    }
    
    /// Generate helper types and utilities
    fn generate_helpers(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        let schema = context.schema;
        let domain_id = schema.domain_id.as_ref();
        
        let mut helpers = HashMap::new();
        
        // Generate types module
        let mut vars = HashMap::new();
        vars.insert("DOMAIN_ID".to_string(), domain_id.to_string());
        vars.insert("DOMAIN_PASCALCASE".to_string(), domain_id.to_pascal_case());
        
        let types_file = templates::apply_template(templates::TYPES_TEMPLATE, &vars)?;
        helpers.insert("types.rs".to_string(), types_file);
        
        // Generate utils module
        let utils_file = templates::apply_template(templates::UTILS_TEMPLATE, &vars)?;
        helpers.insert("utils.rs".to_string(), utils_file);
        
        Ok(helpers)
    }
    
    /// Generate test implementations
    fn generate_tests(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        if !context.options.generate_tests {
            return Ok(HashMap::new());
        }
        
        let schema = context.schema;
        let domain_id = schema.domain_id.as_ref();
        let adapter_name = format!("{}Adapter", domain_id.to_pascal_case());
        
        let mut tests = HashMap::new();
        
        // Generate main adapter tests
        let mut vars = HashMap::new();
        vars.insert("ADAPTER_NAME".to_string(), adapter_name);
        vars.insert("DOMAIN_ID".to_string(), domain_id.to_string());
        
        let adapter_tests = templates::apply_template(templates::ADAPTER_TEST_TEMPLATE, &vars)?;
        tests.insert("adapter_test.rs".to_string(), adapter_tests);
        
        // Generate effect tests
        if !schema.effects.is_empty() {
            let effect_tests = templates::apply_template(templates::EFFECT_TEST_TEMPLATE, &vars)?;
            tests.insert("effect_test.rs".to_string(), effect_tests);
        }
        
        // Generate fact tests
        if !schema.facts.is_empty() {
            let fact_tests = templates::apply_template(templates::FACT_TEST_TEMPLATE, &vars)?;
            tests.insert("fact_test.rs".to_string(), fact_tests);
        }
        
        // Generate proof tests
        if !schema.proofs.is_empty() {
            let proof_tests = templates::apply_template(templates::PROOF_TEST_TEMPLATE, &vars)?;
            tests.insert("proof_test.rs".to_string(), proof_tests);
        }
        
        Ok(tests)
    }
    
    /// Generate documentation
    fn generate_docs(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        if !context.options.generate_docs {
            return Ok(HashMap::new());
        }
        
        let schema = context.schema;
        let domain_id = schema.domain_id.as_ref();
        let adapter_name = format!("{}Adapter", domain_id.to_pascal_case());
        
        let mut docs = HashMap::new();
        
        // Generate README
        let mut vars = HashMap::new();
        vars.insert("ADAPTER_NAME".to_string(), adapter_name);
        vars.insert("DOMAIN_ID".to_string(), domain_id.to_string());
        vars.insert("DOMAIN_TYPE".to_string(), schema.domain_type.clone());
        
        let readme = templates::apply_template(templates::README_TEMPLATE, &vars)?;
        docs.insert("README.md".to_string(), readme);
        
        // Generate API documentation
        let api_docs = templates::apply_template(templates::API_DOCS_TEMPLATE, &vars)?;
        docs.insert("API.md".to_string(), api_docs);
        
        Ok(docs)
    }
    
    /// Generate example code
    fn generate_examples(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        if !context.options.generate_examples {
            return Ok(HashMap::new());
        }
        
        let schema = context.schema;
        let domain_id = schema.domain_id.as_ref();
        let adapter_name = format!("{}Adapter", domain_id.to_pascal_case());
        
        let mut examples = HashMap::new();
        
        // Generate basic usage example
        let mut vars = HashMap::new();
        vars.insert("ADAPTER_NAME".to_string(), adapter_name);
        vars.insert("DOMAIN_ID".to_string(), domain_id.to_string());
        
        let basic_example = templates::apply_template(templates::BASIC_EXAMPLE_TEMPLATE, &vars)?;
        examples.insert("basic_usage.rs".to_string(), basic_example);
        
        // Generate effect examples if there are any effects
        if !schema.effects.is_empty() {
            let effect = &schema.effects[0];
            vars.insert("EFFECT_TYPE".to_string(), effect.effect_type.clone());
            
            let effect_example = templates::apply_template(templates::EFFECT_EXAMPLE_TEMPLATE, &vars)?;
            examples.insert(format!("{}_example.rs", effect.effect_type), effect_example);
        }
        
        // Generate fact examples if there are any facts
        if !schema.facts.is_empty() {
            let fact = &schema.facts[0];
            vars.insert("FACT_TYPE".to_string(), fact.fact_type.clone());
            
            let fact_example = templates::apply_template(templates::FACT_EXAMPLE_TEMPLATE, &vars)?;
            examples.insert(format!("{}_example.rs", fact.fact_type), fact_example);
        }
        
        Ok(examples)
    }
}

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, context: &CodegenContext) -> Result<GeneratedCode> {
        // Validate the schema
        context.schema.validate()?;
        
        let mut code = GeneratedCode::new();
        
        // Generate the main adapter implementation
        let adapter_impl = self.generate_adapter_impl(context)?;
        code.set_adapter_impl(adapter_impl);
        
        // Generate helper files
        let helpers = self.generate_helpers(context)?;
        for (name, content) in helpers {
            code.add_support_file(name, content);
        }
        
        // Generate tests
        let tests = self.generate_tests(context)?;
        for (name, content) in tests {
            code.add_test_file(name, content);
        }
        
        // Generate docs
        let docs = self.generate_docs(context)?;
        for (name, content) in docs {
            code.add_doc_file(name, content);
        }
        
        // Generate examples
        let examples = self.generate_examples(context)?;
        for (name, content) in examples {
            code.add_example_file(name, content);
        }
        
        Ok(code)
    }
    
    fn target(&self) -> CodegenTarget {
        CodegenTarget::Rust
    }
    
    fn write_to_disk(&self, code: &GeneratedCode, output_dir: &Path) -> Result<()> {
        let out_path = if output_dir.is_absolute() {
            output_dir.to_path_buf()
        } else {
            self.base_path.join(output_dir)
        };
        
        // Create output directory if it doesn't exist
        if !out_path.exists() {
            std::fs::create_dir_all(&out_path)?;
        }
        
        // Write adapter implementation
        if !code.adapter_impl.is_empty() {
            let file_path = out_path.join("adapter_impl.rs");
            std::fs::write(file_path, &code.adapter_impl)
                .map_err(|e| Error::IoError(format!("Failed to write adapter_impl.rs: {}", e)))?;
        }
        
        // Write support files
        for (name, content) in &code.support_files {
            let file_path = out_path.join(name);
            std::fs::write(&file_path, content)
                .map_err(|e| Error::IoError(format!("Failed to write {}: {}", name, e)))?;
        }
        
        // Write test files
        let test_dir = out_path.join("tests");
        if !code.test_files.is_empty() && !test_dir.exists() {
            std::fs::create_dir_all(&test_dir)
                .map_err(|e| Error::IoError(format!("Failed to create test directory: {}", e)))?;
        }
        
        for (name, content) in &code.test_files {
            let file_path = test_dir.join(name);
            std::fs::write(&file_path, content)
                .map_err(|e| Error::IoError(format!("Failed to write test file {}: {}", name, e)))?;
        }
        
        // Write documentation files
        let doc_dir = out_path.join("docs");
        if !code.doc_files.is_empty() && !doc_dir.exists() {
            std::fs::create_dir_all(&doc_dir)
                .map_err(|e| Error::IoError(format!("Failed to create docs directory: {}", e)))?;
        }
        
        for (name, content) in &code.doc_files {
            let file_path = doc_dir.join(name);
            std::fs::write(&file_path, content)
                .map_err(|e| Error::IoError(format!("Failed to write doc file {}: {}", name, e)))?;
        }
        
        // Write example files
        let examples_dir = out_path.join("examples");
        if !code.example_files.is_empty() && !examples_dir.exists() {
            std::fs::create_dir_all(&examples_dir)
                .map_err(|e| Error::IoError(format!("Failed to create examples directory: {}", e)))?;
        }
        
        for (name, content) in &code.example_files {
            let file_path = examples_dir.join(name);
            std::fs::write(&file_path, content)
                .map_err(|e| Error::IoError(format!("Failed to write example file {}: {}", name, e)))?;
        }
        
        Ok(())
    }
}

/// Convert a string to snake_case
trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl ToSnakeCase for str {
    fn to_snake_case(&self) -> String {
        let mut result = String::new();
        let mut prev_is_upper = false;
        
        for (i, c) in self.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 && !prev_is_upper {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
                prev_is_upper = true;
            } else {
                result.push(c);
                prev_is_upper = false;
            }
        }
        
        result
    }
}

/// Convert a string to PascalCase
trait ToPascalCase {
    fn to_pascal_case(&self) -> String;
}

impl ToPascalCase for str {
    fn to_pascal_case(&self) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;
        
        for c in self.chars() {
            if c == '_' || c == '-' || c == ' ' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DomainId;
    use crate::effect_adapters::schemas::ethereum::create_ethereum_schema;
    use crate::effect_adapters::codegen::CodegenOptions;
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!("hello_world".to_snake_case(), "hello_world");
        assert_eq!("HelloWorld".to_snake_case(), "hello_world");
        assert_eq!("helloWorld".to_snake_case(), "hello_world");
        assert_eq!("ETH".to_snake_case(), "eth");
        assert_eq!("eth-json-rpc".to_snake_case(), "eth-json-rpc");
    }
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!("hello_world".to_pascal_case(), "HelloWorld");
        assert_eq!("hello-world".to_pascal_case(), "HelloWorld");
        assert_eq!("helloWorld".to_pascal_case(), "HelloWorld");
        assert_eq!("eth".to_pascal_case(), "Eth");
        assert_eq!("eth-json-rpc".to_pascal_case(), "EthJsonRpc");
    }
    
    #[test]
    fn test_ethereum_code_generation() {
        let schema = create_ethereum_schema();
        let options = CodegenOptions::default();
        let context = CodegenContext::new(&schema, options);
        
        let generator = RustCodeGenerator::new();
        let result = generator.generate(&context);
        
        assert!(result.is_ok());
        let code = result.unwrap();
        
        // Check that all expected components are present
        assert!(!code.adapter_impl.is_empty());
        assert!(!code.support_files.is_empty());
        assert!(!code.test_files.is_empty());
        assert!(!code.doc_files.is_empty());
    }
} 