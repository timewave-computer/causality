//! JavaScript/TypeScript Code Generation for Effect Adapters

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use std::collections::HashMap;
#[cfg(feature = "domain")]
use crate::domain_adapters::schemas::{AdapterSchema, EffectDefinition, FactDefinition, ProofDefinition};
use crate::error::Result;
use crate::effect_adapters::codegen::{
    CodeGenerator, 
    CodegenContext, 
    CodegenTarget, 
    GeneratedCode
};
use crate::effect_adapters::codegen::templates::javascript as ts_templates;

/// TypeScript code generator for effect adapters
#[derive(Debug)]
pub struct TypeScriptCodeGenerator {
    /// Base output path for generated code
    base_output_path: PathBuf,
}

impl TypeScriptCodeGenerator {
    /// Create a new TypeScript code generator
    pub fn new() -> Self {
        TypeScriptCodeGenerator {
            base_output_path: PathBuf::from("src/generated"),
        }
    }
    
    /// Set the output path for generated code
    pub fn with_output_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_output_path = path.as_ref().to_path_buf();
        self
    }
    
    /// Generate the main adapter implementation
    fn generate_adapter_impl(&self, context: &CodegenContext) -> Result<String> {
        let mut vars = context.variables.clone();
        
        // Add additional variables
        let schema = context.schema;
        let adapter_name = format!("{}Adapter", to_pascal_case(&schema.domain_id.as_ref()));
        
        vars.insert("ADAPTER_NAME".to_string(), adapter_name.clone());
        vars.insert("DOMAIN_NAME".to_string(), to_pascal_case(&schema.domain_id.as_ref()));
        vars.insert("DOMAIN_ID".to_string(), schema.domain_id.as_ref().to_string());
        vars.insert("DOMAIN_TYPE".to_string(), schema.domain_type.clone());
        
        // Generate effect method switch cases
        let mut effect_cases = String::new();
        for effect in &schema.effects {
            let mut effect_vars = vars.clone();
            effect_vars.insert("EFFECT_TYPE".to_string(), effect.effect_type.clone());
            
            let case_code = ts_templates::apply_js_template(
                ts_templates::EFFECT_SWITCH_CASE_TEMPLATE,
                &effect_vars
            )?;
            
            effect_cases.push_str(&case_code);
            effect_cases.push_str("\n");
        }
        
        vars.insert("EFFECT_SWITCH_CASES".to_string(), effect_cases);
        
        // Generate fact method switch cases
        let mut fact_cases = String::new();
        for fact in &schema.facts {
            let mut fact_vars = vars.clone();
            fact_vars.insert("FACT_TYPE".to_string(), fact.fact_type.clone());
            
            let case_code = ts_templates::apply_js_template(
                ts_templates::FACT_SWITCH_CASE_TEMPLATE,
                &fact_vars
            )?;
            
            fact_cases.push_str(&case_code);
            fact_cases.push_str("\n");
        }
        
        vars.insert("FACT_SWITCH_CASES".to_string(), fact_cases);
        
        // Generate proof method switch cases
        let mut proof_cases = String::new();
        for proof in &schema.proofs {
            let mut proof_vars = vars.clone();
            proof_vars.insert("PROOF_TYPE".to_string(), proof.proof_type.clone());
            
            let case_code = ts_templates::apply_js_template(
                ts_templates::PROOF_SWITCH_CASE_TEMPLATE,
                &proof_vars
            )?;
            
            proof_cases.push_str(&case_code);
            proof_cases.push_str("\n");
        }
        
        vars.insert("PROOF_SWITCH_CASES".to_string(), proof_cases);
        
        // Generate specific methods
        let mut effect_methods = String::new();
        for effect in &schema.effects {
            let method_code = self.generate_effect_method(effect, &vars)?;
            effect_methods.push_str(&method_code);
            effect_methods.push_str("\n\n");
        }
        
        vars.insert("EFFECT_METHODS".to_string(), effect_methods);
        
        let mut fact_methods = String::new();
        for fact in &schema.facts {
            let method_code = self.generate_fact_method(fact, &vars)?;
            fact_methods.push_str(&method_code);
            fact_methods.push_str("\n\n");
        }
        
        vars.insert("FACT_METHODS".to_string(), fact_methods);
        
        let mut proof_methods = String::new();
        for proof in &schema.proofs {
            let method_code = self.generate_proof_method(proof, &vars)?;
            proof_methods.push_str(&method_code);
            proof_methods.push_str("\n\n");
        }
        
        vars.insert("PROOF_METHODS".to_string(), proof_methods);
        
        // Generate RPC client
        let rpc_client = self.generate_rpc_client(&vars)?;
        vars.insert("RPC_CLIENT".to_string(), rpc_client);
        
        // Apply the template
        ts_templates::apply_js_template(ts_templates::ADAPTER_TEMPLATE, &vars)
    }
    
    /// Generate an effect method
    fn generate_effect_method(&self, effect: &EffectDefinition, vars: &HashMap<String, String>) -> Result<String> {
        let mut effect_vars = vars.clone();
        
        effect_vars.insert("EFFECT_TYPE".to_string(), effect.effect_type.clone());
        effect_vars.insert("EFFECT_TYPE_CAMEL".to_string(), to_camel_case(&effect.effect_type));
        
        // Add required fields
        let required_fields = effect.required_fields.join(", ");
        effect_vars.insert("REQUIRED_FIELDS".to_string(), required_fields);
        
        // Add field validation
        let mut field_validation = String::new();
        for field in &effect.required_fields {
            field_validation.push_str(&format!("if (!params.{}) {{\n", field));
            field_validation.push_str(&format!("            throw new Error('Missing required field: {}')\n", field));
            field_validation.push_str("        }\n");
        }
        
        effect_vars.insert("FIELD_VALIDATION".to_string(), field_validation);
        
        ts_templates::apply_js_template(ts_templates::EFFECT_METHOD_TEMPLATE, &effect_vars)
    }
    
    /// Generate a fact method
    fn generate_fact_method(&self, fact: &FactDefinition, vars: &HashMap<String, String>) -> Result<String> {
        let mut fact_vars = vars.clone();
        
        fact_vars.insert("FACT_TYPE".to_string(), fact.fact_type.clone());
        fact_vars.insert("FACT_TYPE_CAMEL".to_string(), to_camel_case(&fact.fact_type));
        
        // Add required fields
        let required_fields = fact.required_fields.join(", ");
        fact_vars.insert("REQUIRED_FIELDS".to_string(), required_fields);
        
        // Add field validation
        let mut field_validation = String::new();
        for field in &fact.required_fields {
            field_validation.push_str(&format!("if (!params.{}) {{\n", field));
            field_validation.push_str(&format!("            throw new Error('Missing required field: {}')\n", field));
            field_validation.push_str("        }\n");
        }
        
        fact_vars.insert("FIELD_VALIDATION".to_string(), field_validation);
        
        // Add RPC call
        fact_vars.insert("RPC_CALL".to_string(), fact.rpc_call.clone());
        
        ts_templates::apply_js_template(ts_templates::FACT_METHOD_TEMPLATE, &fact_vars)
    }
    
    /// Generate a proof method
    fn generate_proof_method(&self, proof: &ProofDefinition, vars: &HashMap<String, String>) -> Result<String> {
        let mut proof_vars = vars.clone();
        
        proof_vars.insert("PROOF_TYPE".to_string(), proof.proof_type.clone());
        proof_vars.insert("PROOF_TYPE_CAMEL".to_string(), to_camel_case(&proof.proof_type));
        
        // Add required fields
        let required_fields = proof.required_fields.join(", ");
        proof_vars.insert("REQUIRED_FIELDS".to_string(), required_fields);
        
        // Add field validation
        let mut field_validation = String::new();
        for field in &proof.required_fields {
            field_validation.push_str(&format!("if (!params.{}) {{\n", field));
            field_validation.push_str(&format!("            throw new Error('Missing required field: {}')\n", field));
            field_validation.push_str("        }\n");
        }
        
        proof_vars.insert("FIELD_VALIDATION".to_string(), field_validation);
        
        ts_templates::apply_js_template(ts_templates::PROOF_METHOD_TEMPLATE, &proof_vars)
    }
    
    /// Generate the RPC client
    fn generate_rpc_client(&self, vars: &HashMap<String, String>) -> Result<String> {
        ts_templates::apply_js_template(ts_templates::RPC_CLIENT_TEMPLATE, vars)
    }
    
    /// Generate helper files (types, utilities)
    fn generate_helpers(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        let mut helpers = HashMap::new();
        let vars = context.variables.clone();
        
        // Add types file
        let types_content = ts_templates::apply_js_template(ts_templates::TYPES_TEMPLATE, &vars)?;
        helpers.insert("types.ts".to_string(), types_content);
        
        // Add utilities file
        let utils_content = ts_templates::apply_js_template(ts_templates::UTILS_TEMPLATE, &vars)?;
        helpers.insert("utils.ts".to_string(), utils_content);
        
        Ok(helpers)
    }
    
    /// Generate test files
    fn generate_tests(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        if !context.options.generate_tests {
            return Ok(HashMap::new());
        }
        
        let mut tests = HashMap::new();
        let schema = context.schema;
        let mut vars = context.variables.clone();
        
        // Add adapter name
        let adapter_name = format!("{}Adapter", to_pascal_case(&schema.domain_id.as_ref()));
        vars.insert("ADAPTER_NAME".to_string(), adapter_name);
        
        // Generate adapter tests
        let adapter_test = ts_templates::apply_js_template(ts_templates::ADAPTER_TEST_TEMPLATE, &vars)?;
        tests.insert("adapter.test.ts".to_string(), adapter_test);
        
        // Generate effect tests
        if !schema.effects.is_empty() {
            let effect = &schema.effects[0]; // Use the first effect as example
            let mut effect_vars = vars.clone();
            effect_vars.insert("EFFECT_TYPE".to_string(), effect.effect_type.clone());
            effect_vars.insert("EFFECT_TYPE_CAMEL".to_string(), to_camel_case(&effect.effect_type));
            
            let effect_test = ts_templates::apply_js_template(ts_templates::EFFECT_TEST_TEMPLATE, &effect_vars)?;
            tests.insert(format!("{}_effect.test.ts", effect.effect_type), effect_test);
        }
        
        // Generate fact tests
        if !schema.facts.is_empty() {
            let fact = &schema.facts[0]; // Use the first fact as example
            let mut fact_vars = vars.clone();
            fact_vars.insert("FACT_TYPE".to_string(), fact.fact_type.clone());
            fact_vars.insert("FACT_TYPE_CAMEL".to_string(), to_camel_case(&fact.fact_type));
            
            let fact_test = ts_templates::apply_js_template(ts_templates::FACT_TEST_TEMPLATE, &fact_vars)?;
            tests.insert(format!("{}_fact.test.ts", fact.fact_type), fact_test);
        }
        
        // Generate proof tests
        if !schema.proofs.is_empty() {
            let proof = &schema.proofs[0]; // Use the first proof as example
            let mut proof_vars = vars.clone();
            proof_vars.insert("PROOF_TYPE".to_string(), proof.proof_type.clone());
            proof_vars.insert("PROOF_TYPE_CAMEL".to_string(), to_camel_case(&proof.proof_type));
            
            let proof_test = ts_templates::apply_js_template(ts_templates::PROOF_TEST_TEMPLATE, &proof_vars)?;
            tests.insert(format!("{}_proof.test.ts", proof.proof_type), proof_test);
        }
        
        Ok(tests)
    }
    
    /// Generate documentation files
    fn generate_docs(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        if !context.options.generate_docs {
            return Ok(HashMap::new());
        }
        
        let mut docs = HashMap::new();
        let schema = context.schema;
        let mut vars = context.variables.clone();
        
        // Add adapter name
        let adapter_name = format!("{}Adapter", to_pascal_case(&schema.domain_id.as_ref()));
        vars.insert("ADAPTER_NAME".to_string(), adapter_name);
        vars.insert("DOMAIN_ID".to_string(), schema.domain_id.as_ref().to_string());
        
        // Add effect types
        let mut effect_list = String::new();
        for effect in &schema.effects {
            effect_list.push_str(&format!("- `{}`: {}\n", effect.effect_type, effect.effect_type));
        }
        vars.insert("EFFECT_LIST".to_string(), effect_list);
        
        // Add fact types
        let mut fact_list = String::new();
        for fact in &schema.facts {
            fact_list.push_str(&format!("- `{}`: {}\n", fact.fact_type, fact.fact_type));
        }
        vars.insert("FACT_LIST".to_string(), fact_list);
        
        // Add proof types
        let mut proof_list = String::new();
        for proof in &schema.proofs {
            proof_list.push_str(&format!("- `{}`: {}\n", proof.proof_type, proof.proof_type));
        }
        vars.insert("PROOF_LIST".to_string(), proof_list);
        
        // Generate README
        let readme = ts_templates::apply_js_template(ts_templates::README_TEMPLATE, &vars)?;
        docs.insert("README.md".to_string(), readme);
        
        // Generate API docs
        let api_docs = ts_templates::apply_js_template(ts_templates::API_DOCS_TEMPLATE, &vars)?;
        docs.insert("API.md".to_string(), api_docs);
        
        Ok(docs)
    }
    
    /// Generate example files
    fn generate_examples(&self, context: &CodegenContext) -> Result<HashMap<String, String>> {
        if !context.options.generate_examples {
            return Ok(HashMap::new());
        }
        
        let mut examples = HashMap::new();
        let schema = context.schema;
        let mut vars = context.variables.clone();
        
        // Add adapter name
        let adapter_name = format!("{}Adapter", to_pascal_case(&schema.domain_id.as_ref()));
        vars.insert("ADAPTER_NAME".to_string(), adapter_name);
        
        // Add simple example
        let basic_example = ts_templates::apply_js_template(ts_templates::BASIC_EXAMPLE_TEMPLATE, &vars)?;
        examples.insert("basic_example.ts".to_string(), basic_example);
        
        // Add package.json
        let package_json = ts_templates::apply_js_template(ts_templates::PACKAGE_JSON_TEMPLATE, &vars)?;
        examples.insert("package.json".to_string(), package_json);
        
        // Add TypeScript definition file
        let d_ts = ts_templates::apply_js_template(ts_templates::TYPESCRIPT_DEFINITION_TEMPLATE, &vars)?;
        examples.insert("index.d.ts".to_string(), d_ts);
        
        Ok(examples)
    }
    
    /// Create the context for code generation
    pub fn create_context<'a>(&self, schema: &'a AdapterSchema, options: &CodegenOptions) -> CodegenContext<'a> {
        let mut context = CodegenContext::new(schema, options.clone());
        
        // Add additional variables
        let adapter_name = format!("{}Adapter", to_pascal_case(&schema.domain_id.as_ref()));
        context.add_variable("ADAPTER_NAME", adapter_name);
        context.add_variable("DOMAIN_NAME", to_pascal_case(&schema.domain_id.as_ref()));
        
        context
    }
}

