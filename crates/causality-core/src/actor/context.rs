// Actor context implementation
//
// This module provides the execution context for actors.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::actor::address::{Address, HierarchicalAddress};
use crate::actor::message::{Message, MessageEnvelope, MessagePriority, SendOptions};
use crate::actor::mailbox::MailboxCapacity;
use crate::actor::{Actor, ActorConfig, ActorStatus, LifecycleEvent};
use crate::time::Duration as TimeDuration;
use crate::time::Timestamp;
use crate::crypto::{ContentId, HashFactory};

/// Options for spawning an actor
#[derive(Debug, Clone)]
pub struct SpawnOptions {
    /// The configuration for the actor
    pub config: ActorConfig,
    
    /// The dispatcher to use for messages
    pub dispatcher: Option<String>,
    
    /// The strategy to use when the mailbox is full
    pub mailbox_full_strategy: MailboxFullStrategy,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            config: ActorConfig::default(),
            dispatcher: None,
            mailbox_full_strategy: MailboxFullStrategy::Drop,
        }
    }
}

impl SpawnOptions {
    /// Create a new set of spawn options with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: ActorConfig::new(name),
            ..Default::default()
        }
    }
    
    /// Set the mailbox capacity
    pub fn with_mailbox_capacity(mut self, capacity: MailboxCapacity) -> Self {
        self.config.mailbox_capacity = capacity;
        self
    }
    
    /// Set whether the actor should restart on failure
    pub fn with_restart_on_failure(mut self, restart: bool) -> Self {
        self.config.restart_on_failure = restart;
        self
    }
    
    /// Set the maximum number of restart attempts
    pub fn with_max_restart_count(mut self, count: Option<usize>) -> Self {
        self.config.max_restart_count = count;
        self
    }
    
    /// Set whether the actor should stop when the parent stops
    pub fn with_stop_with_parent(mut self, stop: bool) -> Self {
        self.config.stop_with_parent = stop;
        self
    }
    
    /// Set the dispatcher to use for messages
    pub fn with_dispatcher(mut self, dispatcher: impl Into<String>) -> Self {
        self.dispatcher = Some(dispatcher.into());
        self
    }
    
    /// Set the strategy to use when the mailbox is full
    pub fn with_mailbox_full_strategy(mut self, strategy: MailboxFullStrategy) -> Self {
        self.mailbox_full_strategy = strategy;
        self
    }
}

/// Strategy for handling messages when the mailbox is full
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailboxFullStrategy {
    /// Drop the new message
    Drop,
    
    /// Block until there is space in the mailbox
    Block,
    
    /// Replace the oldest message in the mailbox
    ReplaceOldest,
    
    /// Replace the lowest priority message in the mailbox
    ReplaceLowestPriority,
}

