// Agent system management
//
// This module provides the foundation for agent system management.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use thiserror::Error;

use crate::resource::{Resource, ResourceId, ResourceAccessor};
use crate::capability::ContentId;
use crate::agent::address::{Address, AddressResolver, AddressPath, HierarchicalAddress};
use crate::agent::{Agent, AgentContext, AgentFactory, LifecycleEvent, BasicAgentContext};
use crate::agent::message::Message;
use crate::agent::supervisor::SupervisionStrategy;

/// Error type for agent system operations
#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Agent already exists: {0}")]
    AgentAlreadyExists(String),
    
    #[error("Mailbox error: {0}")]
    MailboxError(String),
    
    #[error("Context error: {0}")]
    ContextError(String),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Supervision error: {0}")]
    SupervisionError(String),
    
    #[error("System error: {0}")]
    SystemError(String),

    #[error("Resource error: {0}")]
    ResourceError(String),
}

/// Agent system configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    /// The name of the system
    pub name: String,
    
    /// The number of worker threads for the agent system
    pub worker_threads: usize,
    
    /// Whether to shutdown the system when the last agent stops
    pub shutdown_on_last_agent: bool,
    
    /// Default supervision strategy
    pub default_supervision_strategy: SupervisionStrategy,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: "agent-system".to_string(),
            worker_threads: num_cpus::get(),
            shutdown_on_last_agent: false,
            default_supervision_strategy: SupervisionStrategy::Restart {
                max_retries: Some(3),
                backoff_ms: 100,
            },
        }
    }
}

impl SystemConfig {
    /// Create a new system configuration with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    
    /// Set the number of worker threads
    pub fn with_worker_threads(mut self, threads: usize) -> Self {
        self.worker_threads = threads;
        self
    }
    
    /// Set whether to shutdown on last agent stop
    pub fn with_shutdown_on_last_agent(mut self, shutdown: bool) -> Self {
        self.shutdown_on_last_agent = shutdown;
        self
    }
    
    /// Set the default supervision strategy
    pub fn with_default_supervision_strategy(mut self, strategy: SupervisionStrategy) -> Self {
        self.default_supervision_strategy = strategy;
        self
    }
}

/// A trait for agent systems
pub trait AgentSystem: Send + Sync + 'static {
    /// Start the agent system
    fn start(&self) -> Result<(), SystemError>;
    
    /// Stop the agent system
    fn stop(&self) -> Result<(), SystemError>;
    
    /// Spawn a new agent
    fn spawn<A, F>(&self, factory: F, name: Option<String>) -> Result<Address<A::Context>, SystemError>
    where
        A: Agent,
        F: AgentFactory<A> + 'static;
    
    /// Send a system event to an agent
    fn send_system_event(&self, address: &Address<impl Message>, event: LifecycleEvent) -> Result<(), SystemError>;
    
    /// Get the agent system configuration
    fn config(&self) -> &SystemConfig;
    
    /// Get the address resolver for this system
    fn resolver(&self) -> &dyn AddressResolver<Error = SystemError>;
}

/// Entry for an agent in the system registry
struct AgentEntry {
    /// The resource representation of the agent
    resource: Resource,
    
    /// The context of the agent
    context: Box<dyn AgentContext>,
    
    /// The name of the agent
    name: String,
    
    /// The type of the agent
    agent_type: String,
}

impl AgentEntry {
    /// Create a new agent entry
    fn new(resource: Resource, context: Box<dyn AgentContext>, name: String, agent_type: String) -> Self {
        Self {
            resource,
            context,
            name,
            agent_type,
        }
    }
    
    /// Get the content ID of the agent
    fn content_id(&self) -> ContentId {
        self.resource.content_id()
    }
}

/// Basic agent system implementation
pub struct BasicAgentSystem {
    /// The configuration of the system
    config: SystemConfig,
    
    /// The registry of agents in the system
    agents: Arc<RwLock<HashMap<ContentId, AgentEntry>>>,
    
    /// The resolver for agent addresses
    resolver: Arc<BasicAddressResolver>,
}

/// Basic address resolver implementation
struct BasicAddressResolver {
    /// Map of agent names to their content IDs
    agents_by_name: Arc<RwLock<HashMap<String, ContentId>>>,
    
    /// Map of agent types to vectors of content IDs
    agents_by_type: Arc<RwLock<HashMap<String, Vec<ContentId>>>>,
    
    /// Map of content IDs to agent names
    agents_by_id: Arc<RwLock<HashMap<ContentId, String>>>,
}

