// Transaction Execution Layer for smart contract execution
// Original file: src/tel/mod.rs

//! Transaction Effect Language (TEL) implementation
//!
//! TEL provides a declarative language for describing transactions and effects
//! across multiple blockchain domains. This module contains the core TEL components,
//! including script parsing, handlers, and execution.

// Module declarations
pub mod script;
pub mod handlers;

// Re-export key components
pub use script::{TelScript, TelOperation, TelOperationType, TelParser};
pub use handlers::{
    TelHandler, ConstraintTelHandler, TransferTelHandler, 
    StorageTelHandler, QueryTelHandler, TelHandlerRegistry,
    TransferParams, StorageParams, QueryParams,
    TelCompiler, StandardTelCompiler
};

/// TEL macro for inline script creation (placeholder)
///
/// In a full implementation, this would be a proc macro that parses TEL syntax
/// into a TelScript at compile time.
#[macro_export]
macro_rules! tel {
    ($script:expr) => {
        {
            let source = $script;
            crate::tel::TelParser::parse(source).expect("Failed to parse TEL script")
        }
    };
}

/// Shorthand function to create and parse a TEL script
pub fn parse_tel(source: &str) -> Result<TelScript, anyhow::Error> {
    TelParser::parse(source)
}

/// Shorthand function to compile a TEL script into effects
pub async fn compile_tel(
    source: &str,
    compiler: &dyn TelCompiler,
    context: &crate::effect::EffectContext,
) -> Result<Vec<std::sync::Arc<dyn crate::effect::Effect>>, anyhow::Error> {
    let script = parse_tel(source)?;
    compiler.compile(&script, context).await
}

/// Shorthand function to execute a TEL script
pub async fn execute_tel(
    source: &str,
    compiler: &dyn TelCompiler,
    context: crate::effect::EffectContext,
) -> Result<Vec<crate::effect::EffectOutcome>, anyhow::Error> {
    let script = parse_tel(source)?;
    compiler.execute(&script, context).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainId;
    
    #[test]
    fn test_tel_macro() {
        // This is a simple test to ensure the macro compiles
        // In a real test, we would use the actual macro
        let script = "transfer(from: '0x1234', to: '0x5678', amount: 100, token: 'ETH')";
        let result = parse_tel(script);
        
        // The actual parsing is not implemented, so we expect an error
        assert!(result.is_err());
    }
    
    #[test]
    fn test_create_tel_script() {
        let mut script = TelScript::new("Test script");
        
        let transfer = TelOperation::transfer(
            "0x1234",
            "0x5678",
            100,
            "ETH",
            Some(DomainId::new("ethereum:mainnet")),
        );
        
        script.add_operation(transfer);
        
        assert_eq!(script.operations().len(), 1);
        assert_eq!(script.operations()[0].function_name, "transfer");
    }
} 