use std::sync::Arc;

use causality::builder::prelude::*;
use causality::effect::{CoreEffect, Effect};
use causality::error::{Error, Result};
use causality::handler::{compose, EffectHandler, NoopHandler};
use causality::types::{Account, Amount, Balance, Timestamp};

// Test handler that only handles deposits
struct DepositOnlyHandler;

impl DepositOnlyHandler {
    fn new() -> Self {
        DepositOnlyHandler
    }
}

impl EffectHandler for DepositOnlyHandler {
    fn handle_deposit(
        &self,
        _account: Account,
        _amount: Amount,
        _timestamp: Timestamp,
    ) -> Result<()> {
        Ok(())
    }

    fn handle_withdrawal(
        &self,
        _account: Account,
        _amount: Amount,
        _timestamp: Timestamp,
    ) -> Result<()> {
        Err(Error::OperationFailed(
            "Withdrawals not supported".to_string(),
        ))
    }

    fn handle_observation(&self, _account: Account, _timestamp: Timestamp) -> Result<Balance> {
        Err(Error::OperationFailed(
            "Observations not supported".to_string(),
        ))
    }
}

// Test handler that only handles observations
struct ObservationOnlyHandler;

impl ObservationOnlyHandler {
    fn new() -> Self {
        ObservationOnlyHandler
    }
}

impl EffectHandler for ObservationOnlyHandler {
    fn handle_deposit(
        &self,
        _account: Account,
        _amount: Amount,
        _timestamp: Timestamp,
    ) -> Result<()> {
        Err(Error::OperationFailed("Deposits not supported".to_string()))
    }

    fn handle_withdrawal(
        &self,
        _account: Account,
        _amount: Amount,
        _timestamp: Timestamp,
    ) -> Result<()> {
        Err(Error::OperationFailed(
            "Withdrawals not supported".to_string(),
        ))
    }

    fn handle_observation(&self, _account: Account, _timestamp: Timestamp) -> Result<Balance> {
        Ok(Balance(500))
    }
}

#[test]
fn test_handler_composition() {
    // Create two handlers with different capabilities
    let deposit_handler = Arc::new(DepositOnlyHandler::new());
    let observation_handler = Arc::new(ObservationOnlyHandler::new());

    // Compose them together
    let composite_handler = compose(deposit_handler.clone(), observation_handler.clone());

    // Create test data
    let account = Account("test".to_string());
    let amount = Amount(100);
    let timestamp = Timestamp(12345);

    // Create a deposit effect (should use the deposit handler)
    let deposit_effect = deposit(account.clone(), amount.clone(), timestamp.clone());

    // Create an observation effect (should use the observation handler)
    let observation_effect = observation(account.clone(), timestamp.clone());

    // Execute the effects
    let deposit_result = deposit_effect.execute(&composite_handler);
    assert!(deposit_result == (), "Deposit should succeed");

    let balance = observation_effect.execute(&composite_handler);
    assert_eq!(
        balance,
        Balance(500),
        "Observation should return Balance(500)"
    );

    // Create a withdrawal effect (neither handler supports this)
    let withdrawal_effect = withdrawal(account.clone(), amount.clone(), timestamp.clone());

    // Execute the withdrawal (should panic due to the builder setup)
    // So we'll catch the panic and verify it happened
    let result = std::panic::catch_unwind(|| {
        withdrawal_effect.execute(&composite_handler);
    });
    assert!(result.is_err(), "Withdrawal should panic");
}

#[test]
fn test_effect_sequence() {
    // Create a handler that supports deposits
    let deposit_handler = Arc::new(DepositOnlyHandler::new());

    // Create test data
    let account = Account("test".to_string());
    let amount = Amount(100);
    let timestamp = Timestamp(12345);

    // Create a sequence of deposit effects
    let effects = vec![
        deposit(account.clone(), amount.clone(), timestamp.clone()),
        deposit(Account("test2".to_string()), Amount(200), Timestamp(12346)),
    ];

    let sequence = sequence(effects);

    // Execute the sequence
    let result = sequence.execute(&*deposit_handler);

    // Should succeed
    assert_eq!(result, (), "Sequence should succeed");
}

#[test]
fn test_full_workflow() {
    // Create handlers
    let deposit_handler = Arc::new(DepositOnlyHandler::new());
    let observation_handler = Arc::new(ObservationOnlyHandler::new());
    let composite_handler = compose(deposit_handler, observation_handler);

    // Create test data
    let account = Account("test".to_string());
    let amount = Amount(100);
    let timestamp = Timestamp(12345);

    // Step 1: Make a deposit
    let step1 = deposit(account.clone(), amount.clone(), timestamp.clone());

    // Step 2: Check the balance
    let step2 = observation(account.clone(), timestamp.clone());

    // Execute step 1
    let result1 = step1.execute(&composite_handler);
    assert_eq!(result1, (), "Deposit should succeed");

    // Execute step 2
    let balance = step2.execute(&composite_handler);

    // Verify the balance
    assert_eq!(balance, Balance(500), "Balance should be 500");

    // Create a complex sequence and execute it
    let complex_workflow = sequence(vec![
        deposit(account.clone(), amount.clone(), timestamp.clone()),
        deposit(
            account.clone(),
            Amount(amount.0 * 2),
            Timestamp(timestamp.0 + 1),
        ),
        deposit(
            account.clone(),
            Amount(amount.0 * 3),
            Timestamp(timestamp.0 + 2),
        ),
    ]);

    let result = complex_workflow.execute(&composite_handler);
    assert_eq!(result, (), "Complex workflow should succeed");
}