/// A trait for actor contexts
pub trait ActorContext: Send + 'static {
    /// The address type for this context
    type Address;
    
    /// Get the address of the actor
    fn address(&self) -> Self::Address;
    
    /// Get the path of the actor in the hierarchy
    fn path(&self) -> String;
    
    /// Get the parent of the actor
    fn parent(&self) -> Option<Self::Address>;
    
    /// Get the configuration of the actor
    fn config(&self) -> &ActorConfig;
    
    /// Get the current status of the actor
    fn status(&self) -> ActorStatus;
    
    /// Stop the actor
    fn stop(&mut self);
    
    /// Restart the actor
    fn restart(&mut self);
    
    /// Schedule a message to be sent after a delay
    fn schedule<M: Message>(
        &self,
        recipient: Address<M>,
        message: M,
        delay: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Schedule a message to be sent periodically
    fn schedule_periodic<M: Message>(
        &self,
        recipient: Address<M>,
        message: M,
        initial_delay: Duration,
        interval: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Cancel a scheduled message
    fn cancel_schedule(&self, schedule_id: &str) -> bool;
    
    /// Spawn a child actor
    fn spawn<A: Actor>(
        &self,
        actor: A,
        options: SpawnOptions,
    ) -> Result<Address<A::Context>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Watch an actor for termination
    fn watch(&self, address: &Address<impl Message>) -> bool;
    
    /// Unwatch an actor
    fn unwatch(&self, address: &Address<impl Message>) -> bool;
    
    /// Set a state value
    fn set_state<T: Any + Send + Sync>(&mut self, key: &str, value: T);
    
    /// Get a state value
    fn get_state<T: Any + Send + Sync>(&self, key: &str) -> Option<&T>;
    
    /// Get a mutable reference to a state value
    fn get_state_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T>;
    
    /// Remove a state value
    fn remove_state(&mut self, key: &str) -> Option<Box<dyn Any + Send + Sync>>;
    
    /// Log a message at the debug level
    fn debug(&self, message: impl Into<String>);
    
    /// Log a message at the info level
    fn info(&self, message: impl Into<String>);
    
    /// Log a message at the warning level
    fn warn(&self, message: impl Into<String>);
    
    /// Log a message at the error level
    fn error(&self, message: impl Into<String>);
}

/// A basic implementation of actor context
pub struct BasicActorContext<M: Message> {
    /// The address of the actor
    address: Address<M>,
    
    /// The path of the actor in the hierarchy
    path: String,
    
    /// The parent of the actor
    parent: Option<Address<M>>,
    
    /// The configuration of the actor
    config: ActorConfig,
    
    /// The current status of the actor
    status: ActorStatus,
    
    /// State values
    state: HashMap<String, Box<dyn Any + Send + Sync>>,
    
    /// Scheduled messages
    schedules: Vec<ScheduledMessage>,
    
    /// Watched actors
    watched: Vec<WatchedActor>,
    
    /// The system context
    system: Arc<Mutex<SystemContext>>,
}

/// A scheduled message
struct ScheduledMessage {
    /// The ID of the schedule
    id: String,
    
    /// The time to send the message
    time: Timestamp,
    
    /// The interval for periodic messages
    interval: Option<TimeDuration>,
    
    /// The message envelope
    envelope: Box<dyn Any + Send + Sync>,
    
    /// The recipient address
    recipient: Box<dyn Any + Send + Sync>,
}

/// A watched actor
struct WatchedActor {
    /// The address of the watched actor
    address: Box<dyn Any + Send + Sync>,
    
    /// Whether the actor is terminated
    terminated: bool,
}

/// A system context shared by all actors
struct SystemContext {
    // System-wide resources and services would go here
}

impl<M: Message> BasicActorContext<M> {
    /// Create a new basic actor context
    pub fn new(
        address: Address<M>,
        path: String,
        parent: Option<Address<M>>,
        config: ActorConfig,
        system: Arc<Mutex<SystemContext>>,
    ) -> Self {
        Self {
            address,
            path,
            parent,
            config,
            status: ActorStatus::Initializing,
            state: HashMap::new(),
            schedules: Vec::new(),
            watched: Vec::new(),
            system,
        }
    }
    
    /// Set the status of the actor
    pub fn set_status(&mut self, status: ActorStatus) {
        self.status = status;
    }
    
    /// Process a lifecycle event
    pub fn process_lifecycle_event(&mut self, event: LifecycleEvent) {
        match event {
            LifecycleEvent::Initialize => {
                self.set_status(ActorStatus::Initializing);
            }
            LifecycleEvent::Start => {
                self.set_status(ActorStatus::Running);
            }
            LifecycleEvent::Stop => {
                self.set_status(ActorStatus::Stopping);
            }
            LifecycleEvent::Restart => {
                self.set_status(ActorStatus::Initializing);
            }
        }
    }
}

impl<M: Message> ActorContext for BasicActorContext<M> {
    type Address = Address<M>;
    
    fn address(&self) -> Self::Address {
        self.address.clone()
    }
    
    fn path(&self) -> String {
        self.path.clone()
    }
    
    fn parent(&self) -> Option<Self::Address> {
        self.parent.clone()
    }
    
    fn config(&self) -> &ActorConfig {
        &self.config
    }
    
    fn status(&self) -> ActorStatus {
        self.status
    }
    
    fn stop(&mut self) {
        self.process_lifecycle_event(LifecycleEvent::Stop);
    }
    
    fn restart(&mut self) {
        self.process_lifecycle_event(LifecycleEvent::Restart);
    }
    
    fn schedule<T: Message>(
        &self,
        recipient: Address<T>,
        message: T,
        delay: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation would go here
        // For now, we just store the scheduled message
        // Generate a content-based ID for the schedule
        let id = ContentId::new(format!("schedule:{}", rand::random::<u64>())).to_string();
        let time = crate::time::now() + crate::time::Duration::from_millis(delay.as_millis() as u64);
        let envelope = Box::new(MessageEnvelope::new(message));
        let recipient_box = Box::new(recipient);
        
        let schedule = ScheduledMessage {
            id,
            time,
            interval: None,
            envelope,
            recipient: recipient_box,
        };
        
        // In a real implementation, we would register this with a scheduler
        // For now, we'll just skip that part
        
        Ok(())
    }
    
    fn schedule_periodic<T: Message>(
        &self,
        recipient: Address<T>,
        message: T,
        initial_delay: Duration,
        interval: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation would go here
        // For now, we just store the scheduled message
        // Generate a content-based ID for the schedule
        let id = ContentId::new(format!("periodic-schedule:{}", rand::random::<u64>())).to_string();
        let time = crate::time::now() + crate::time::Duration::from_millis(initial_delay.as_millis() as u64);
        let interval = Some(crate::time::Duration::from_millis(interval.as_millis() as u64));
        let envelope = Box::new(MessageEnvelope::new(message));
        let recipient_box = Box::new(recipient);
        
        let schedule = ScheduledMessage {
            id,
            time,
            interval,
            envelope,
            recipient: recipient_box,
        };
        
        // In a real implementation, we would register this with a scheduler
        // For now, we'll just skip that part
        
        Ok(())
    }
    
    fn cancel_schedule(&self, schedule_id: &str) -> bool {
        // Implementation would go here
        // For now, just return false
        false
    }
    
    fn spawn<A: Actor>(
        &self,
        actor: A,
        options: SpawnOptions,
    ) -> Result<Address<A::Context>, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation would go here
        // For now, just return an error
        Err("Not implemented".into())
    }
    
    fn watch(&self, address: &Address<impl Message>) -> bool {
        // Implementation would go here
        // For now, just return false
        false
    }
    
    fn unwatch(&self, address: &Address<impl Message>) -> bool {
        // Implementation would go here
        // For now, just return false
        false
    }
    
    fn set_state<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        self.state.insert(key.to_string(), Box::new(value));
    }
    
    fn get_state<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.state.get(key).and_then(|value| value.downcast_ref::<T>())
    }
    
    fn get_state_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.state.get_mut(key).and_then(|value| value.downcast_mut::<T>())
    }
    
    fn remove_state(&mut self, key: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.state.remove(key)
    }
    
    fn debug(&self, message: impl Into<String>) {
        println!("[DEBUG] [{}] {}", self.address, message.into());
    }
    
    fn info(&self, message: impl Into<String>) {
        println!("[INFO] [{}] {}", self.address, message.into());
    }
    
    fn warn(&self, message: impl Into<String>) {
        println!("[WARN] [{}] {}", self.address, message.into());
    }
    
    fn error(&self, message: impl Into<String>) {
        println!("[ERROR] [{}] {}", self.address, message.into());
    }
}

