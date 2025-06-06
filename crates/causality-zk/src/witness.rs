//! ZK witness schema and validation

use crate::{ZkWitness, error::{WitnessError, WitnessResult}};
use causality_core::machine::instruction::Instruction;
use serde::{Serialize, Deserialize};

/// Schema for validating ZK witnesses
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessSchema {
    /// Expected number of private inputs
    pub num_private_inputs: u32,
    
    /// Execution trace format
    pub trace_format: TraceFormat,
    
    /// Validation rules for the witness
    pub validation_rules: Vec<ValidationRule>,
}

/// Format for witness field validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: WitnessFieldType,
    /// Optional field description
    pub description: String,
}

/// Type of witness field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessFieldType {
    /// Integer value
    Integer,
    /// Boolean value
    Boolean,
    /// Register state
    Register,
    /// Resource reference
    Resource,
}

/// Execution trace format for witness validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceFormat {
    /// Raw register machine states
    RegisterMachine,
    /// Compressed trace format
    Compressed,
    /// Custom trace format
    Custom { format: String },
}

/// Validation rule for witness data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Range check: min ≤ value ≤ max
    Range { min: i64, max: i64 },
    /// Non-zero check
    NonZero,
    /// Boolean check (0 or 1)
    Boolean,
    /// Custom validation logic
    Custom { rule: String },
}

impl WitnessSchema {
    /// Create witness schema for instruction sequence
    pub fn for_instructions(instructions: &[Instruction]) -> Self {
        let mut num_private_inputs = 0;
        let mut validation_rules = Vec::new();
        
        // Analyze instructions to determine witness requirements
        for instruction in instructions {
            match instruction {
                Instruction::Alloc { .. } => {
                    num_private_inputs += 2; // Type and value inputs
                    validation_rules.push(ValidationRule::NonZero);
                }
                Instruction::Consume { .. } => {
                    num_private_inputs += 1; // Resource input
                }
                Instruction::Witness { .. } => {
                    num_private_inputs += 1; // Witness value
                }
                _ => {}
            }
        }
        
        Self {
            num_private_inputs,
            trace_format: TraceFormat::RegisterMachine,
            validation_rules,
        }
    }
    
    /// Validate witness against schema
    pub fn validate_witness(&self, witness: &ZkWitness) -> WitnessResult<()> {
        // Check number of private inputs
        if witness.private_inputs.len() != self.num_private_inputs as usize {
            return Err(WitnessError::ValidationFailed(
                format!("Expected {} private inputs, got {}", 
                    self.num_private_inputs, witness.private_inputs.len())
            ));
        }
        
        // Apply validation rules
        for (i, value) in witness.private_inputs.iter().enumerate() {
            if let Some(rule) = self.validation_rules.get(i) {
                self.apply_validation_rule(rule, (*value).into())?;
            }
        }
        
        Ok(())
    }
    
    /// Apply a validation rule to a value
    fn apply_validation_rule(&self, rule: &ValidationRule, value: i64) -> WitnessResult<()> {
        match rule {
            ValidationRule::Range { min, max } => {
                if value < *min || value > *max {
                    return Err(WitnessError::ValidationFailed(
                        format!("Value {} not in range [{}, {}]", value, min, max)
                    ));
                }
            }
            ValidationRule::NonZero => {
                if value == 0 {
                    return Err(WitnessError::ValidationFailed(
                        "Value must be non-zero".to_string()
                    ));
                }
            }
            ValidationRule::Boolean => {
                if value != 0 && value != 1 {
                    return Err(WitnessError::ValidationFailed(
                        "Value must be 0 or 1".to_string()
                    ));
                }
            }
            ValidationRule::Custom { rule: _ } => {
                // For now, just pass - would implement custom validation logic
                // based on the rule string
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::machine::instruction::{Instruction, RegisterId};
    
    #[test]
    fn test_witness_schema_creation() {
        let instructions = vec![
            Instruction::Alloc { 
                type_reg: RegisterId(0), 
                val_reg: RegisterId(1), 
                out_reg: RegisterId(2) 
            }
        ];
        
        let schema = WitnessSchema::for_instructions(&instructions);
        assert_eq!(schema.num_private_inputs, 2);
        assert_eq!(schema.validation_rules.len(), 1);
    }
} 