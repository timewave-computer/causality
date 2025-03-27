// Actor system framework
//
// This module provides an actor-based concurrency framework following the actor model paradigm.
// Actors are isolated, concurrent units of computation that communicate exclusively via message passing.

// Core submodules
pub mod message;
pub mod address;
pub mod mailbox;
pub mod context;
pub mod system;
pub mod supervisor;

// Re-exports of key types from submodules
pub use message::{Message, MessageHandler, HandleResult};
pub use address::{Address, AddressResolver};
pub use context::{ActorContext, BasicActorContext, HierarchicalActorContext, SpawnOptions};
pub use system::{ActorSystem, SystemConfig};
pub use supervisor::{Supervisor, SupervisionDecision, SupervisionStrategy};

/// A trait for actors
pub trait Actor: Send + 'static {
    /// The type of context this actor uses
    type Context: ActorContext;
    
    /// Initialize the actor
    fn initialize(&mut self, context: &mut Self::Context);
    
    /// Called when the actor is stopped
    fn on_stop(&mut self, context: &mut Self::Context);
    
    /// Called when the actor is restarted
    fn on_restart(&mut self, context: &mut Self::Context);
}

/// A trait for actor factories
pub trait ActorFactory<A: Actor>: Send + 'static {
    /// Create a new instance of the actor
    fn create(&self) -> A;
}

/// A trait for message dispatchers
pub trait MessageDispatcher<M: Message>: Send + Sync + 'static {
    /// Dispatch a message to an actor
    fn dispatch(&self, message: M, context: &mut dyn ActorContext) -> HandleResult<M::Response>;
}

/// A trait for actor references
pub trait ActorRef<M: Message>: Send + Sync + 'static {
    /// Send a message to the actor
    fn send(&self, message: M) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Stop the actor
    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Restart the actor
    fn restart(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Lifecycle events for actors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// Actor is being initialized
    Initialize,
    
    /// Actor is being started
    Start,
    
    /// Actor is being stopped
    Stop,
    
    /// Actor is being restarted
    Restart,
}

/// The result of a lifecycle event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleResult {
    /// The event was handled successfully
    Success,
    
    /// The event handling failed
    Failure,
}

/// The status of an actor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorStatus {
    /// Actor is being initialized
    Initializing,
    
    /// Actor is running
    Running,
    
    /// Actor is stopping
    Stopping,
    
    /// Actor is stopped
    Stopped,
    
    /// Actor has failed
    Failed,
}

/// Configuration for an actor
#[derive(Debug, Clone)]
pub struct ActorConfig {
    /// The name of the actor
    pub name: String,
    
    /// The mailbox capacity
    pub mailbox_capacity: mailbox::MailboxCapacity,
    
    /// Whether the actor should restart on failure
    pub restart_on_failure: bool,
    
    /// The maximum number of restart attempts
    pub max_restart_count: Option<usize>,
    
    /// Whether the actor should stop when the parent stops
    pub stop_with_parent: bool,
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            mailbox_capacity: mailbox::MailboxCapacity::Bounded(100),
            restart_on_failure: true,
            max_restart_count: None,
            stop_with_parent: true,
        }
    }
}

impl ActorConfig {
    /// Create a new actor configuration with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    
    /// Set the mailbox capacity
    pub fn with_mailbox_capacity(mut self, capacity: mailbox::MailboxCapacity) -> Self {
        self.mailbox_capacity = capacity;
        self
    }
    
    /// Set whether the actor should restart on failure
    pub fn with_restart_on_failure(mut self, restart: bool) -> Self {
        self.restart_on_failure = restart;
        self
    }
    
    /// Set the maximum number of restart attempts
    pub fn with_max_restart_count(mut self, count: Option<usize>) -> Self {
        self.max_restart_count = count;
        self
    }
    
    /// Set whether the actor should stop when the parent stops
    pub fn with_stop_with_parent(mut self, stop: bool) -> Self {
        self.stop_with_parent = stop;
        self
    }
}

/// Helper functions for working with actors
pub mod helpers {
    use super::*;
    
    /// Create a simple actor
    pub fn simple_actor<F, C>(
        name: impl Into<String>,
        initialize: F,
    ) -> impl Actor<Context = C>
    where
        F: FnOnce(&mut C) + Send + 'static,
        C: ActorContext,
    {
        struct SimpleActor<F, C> {
            name: String,
            initialize: Option<F>,
            _marker: std::marker::PhantomData<C>,
        }
        
        impl<F, C> Actor for SimpleActor<F, C>
        where
            F: FnOnce(&mut C) + Send + 'static,
            C: ActorContext,
        {
            type Context = C;
            
            fn initialize(&mut self, context: &mut Self::Context) {
                if let Some(initialize) = self.initialize.take() {
                    initialize(context);
                }
            }
            
            fn on_stop(&mut self, _context: &mut Self::Context) {
                // Nothing to do
            }
            
            fn on_restart(&mut self, _context: &mut Self::Context) {
                // Nothing to do
            }
        }
        
        SimpleActor {
            name: name.into(),
            initialize: Some(initialize),
            _marker: std::marker::PhantomData,
        }
    }
    
    /// Create a supervisor actor
    pub fn supervisor_actor<A, C>(
        name: impl Into<String>,
        supervisor: impl Supervisor<A> + 'static,
    ) -> impl Actor<Context = C>
    where
        A: Actor,
        C: ActorContext,
    {
        struct SupervisorActor<A, C, S> {
            name: String,
            supervisor: S,
            _marker: std::marker::PhantomData<(A, C)>,
        }
        
        impl<A, C, S> Actor for SupervisorActor<A, C, S>
        where
            A: Actor,
            C: ActorContext,
            S: Supervisor<A> + 'static,
        {
            type Context = C;
            
            fn initialize(&mut self, _context: &mut Self::Context) {
                // Nothing to do
            }
            
            fn on_stop(&mut self, _context: &mut Self::Context) {
                // Nothing to do
            }
            
            fn on_restart(&mut self, _context: &mut Self::Context) {
                // Nothing to do
            }
        }
        
        SupervisorActor {
            name: name.into(),
            supervisor,
            _marker: std::marker::PhantomData,
        }
    }
    
    /// Create an actor system
    pub fn actor_system(name: impl Into<String>) -> impl ActorSystem {
        system::helpers::basic_actor_system(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_actor_config() {
        let config = ActorConfig::new("test");
        assert_eq!(config.name, "test");
        assert_eq!(config.restart_on_failure, true);
        
        let config = config
            .with_mailbox_capacity(mailbox::MailboxCapacity::Bounded(10))
            .with_restart_on_failure(false)
            .with_max_restart_count(Some(3))
            .with_stop_with_parent(false);
        
        assert_eq!(config.name, "test");
        assert_eq!(config.mailbox_capacity, mailbox::MailboxCapacity::Bounded(10));
        assert_eq!(config.restart_on_failure, false);
        assert_eq!(config.max_restart_count, Some(3));
        assert_eq!(config.stop_with_parent, false);
    }
} 