/// A hierarchical actor context
pub struct HierarchicalActorContext<M: Message> {
    /// The basic context
    basic: BasicActorContext<M>,
    
    /// The hierarchical address
    hierarchical_address: HierarchicalAddress<M>,
    
    /// The children of this actor
    children: HashMap<String, Address<M>>,
}

impl<M: Message> HierarchicalActorContext<M> {
    /// Create a new hierarchical actor context
    pub fn new(
        hierarchical_address: HierarchicalAddress<M>,
        parent: Option<Address<M>>,
        config: ActorConfig,
        system: Arc<Mutex<SystemContext>>,
    ) -> Self {
        let address = hierarchical_address.address().clone();
        let path = hierarchical_address.path().to_string();
        
        Self {
            basic: BasicActorContext::new(address, path.clone(), parent, config, system),
            hierarchical_address,
            children: HashMap::new(),
        }
    }
    
    /// Get the hierarchical address of the actor
    pub fn hierarchical_address(&self) -> HierarchicalAddress<M> {
        self.hierarchical_address.clone()
    }
    
    /// Add a child actor
    pub fn add_child(&mut self, name: impl Into<String>, address: Address<M>) {
        self.children.insert(name.into(), address);
    }
    
    /// Remove a child actor
    pub fn remove_child(&mut self, name: &str) -> Option<Address<M>> {
        self.children.remove(name)
    }
    
    /// Get a child actor by name
    pub fn get_child(&self, name: &str) -> Option<&Address<M>> {
        self.children.get(name)
    }
    
    /// Get all children
    pub fn children(&self) -> impl Iterator<Item = (&String, &Address<M>)> {
        self.children.iter()
    }
}

impl<M: Message> ActorContext for HierarchicalActorContext<M> {
    type Address = HierarchicalAddress<M>;
    
    fn address(&self) -> Self::Address {
        self.hierarchical_address.clone()
    }
    
    fn path(&self) -> String {
        self.basic.path()
    }
    
    fn parent(&self) -> Option<Self::Address> {
        self.hierarchical_address.parent()
    }
    
    fn config(&self) -> &ActorConfig {
        self.basic.config()
    }
    
    fn status(&self) -> ActorStatus {
        self.basic.status()
    }
    
    fn stop(&mut self) {
        self.basic.stop();
    }
    
    fn restart(&mut self) {
        self.basic.restart();
    }
    
    fn schedule<T: Message>(
        &self,
        recipient: Address<T>,
        message: T,
        delay: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.basic.schedule(recipient, message, delay)
    }
    
