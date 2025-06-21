//! Gas metering for the minimal 5-operation instruction set
//!
//! This module implements gas metering and cost calculation for the
//! mathematically minimal instruction set.

use crate::machine::instruction::{Instruction, RegisterId};
use crate::system::content_addressing::ResourceId;
use serde::{Serialize, Deserialize};

/// Gas metering for the minimal instruction set
#[derive(Debug, Clone)]
pub struct GasMeter {
    /// Current gas consumed
    pub gas_used: u64,
    
    /// Gas limit for execution
    pub gas_limit: u64,
    
    /// Instruction costs for the 5 operations
    pub instruction_costs: InstructionCosts,
}

/// Cost configuration for the 5 minimal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionCosts {
    /// Cost for Transform operations (morphism application)
    pub transform_cost: u64,
    
    /// Cost for Alloc operations (resource creation)
    pub alloc_cost: u64,
    
    /// Cost for Consume operations (resource destruction)
    pub consume_cost: u64,
    
    /// Cost for Compose operations (morphism composition)
    pub compose_cost: u64,
    
    /// Cost for Tensor operations (parallel composition)
    pub tensor_cost: u64,
}

impl Default for InstructionCosts {
    fn default() -> Self {
        Self {
            transform_cost: 3,  // Transform is the most general operation
            alloc_cost: 5,      // Resource allocation is expensive
            consume_cost: 2,    // Resource deallocation is cheaper
            compose_cost: 1,    // Composition is just creating a new morphism
            tensor_cost: 2,     // Tensor product is structural
        }
    }
}

impl GasMeter {
    /// Create a new gas meter with the given limit
    pub fn new(gas_limit: u64) -> Self {
        Self {
            gas_used: 0,
            gas_limit,
            instruction_costs: InstructionCosts::default(),
        }
    }
    
    /// Create a gas meter with custom instruction costs
    pub fn with_costs(gas_limit: u64, costs: InstructionCosts) -> Self {
        Self {
            gas_used: 0,
            gas_limit,
            instruction_costs: costs,
        }
    }
    
    /// Check if we have enough gas for an instruction
    pub fn can_execute(&self, instruction: &Instruction) -> bool {
        let cost = self.instruction_cost(instruction);
        self.gas_used + cost <= self.gas_limit
    }
    
    /// Consume gas for an instruction
    pub fn consume_gas(&mut self, instruction: &Instruction) -> Result<(), GasError> {
        let cost = self.instruction_cost(instruction);
        
        if self.gas_used + cost > self.gas_limit {
            return Err(GasError::OutOfGas {
                required: cost,
                available: self.gas_limit - self.gas_used,
            });
        }
        
        self.gas_used += cost;
        Ok(())
    }
    
    /// Get the gas cost for an instruction
    pub fn instruction_cost(&self, instruction: &Instruction) -> u64 {
        match instruction {
            Instruction::Transform { .. } => self.instruction_costs.transform_cost,
            Instruction::Alloc { .. } => self.instruction_costs.alloc_cost,
            Instruction::Consume { .. } => self.instruction_costs.consume_cost,
            Instruction::Compose { .. } => self.instruction_costs.compose_cost,
            Instruction::Tensor { .. } => self.instruction_costs.tensor_cost,
        }
    }
    
    /// Get remaining gas
    pub fn remaining_gas(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }
    
    /// Get gas usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.gas_limit == 0 {
            0.0
        } else {
            (self.gas_used as f64 / self.gas_limit as f64) * 100.0
        }
    }
    
    /// Reset gas usage
    pub fn reset(&mut self) {
        self.gas_used = 0;
    }
    
    /// Estimate gas for a sequence of instructions
    pub fn estimate_gas(&self, instructions: &[Instruction]) -> u64 {
        instructions.iter()
            .map(|instr| self.instruction_cost(instr))
            .sum()
    }
}

/// Gas-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GasError {
    /// Out of gas error
    OutOfGas {
        required: u64,
        available: u64,
    },
    
    /// Gas limit exceeded
    GasLimitExceeded {
        limit: u64,
        used: u64,
    },
}

impl std::fmt::Display for GasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GasError::OutOfGas { required, available } => {
                write!(f, "Out of gas: required {}, available {}", required, available)
            }
            GasError::GasLimitExceeded { limit, used } => {
                write!(f, "Gas limit exceeded: limit {}, used {}", limit, used)
            }
        }
    }
}

impl std::error::Error for GasError {}

