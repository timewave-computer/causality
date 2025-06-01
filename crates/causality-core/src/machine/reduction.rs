//! Operational semantics and reduction rules
//!
//! This module implements the operational semantics for the register machine,
//! defining how instructions are executed and values are reduced.
//! 
//! The reduction rules implement the formal semantics of the 9-instruction set.

use super::{
    instruction::{Instruction, RegisterId, ConstraintExpr, Effect, Label},
    state::MachineState,
    value::MachineValue,
    resource::ResourceManager,
    metering::Metering,
};
use crate::lambda::{TypeInner, BaseType, Symbol};
use crate::system::error::ReductionError;
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Reduction Engine
//-----------------------------------------------------------------------------

/// The reduction engine executes instructions according to operational semantics
pub struct ReductionEngine {
    /// Current machine state
    state: MachineState,
    
    /// Instruction sequence being executed
    program: Vec<Instruction>,
    
    /// Label map for jumps
    labels: HashMap<Label, usize>,
    
    /// Maximum number of reduction steps (for termination)
    max_steps: usize,
    
    /// Current step count
    step_count: usize,
    
    /// Witness provider (for non-deterministic inputs)
    witness_provider: Option<Box<dyn WitnessProvider>>,
    
    /// Computational metering
    metering: Option<Metering>,
}

/// Trait for providing witness values
pub trait WitnessProvider {
    /// Get a witness value for the given register
    fn get_witness(&mut self, reg: RegisterId) -> MachineValue;
}

impl ReductionEngine {
    /// Create a new reduction engine
    pub fn new(program: Vec<Instruction>, max_steps: usize) -> Self {
        // Build label map for efficient jumps
        let mut labels = HashMap::new();
        for (index, instruction) in program.iter().enumerate() {
            if let Instruction::LabelMarker(label) = instruction {
                if labels.insert(label.clone(), index).is_some() {
                    // Potentially handle or log duplicate label definitions if necessary
                    // For now, we'll just overwrite, but this could be an error condition
                    // depending on language semantics.
                    // TODO: revisit once language is more mature
                    eprintln!("Warning: Duplicate label definition for {:?}", label);
                }
            }
        }
        
        Self {
            state: MachineState::new(),
            program,
            labels,
            max_steps,
            step_count: 0,
            witness_provider: None,
            metering: None,
        }
    }
    
    /// Enable computational metering with the given budget
    pub fn enable_metering(&mut self, budget: u64) -> Result<(), ReductionError> {
        let mut metering = Metering::new();
        metering.initialize_budget(&mut self.state, budget)
            .map_err(|_| ReductionError::NotImplemented("Failed to initialize metering".to_string()))?;
        self.metering = Some(metering);
        Ok(())
    }
    
    /// Set the witness provider
    pub fn set_witness_provider(&mut self, provider: Box<dyn WitnessProvider>) {
        self.witness_provider = Some(provider);
    }
    
    /// Run the program until completion or max steps
    pub fn run(&mut self) -> Result<&MachineState, ReductionError> {
        while self.state.pc < self.program.len() && self.step_count < self.max_steps {
            self.step()?;
            self.step_count += 1;
        }
        
        if self.step_count >= self.max_steps {
            Err(ReductionError::MaxStepsExceeded)
        } else {
            Ok(&self.state)
        }
    }
    
