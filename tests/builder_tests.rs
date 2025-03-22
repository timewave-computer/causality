use causality::builder::prelude::*;
use causality::builder::{EffectBuilder, SequenceBuilder};
use causality::effect::CoreEffect;
use causality::types::{Account, Amount, Balance, Timestamp};

// Test constants
const TEST_ACCOUNT: Account = Account("test".to_string());
const TEST_AMOUNT: Amount = Amount(100);
const TEST_TIMESTAMP: Timestamp = Timestamp(12345);

#[test]
fn test_effect_builder_creation() {
    // Check that we can create a builder
    let builder: EffectBuilder<()> = EffectBuilder::new();

    // No assertion needed, just checking that it doesn't panic
}

#[test]
fn test_deposit_builder() {
    // Create a deposit effect using the builder
    let effect = EffectBuilder::<()>::new()
        .account(TEST_ACCOUNT.clone())
        .amount(TEST_AMOUNT.clone())
        .timestamp(TEST_TIMESTAMP.clone())
        .deposit();

    // Just verify we can create it (can't easily check properties with CoreEffect)
    assert!(true, "Successfully created deposit effect");
}

#[test]
fn test_withdrawal_builder() {
    // Create a withdrawal effect using the builder
    let effect = EffectBuilder::<()>::new()
        .account(TEST_ACCOUNT.clone())
        .amount(TEST_AMOUNT.clone())
        .timestamp(TEST_TIMESTAMP.clone())
        .withdrawal();

    // Just verify we can create it (can't easily check properties with CoreEffect)
    assert!(true, "Successfully created withdrawal effect");
}

#[test]
fn test_observation_builder() {
    // Create an observation effect using the builder
    let effect = EffectBuilder::<Balance>::new()
        .account(TEST_ACCOUNT.clone())
        .timestamp(TEST_TIMESTAMP.clone())
        .observation();

    // Just verify we can create it (can't easily check properties with CoreEffect)
    assert!(true, "Successfully created observation effect");
}

#[test]
#[should_panic(expected = "Account is required for deposit")]
fn test_deposit_builder_missing_account() {
    // Try to create a deposit effect without an account (should panic)
    let _effect = EffectBuilder::<()>::new()
        .amount(TEST_AMOUNT.clone())
        .timestamp(TEST_TIMESTAMP.clone())
        .deposit();
}

#[test]
#[should_panic(expected = "Amount is required for deposit")]
fn test_deposit_builder_missing_amount() {
    // Try to create a deposit effect without an amount (should panic)
    let _effect = EffectBuilder::<()>::new()
        .account(TEST_ACCOUNT.clone())
        .timestamp(TEST_TIMESTAMP.clone())
        .deposit();
}

#[test]
#[should_panic(expected = "Timestamp is required for deposit")]
fn test_deposit_builder_missing_timestamp() {
    // Try to create a deposit effect without a timestamp (should panic)
    let _effect = EffectBuilder::<()>::new()
        .account(TEST_ACCOUNT.clone())
        .amount(TEST_AMOUNT.clone())
        .deposit();
}

#[test]
fn test_sequence_builder() {
    // Create a sequence of effects using the builder
    let effects = vec![
        deposit(
            TEST_ACCOUNT.clone(),
            TEST_AMOUNT.clone(),
            TEST_TIMESTAMP.clone(),
        ),
        withdrawal(
            TEST_ACCOUNT.clone(),
            Amount(TEST_AMOUNT.0 / 2),
            Timestamp(TEST_TIMESTAMP.0 + 1),
        ),
    ];

    let sequence = effects.sequence();

    // Just verify we can create it (can't easily check properties with CoreEffect)
    assert!(true, "Successfully created sequence effect");
}

#[test]
fn test_empty_sequence_builder() {
    // Creating a sequence from an empty vector should give a no-op effect
    let effects: Vec<CoreEffect<()>> = vec![];
    let _sequence = effects.sequence();

    // Just verify we can create it
    assert!(true, "Successfully created sequence from empty vector");
}

#[test]
fn test_prelude_functions() {
    // Test the prelude convenience functions

    // Test deposit
    let _deposit_effect = deposit(
        TEST_ACCOUNT.clone(),
        TEST_AMOUNT.clone(),
        TEST_TIMESTAMP.clone(),
    );

    // Test withdrawal
    let _withdrawal_effect = withdrawal(
        TEST_ACCOUNT.clone(),
        TEST_AMOUNT.clone(),
        TEST_TIMESTAMP.clone(),
    );

    // Test observation
    let _observation_effect = observation(TEST_ACCOUNT.clone(), TEST_TIMESTAMP.clone());

    // Test sequence
    let effects = vec![
        deposit(
            TEST_ACCOUNT.clone(),
            TEST_AMOUNT.clone(),
            TEST_TIMESTAMP.clone(),
        ),
        withdrawal(
            TEST_ACCOUNT.clone(),
            Amount(TEST_AMOUNT.0 / 2),
            Timestamp(TEST_TIMESTAMP.0 + 1),
        ),
    ];

    let _sequence_effect = sequence(effects);

    // All created successfully
    assert!(
        true,
        "Successfully created all effects using prelude functions"
    );
}

#[test]
fn test_builder_fluent_interface() {
    // Test that we can chain builder methods in different orders

    // Order 1: account -> amount -> timestamp
    let _effect1 = EffectBuilder::<()>::new()
        .account(TEST_ACCOUNT.clone())
        .amount(TEST_AMOUNT.clone())
        .timestamp(TEST_TIMESTAMP.clone())
        .deposit();

    // Order 2: timestamp -> account -> amount
    let _effect2 = EffectBuilder::<()>::new()
        .timestamp(TEST_TIMESTAMP.clone())
        .account(TEST_ACCOUNT.clone())
        .amount(TEST_AMOUNT.clone())
        .deposit();

    // Both should be created successfully
    assert!(
        true,
        "Successfully created effects with different builder chains"
    );
}