/// Gas accounting for complex operations
impl GasMeter {
    /// Calculate gas for morphism application based on complexity
    pub fn morphism_application_cost(&self, complexity: MorphismComplexity) -> u64 {
        let base_cost = self.instruction_costs.transform_cost;
        
        match complexity {
            MorphismComplexity::Simple => base_cost,
            MorphismComplexity::Moderate => base_cost * 2,
            MorphismComplexity::Complex => base_cost * 5,
            MorphismComplexity::Composition(depth) => base_cost * (1 + depth as u64),
            MorphismComplexity::Tensor(components) => base_cost * components as u64,
        }
    }
    
    /// Calculate gas for resource operations based on resource type
    pub fn resource_operation_cost(&self, operation: ResourceOperation, size: u64) -> u64 {
        let base_cost = match operation {
            ResourceOperation::Alloc => self.instruction_costs.alloc_cost,
            ResourceOperation::Consume => self.instruction_costs.consume_cost,
        };
        
        // Scale cost by resource size (with a minimum)
        base_cost + (size / 1024).max(1)
    }
}

/// Morphism complexity for gas calculation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MorphismComplexity {
    /// Simple morphism (identity, basic function)
    Simple,
    
    /// Moderate complexity (protocol, effect)
    Moderate,
    
    /// Complex morphism (nested protocols, complex effects)
    Complex,
    
    /// Composed morphism with given depth
    Composition(u32),
    
    /// Tensor product with given number of components
    Tensor(u32),
}

/// Resource operation type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOperation {
    /// Resource allocation
    Alloc,
    
    /// Resource consumption
    Consume,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_gas_metering() {
        let mut meter = GasMeter::new(100);
        
        let transform = Instruction::Transform {
            morph_reg: RegisterId::new(1),
            input_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        };
        
        assert!(meter.can_execute(&transform));
        assert_eq!(meter.instruction_cost(&transform), 3);
        
        meter.consume_gas(&transform).unwrap();
        assert_eq!(meter.gas_used, 3);
        assert_eq!(meter.remaining_gas(), 97);
    }
    
    #[test]
    fn test_out_of_gas() {
        let mut meter = GasMeter::new(2);
        
        let alloc = Instruction::Alloc {
            type_reg: RegisterId::new(1),
            init_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        };
        
        assert!(!meter.can_execute(&alloc));
        
        let result = meter.consume_gas(&alloc);
        assert!(matches!(result, Err(GasError::OutOfGas { .. })));
    }
    
    #[test]
    fn test_gas_estimation() {
        let meter = GasMeter::new(1000);
        
        let instructions = vec![
            Instruction::Alloc {
                type_reg: RegisterId::new(1),
                init_reg: RegisterId::new(2),
                output_reg: RegisterId::new(3),
            },
            Instruction::Transform {
                morph_reg: RegisterId::new(3),
                input_reg: RegisterId::new(4),
                output_reg: RegisterId::new(5),
            },
            Instruction::Consume {
                resource_reg: RegisterId::new(3),
                output_reg: RegisterId::new(6),
            },
        ];
        
        let estimated = meter.estimate_gas(&instructions);
        assert_eq!(estimated, 5 + 3 + 2); // alloc + transform + consume
    }
    
    #[test]
    fn test_custom_costs() {
        let custom_costs = InstructionCosts {
            transform_cost: 10,
            alloc_cost: 20,
            consume_cost: 5,
            compose_cost: 3,
            tensor_cost: 7,
        };
        
        let meter = GasMeter::with_costs(1000, custom_costs);
        
        let tensor = Instruction::Tensor {
            left_reg: RegisterId::new(1),
            right_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        };
        
        assert_eq!(meter.instruction_cost(&tensor), 7);
    }
    
    #[test]
    fn test_morphism_complexity_costs() {
        let meter = GasMeter::new(1000);
        
        assert_eq!(meter.morphism_application_cost(MorphismComplexity::Simple), 3);
        assert_eq!(meter.morphism_application_cost(MorphismComplexity::Moderate), 6);
        assert_eq!(meter.morphism_application_cost(MorphismComplexity::Complex), 15);
        assert_eq!(meter.morphism_application_cost(MorphismComplexity::Composition(3)), 12);
        assert_eq!(meter.morphism_application_cost(MorphismComplexity::Tensor(4)), 12);
    }
    
    #[test]
    fn test_resource_operation_costs() {
        let meter = GasMeter::new(1000);
        
        // Small resource
        assert_eq!(meter.resource_operation_cost(ResourceOperation::Alloc, 100), 6); // 5 + 1
        
        // Large resource
        assert_eq!(meter.resource_operation_cost(ResourceOperation::Alloc, 2048), 7); // 5 + 2
        
        // Consume is cheaper
        assert_eq!(meter.resource_operation_cost(ResourceOperation::Consume, 100), 3); // 2 + 1
    }
} 