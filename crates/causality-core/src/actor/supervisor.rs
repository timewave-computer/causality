// Actor supervision implementation
//
// This module provides supervision strategies for the actor system.

use std::fmt::Debug;
use std::time::Duration;

use crate::actor::{Actor, ActorConfig, ActorStatus};

/// A trait for actor supervisors
pub trait Supervisor<A: Actor>: Send + 'static {
    /// Handle a failed actor
    fn handle_failure(&self, actor: &mut A, error: Box<dyn std::error::Error + Send + Sync>, context: &mut A::Context) -> SupervisionDecision;
    
    /// Get the supervision strategy
    fn strategy(&self) -> SupervisionStrategy;
    
    /// Get the maximum number of restart attempts
    fn max_restarts(&self) -> Option<usize>;
    
    /// Get the time window for counting restarts
    fn restart_window(&self) -> Option<Duration>;
}

/// The decision made by a supervisor when an actor fails
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionDecision {
    /// Restart the actor
    Restart,
    
    /// Stop the actor
    Stop,
    
    /// Escalate the failure to the parent supervisor
    Escalate,
    
    /// Resume the actor without restarting
    Resume,
}

/// The strategy to use for supervision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionStrategy {
    /// One for one: only the failed actor is affected
    OneForOne,
    
    /// All for one: all siblings are affected by one failure
    AllForOne,
    
    /// Rest for one: all actors started after the failed one are affected
    RestForOne,
}

/// A simple supervisor that always follows the same strategy
pub struct SimpleSupervisor {
    /// The supervision strategy
    strategy: SupervisionStrategy,
    
    /// The maximum number of restart attempts
    max_restarts: Option<usize>,
    
    /// The time window for counting restarts
    restart_window: Option<Duration>,
    
    /// The default decision for failures
    default_decision: SupervisionDecision,
}

impl SimpleSupervisor {
    /// Create a new simple supervisor
    pub fn new(
        strategy: SupervisionStrategy,
        max_restarts: Option<usize>,
        restart_window: Option<Duration>,
        default_decision: SupervisionDecision,
    ) -> Self {
        Self {
            strategy,
            max_restarts,
            restart_window,
            default_decision,
        }
    }
    
    /// Create a new supervisor that always restarts actors
    pub fn always_restart() -> Self {
        Self::new(
            SupervisionStrategy::OneForOne,
            None,
            None,
            SupervisionDecision::Restart,
        )
    }
    
    /// Create a new supervisor that always stops actors
    pub fn always_stop() -> Self {
        Self::new(
            SupervisionStrategy::OneForOne,
            None,
            None,
            SupervisionDecision::Stop,
        )
    }
    
    /// Create a new supervisor that always escalates failures
    pub fn always_escalate() -> Self {
        Self::new(
            SupervisionStrategy::OneForOne,
            None,
            None,
            SupervisionDecision::Escalate,
        )
    }
    
    /// Create a new supervisor that always resumes actors
    pub fn always_resume() -> Self {
        Self::new(
            SupervisionStrategy::OneForOne,
            None,
            None,
            SupervisionDecision::Resume,
        )
    }
    
    /// Create a new supervisor with a limited number of restarts
    pub fn with_limited_restarts(max_restarts: usize, window: Option<Duration>) -> Self {
        Self::new(
            SupervisionStrategy::OneForOne,
            Some(max_restarts),
            window,
            SupervisionDecision::Restart,
        )
    }
}

impl<A: Actor> Supervisor<A> for SimpleSupervisor {
    fn handle_failure(&self, _actor: &mut A, _error: Box<dyn std::error::Error + Send + Sync>, context: &mut A::Context) -> SupervisionDecision {
        // In a real implementation, we would check the number of restarts
        // and decide based on that. For now, we just return the default decision.
        if let Some(max) = self.max_restarts {
            // Get the current restart count from the context
            let restart_count = context.get_state::<usize>("restart_count").cloned().unwrap_or(0);
            
            if restart_count >= max {
                // Too many restarts, stop the actor
                return SupervisionDecision::Stop;
            } else {
                // Increment the restart count
                context.set_state("restart_count", restart_count + 1);
                
                // Check if we need to expire old restarts
                if let Some(window) = self.restart_window {
                    // In a real implementation, we would track the timestamps of restarts
                    // and expire old ones. For now, we'll just use the simple count.
                }
                
                return self.default_decision;
            }
        }
        
        self.default_decision
    }
    
