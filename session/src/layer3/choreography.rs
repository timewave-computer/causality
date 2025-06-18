// Layer 3 Choreography language - multi-party interaction patterns

use crate::layer3::agent::AgentId;
use crate::layer2::outcome::Value;

/// A choreography describes multi-party interactions
#[derive(Debug, Clone)]
pub enum Choreography {
    /// Single step
    Step(ChoreographyStep),
    
    /// Sequential composition
    Sequence(Vec<Choreography>),
    
    /// Parallel composition
    Parallel(Vec<Choreography>),
    
    /// Choice between alternatives
    Choice(Vec<Choreography>),
}

/// Individual choreography step
#[derive(Debug, Clone)]
pub enum ChoreographyStep {
    /// Agent sends a message to another agent
    Send {
        from: AgentId,
        to: AgentId,
        message: Message,
    },
    
    /// Agent creates a new agent
    Spawn {
        parent: AgentId,
        agent: crate::layer3::agent::Agent,
    },
    
    /// Parallel execution of steps
    Parallel(Vec<ChoreographyStep>),
    
    /// Sequential execution of steps
    Sequence(Vec<ChoreographyStep>),
}

/// Message content
#[derive(Debug, Clone)]
pub enum Message {
    /// Simple text message
    Text(String),
    
    /// Structured data message
    Data(Value),
    
    /// Request message expecting response
    Request(String, Value), // (request_id, value)
    
    /// Response to a request
    Response(String, Value), // (request_id, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_choreography_creation() {
        let step = ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Hello".to_string()),
        };
        
        let choreo = Choreography::Step(step);
        
        match choreo {
            Choreography::Step(ChoreographyStep::Send { from, to, message }) => {
                assert_eq!(from, AgentId::new("Alice"));
                assert_eq!(to, AgentId::new("Bob"));
                match message {
                    Message::Text(msg) => assert_eq!(msg, "Hello"),
                    _ => panic!("Expected text message"),
                }
            }
            _ => panic!("Expected Send step"),
        }
    }
    
    #[test]
    fn test_sequence() {
        let step1 = ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Request".to_string()),
        };
        
        let step2 = ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Text("Response".to_string()),
        };
        
        let choreo = Choreography::Sequence(vec![
            Choreography::Step(step1),
            Choreography::Step(step2),
        ]);
        
        match choreo {
            Choreography::Sequence(steps) => {
                assert_eq!(steps.len(), 2);
            }
            _ => panic!("Expected Sequence"),
        }
    }
    
    #[test]
    fn test_spawn() {
        let new_agent = crate::layer3::agent::Agent::new("Worker");
        let step = ChoreographyStep::Spawn {
            parent: AgentId::new("Alice"),
            agent: new_agent,
        };
        
        let choreo = Choreography::Step(step);
        
        match choreo {
            Choreography::Step(ChoreographyStep::Spawn { parent, agent }) => {
                assert_eq!(parent, AgentId::new("Alice"));
                assert_eq!(agent.id, AgentId::new("Worker"));
            }
            _ => panic!("Expected Spawn step"),
        }
    }
}
