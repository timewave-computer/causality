// Layer 1 to Layer 0 compiler - translates typed terms to register machine instructions

use crate::layer0::{Instruction, Register};
use crate::layer1::{Term, Variable};
use std::collections::BTreeMap;

/// Compiler state for tracking register allocation and variable bindings
pub struct CompilerState {
    /// Next available register
    next_register: u32,
    
    /// Variable to register mapping - deterministic ordering
    var_map: BTreeMap<Variable, Register>,
    
    /// Generated instructions
    instructions: Vec<Instruction>,
    
    /// Record field layouts (for monomorphization) - deterministic ordering
    record_layouts: BTreeMap<String, Vec<String>>,
}

/// Compilation errors
#[derive(Debug)]
pub enum CompileError {
    UnboundVariable(Variable),
    TypeError(String),
}

impl Default for CompilerState {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerState {
    /// Create a new compiler state
    pub fn new() -> Self {
        CompilerState {
            next_register: 0,
            var_map: BTreeMap::new(),
            instructions: Vec::new(),
            record_layouts: BTreeMap::new(),
        }
    }
    
    /// Allocate a fresh register
    fn fresh_register(&mut self) -> Register {
        let reg = Register(self.next_register);
        self.next_register += 1;
        reg
    }
    
    /// Get or allocate register for a variable
    fn get_var_register(&mut self, var: &Variable) -> Result<Register, CompileError> {
        self.var_map.get(var)
            .cloned()
            .ok_or_else(|| CompileError::UnboundVariable(var.clone()))
    }
    
    /// Bind a variable to a register
    fn bind_var(&mut self, var: Variable, reg: Register) {
        self.var_map.insert(var, reg);
    }
    
    /// Add an instruction
    fn emit(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }
    
    /// Get current PC (for labels)
    fn current_pc(&self) -> usize {
        self.instructions.len()
    }
    
    /// Reserve space for a future instruction
    fn reserve_instruction(&mut self) -> usize {
        let pc = self.current_pc();
        self.instructions.push(Instruction::Create { 
            value_reg: Register(0), 
            msg_reg: Register(0) 
        }); // Placeholder
        pc
    }
    