impl CodeGenerator for TypeScriptCodeGenerator {
    fn generate(&self, context: &CodegenContext) -> Result<GeneratedCode> {
        let mut code = HashMap::new();
        
        // Generate main adapter implementation
        let adapter_impl = self.generate_adapter_impl(context)?;
        let schema = context.schema;
        let adapter_name = format!("{}Adapter", to_pascal_case(&schema.domain_id.as_ref()));
        code.insert(format!("{}.ts", adapter_name), adapter_impl);
        
        // Generate helper files
        let helpers = self.generate_helpers(context)?;
        for (name, content) in helpers {
            code.insert(name, content);
        }
        
        // Generate test files
        let tests = self.generate_tests(context)?;
        for (name, content) in tests {
            code.insert(format!("tests/{}", name), content);
        }
        
        // Generate documentation files
        let docs = self.generate_docs(context)?;
        for (name, content) in docs {
            code.insert(format!("docs/{}", name), content);
        }
        
        // Generate example files
        let examples = self.generate_examples(context)?;
        for (name, content) in examples {
            code.insert(format!("examples/{}", name), content);
        }
        
        Ok(code)
    }
    
    fn target(&self) -> CodegenTarget {
        CodegenTarget::TypeScript
    }
    
    fn write_to_disk(&self, code: &GeneratedCode, output_dir: &Path) -> Result<()> {
        // This method is not used with the new implementation
        // that returns a HashMap<String, String> from generate()
        Ok(())
    }
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Convert a string to camelCase
fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    if pascal.is_empty() {
        return pascal;
    }
    
