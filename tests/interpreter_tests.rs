use std::sync::Arc;

use causality::builder::prelude::*;
use causality::effect::{Effect, EffectKind};
use causality::error::{Error, Result};
use causality::handler::{EffectHandler, NoopHandler, SharedHandler};
use causality::interpreter::{Interpret, Interpreter, MockInterpreter, TracingInterpreter};
use causality::types::{Account, Amount, Balance, Timestamp};

// Basic test account and values
const TEST_ACCOUNT: Account = 42;
const TEST_AMOUNT: Amount = 100;
const TEST_TIMESTAMP: Timestamp = 12345;

#[test]
fn test_interpreter_creation() {
    let handler = Arc::new(NoopHandler::new());
    let interpreter = Interpreter::new(handler);

    // No assertion needed, just checking that it doesn't panic
}

#[test]
fn test_noop_handler_fails() {
    let handler = Arc::new(NoopHandler::new());
    let interpreter = Interpreter::new(handler);

    // Create a deposit effect
    let effect = deposit(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);

    // Execute with the interpreter
    let result = interpreter.execute(&*effect);

    // Since we're using NoopHandler, this should fail
    assert!(result.is_err());
}

struct TestHandler {
    deposit_result: Result<()>,
    withdrawal_result: Result<()>,
    observation_result: Result<Balance>,
}

impl TestHandler {
    fn new() -> Self {
        TestHandler {
            deposit_result: Ok(()),
            withdrawal_result: Ok(()),
            observation_result: Ok(Balance::from(500)),
        }
    }

    fn with_deposit_result(mut self, result: Result<()>) -> Self {
        self.deposit_result = result;
        self
    }

    fn with_withdrawal_result(mut self, result: Result<()>) -> Self {
        self.withdrawal_result = result;
        self
    }

    fn with_observation_result(mut self, result: Result<Balance>) -> Self {
        self.observation_result = result;
        self
    }
}

impl EffectHandler for TestHandler {
    fn handle_deposit(
        &self,
        _account: Account,
        _amount: Amount,
        _timestamp: Timestamp,
    ) -> Result<()> {
        self.deposit_result.clone()
    }

    fn handle_withdrawal(
        &self,
        _account: Account,
        _amount: Amount,
        _timestamp: Timestamp,
    ) -> Result<()> {
        self.withdrawal_result.clone()
    }

    fn handle_observation(&self, _account: Account, _timestamp: Timestamp) -> Result<Balance> {
        self.observation_result.clone()
    }
}

#[test]
fn test_deposit_effect() {
    let handler = Arc::new(TestHandler::new());
    let interpreter = Interpreter::new(handler);

    // Create a deposit effect
    let effect = deposit(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);

    // Execute with the interpreter
    let result = interpreter.execute(&*effect);

    // Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_withdrawal_effect() {
    let handler = Arc::new(TestHandler::new());
    let interpreter = Interpreter::new(handler);

    // Create a withdrawal effect
    let effect = withdrawal(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);

    // Execute with the interpreter
    let result = interpreter.execute(&*effect);

    // Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_observation_effect() {
    let handler = Arc::new(TestHandler::new());
    let interpreter = Interpreter::new(handler);

    // Create an observation effect
    let effect = observation(TEST_ACCOUNT, TEST_TIMESTAMP);

    // Execute with the interpreter
    let result = interpreter.execute(&*effect);

    // Should succeed with the expected balance
    assert_eq!(result.unwrap(), Balance::from(500));
}

#[test]
fn test_failed_effects() {
    let handler = Arc::new(
        TestHandler::new()
            .with_deposit_result(Err(Error::OperationFailed("Deposit failed".to_string())))
            .with_withdrawal_result(Err(Error::OperationFailed("Withdrawal failed".to_string())))
            .with_observation_result(Err(Error::OperationFailed(
                "Observation failed".to_string(),
            ))),
    );
    let interpreter = Interpreter::new(handler);

    // Test deposit
    let deposit_effect = deposit(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);
    assert!(interpreter.execute(&*deposit_effect).is_err());

    // Test withdrawal
    let withdrawal_effect = withdrawal(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);
    assert!(interpreter.execute(&*withdrawal_effect).is_err());

    // Test observation
    let observation_effect = observation(TEST_ACCOUNT, TEST_TIMESTAMP);
    assert!(interpreter.execute(&*observation_effect).is_err());
}