    /// Execute a single reduction step
    pub fn step(&mut self) -> Result<(), ReductionError> {
        if self.state.pc >= self.program.len() {
            return Err(ReductionError::ProgramCounterOutOfBounds);
        }
        
        let instruction = self.program.get(self.state.pc).ok_or(ReductionError::PCOutOfBounds)?;

        self.state.jumped = false; // Reset jumped flag for the current instruction

        // Apply computational metering if enabled
        if let Some(ref metering) = self.metering {
            metering.consume_for_instruction(&mut self.state, instruction)
                .map_err(|_| ReductionError::NotImplemented("Compute budget exhausted".to_string()))?;
        }
        
        self.execute_instruction(instruction.clone())?;
        
        if !self.state.jumped {
            self.state.pc += 1;
        }
        
        Ok(())
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), ReductionError> {
        match instruction {
            Instruction::Move { src, dst } => {
                self.execute_move(src, dst)?;
            }
            
            Instruction::Apply { fn_reg, arg_reg, out_reg } => {
                self.execute_apply(fn_reg, arg_reg, out_reg)?;
            }
            
            Instruction::Match { sum_reg, left_reg, right_reg, left_label, right_label } => {
                self.execute_match(sum_reg, left_reg, right_reg, left_label, right_label)?;
            }
            
            Instruction::Alloc { type_reg, val_reg, out_reg } => {
                self.execute_alloc(type_reg, val_reg, out_reg)?;
            }
            
            Instruction::Consume { resource_reg, out_reg } => {
                self.execute_consume(resource_reg, out_reg)?;
            }
            
            Instruction::Check { constraint } => {
                self.execute_check(constraint)?;
            }
            
            Instruction::Perform { effect, out_reg } => {
                self.execute_perform(effect, out_reg)?;
            }
            
            Instruction::Select { cond_reg, true_reg, false_reg, out_reg } => {
                self.execute_select(cond_reg, true_reg, false_reg, out_reg)?;
            }
            
            Instruction::Witness { out_reg } => self.execute_witness(out_reg)?,
            Instruction::LabelMarker(_) => { /* Do nothing, it's a marker. PC will advance if not jumped. */ }
        }

        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Reduction Rules Implementation
//-----------------------------------------------------------------------------

impl ReductionEngine {
    /// Execute move instruction: Transfer ownership, invalidate source
    /// ⟨move r₁ r₂, σ⟩ → ⟨σ[r₂ ↦ σ(r₁), r₁ ↦ ⊥]⟩
    fn execute_move(&mut self, src: RegisterId, dst: RegisterId) -> Result<(), ReductionError> {
        // Load value from source register
        let src_reg = self.state.load_register(src)
            .map_err(|_| ReductionError::RegisterNotFound(src))?;
        
        if src_reg.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(src));
        }
        
        // Store value in destination register
        self.state.store_register(dst, src_reg.value.clone(), src_reg.value_type.clone());
        
        // Consume source register (linear consumption)
        self.state.consume_register(src)
            .map_err(|_| ReductionError::RegisterConsumptionFailed(src))?;
        
        Ok(())
    }
    