impl BasicAddressResolver {
    /// Create a new basic address resolver
    fn new() -> Self {
        Self {
            agents_by_name: Arc::new(RwLock::new(HashMap::new())),
            agents_by_type: Arc::new(RwLock::new(HashMap::new())),
            agents_by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register an agent in the resolver
    fn register_agent(&self, id: ContentId, name: &str, agent_type: &str) {
        // Register by name
        {
            let mut agents_by_name = self.agents_by_name.write().unwrap();
            agents_by_name.insert(name.to_string(), id);
        }
        
        // Register by type
        {
            let mut agents_by_type = self.agents_by_type.write().unwrap();
            let entry = agents_by_type.entry(agent_type.to_string()).or_insert_with(Vec::new);
            entry.push(id);
        }
        
        // Register by ID
        {
            let mut agents_by_id = self.agents_by_id.write().unwrap();
            agents_by_id.insert(id, name.to_string());
        }
    }
    
    /// Unregister an agent from the resolver
    fn unregister_agent(&self, id: ContentId) {
        // Find the name and type
        let name = {
            let agents_by_id = self.agents_by_id.read().unwrap();
            agents_by_id.get(&id).cloned()
        };
        
        if let Some(name) = name {
            // Unregister by name
            {
                let mut agents_by_name = self.agents_by_name.write().unwrap();
                agents_by_name.remove(&name);
            }
            
            // Unregister by ID
            {
                let mut agents_by_id = self.agents_by_id.write().unwrap();
                agents_by_id.remove(&id);
            }
            
            // Unregister by type (requires finding all types that contain this id)
            {
                let mut agents_by_type = self.agents_by_type.write().unwrap();
                
                for (_type, ids) in agents_by_type.iter_mut() {
                    ids.retain(|i| *i != id);
                }
                
                // Remove empty type entries
                agents_by_type.retain(|_, ids| !ids.is_empty());
            }
        }
    }
}

impl AddressResolver for BasicAddressResolver {
    type Error = SystemError;
    
    fn resolve_by_name<M: Message>(&self, name: &str) -> Result<Address<M>, Self::Error> {
        let agents_by_name = self.agents_by_name.read().unwrap();
        let id = agents_by_name.get(name).ok_or_else(|| SystemError::AgentNotFound(name.to_string()))?;
        
        // Get the type
        let agents_by_id = self.agents_by_id.read().unwrap();
        let agent_name = agents_by_id.get(id).ok_or_else(|| SystemError::InvalidAddress(format!("{:?}", id)))?;
        
        // Get the agent type (in a real implementation, we'd look this up properly)
        let agent_type = "agent";
        
        Ok(Address::new(agent_name, agent_type))
    }
    
    fn resolve_by_hash<M: Message>(&self, hash: ContentId) -> Result<Address<M>, Self::Error> {
        let agents_by_id = self.agents_by_id.read().unwrap();
        let name = agents_by_id.get(&hash).ok_or_else(|| SystemError::AgentNotFound(format!("{:?}", hash)))?;
        
        // Get the agent type (in a real implementation, we'd look this up properly)
        let agent_type = "agent";
        
        Ok(Address::new(name, agent_type))
    }
    
    fn find_by_type<M: Message>(&self, agent_type: &str) -> Result<Vec<Address<M>>, Self::Error> {
        let agents_by_type = self.agents_by_type.read().unwrap();
        let ids = agents_by_type.get(agent_type).cloned().unwrap_or_default();
        
        let mut addresses = Vec::new();
        
        for id in ids {
            if let Ok(addr) = self.resolve_by_hash::<M>(id) {
                addresses.push(addr);
            }
        }
        
        Ok(addresses)
    }
    
    fn exists(&self, address: &Address<impl Message>) -> bool {
        let id = address.content_id();
        let agents_by_id = self.agents_by_id.read().unwrap();
        agents_by_id.contains_key(&id)
    }
}

impl BasicAgentSystem {
    /// Create a new basic agent system
    pub fn new(config: SystemConfig) -> Self {
        let resolver = Arc::new(BasicAddressResolver::new());
        
        Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
            resolver,
        }
    }
}

impl AgentSystem for BasicAgentSystem {
    fn start(&self) -> Result<(), SystemError> {
        // In a real implementation, we would start worker threads here
        Ok(())
    }
    
    fn stop(&self) -> Result<(), SystemError> {
        // In a real implementation, we would stop worker threads here
        Ok(())
    }
    