    fn strategy(&self) -> SupervisionStrategy {
        self.strategy
    }
    
    fn max_restarts(&self) -> Option<usize> {
        self.max_restarts
    }
    
    fn restart_window(&self) -> Option<Duration> {
        self.restart_window
    }
}

/// A supervisor that makes decisions based on the error type
pub struct ErrorMatchingSupervisor<A: Actor> {
    /// The base supervisor
    base: SimpleSupervisor,
    
    /// Error matchers
    matchers: Vec<Box<dyn Fn(&(dyn std::error::Error + Send + Sync), &mut A, &mut A::Context) -> Option<SupervisionDecision> + Send + 'static>>,
}

impl<A: Actor> ErrorMatchingSupervisor<A> {
    /// Create a new error matching supervisor
    pub fn new(base: SimpleSupervisor) -> Self {
        Self {
            base,
            matchers: Vec::new(),
        }
    }
    
    /// Add a matcher for a specific error type
    pub fn with_matcher<E, F>(mut self, matcher: F) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
        F: Fn(&E, &mut A, &mut A::Context) -> SupervisionDecision + Send + 'static,
    {
        let boxed_matcher = Box::new(move |error: &(dyn std::error::Error + Send + Sync), actor: &mut A, context: &mut A::Context| {
            if let Some(typed_error) = error.downcast_ref::<E>() {
                Some(matcher(typed_error, actor, context))
            } else {
                None
            }
        });
        
        self.matchers.push(boxed_matcher);
        self
    }
}

impl<A: Actor> Supervisor<A> for ErrorMatchingSupervisor<A> {
    fn handle_failure(&self, actor: &mut A, error: Box<dyn std::error::Error + Send + Sync>, context: &mut A::Context) -> SupervisionDecision {
        // Try to match the error to a specific matcher
        for matcher in &self.matchers {
            if let Some(decision) = matcher(&*error, actor, context) {
                return decision;
            }
        }
        
        // Fall back to the base supervisor
        self.base.handle_failure(actor, error, context)
    }
    
    fn strategy(&self) -> SupervisionStrategy {
        self.base.strategy()
    }
    
    fn max_restarts(&self) -> Option<usize> {
        self.base.max_restarts()
    }
    
    fn restart_window(&self) -> Option<Duration> {
        self.base.restart_window()
    }
}

/// A hierarchical supervisor that can supervise multiple actors
pub struct HierarchicalSupervisor<A: Actor> {
    /// The current supervisor
    supervisor: Box<dyn Supervisor<A>>,
    
    /// The parent supervisor
    parent: Option<Box<dyn Supervisor<A>>>,
    
    /// Child actors supervised by this supervisor
    children: Vec<(String, A)>,
}

impl<A: Actor> HierarchicalSupervisor<A> {
    /// Create a new hierarchical supervisor
    pub fn new(supervisor: Box<dyn Supervisor<A>>) -> Self {
        Self {
            supervisor,
            parent: None,
            children: Vec::new(),
        }
    }
    
    /// Set the parent supervisor
    pub fn with_parent(mut self, parent: Box<dyn Supervisor<A>>) -> Self {
        self.parent = Some(parent);
        self
    }
    
    /// Add a child actor
    pub fn add_child(&mut self, name: impl Into<String>, actor: A) {
        self.children.push((name.into(), actor));
    }
    
    /// Remove a child actor
    pub fn remove_child(&mut self, name: &str) -> Option<A> {
        if let Some(index) = self.children.iter().position(|(n, _)| n == name) {
            let (_, actor) = self.children.remove(index);
            Some(actor)
        } else {
            None
        }
    }
    