    /// Execute function application
    /// ⟨apply r_fn r_arg r_out, σ⟩ → ⟨σ[r_out ↦ f(v)]⟩ where σ(r_fn) = f, σ(r_arg) = v
    fn execute_apply(&mut self, fn_reg: RegisterId, arg_reg: RegisterId, out_reg: RegisterId) -> Result<(), ReductionError> {
        // Load function value
        let fn_val = self.state.load_register(fn_reg)
            .map_err(|_| ReductionError::RegisterNotFound(fn_reg))?;
        
        if fn_val.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(fn_reg));
        }
        
        // Load argument value
        let arg_val = self.state.load_register(arg_reg)
            .map_err(|_| ReductionError::RegisterNotFound(arg_reg))?;
        
        if arg_val.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(arg_reg));
        }
        
        // Apply function based on its type
        let result = match &fn_val.value {
            MachineValue::Function { params: _, body: _ } => {
                // For now, we don't support user-defined functions
                // This would require substitution and evaluation
                return Err(ReductionError::NotImplemented("User-defined functions".to_string()));
            }
            
            MachineValue::BuiltinFunction(name) => {
                self.apply_builtin(name, &arg_val.value)?
            }
            
            _ => return Err(ReductionError::NotAFunction(fn_reg)),
        };
        
        // Store result
        self.state.store_register(out_reg, result, None);
        
        // Consume function and argument (linear)
        self.state.consume_register(fn_reg)
            .map_err(|_| ReductionError::RegisterConsumptionFailed(fn_reg))?;
        self.state.consume_register(arg_reg)
            .map_err(|_| ReductionError::RegisterConsumptionFailed(arg_reg))?;
        
        Ok(())
    }
    
    /// Execute pattern matching on sum types
    /// ⟨match r_sum r_left r_right label_l label_r, σ⟩ → 
    ///   if σ(r_sum) = inl(v) then ⟨σ[r_left ↦ v], pc ↦ label_l⟩
    ///   if σ(r_sum) = inr(v) then ⟨σ[r_right ↦ v], pc ↦ label_r⟩
    fn execute_match(
        &mut self, 
        sum_reg: RegisterId, 
        left_reg: RegisterId, 
        right_reg: RegisterId,
        left_label: Label,
        right_label: Label
    ) -> Result<(), ReductionError> {
        // Load sum value
        let sum_val = self.state.load_register(sum_reg)
            .map_err(|_| ReductionError::RegisterNotFound(sum_reg))?;
        
        if sum_val.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(sum_reg));
        }
        
        match &sum_val.value {
            MachineValue::Sum { tag, value } => {
                if tag.as_str() == "inl" || tag.as_str() == "left" {
                    // Left branch
                    self.state.store_register(left_reg, (**value).clone(), None);
                    self.jump_to_label(&left_label)?;
                } else if tag.as_str() == "inr" || tag.as_str() == "right" {
                    // Right branch
                    self.state.store_register(right_reg, (**value).clone(), None);
                    self.jump_to_label(&right_label)?;
                } else {
                    return Err(ReductionError::InvalidSumTag(tag.clone()));
                }
                
                // Consume the sum value
                self.state.consume_register(sum_reg)
                    .map_err(|_| ReductionError::RegisterConsumptionFailed(sum_reg))?;
                
                Ok(())
            }
            _ => Err(ReductionError::NotASum(sum_reg)),
        }
    }
    
    /// Execute resource allocation
    /// ⟨alloc r_τ r_v r_out, σ⟩ →
    ///   let id = fresh_id() in
    ///   let res = Resource{id, type=σ(r_τ), value=σ(r_v), consumed=false} in
    ///   ⟨σ[r_out ↦ res, heap ↦ heap ∪ {id ↦ res}]⟩
    fn execute_alloc(&mut self, type_reg: RegisterId, val_reg: RegisterId, out_reg: RegisterId) -> Result<(), ReductionError> {
        // Load type value
        let type_val = self.state.load_register(type_reg)
            .map_err(|_| ReductionError::RegisterNotFound(type_reg))?;
        
        // Load value to store
        let val = self.state.load_register(val_reg)
            .map_err(|_| ReductionError::RegisterNotFound(val_reg))?;
        
        if val.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(val_reg));
        }
        
        // Extract type from register value
        let resource_type = match &type_val.value {
            MachineValue::Type(t) => t.clone(),
            _ => {
                // If not a type value, use the value's own type
                val.value_type.clone().unwrap_or(TypeInner::Base(BaseType::Unit))
            }
        };
        
        // Allocate resource on heap
        let resource_id = self.state.alloc_resource(val.value.clone(), resource_type);
        
        // Store resource reference in output register
        self.state.store_register(out_reg, MachineValue::ResourceRef(resource_id), None);
        
        // Consume input value register
        self.state.consume_register(val_reg)
            .map_err(|_| ReductionError::RegisterConsumptionFailed(val_reg))?;
        
        Ok(())
    }
    
    /// Execute resource consumption
    /// ⟨consume r_res r_out, σ⟩ →
    ///   if σ.heap[σ(r_res).id].consumed then
    ///     error(AlreadyConsumed)
    ///   else
    ///     ⟨σ[r_out ↦ σ(r_res).value, 
    ///        heap ↦ heap[σ(r_res).id.consumed ↦ true]]⟩
    fn execute_consume(&mut self, resource_reg: RegisterId, out_reg: RegisterId) -> Result<(), ReductionError> {
        // Load resource reference
        let res_val = self.state.load_register(resource_reg)
            .map_err(|_| ReductionError::RegisterNotFound(resource_reg))?;
        
        if res_val.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(resource_reg));
        }
        
        match &res_val.value {
            MachineValue::ResourceRef(id) => {
                // Clone the id to avoid borrow issues
                let resource_id = id.clone();
                
                // Consume resource from heap
                let value = self.state.consume_resource(resource_id.clone())
                    .map_err(|_| ReductionError::ResourceAlreadyConsumed(resource_id))?;
                
                // Store extracted value in output register
                self.state.store_register(out_reg, value, None);
                
                // Consume the resource reference register
                self.state.consume_register(resource_reg)
                    .map_err(|_| ReductionError::RegisterConsumptionFailed(resource_reg))?;
                
                Ok(())
            }
            _ => Err(ReductionError::NotAResource(resource_reg)),
        }
    }
    
    /// Execute constraint checking
    /// ⟨check c, σ⟩ → ⟨σ⟩ if eval(c, σ) = true, else error
    fn execute_check(&mut self, constraint: ConstraintExpr) -> Result<(), ReductionError> {
        let satisfied = self.evaluate_constraint(&constraint)?;
        
        if !satisfied {
            Err(ReductionError::ConstraintViolation)
        } else {
            Ok(())
        }
    }
    
    /// Execute effect
    /// ⟨perform eff r_out, σ⟩ → ⟨σ[effects ↦ effects ++ [eff]]⟩
    fn execute_perform(&mut self, effect: Effect, out_reg: RegisterId) -> Result<(), ReductionError> {
        // Check preconditions
        let pre_satisfied = self.evaluate_constraint(&effect.pre)?;
        if !pre_satisfied {
            return Err(ReductionError::EffectPreconditionFailed);
        }
        
        // Clone the tag to avoid move issue
        let effect_tag = effect.tag.clone();
        
        // Add effect to state
        self.state.add_effect(super::effect::Effect {
            call: super::instruction::EffectCall {
                tag: effect.tag,
                args: effect.params.clone(),
                return_type: None,
            },
            result_register: Some(out_reg),
        });
        
        // Effects are handled by the interpreter, not the reduction engine
        // For now, we just store a placeholder in the result register
        self.state.store_register(out_reg, MachineValue::EffectResult(effect_tag), None);
        
        Ok(())
    }
    
    /// Execute conditional selection
    /// ⟨select r_cond r_true r_false r_out, σ⟩ →
    ///   if σ(r_cond) = true then ⟨σ[r_out ↦ σ(r_true)]⟩
    ///   else ⟨σ[r_out ↦ σ(r_false)]⟩
    fn execute_select(
        &mut self,
        cond_reg: RegisterId,
        true_reg: RegisterId,
        false_reg: RegisterId,
        out_reg: RegisterId
    ) -> Result<(), ReductionError> {
        // Load condition value
        let cond_val = self.state.load_register(cond_reg)
            .map_err(|_| ReductionError::RegisterNotFound(cond_reg))?;
        
        if cond_val.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(cond_reg));
        }
        
        // Evaluate condition
        let condition = match &cond_val.value {
            MachineValue::Bool(b) => *b,
            _ => return Err(ReductionError::NotABoolean(cond_reg)),
        };
        
        // Select value based on condition
        let selected_reg = if condition { true_reg } else { false_reg };
        let selected_val = self.state.load_register(selected_reg)
            .map_err(|_| ReductionError::RegisterNotFound(selected_reg))?;
        
        if selected_val.consumed {
            return Err(ReductionError::RegisterAlreadyConsumed(selected_reg));
        }
        
        // Store selected value in output
        self.state.store_register(out_reg, selected_val.value.clone(), selected_val.value_type.clone());
        
        // Consume condition and selected register (linear)
        self.state.consume_register(cond_reg)
            .map_err(|_| ReductionError::RegisterConsumptionFailed(cond_reg))?;
        self.state.consume_register(selected_reg)
            .map_err(|_| ReductionError::RegisterConsumptionFailed(selected_reg))?;
        
        Ok(())
    }
    
    /// Execute witness reading
    /// ⟨witness r_out, σ⟩ → ⟨σ[r_out ↦ witness_value]⟩
    fn execute_witness(&mut self, out_reg: RegisterId) -> Result<(), ReductionError> {
        // Get witness value from provider
        let witness_value = match &mut self.witness_provider {
            Some(provider) => provider.get_witness(out_reg),
            None => return Err(ReductionError::NoWitnessProvider),
        };
        
        // Store witness value in output register
        self.state.store_register(out_reg, witness_value, None);
        
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

impl ReductionEngine {
    /// Apply a builtin function
    fn apply_builtin(&self, name: &Symbol, arg: &MachineValue) -> Result<MachineValue, ReductionError> {
        match name.as_str() {
            // Sum constructors
            "inl" => Ok(MachineValue::Sum {
                tag: Symbol::new("inl"),
                value: Box::new(arg.clone()),
            }),
            "inr" => Ok(MachineValue::Sum {
                tag: Symbol::new("inr"),
                value: Box::new(arg.clone()),
            }),
            
            // Product constructor
            "pair" => {
                // This would need currying for proper implementation
                Ok(MachineValue::PartiallyApplied {
                    name: name.clone(),
                    args: vec![arg.clone()],
                })
            }
            
            // Arithmetic operations
            "add" | "sub" | "mul" | "div" => {
                // These need currying too
                Ok(MachineValue::PartiallyApplied {
                    name: name.clone(),
                    args: vec![arg.clone()],
                })
            }
            
            _ => Err(ReductionError::UnknownBuiltin(name.clone())),
        }
    }
    
    /// Jump to a label
    fn jump_to_label(&mut self, label: &Label) -> Result<(), ReductionError> {
        if let Some(&target_pc) = self.labels.get(label) {
            self.state.pc = target_pc;
            self.state.jumped = true; // Indicate that a jump occurred
            Ok(())
        } else {
            Err(ReductionError::LabelNotFound(label.clone()))
        }
    }
    
    /// Evaluate a constraint expression
    fn evaluate_constraint(&self, constraint: &ConstraintExpr) -> Result<bool, ReductionError> {
        match constraint {
            ConstraintExpr::True => Ok(true),
            ConstraintExpr::False => Ok(false),
            
            ConstraintExpr::And(left, right) => {
                Ok(self.evaluate_constraint(left)? && self.evaluate_constraint(right)?)
            }
            
            ConstraintExpr::Or(left, right) => {
                Ok(self.evaluate_constraint(left)? || self.evaluate_constraint(right)?)
            }
            
            ConstraintExpr::Not(expr) => {
                Ok(!self.evaluate_constraint(expr)?)
            }
            
            ConstraintExpr::Equal(a, b) => {
                let val_a = self.state.load_register(*a)
                    .map_err(|_| ReductionError::RegisterNotFound(*a))?;
                let val_b = self.state.load_register(*b)
                    .map_err(|_| ReductionError::RegisterNotFound(*b))?;
                
                Ok(val_a.value == val_b.value)
            }
            
            ConstraintExpr::LessThan(a, b) => {
                self.compare_numeric(*a, *b, |x, y| x < y)
            }
            
            ConstraintExpr::GreaterThan(a, b) => {
                self.compare_numeric(*a, *b, |x, y| x > y)
            }
            
            ConstraintExpr::LessEqual(a, b) => {
                self.compare_numeric(*a, *b, |x, y| x <= y)
            }
            
            ConstraintExpr::GreaterEqual(a, b) => {
                self.compare_numeric(*a, *b, |x, y| x >= y)
            }
            
            ConstraintExpr::HasType(reg, expected_type) => {
                let reg_val = self.state.load_register(*reg)
                    .map_err(|_| ReductionError::RegisterNotFound(*reg))?;
                
                match &reg_val.value_type {
                    Some(actual_type) => Ok(actual_type == expected_type),
                    None => Ok(false),
                }
            }
            
            ConstraintExpr::IsConsumed(reg) => {
                match self.state.load_register(*reg) {
                    Ok(reg_val) => Ok(reg_val.consumed),
                    Err(_) => Ok(true), // Non-existent registers are considered consumed
                }
            }
            
            ConstraintExpr::HasCapability(res_reg, cap) => {
                // This would check if a resource has a specific capability
                // For now, we'll return false as capabilities aren't implemented
                let _ = (res_reg, cap);
                Ok(false)
            }
            
            ConstraintExpr::IsOwner(res_reg, owner_reg) => {
                // This would check ownership of a resource
                // For now, we'll return false as ownership isn't implemented
                let _ = (res_reg, owner_reg);
                Ok(false)
            }
            
            ConstraintExpr::Predicate { .. } => {
                Err(ReductionError::NotImplemented("Custom predicates".to_string()))
            }
        }
    }
    
    /// Compare two numeric values
    fn compare_numeric(&self, a: RegisterId, b: RegisterId, op: fn(u32, u32) -> bool) -> Result<bool, ReductionError> {
        let val_a = self.state.load_register(a)
            .map_err(|_| ReductionError::RegisterNotFound(a))?;
        let val_b = self.state.load_register(b)
            .map_err(|_| ReductionError::RegisterNotFound(b))?;
        
        match (&val_a.value, &val_b.value) {
            (MachineValue::Int(x), MachineValue::Int(y)) => Ok(op(*x, *y)),
            _ => Err(ReductionError::TypeMismatch),
        }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::instruction::RegisterId;
    use crate::lambda::BaseType;
    
    #[test]
    fn test_move_instruction() {
        let mut engine = ReductionEngine::new(vec![
            Instruction::Witness { out_reg: RegisterId::new(1) },
            Instruction::Move {
                src: RegisterId::new(1),
                dst: RegisterId::new(2),
            },
        ], 10);
        
        // Provide witness values
        struct MoveWitness;
        impl WitnessProvider for MoveWitness {
            fn get_witness(&mut self, reg: RegisterId) -> MachineValue {
                match reg.id() {
                    1 => MachineValue::Int(42),
                    _ => MachineValue::Unit,
                }
            }
        }
        
        engine.set_witness_provider(Box::new(MoveWitness));
        
        // Run the program
        let result = engine.run();
        assert!(result.is_ok());
        
        let state = result.unwrap();
        
        // Check that source register is consumed
        let src_reg = state.load_register(RegisterId::new(1)).unwrap();
        assert!(src_reg.consumed);
        
        // Check that destination register has the value
        let dst_reg = state.load_register(RegisterId::new(2)).unwrap();
        assert!(!dst_reg.consumed);
        assert_eq!(dst_reg.value, MachineValue::Int(42));
    }
    
    #[test]
    fn test_alloc_consume() {
        let mut engine = ReductionEngine::new(vec![
            // Create a type value
            Instruction::Witness { out_reg: RegisterId::new(1) }, // Type
            // Create a value
            Instruction::Witness { out_reg: RegisterId::new(2) }, // Value
            // Allocate resource
            Instruction::Alloc {
                type_reg: RegisterId::new(1),
                val_reg: RegisterId::new(2),
                out_reg: RegisterId::new(3),
            },
            // Consume resource
            Instruction::Consume {
                resource_reg: RegisterId::new(3),
                out_reg: RegisterId::new(4),
            },
        ], 10);
        
        // Provide witness values
        struct TestWitness;
        impl WitnessProvider for TestWitness {
            fn get_witness(&mut self, reg: RegisterId) -> MachineValue {
                match reg.id() {
                    1 => MachineValue::Type(TypeInner::Base(BaseType::Int)),
                    2 => MachineValue::Int(42),
                    _ => MachineValue::Unit,
                }
            }
        }
        
        engine.set_witness_provider(Box::new(TestWitness));
        
        // Run the program
        let result = engine.run();
        assert!(result.is_ok());
        
        let state = result.unwrap();
        
        // Check that the final register has the consumed value
        let final_reg = state.load_register(RegisterId::new(4)).unwrap();
        assert_eq!(final_reg.value, MachineValue::Int(42));
    }
    
    #[test]
    fn test_computational_metering() {
        let mut engine = ReductionEngine::new(vec![
            Instruction::Witness { out_reg: RegisterId::new(1) },
            Instruction::Witness { out_reg: RegisterId::new(2) },
            Instruction::Witness { out_reg: RegisterId::new(3) },
        ], 10);
        
        // Enable metering with limited budget
        assert!(engine.enable_metering(200).is_ok()); // witness costs 100 each
        
        // Run should fail due to budget exhaustion
        let result = engine.run();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_select_instruction() {
        let mut engine = ReductionEngine::new(vec![
            // Create condition (true)
            Instruction::Witness { out_reg: RegisterId::new(1) },
            // Create true value
            Instruction::Witness { out_reg: RegisterId::new(2) },
            // Create false value
            Instruction::Witness { out_reg: RegisterId::new(3) },
            // Select based on condition
            Instruction::Select {
                cond_reg: RegisterId::new(1),
                true_reg: RegisterId::new(2),
                false_reg: RegisterId::new(3),
                out_reg: RegisterId::new(4),
            },
        ], 10);
        
        struct SelectWitness;
        impl WitnessProvider for SelectWitness {
            fn get_witness(&mut self, reg: RegisterId) -> MachineValue {
                match reg.id() {
                    1 => MachineValue::Bool(true),
                    2 => MachineValue::Int(42),
                    3 => MachineValue::Int(99),
                    _ => MachineValue::Unit,
                }
            }
        }
        
        engine.set_witness_provider(Box::new(SelectWitness));
        
        let result = engine.run();
        assert!(result.is_ok());
        
        let state = result.unwrap();
        let final_reg = state.load_register(RegisterId::new(4)).unwrap();
        assert_eq!(final_reg.value, MachineValue::Int(42)); // Should select true branch
    }
    
    #[test]
    fn test_builtin_functions() {
        let _state = MachineState::new();
        // ... existing code ...
    }
} 