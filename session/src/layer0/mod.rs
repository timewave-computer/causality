// Layer 0: Content-Addressed Message Machine
// The foundational layer providing minimal, deterministic execution

pub mod content;
pub mod machine;
pub mod instruction;

// Re-export key types
pub use content::MessageId;
pub use machine::{
    MessageValue, MachineState, MachineError, 
    Register, Binding, ChannelId, SumVariant
};
pub use instruction::{Instruction, execute_instruction};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_instruction() {
        let mut state = MachineState::new();
        
        // Put a value in register 0
        state.bind_register(Register(0), Binding::Value(MessageValue::Int(42)));
        
        // Create message from value
        let inst = Instruction::Create { 
            value_reg: Register(0), 
            msg_reg: Register(1) 
        };
        
        execute_instruction(&mut state, &inst).unwrap();
        
        // Check that message was created
        match state.get_binding(Register(1)).unwrap() {
            Binding::Message(id) => {
                assert!(state.has_message(id));
            }
            _ => panic!("Expected message binding"),
        }
    }
    
    #[test]
    fn test_consume_enforces_linearity() {
        let mut state = MachineState::new();
        
        // Create a message
        let value = MessageValue::Int(100);
        let msg_id = value.to_message_id();
        state.add_message(msg_id, value);
        state.bind_register(Register(0), Binding::Message(msg_id));
        
        // Consume the message
        let inst = Instruction::Consume {
            msg_reg: Register(0),
            value_reg: Register(1),
        };
        
        execute_instruction(&mut state, &inst).unwrap();
        
        // Check value was extracted
        match state.get_binding(Register(1)).unwrap() {
            Binding::Value(MessageValue::Int(100)) => {}
            _ => panic!("Expected value 100"),
        }
        
        // Check message no longer exists (linear consumption)
        assert!(!state.has_message(&msg_id));
        
        // Check source register was cleared
        assert!(state.get_binding(Register(0)).is_err());
    }
    
    #[test]
    fn test_send_receive() {
        let mut state = MachineState::new();
        let channel = ChannelId(1);
        
        // Setup: create message and channel
        let value = MessageValue::Bool(true);
        let msg_id = value.to_message_id();
        state.add_message(msg_id, value);
        state.bind_register(Register(0), Binding::Message(msg_id));
        state.bind_register(Register(1), Binding::Value(MessageValue::Channel(channel)));
        
        // Send message
        let send_inst = Instruction::Send {
            msg_reg: Register(0),
            chan_reg: Register(1),
        };
        execute_instruction(&mut state, &send_inst).unwrap();
        
        // Receive message
        let recv_inst = Instruction::Receive {
            chan_reg: Register(1),
            msg_reg: Register(2),
        };
        execute_instruction(&mut state, &recv_inst).unwrap();
        
        // Check received message
        match state.get_binding(Register(2)).unwrap() {
            Binding::Message(id) => {
                assert_eq!(*id, msg_id);
            }
            _ => panic!("Expected message binding"),
        }
    }
    
    #[test]
    fn test_match_sum() {
        let mut state = MachineState::new();
        
        // Create a Left sum
        let sum_value = MessageValue::Sum(
            SumVariant::Left, 
            Box::new(MessageValue::Int(42))
        );
        let msg_id = sum_value.to_message_id();
        state.add_message(msg_id, sum_value);
        state.bind_register(Register(0), Binding::Message(msg_id));
        
        // Match on sum
        let inst = Instruction::Match {
            msg_reg: Register(0),
            left_reg: Register(1),
            right_reg: Register(2),
            left_target: 10,
            right_target: 20,
        };
        
        execute_instruction(&mut state, &inst).unwrap();
        
        // Check we took left branch
        assert_eq!(state.pc(), 10);
        
        // Check left register has value
        match state.get_binding(Register(1)).unwrap() {
            Binding::Value(MessageValue::Int(42)) => {}
            _ => panic!("Expected Int(42) in left register"),
        }
    }
    
    #[test]
    fn test_program_execution() {
        let mut state = MachineState::new();
        
        // Setup initial values
        state.bind_register(Register(0), Binding::Value(MessageValue::Int(10)));
        state.bind_register(Register(1), Binding::Value(MessageValue::Int(20)));
        
        let program = vec![
            // Create messages from values
            Instruction::Create { value_reg: Register(0), msg_reg: Register(2) },
            Instruction::Create { value_reg: Register(1), msg_reg: Register(3) },
            // Consume first message
            Instruction::Consume { msg_reg: Register(2), value_reg: Register(4) },
        ];
        
        state.execute_program(&program).unwrap();
        
        // Check final state
        assert_eq!(state.pc(), 3); // After 3 instructions
        match state.get_binding(Register(4)).unwrap() {
            Binding::Value(MessageValue::Int(10)) => {}
            _ => panic!("Expected Int(10) in register 4"),
        }
    }
}