#[test]
fn test_sequence_execution() {
    let handler = Arc::new(TestHandler::new());
    let interpreter = Interpreter::new(handler);

    // Create a sequence of effects
    let effects = vec![
        deposit(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP),
        withdrawal(TEST_ACCOUNT, TEST_AMOUNT / 2, TEST_TIMESTAMP + 1),
    ];

    let sequence = sequence(effects);

    // Execute the sequence
    let result = interpreter.execute(&*sequence);

    // Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_sequence_execution_with_failure() {
    let handler = Arc::new(
        TestHandler::new()
            .with_withdrawal_result(Err(Error::OperationFailed("Withdrawal failed".to_string()))),
    );
    let interpreter = Interpreter::new(handler);

    // Create a sequence of effects where the second will fail
    let effects = vec![
        deposit(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP),
        withdrawal(TEST_ACCOUNT, TEST_AMOUNT / 2, TEST_TIMESTAMP + 1),
    ];

    let sequence = sequence(effects);

    // Execute the sequence
    let result = interpreter.execute(&*sequence);

    // Should fail because one effect in the sequence fails
    assert!(result.is_err());
}

#[test]
fn test_interpret_trait() {
    let handler = Arc::new(TestHandler::new());
    let interpreter = Interpreter::new(handler);

    // Create an effect
    let effect = observation(TEST_ACCOUNT, TEST_TIMESTAMP);

    // Use the Interpret trait
    let result = effect.interpret(&interpreter);

    // Should succeed with the expected balance
    assert_eq!(result.unwrap(), Balance::from(500));
}

#[test]
fn test_tracing_interpreter() {
    let handler = Arc::new(TestHandler::new());
    let mut tracing_interpreter = TracingInterpreter::new(handler);

    // Create and execute some effects
    let effect1 = deposit(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);
    let effect2 = observation(TEST_ACCOUNT, TEST_TIMESTAMP);

    tracing_interpreter.execute(&*effect1).unwrap();
    tracing_interpreter.execute(&*effect2).unwrap();

    // Check the trace
    let trace = tracing_interpreter.trace();
    assert_eq!(trace.len(), 2);
    assert_eq!(trace[0].kind(), EffectKind::Deposit);
    assert_eq!(trace[1].kind(), EffectKind::Observation);

    // Clear the trace and check it's empty
    tracing_interpreter.clear_trace();
    assert_eq!(tracing_interpreter.trace().len(), 0);
}

#[test]
fn test_mock_interpreter() {
    let mut mock_interpreter = MockInterpreter::new();

    // Set up mock responses
    mock_interpreter.set_response(EffectKind::Deposit, Ok(()));
    mock_interpreter.set_response(
        EffectKind::Withdrawal,
        Err(Error::OperationFailed("Mock withdrawal failed".to_string())),
    );
    mock_interpreter.set_response(EffectKind::Observation, Ok(Balance::from(1000)));

    // Test deposit (should succeed)
    let deposit_effect = deposit(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);
    assert!(mock_interpreter.execute(&*deposit_effect).is_ok());

    // Test withdrawal (should fail)
    let withdrawal_effect = withdrawal(TEST_ACCOUNT, TEST_AMOUNT, TEST_TIMESTAMP);
    assert!(mock_interpreter.execute(&*withdrawal_effect).is_err());

    // Test observation (should return 1000)
    let observation_effect = observation(TEST_ACCOUNT, TEST_TIMESTAMP);
    assert_eq!(
        mock_interpreter.execute(&*observation_effect).unwrap(),
        Balance::from(1000)
    );
}