    /// Compile a record to a nested pair structure
    /// Records are compiled as right-nested pairs: {a,b,c} -> (a,(b,c))
    fn compile_record_fields(
        &mut self,
        fields: &[(String, Box<Term>)]
    ) -> Result<Register, CompileError> {
        if fields.is_empty() {
            // Empty record is unit
            let reg = self.fresh_register();
            // In real impl, would store Unit value
            Ok(reg)
        } else if fields.len() == 1 {
            // Single field - just compile the value
            compile_term_inner(self, &fields[0].1)
        } else {
            // Multiple fields - create nested pairs
            let _first_reg = compile_term_inner(self, &fields[0].1)?;
            let _rest_reg = self.compile_record_fields(&fields[1..])?;
            
            // Create pair (first, rest)
            let pair_reg = self.fresh_register();
            // In real impl, would create Pair value
            Ok(pair_reg)
        }
    }
}

/// Compile a Layer 1 term to Layer 0 instructions
pub fn compile_term(term: &Term) -> Result<(Vec<Instruction>, Register), CompileError> {
    let mut state = CompilerState::new();
    let result_reg = compile_term_inner(&mut state, term)?;
    Ok((state.instructions, result_reg))
}

/// Internal compilation function that modifies state
fn compile_term_inner(
    state: &mut CompilerState, 
    term: &Term
) -> Result<Register, CompileError> {
    match term {
        Term::Var(v) => {
            state.get_var_register(v)
        }
        
        Term::Unit => {
            let reg = state.fresh_register();
            // Store unit value directly (no instruction needed, just bind)
            // In a real implementation, we'd need to handle this properly
            Ok(reg)
        }
        
        Term::Bool(_b) => {
            let reg = state.fresh_register();
            // For now, store bool as a value that will be converted to message later
            Ok(reg)
        }
        
        Term::Int(_n) => {
            let reg = state.fresh_register();
            // For now, store int as a value that will be converted to message later
            Ok(reg)
        }
        
        Term::Pair(left, right) => {
            let r1 = compile_term_inner(state, left)?;
            let _r2 = compile_term_inner(state, right)?;
            // Would need to create a pair value - simplified for now
            Ok(r1)
        }
        
        Term::Record(fields) => {
            // Convert HashMap to sorted Vec for deterministic layout
            let mut sorted_fields: Vec<_> = fields.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            sorted_fields.sort_by(|a, b| a.0.cmp(&b.0));
            
            // Store the layout for later field access
            let layout_key = format!("record_{}", state.next_register);
            let field_names: Vec<String> = sorted_fields.iter()
                .map(|(name, _)| name.clone())
                .collect();
            state.record_layouts.insert(layout_key.clone(), field_names);
            
            // Compile fields to nested pairs
            let value_reg = state.compile_record_fields(&sorted_fields)?;
            
            // Create message from the value
            let msg_reg = state.fresh_register();
            state.emit(Instruction::Create { value_reg, msg_reg });
            Ok(msg_reg)
        }
        
        Term::Project { record, label: _ } => {
            let record_reg = compile_term_inner(state, record)?;
            
            // For now, simplified - just return the record
            // In full impl, would look up layout and compile field access
            Ok(record_reg)
        }
        
        Term::Extend { record, label: _, value } => {
            // For now, simplified - compile both and return record
            let _record_reg = compile_term_inner(state, record)?;
            let _value_reg = compile_term_inner(state, value)?;
            let result_reg = state.fresh_register();
            Ok(result_reg)
        }
        
        Term::Restrict { record, labels: _ } => {
            // For now, just compile and return the record
            compile_term_inner(state, record)
        }
        
        Term::Send { channel, value } => {
            let chan_reg = compile_term_inner(state, channel)?;
            let val_reg = compile_term_inner(state, value)?;
            // Records are already messages, so just send directly
            state.emit(Instruction::Send { msg_reg: val_reg, chan_reg });
            Ok(chan_reg) // Return updated channel
        }
        
        Term::Receive(channel) => {
            let chan_reg = compile_term_inner(state, channel)?;
            let msg_reg = state.fresh_register();
            state.emit(Instruction::Receive { chan_reg, msg_reg });
            Ok(msg_reg)
        }
        
        Term::Let { var, value, body } => {
            let val_reg = compile_term_inner(state, value)?;
            state.bind_var(var.clone(), val_reg);
            compile_term_inner(state, body)
        }
        
        Term::Case { scrutinee, left_var: _, left_body, right_var: _, right_body } => {
            let _guard_reg = compile_term_inner(state, scrutinee)?;
            let _result_reg = state.fresh_register();
            
            // Simplified - just emit regular instruction and return left result
            // In full implementation, would emit proper branching instructions
            
            // Left branch
            let left_result = compile_term_inner(state, left_body)?;
            
            let _left_end_jump = state.reserve_instruction();
            
            // Right branch  
            let _right_result = compile_term_inner(state, right_body)?;
            
            let _end_pc = state.current_pc();
            
            // For simplicity, return left result register
            Ok(left_result)
        }
        
        _ => Err(CompileError::TypeError("Unsupported term".to_string())),
    }
}

/// Compile a simple example program
pub fn compile_example() -> Vec<Instruction> {
    // Example: create a message with value 42, then consume it
    vec![
        // Assume register 0 has value Int(42)
        Instruction::Create { 
            value_reg: Register(0), 
            msg_reg: Register(1) 
        },
        Instruction::Consume { 
            msg_reg: Register(1), 
            value_reg: Register(2) 
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::Term;
    
    #[test]
    fn test_compile_record_operations() {
        // Create a record and project from it
        let term = Term::let_bind(
            "msg",
            Term::record(vec![("value", Term::Int(42))]),
            Term::project(Term::var("msg"), "value")
        );
        
        let (instructions, _) = compile_term(&term).unwrap();
        
        // Should have Create instruction for the record
        assert!(instructions.iter().any(|i| matches!(i, Instruction::Create { .. })));
    }
    
    #[test]
    fn test_compile_let_binding() {
        // let x = record { value: 5 } in x
        let term = Term::let_bind(
            "x",
            Term::record(vec![("value", Term::Int(5))]),
            Term::var("x")
        );
        
        let result = compile_term(&term);
        assert!(result.is_ok());
        
        let (instructions, _) = result.unwrap();
        assert!(!instructions.is_empty()); // At least Create
    }
}