    let mut result = String::new();
    let mut chars = pascal.chars();
    if let Some(first) = chars.next() {
        result.push(first.to_ascii_lowercase());
    }
    result.extend(chars);
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::default::Default;
    
    /// Mock schema for testing
    #[derive(Debug, Clone)]
    struct MockSchema {
        id: String,
        domain_type: String,
        version: String,
        effect_definitions: Vec<MockEffectDefinition>,
        fact_definitions: Vec<MockFactDefinition>,
        proof_definitions: Vec<MockProofDefinition>,
    }
    
    impl Default for MockSchema {
        fn default() -> Self {
            MockSchema {
                id: "ethereum".to_string(),
                domain_type: "blockchain".to_string(),
                version: "1.0.0".to_string(),
                effect_definitions: vec![MockEffectDefinition::default()],
                fact_definitions: vec![MockFactDefinition::default()],
                proof_definitions: vec![MockProofDefinition::default()],
            }
        }
    }
    
    #[derive(Debug, Clone, Default)]
    struct MockEffectDefinition {
        effect_type: String,
        required_fields: Vec<String>,
        optional_fields: Vec<String>,
        metadata: HashMap<String, String>,
    }
    
    #[derive(Debug, Clone, Default)]
    struct MockFactDefinition {
        fact_type: String,
        required_fields: Vec<String>,
        optional_fields: Vec<String>,
        metadata: HashMap<String, String>,
    }
    
    #[derive(Debug, Clone, Default)]
    struct MockProofDefinition {
        proof_type: String,
        required_fields: Vec<String>,
        metadata: HashMap<String, String>,
    }
    
    #[derive(Debug)]
    struct MockCodegenContext<'a> {
        schema: &'a MockSchema,
        options: CodegenOptions,
        variables: HashMap<String, String>,
    }
    
    impl<'a> MockCodegenContext<'a> {
        fn new(schema: &'a MockSchema, options: CodegenOptions) -> Self {
            let mut variables = HashMap::new();
            variables.insert("DOMAIN_ID".to_string(), schema.id.clone());
            variables.insert("DOMAIN_TYPE".to_string(), schema.domain_type.clone());
            variables.insert("SCHEMA_VERSION".to_string(), schema.version.clone());
            
            MockCodegenContext {
                schema,
                options,
                variables,
            }
        }
    }
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld");
    }
    
    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("helloWorld"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
    }
    
    #[test]
    fn test_typescript_generator_basic() {
        let generator = TypeScriptCodeGenerator::new();
        assert_eq!(generator.target(), CodegenTarget::TypeScript);
    }
} 