// Unified Valence-Causality Architecture Implementation
// A minimal prototype demonstrating all four layers of the architecture

pub mod layer0;
pub mod layer1;
pub mod layer2;
pub mod layer3;
pub mod interpreter;
pub mod ir;  // New IR module for content-addressed representation
pub mod blockchain;  // Blockchain-native types with linear semantics

// Re-export key types from each layer
pub use layer0::{MessageId, MessageValue, MachineState};
pub use layer1::{Type, SessionType, Term};
pub use layer2::{Outcome, StateTransition, Effect};
pub use layer3::{Agent, AgentId, Choreography, Capability};

// Re-export blockchain types
pub use blockchain::{Mailbox, Token, StateDiff};

// Remove the unused add function
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    
    #[test] 
    fn test_deterministic_collections() {
        // Test that our BTreeMap usage gives deterministic iteration
        let mut map1 = BTreeMap::new();
        map1.insert("c", 3);
        map1.insert("a", 1);
        map1.insert("b", 2);
        
        let mut map2 = BTreeMap::new();
        map2.insert("b", 2);
        map2.insert("c", 3);
        map2.insert("a", 1);
        
        // Both maps should iterate in the same order: a, b, c
        let keys1: Vec<_> = map1.keys().collect();
        let keys2: Vec<_> = map2.keys().collect();
        
        assert_eq!(keys1, keys2);
        assert_eq!(keys1, vec![&"a", &"b", &"c"]);
    }
}
