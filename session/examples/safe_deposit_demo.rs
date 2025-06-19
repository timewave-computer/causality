// Example: Safe Deposit Mode - Demonstrating constrained mailbox accounts

use session::blockchain::mailbox::{
    Mailbox, MailboxId, TokenId, SafeDepositConfig, DepositCondition
};
use session::layer1::SessionType;

fn main() {
    println!("=== Safe Deposit Mode Demo ===\n");
    
    // Create a mailbox with safe deposit constraints
    let safe_config = SafeDepositConfig {
        rejection_conditions: vec![
            // Maximum 1000 ETH total
            DepositCondition::DepositCap {
                token: TokenId("ETH".to_string()),
                max_amount: 1000,
            },
            // No single deposit over 500 ETH
            DepositCondition::MaxSingleDeposit { max_amount: 500 },
            // Only accept deposits from known addresses
            DepositCondition::AllowedDepositors(vec![
                "alice.eth".to_string(),
                "bob.eth".to_string(),
                "treasury.eth".to_string(),
            ]),
            // Deadline at block 1000
            DepositCondition::DeadlineBlock(1000),
        ],
        auto_refund: true,
        refund_timeout_blocks: 100,
    };
    
    let mut mailbox = Mailbox::new_safe(
        MailboxId("constrained_account".to_string()),
        SessionType::End,
        safe_config,
    );
    
    println!("Created constrained mailbox account with:");
    println!("  - Maximum balance: 1000 ETH");
    println!("  - Maximum single deposit: 500 ETH");
    println!("  - Allowed depositors: alice.eth, bob.eth, treasury.eth");
    println!("  - Deadline: block 1000");
    println!("  - Auto-refund: enabled\n");
    
    // Simulate deposits at different blocks
    let current_block = 100;
    
    // Valid deposit from Alice
    println!("Block {}: Alice deposits 400 ETH", current_block);
    match mailbox.receive_deposit_safe(
        "alice.eth".to_string(),
        TokenId("ETH".to_string()),
        400,
        current_block,
    ) {
        Ok(receipt) => println!("  ✅ Accepted - nonce: {}", receipt.nonce),
        Err(e) => println!("  ❌ Rejected: {}", e),
    }
    
    // Valid deposit from Bob
    println!("\nBlock {}: Bob deposits 400 ETH", current_block + 10);
    match mailbox.receive_deposit_safe(
        "bob.eth".to_string(),
        TokenId("ETH".to_string()),
        400,
        current_block + 10,
    ) {
        Ok(receipt) => println!("  ✅ Accepted - nonce: {}", receipt.nonce),
        Err(e) => println!("  ❌ Rejected: {}", e),
    }
    
    // Rejected: Would exceed cap
    println!("\nBlock {}: Treasury deposits 300 ETH", current_block + 20);
    match mailbox.receive_deposit_safe(
        "treasury.eth".to_string(),
        TokenId("ETH".to_string()),
        300,
        current_block + 20,
    ) {
        Ok(receipt) => println!("  ✅ Accepted - nonce: {}", receipt.nonce),
        Err(e) => println!("  ❌ Rejected: {}", e),
    }
    
    // Rejected: Single deposit too large
    println!("\nBlock {}: Alice tries to deposit 600 ETH", current_block + 30);
    match mailbox.receive_deposit_safe(
        "alice.eth".to_string(),
        TokenId("ETH".to_string()),
        600,
        current_block + 30,
    ) {
        Ok(receipt) => println!("  ✅ Accepted - nonce: {}", receipt.nonce),
        Err(e) => println!("  ❌ Rejected: {}", e),
    }
    
    // Rejected: Unauthorized depositor
    println!("\nBlock {}: Charlie tries to deposit 100 ETH", current_block + 40);
    match mailbox.receive_deposit_safe(
        "charlie.eth".to_string(),
        TokenId("ETH".to_string()),
        100,
        current_block + 40,
    ) {
        Ok(receipt) => println!("  ✅ Accepted - nonce: {}", receipt.nonce),
        Err(e) => println!("  ❌ Rejected: {}", e),
    }
    
    // Check pending refunds
    println!("\n=== Pending Refunds ===");
    let refunds = mailbox.get_pending_refunds(current_block + 50);
    println!("Found {} refunds ready to process:", refunds.len());
    for refund in &refunds {
        println!("  - {} ETH to {} (reason: {:?})", 
            refund.amount, refund.recipient, refund.reason);
    }
    
    // Process refunds
    let nonces: Vec<u64> = refunds.iter().map(|r| r.nonce).collect();
    mailbox.process_refunds(&nonces);
    println!("\n✅ Processed {} refunds", nonces.len());
    
    // Rejected: After deadline
    println!("\nBlock {}: Bob tries to deposit after deadline", 1001);
    match mailbox.receive_deposit_safe(
        "bob.eth".to_string(),
        TokenId("ETH".to_string()),
        100,
        1001,
    ) {
        Ok(receipt) => println!("  ✅ Accepted - nonce: {}", receipt.nonce),
        Err(e) => println!("  ❌ Rejected: {}", e),
    }
    
    // Show final state
    println!("\n=== Final State ===");
    println!("Pending deposits: {}", mailbox.pending_deposits.len());
    println!("Rejected deposits: {}", mailbox.rejected_deposits.len());
    
    // Consume the accepted deposits
    let consumed = mailbox.consume_deposits();
    println!("\nConsumed {} deposits into the linear system", consumed.len());
    for (token, balance) in &mailbox.consumed_balance {
        println!("  {}: {} (now linearly tracked)", token.0, balance);
    }
    
    println!("\n=== Key Properties ===");
    println!("• Deposits can be rejected based on configurable conditions");
    println!("• Rejected deposits NEVER enter the linear session system");
    println!("• Anyone can trigger refunds for rejected deposits");
    println!("• The blockchain account can receive tokens, but the session");
    println!("  system only sees deposits that pass all constraints");
    println!("• This enables constrained recipient accounts that serve as");
    println!("  safe intermediate holding points in protocols");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_safe_deposit_demo() {
        super::main();
    }
} 