    fn schedule_periodic<T: Message>(
        &self,
        recipient: Address<T>,
        message: T,
        initial_delay: Duration,
        interval: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.basic.schedule_periodic(recipient, message, initial_delay, interval)
    }
    
    fn cancel_schedule(&self, schedule_id: &str) -> bool {
        self.basic.cancel_schedule(schedule_id)
    }
    
    fn spawn<A: Actor>(
        &self,
        actor: A,
        options: SpawnOptions,
    ) -> Result<Address<A::Context>, Box<dyn std::error::Error + Send + Sync>> {
        self.basic.spawn(actor, options)
    }
    
    fn watch(&self, address: &Address<impl Message>) -> bool {
        self.basic.watch(address)
    }
    
    fn unwatch(&self, address: &Address<impl Message>) -> bool {
        self.basic.unwatch(address)
    }
    
    fn set_state<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        self.basic.set_state(key, value);
    }
    
    fn get_state<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.basic.get_state(key)
    }
    
    fn get_state_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.basic.get_state_mut(key)
    }
    
    fn remove_state(&mut self, key: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.basic.remove_state(key)
    }
    
    fn debug(&self, message: impl Into<String>) {
        self.basic.debug(message);
    }
    
    fn info(&self, message: impl Into<String>) {
        self.basic.info(message);
    }
    
    fn warn(&self, message: impl Into<String>) {
        self.basic.warn(message);
    }
    
    fn error(&self, message: impl Into<String>) {
        self.basic.error(message);
    }
}

/// Helper functions for working with actor contexts
pub mod helpers {
    use super::*;
    
    /// Create a basic actor context
    pub fn basic_context<M: Message>(
        address: Address<M>,
        config: ActorConfig,
    ) -> BasicActorContext<M> {
        let system = Arc::new(Mutex::new(SystemContext {}));
        BasicActorContext::new(address, address.name().to_string(), None, config, system)
    }
    
    /// Create a hierarchical actor context
    pub fn hierarchical_context<M: Message>(
        address: HierarchicalAddress<M>,
        config: ActorConfig,
    ) -> HierarchicalActorContext<M> {
        let system = Arc::new(Mutex::new(SystemContext {}));
        HierarchicalActorContext::new(address, None, config, system)
    }
    
    /// Create a child context
    pub fn child_context<M: Message>(
        parent: &HierarchicalActorContext<M>,
        name: impl Into<String>,
        config: ActorConfig,
    ) -> HierarchicalActorContext<M> {
        let parent_address = parent.hierarchical_address();
        let child_address = parent_address.child(name);
        let system = Arc::new(Mutex::new(SystemContext {}));
        
        HierarchicalActorContext::new(
            child_address,
            Some(parent_address.address().clone()),
            config,
            system,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::message::Message;
    
    // A test message
    struct TestMessage;
    
    impl Message for TestMessage {
        type Response = ();
    }
    
    #[test]
    fn test_basic_context() {
        let address = Address::<TestMessage>::new("test", "TestActor");
        let config = ActorConfig::new("test");
        let mut context = helpers::basic_context(address, config);
        
        assert_eq!(context.path(), "test");
        assert_eq!(context.status(), ActorStatus::Initializing);
        
        context.set_state("key1", "value1");
        context.set_state("key2", 42);
        
        assert_eq!(context.get_state::<String>("key1"), Some(&"value1".to_string()));
        assert_eq!(context.get_state::<i32>("key2"), Some(&42));
        assert_eq!(context.get_state::<bool>("key3"), None);
        
        context.stop();
        assert_eq!(context.status(), ActorStatus::Stopping);
    }
    
    #[test]
    fn test_hierarchical_context() {
        use crate::actor::address::{Address, AddressPath, HierarchicalAddress};
        
        let address = Address::<TestMessage>::new("parent", "TestActor");
        let path = AddressPath::from_string("/parent");
        let hierarchical_address = HierarchicalAddress::new(address, path);
        let config = ActorConfig::new("parent");
        
        let mut parent = helpers::hierarchical_context(hierarchical_address, config);
        
        let child_config = ActorConfig::new("child");
        let mut child = helpers::child_context(&parent, "child", child_config);
        
        assert_eq!(child.path(), "/parent/child");
        assert!(child.parent().is_some());
        
        // Add the child to the parent
        let child_address = child.address().address().clone();
        parent.add_child("child", child_address);
        
        assert!(parent.get_child("child").is_some());
        assert_eq!(parent.children().count(), 1);
        
        // Remove the child
        let removed = parent.remove_child("child");
        assert!(removed.is_some());
        assert_eq!(parent.children().count(), 0);
    }
} 