    /// Handle a failed child actor
    pub fn handle_child_failure(&mut self, name: &str, error: Box<dyn std::error::Error + Send + Sync>, context: &mut A::Context) -> SupervisionDecision {
        // Find the child actor
        if let Some(index) = self.children.iter().position(|(n, _)| n == name) {
            let (_, actor) = &mut self.children[index];
            
            // Let the supervisor handle the failure
            let decision = self.supervisor.handle_failure(actor, error, context);
            
            // Handle the decision
            match decision {
                SupervisionDecision::Restart => {
                    // Actor would be restarted here
                }
                SupervisionDecision::Stop => {
                    // Actor would be stopped here
                    self.children.remove(index);
                }
                SupervisionDecision::Escalate => {
                    // Escalate to the parent supervisor
                    if let Some(parent) = &self.parent {
                        return parent.handle_failure(actor, error, context);
                    } else {
                        // No parent, stop the actor
                        self.children.remove(index);
                        return SupervisionDecision::Stop;
                    }
                }
                SupervisionDecision::Resume => {
                    // Actor continues running
                }
            }
            
            decision
        } else {
            // Child not found
            SupervisionDecision::Stop
        }
    }
}

impl<A: Actor> Supervisor<A> for HierarchicalSupervisor<A> {
    fn handle_failure(&self, actor: &mut A, error: Box<dyn std::error::Error + Send + Sync>, context: &mut A::Context) -> SupervisionDecision {
        self.supervisor.handle_failure(actor, error, context)
    }
    
    fn strategy(&self) -> SupervisionStrategy {
        self.supervisor.strategy()
    }
    
    fn max_restarts(&self) -> Option<usize> {
        self.supervisor.max_restarts()
    }
    
    fn restart_window(&self) -> Option<Duration> {
        self.supervisor.restart_window()
    }
}

/// Helper functions for working with supervisors
pub mod helpers {
    use super::*;
    
    /// Create a supervisor that always restarts actors
    pub fn always_restart<A: Actor>() -> impl Supervisor<A> {
        SimpleSupervisor::always_restart()
    }
    
    /// Create a supervisor that always stops actors
    pub fn always_stop<A: Actor>() -> impl Supervisor<A> {
        SimpleSupervisor::always_stop()
    }
    
    /// Create a supervisor that always escalates failures
    pub fn always_escalate<A: Actor>() -> impl Supervisor<A> {
        SimpleSupervisor::always_escalate()
    }
    
    /// Create a supervisor that always resumes actors
    pub fn always_resume<A: Actor>() -> impl Supervisor<A> {
        SimpleSupervisor::always_resume()
    }
    
    /// Create a supervisor with a limited number of restarts
    pub fn with_limited_restarts<A: Actor>(max_restarts: usize, window: Option<Duration>) -> impl Supervisor<A> {
        SimpleSupervisor::with_limited_restarts(max_restarts, window)
    }
    
    /// Create an error matching supervisor
    pub fn error_matching<A: Actor, E>(
        base: impl Supervisor<A> + 'static,
        error_type: fn(&(dyn std::error::Error + Send + Sync)) -> Option<&E>,
        decision: SupervisionDecision,
    ) -> impl Supervisor<A>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let base = SimpleSupervisor::new(
            base.strategy(),
            base.max_restarts(),
            base.restart_window(),
            SupervisionDecision::Restart,
        );
        
