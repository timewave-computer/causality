// Layer 0 instructions - the minimal set for message passing

use crate::layer0::{
    MessageValue, MachineState, MachineError,
    Register, Binding, SumVariant
};

/// Layer 0 instruction set (5 instructions)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Create a new message from a value
    Create { value_reg: Register, msg_reg: Register },
    
    /// Consume a message to extract its value
    Consume { msg_reg: Register, value_reg: Register },
    
    /// Send a message through a channel
    Send { msg_reg: Register, chan_reg: Register },
    
    /// Receive a message from a channel
    Receive { chan_reg: Register, msg_reg: Register },
    
    /// Pattern match on message structure
    Match { 
        msg_reg: Register, 
        left_reg: Register,
        right_reg: Register,
        left_target: usize,
        right_target: usize,
    },
}

/// Execute a single instruction on the machine state
pub fn execute_instruction(
    state: &mut MachineState, 
    instruction: &Instruction
) -> Result<(), MachineError> {
    match instruction {
        Instruction::Create { value_reg, msg_reg } => {
            // Get value from register
            let value = match state.get_binding(*value_reg)? {
                Binding::Value(v) => v.clone(),
                Binding::Message(_id) => {
                    return Err(MachineError::TypeMismatch {
                        expected: "value".to_string(),
                        got: "message".to_string(),
                    });
                }
            };
            
            // Create content-addressed message
            let msg_id = value.to_message_id();
            state.add_message(msg_id, value);
            
            // Bind message ID to register
            state.bind_register(*msg_reg, Binding::Message(msg_id));
            state.increment_pc();
            Ok(())
        }
        
        Instruction::Consume { msg_reg, value_reg } => {
            // Get message ID from register
            let msg_id = match state.get_binding(*msg_reg)? {
                Binding::Message(id) => *id,
                Binding::Value(_) => {
                    return Err(MachineError::TypeMismatch {
                        expected: "message".to_string(),
                        got: "value".to_string(),
                    });
                }
            };
            
            // Consume message (enforces linearity)
            let value = state.consume_message(msg_id)?;
            
            // Bind value to register and clear message register
            state.bind_register(*value_reg, Binding::Value(value));
            state.clear_register(*msg_reg);
            state.increment_pc();
            Ok(())
        }
        
        Instruction::Send { msg_reg, chan_reg } => {
            // Get message ID
            let msg_id = match state.get_binding(*msg_reg)? {
                Binding::Message(id) => *id,
                Binding::Value(_) => {
                    return Err(MachineError::TypeMismatch {
                        expected: "message".to_string(),
                        got: "value".to_string(),
                    });
                }
            };
            
            // Get channel ID
            let chan_id = match state.get_binding(*chan_reg)? {
                Binding::Value(MessageValue::Channel(id)) => *id,
                _ => {
                    return Err(MachineError::TypeMismatch {
                        expected: "channel".to_string(),
                        got: "other".to_string(),
                    });
                }
            };
            
            // Send message through channel
            state.send_message(chan_id, msg_id);
            state.clear_register(*msg_reg); // Message is sent, no longer accessible
            state.increment_pc();
            Ok(())
        }
        
        Instruction::Receive { chan_reg, msg_reg } => {
            // Get channel ID
            let chan_id = match state.get_binding(*chan_reg)? {
                Binding::Value(MessageValue::Channel(id)) => *id,
                _ => {
                    return Err(MachineError::TypeMismatch {
                        expected: "channel".to_string(),
                        got: "other".to_string(),
                    });
                }
            };
            
            // Receive message from channel
            match state.receive_message(chan_id) {
                Some(msg_id) => {
                    state.bind_register(*msg_reg, Binding::Message(msg_id));
                    state.increment_pc();
                    Ok(())
                }
                None => {
                    Err(MachineError::InvalidOperation(
                        "No message available on channel".to_string()
                    ))
                }
            }
        }
        
        Instruction::Match { 
            msg_reg, 
            left_reg, 
            right_reg,
            left_target,
            right_target 
        } => {
            // Get message ID
            let msg_id = match state.get_binding(*msg_reg)? {
                Binding::Message(id) => *id,
                Binding::Value(_) => {
                    return Err(MachineError::TypeMismatch {
                        expected: "message".to_string(),
                        got: "value".to_string(),
                    });
                }
            };
            
            // Consume message to inspect
            let value = state.consume_message(msg_id)?;
            
            // Pattern match on sum type
            match value {
                MessageValue::Sum(variant, inner) => {
                    match variant {
                        SumVariant::Left => {
                            state.bind_register(*left_reg, Binding::Value(*inner));
                            state.set_pc(*left_target);
                        }
                        SumVariant::Right => {
                            state.bind_register(*right_reg, Binding::Value(*inner));
                            state.set_pc(*right_target);
                        }
                    }
                    state.clear_register(*msg_reg);
                    Ok(())
                }
                _ => Err(MachineError::TypeMismatch {
                    expected: "sum".to_string(),
                    got: "other".to_string(),
                })
            }
        }
    }
}
