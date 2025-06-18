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
    Choice {
        chooser: AgentId,
        branches: Vec<(String, Choreography)>,
    },
    
    /// Conditional execution
    If {
        condition: Condition,
        then_branch: Box<Choreography>,
        else_branch: Option<Box<Choreography>>,
    },
    
    /// Loop
    While {
        condition: Condition,
        body: Box<Choreography>,
    },
    
    /// Empty choreography (no-op)
    Empty,
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
    
    /// Agent performs a local computation
    Local {
        agent: AgentId,
        action: LocalAction,
    },
    
    /// Agent creates a new agent
    Spawn {
        creator: AgentId,
        new_agent: AgentId,
        agent_type: String,
    },
    
    /// Agent delegates capability to another
    Delegate {
        from: AgentId,
        to: AgentId,
        capability: String, // Simplified for parsing
    },
}

/// Message content
#[derive(Debug, Clone)]
pub enum Message {
    /// Simple text message
    Text(String),
    
    /// Structured message with type and value
    Typed {
        msg_type: String,
        value: Value,
    },
    
    /// Request message expecting response
    Request {
        request_type: String,
        payload: Box<Message>,
    },
    
    /// Response to a request
    Response {
        response_type: String,
        payload: Box<Message>,
    },
}

/// Local action performed by an agent
#[derive(Debug, Clone)]
pub enum LocalAction {
    /// Update local state
    UpdateState {
        key: String,
        value: String,
    },
    
    /// Compute a value
    Compute {
        operation: String,
        args: Vec<String>,
    },
    
    /// Validate something
    Validate {
        what: String,
    },
    
    /// Log a message
    Log(String),
}

/// Condition for conditional choreographies
#[derive(Debug, Clone)]
pub enum Condition {
    /// Check agent's local state
    StateEquals {
        agent: AgentId,
        key: String,
        value: String,
    },
    
    /// Check if agent has capability
    HasCapability {
        agent: AgentId,
        capability: String,
    },
    
    /// Boolean operations
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
    
    /// Always true/false
    True,
    False,
}

/// Simple choreography parser
pub struct ChoreographyParser;

impl Default for ChoreographyParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ChoreographyParser {
    pub fn new() -> Self {
        ChoreographyParser
    }
    
    /// Parse a simple choreography from text
    /// Format: "Alice sends PaymentRequest to Bob"
    ///         "Bob validates payment"
    ///         "Bob sends Payment to Alice"
    pub fn parse_simple(&self, text: &str) -> Result<Choreography, ParseError> {
        let lines: Vec<&str> = text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        
        if lines.is_empty() {
            return Ok(Choreography::Empty);
        }
        
        let mut steps = Vec::new();
        
        for line in lines {
            if let Some(step) = self.parse_line(line)? {
                steps.push(Choreography::Step(step));
            }
        }
        
        match steps.len() {
            0 => Ok(Choreography::Empty),
            1 => Ok(steps.into_iter().next().unwrap()),
            _ => Ok(Choreography::Sequence(steps)),
        }
    }
    
    /// Parse a single line into a choreography step
    fn parse_line(&self, line: &str) -> Result<Option<ChoreographyStep>, ParseError> {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        if tokens.is_empty() {
            return Ok(None);
        }
        
        // Pattern: "Alice sends MESSAGE to Bob"
        if tokens.len() >= 4 && tokens[1] == "sends" && tokens.contains(&"to") {
            let from = AgentId::new(tokens[0]);
            let to_idx = tokens.iter().position(|&t| t == "to").unwrap();
            let to = AgentId::new(tokens[to_idx + 1]);
            
            let message_tokens = &tokens[2..to_idx];
            let message = Message::Text(message_tokens.join(" "));
            
            return Ok(Some(ChoreographyStep::Send { from, to, message }));
        }
        
        // Pattern: "Alice validates SOMETHING"
        if tokens.len() >= 2 && tokens[1] == "validates" {
            let agent = AgentId::new(tokens[0]);
            let what = tokens[2..].join(" ");
            
            return Ok(Some(ChoreographyStep::Local {
                agent,
                action: LocalAction::Validate { what },
            }));
        }
        
        // Pattern: "Alice logs MESSAGE"
        if tokens.len() >= 2 && tokens[1] == "logs" {
            let agent = AgentId::new(tokens[0]);
            let message = tokens[2..].join(" ");
            
            return Ok(Some(ChoreographyStep::Local {
                agent,
                action: LocalAction::Log(message),
            }));
        }
        
        // Pattern: "Alice spawns Bob as TYPE"
        if tokens.len() >= 4 && tokens[1] == "spawns" && tokens[3] == "as" {
            let creator = AgentId::new(tokens[0]);
            let new_agent = AgentId::new(tokens[2]);
            let agent_type = tokens[4..].join(" ");
            
            return Ok(Some(ChoreographyStep::Spawn {
                creator,
                new_agent,
                agent_type,
            }));
        }
        
        Err(ParseError {
            line: line.to_string(),
            reason: "Unrecognized choreography pattern".to_string(),
        })
    }
}

/// Parse error
#[derive(Debug)]
pub struct ParseError {
    pub line: String,
    pub reason: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error in '{}': {}", self.line, self.reason)
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_send() {
        let parser = ChoreographyParser::new();
        let choreo = parser.parse_simple("Alice sends Hello to Bob").unwrap();
        
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
        let parser = ChoreographyParser::new();
        let text = r#"
            Alice sends PaymentRequest to Bob
            Bob validates payment
            Bob sends Payment to Alice
            Alice logs Payment received
        "#;
        
        let choreo = parser.parse_simple(text).unwrap();
        
        match choreo {
            Choreography::Sequence(steps) => {
                assert_eq!(steps.len(), 4);
            }
            _ => panic!("Expected Sequence"),
        }
    }
    
    #[test]
    fn test_local_actions() {
        let parser = ChoreographyParser::new();
        
        let validate = parser.parse_simple("Alice validates request").unwrap();
        match validate {
            Choreography::Step(ChoreographyStep::Local { agent, action }) => {
                assert_eq!(agent, AgentId::new("Alice"));
                match action {
                    LocalAction::Validate { what } => assert_eq!(what, "request"),
                    _ => panic!("Expected Validate action"),
                }
            }
            _ => panic!("Expected Local step"),
        }
        
        let log = parser.parse_simple("Bob logs Transaction complete").unwrap();
        match log {
            Choreography::Step(ChoreographyStep::Local { agent, action }) => {
                assert_eq!(agent, AgentId::new("Bob"));
                match action {
                    LocalAction::Log(msg) => assert_eq!(msg, "Transaction complete"),
                    _ => panic!("Expected Log action"),
                }
            }
            _ => panic!("Expected Local step"),
        }
    }
    
    #[test]
    fn test_spawn() {
        let parser = ChoreographyParser::new();
        let choreo = parser.parse_simple("Alice spawns Worker as compute agent").unwrap();
        
        match choreo {
            Choreography::Step(ChoreographyStep::Spawn { creator, new_agent, agent_type }) => {
                assert_eq!(creator, AgentId::new("Alice"));
                assert_eq!(new_agent, AgentId::new("Worker"));
                assert_eq!(agent_type, "compute agent");
            }
            _ => panic!("Expected Spawn step"),
        }
    }
}