        ErrorMatchingSupervisor::new(base).with_matcher(move |_: &E, _: &mut A, _: &mut A::Context| decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::context::{ActorContext, BasicActorContext};
    use crate::actor::address::Address;
    use crate::actor::message::Message;
    use std::error::Error;
    use std::fmt;
    
    // A test message
    struct TestMessage;
    
    impl Message for TestMessage {
        type Response = ();
    }
    
    // A test actor
    struct TestActor {
        name: String,
    }
    
    impl Actor for TestActor {
        type Context = BasicActorContext<TestMessage>;
        
        fn initialize(&mut self, ctx: &mut Self::Context) {
            ctx.set_state("initialized", true);
        }
        
        fn on_stop(&mut self, _ctx: &mut Self::Context) {
            // Actor is stopping
        }
        
        fn on_restart(&mut self, _ctx: &mut Self::Context) {
            // Actor is restarting
        }
    }
    
    // A test error
    #[derive(Debug)]
    struct TestError {
        message: String,
    }
    
    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestError: {}", self.message)
        }
    }
    
    impl Error for TestError {}
    
    #[test]
    fn test_simple_supervisor() {
        use crate::actor::ActorConfig;
        use crate::actor::context::helpers::basic_context;
        
        let supervisor = SimpleSupervisor::always_restart();
        let mut actor = TestActor { name: "test".to_string() };
        let address = Address::<TestMessage>::new("test", "TestActor");
        let config = ActorConfig::new("test");
        let mut context = basic_context(address, config);
        
        let error = Box::new(TestError { message: "Test failure".to_string() });
        let decision = supervisor.handle_failure(&mut actor, error, &mut context);
        
        assert_eq!(decision, SupervisionDecision::Restart);
    }
    
    #[test]
    fn test_limited_restarts() {
        use crate::actor::ActorConfig;
        use crate::actor::context::helpers::basic_context;
        
        let supervisor = SimpleSupervisor::with_limited_restarts(2, None);
        let mut actor = TestActor { name: "test".to_string() };
        let address = Address::<TestMessage>::new("test", "TestActor");
        let config = ActorConfig::new("test");
        let mut context = basic_context(address, config);
        
        let error = Box::new(TestError { message: "Test failure".to_string() });
        
        // First restart should succeed
        let decision1 = supervisor.handle_failure(&mut actor, error.clone(), &mut context);
        assert_eq!(decision1, SupervisionDecision::Restart);
        
        // Second restart should succeed
        let decision2 = supervisor.handle_failure(&mut actor, error.clone(), &mut context);
        assert_eq!(decision2, SupervisionDecision::Restart);
        
        // Third restart should fail and stop the actor
        let decision3 = supervisor.handle_failure(&mut actor, error.clone(), &mut context);
        assert_eq!(decision3, SupervisionDecision::Stop);
    }
    
    #[test]
    fn test_error_matching_supervisor() {
        use crate::actor::ActorConfig;
        use crate::actor::context::helpers::basic_context;
        
        let base = SimpleSupervisor::always_restart();
        let supervisor = ErrorMatchingSupervisor::new(base)
            .with_matcher(|err: &TestError, _: &mut TestActor, _: &mut BasicActorContext<TestMessage>| {
                if err.message.contains("critical") {
                    SupervisionDecision::Stop
                } else {
                    SupervisionDecision::Restart
                }
            });
        
        let mut actor = TestActor { name: "test".to_string() };
        let address = Address::<TestMessage>::new("test", "TestActor");
        let config = ActorConfig::new("test");
        let mut context = basic_context(address, config);
        
        // Non-critical error should restart
        let error1 = Box::new(TestError { message: "Test failure".to_string() });
        let decision1 = supervisor.handle_failure(&mut actor, error1, &mut context);
        assert_eq!(decision1, SupervisionDecision::Restart);
        
        // Critical error should stop
        let error2 = Box::new(TestError { message: "Critical failure".to_string() });
        let decision2 = supervisor.handle_failure(&mut actor, error2, &mut context);
        assert_eq!(decision2, SupervisionDecision::Stop);
    }
    
    #[test]
    fn test_hierarchical_supervisor() {
        use crate::actor::ActorConfig;
        use crate::actor::context::helpers::basic_context;
        
        let child_supervisor = Box::new(SimpleSupervisor::always_restart());
        let mut supervisor = HierarchicalSupervisor::new(child_supervisor);
        
        let actor1 = TestActor { name: "actor1".to_string() };
        let actor2 = TestActor { name: "actor2".to_string() };
        
        supervisor.add_child("actor1", actor1);
        supervisor.add_child("actor2", actor2);
        
        let address = Address::<TestMessage>::new("test", "TestActor");
        let config = ActorConfig::new("test");
        let mut context = basic_context(address, config);
        
        let error = Box::new(TestError { message: "Test failure".to_string() });
        let decision = supervisor.handle_child_failure("actor1", error, &mut context);
        
        assert_eq!(decision, SupervisionDecision::Restart);
    }
} 