    fn spawn<A, F>(&self, factory: F, name: Option<String>) -> Result<Address<A::Context>, SystemError>
    where
        A: Agent,
        F: AgentFactory<A> + 'static,
    {
        let name = name.unwrap_or_else(|| format!("agent-{:x}", rand::random::<u64>()));
        let agent_type = std::any::type_name::<A>();
        
        // Create the resource for the agent
        let resource = Resource::new_with_type("Agent");
        
        // Create the agent and its context
        let mut agent = factory.create();
        let context = Box::new(BasicAgentContext::new(name.clone(), agent_type.to_string()));
        
        // Initialize the agent
        agent.initialize(&mut *context);
        
        // Create an address for the agent
        let address = Address::<A::Context>::new(&name, agent_type);
        let content_id = resource.content_id();
        
        // Register the agent
        {
            let mut agents = self.agents.write().unwrap();
            let entry = AgentEntry::new(resource, context, name.clone(), agent_type.to_string());
            
            agents.insert(content_id, entry);
        }
        
        // Register with the resolver
        self.resolver.register_agent(content_id, &name, agent_type);
        
        Ok(address)
    }
    
    fn send_system_event(&self, address: &Address<impl Message>, event: LifecycleEvent) -> Result<(), SystemError> {
        let id = address.content_id();
        
        let agents = self.agents.read().unwrap();
        let entry = agents.get(&id).ok_or_else(|| SystemError::AgentNotFound(address.name().to_string()))?;
        
        // Send the event to the agent's context
        match event {
            LifecycleEvent::Initialize => {
                // Agent is already initialized during spawn
                Ok(())
            },
            LifecycleEvent::Start => {
                // In a real implementation, we would start processing messages
                Ok(())
            },
            LifecycleEvent::Stop => {
                // In a real implementation, we would stop the agent
                Ok(())
            },
            LifecycleEvent::Restart => {
                // In a real implementation, we would restart the agent
                Ok(())
            },
        }
    }
    
    fn config(&self) -> &SystemConfig {
        &self.config
    }
    
    fn resolver(&self) -> &dyn AddressResolver<Error = SystemError> {
        self.resolver.as_ref()
    }
}

/// Hierarchical agent system implementation that supports parent-child relationships
pub struct HierarchicalAgentSystem {
    /// The base agent system
    base: BasicAgentSystem,
    
    /// Resolver for hierarchical addresses
    hierarchical_resolver: Arc<HierarchicalAddressResolver>,
}

/// Hierarchical address resolver implementation
struct HierarchicalAddressResolver {
    /// The base address resolver
    base: Arc<BasicAddressResolver>,
    
    /// Map of agent paths to their content IDs
    agents_by_path: Arc<RwLock<HashMap<String, ContentId>>>,
}

impl HierarchicalAddressResolver {
    /// Create a new hierarchical address resolver
    fn new(base: Arc<BasicAddressResolver>) -> Self {
        Self {
            base,
            agents_by_path: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl HierarchicalAgentSystem {
    /// Create a new hierarchical agent system
    pub fn new(config: SystemConfig) -> Self {
        let base = BasicAgentSystem::new(config);
        let hierarchical_resolver = Arc::new(HierarchicalAddressResolver::new(base.resolver.clone()));
        
        Self {
            base,
            hierarchical_resolver,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestAgent;
    
    impl Agent for TestAgent {
        type Context = BasicAgentContext;
        
        fn initialize(&mut self, _context: &mut Self::Context) {
            // Initialize the agent
        }
        
        fn on_stop(&mut self, _context: &mut Self::Context) {
            // Handle agent stop
        }
        
        fn on_restart(&mut self, _context: &mut Self::Context) {
            // Handle agent restart
        }
    }
    
    struct TestAgentFactory;
    
    impl AgentFactory<TestAgent> for TestAgentFactory {
        fn create(&self) -> TestAgent {
            TestAgent
        }
    }
    
    #[test]
    fn test_basic_agent_system() {
        let config = SystemConfig::new("test-system");
        let system = BasicAgentSystem::new(config);
        
        // Start the system
        system.start().unwrap();
        
        // Spawn an agent
        let factory = TestAgentFactory;
        let address = system.spawn::<TestAgent, _>(factory, Some("test-agent".to_string())).unwrap();
        
        // Check that the agent exists
        assert!(system.resolver().exists(&address));
        
        // Resolve the agent by name
        let resolved = system.resolver().resolve_by_name::<BasicAgentContext>("test-agent").unwrap();
        assert_eq!(resolved.name(), "test-agent");
        
        // Resolve the agent by content hash
        let resolved = system.resolver().resolve_by_hash::<BasicAgentContext>(address.content_id()).unwrap();
        assert_eq!(resolved.name(), "test-agent");
        
        // Stop the system
        system.stop().unwrap();
    }
} 