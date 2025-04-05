# TEL Syntax Redesign - Workshop Document

## Goals

- Create a **clean, intuitive syntax** for TEL that can be learned in an afternoon
- Provide **better abstractions** for expressing concurrency, control flow, and state management
- Support the pure functional architecture with a unified effect system
- Use a more functional style with fewer braces
- Integrate **algebraic effects** as the foundation of the language
- Make **temporal effects** first-class citizens
- Unify all language concepts through a cohesive design
- Leverage **content addressing** for all data references and state management

## Core Design Principles

1. **Declarative over imperative** - Focus on describing "what" should happen, not "how"
2. **Functional paradigm** - Embrace immutability and expression-based programming
3. **Effect-driven design** - All computations with side effects are represented as algebraic effects
4. **Unified language model** - Core concepts (state, time, queries, resources) share the same foundation
5. **Whitespace significance** - Use indentation rather than braces for blocks 
6. **Pipeline style** - Use pipe operators for chaining operations
7. **Temporal awareness** - Time is a first-class concept in the language
8. **Content addressing** - All entities are referenced by content hash, not arbitrary identifiers

## Content Addressing as a Fundamental Concept

TEL incorporates content addressing at its core, where all entities are identified by their content hash rather than arbitrary identifiers:

```tel
-- Content-addressed resource reference (automatically hashed)
let token_data = {
  name: "Transfer Token",
  symbol: "TT",
  total_supply: 1000000
}

-- Reference is the hash of the content
let token_id = content_id(token_data)  -- Returns Hash<TokenData>

-- Content-addressed operations
perform transfer {
  from: sender_id,  -- Hash<Account>
  to: receiver_id,  -- Hash<Account>
  token: token_id,  -- Hash<TokenData>
  amount: amount
}
```

Content addressing provides several key benefits:

1. **Verifiability** - Any reference can be verified against its content
2. **Immutability** - Data is inherently immutable; changes produce new content IDs
3. **Deduplication** - Identical data has identical references
4. **Deterministic execution** - Operations on the same content always yield the same results
5. **Transparency** - All data dependencies are explicit through their content IDs

### Content-Addressed State Transitions

State transitions produce new content-addressed states:

```tel
-- State transitions generate new content-addressed states
effect transition : State -> <state, logging> Hash<State>

handler content_addressed_state initial_state_hash
  var current_state_hash = initial_state_hash
  
  effect transition new_state ->
    -- Compute new state hash
    let new_state_hash = content_id(new_state)
    
    -- Record state transition
    perform record_state_transition {
      from: current_state_hash,
      to: new_state_hash,
      timestamp: now()
    }
    
    -- Update current state
    current_state_hash <- new_state_hash
    
    -- Return new state hash
    resume new_state_hash
```

### Content-Addressed Data Queries

Queries operate on and return content-addressed data:

```tel
-- Query returns content-addressed results
result_hash <- perform query
  from accounts
  where account.owner == owner_hash
  select account

-- Load content by hash
account <- perform load result_hash
```

## Language Foundations

TEL is built on a unified foundation where all language features are expressed through algebraic effects and content addressing:

```tel
-- Define a program
program TokenSwap
  -- Declare input parameters with types
  input
    sender : Hash<Account>
    token_a : Hash<Token>
    amount_a : Amount
    token_b : Hash<Token>
    min_amount_b : Amount
    deadline : Timestamp

  -- Main workflow
  flow main = do
    -- Check we're within the deadline
    valid <- perform within_window (now(), deadline)
    unless valid do
      failed_state <- perform transition Failed "Outside execution window"
      return ()
      
    -- Check approvals
    approved_state <- perform transition Approved
    
    -- Transfer tokens from sender
    transfer_result <- perform transfer {
      from: sender,
      token: token_a,
      amount: amount_a,
      to: self()
    }

    -- Handle potential failure
    if not transfer_result.success then
      failed_state <- perform transition Failed transfer_result.reason
      return ()
    
    -- Execute the swap
    swapping_state <- perform transition Swapping
    swap_result <- perform swap {
      token_in: token_a,
      amount_in: amount_a,
      token_out: token_b,
      min_amount_out: min_amount_b
    }

    -- Handle result using pattern matching
    perform match swap_result {
      Success result -> do
        -- Transfer the output tokens to sender
        return_result <- perform transfer {
          from: self(),
          token: token_b,
          amount: result.amount_out,
          to: sender
        }
        
        perform match return_result {
          Success _ -> 
            completed_state <- perform transition Completed
          Failure reason -> 
            failed_state <- perform transition Failed reason
        }
            
      Failure reason ->
        failed_state <- perform transition Failed reason
    }
```

## Unified Effect System

All aspects of TEL are expressed through a single, unified effect system with content addressing:

### 1. Effect Operations and Handlers

Effects operate on content-addressed data:

```tel
-- Define a content-addressed effect operation
effect transfer : TransferParams -> <transfer, failure> Hash<TransferResult>

-- Define an effect handler
handler transfer_handler
  effect transfer params ->
    -- Verify all referenced entities exist
    let from_account = perform load params.from
    let token_data = perform load params.token
    
    -- Implement the transfer logic
    if has_sufficient_balance(from_account, token_data, params.amount) then
      -- Execute transfer and compute result
      result <- perform_transfer(params)
      
      -- Return content ID of the result
      let result_hash = content_id(result)
      
      -- Store the result
      perform store result_hash result
      
      -- Return the content hash
      resume result_hash
    else
      -- Create failure result
      let failure = { success: false, reason: "Insufficient balance" }
      
      -- Store and return failure
      let failure_hash = content_id(failure)
      perform store failure_hash failure
      resume failure_hash
```

### 2. Content-Addressed State Machine

States are content-addressed and transitions produce new state hashes:

```tel
-- State machine definition
state
  initial Pending
  Approved
  Swapping
  final Completed
  final Failed reason:String

-- State transition effect returns content hash of new state
effect transition : State -> <state, logging> Hash<State>

-- State machine handler
handler state_machine_handler initial_state
  -- Store initial state and get its hash
  var current_state_hash = perform store_state initial_state
  
  effect transition new_state ->
    -- Load current state
    let current_state = perform load_state current_state_hash
    
    -- Validate the transition
    if valid_transition(current_state, new_state) then
      -- Log the transition
      perform log "Transitioning from {current_state} to {new_state}"
      
      -- Store new state
      let new_state_hash = perform store_state new_state
      
      -- Create a causal link between states
      perform link {
        from: current_state_hash,
        to: new_state_hash,
        relationship: "state_transition"
      }
      
      -- Update current state
      current_state_hash <- new_state_hash
      
      -- Return new state hash
      resume new_state_hash
    else
      perform failure "Invalid transition: {current_state} -> {new_state}"
      resume default_hash
      
  effect get_state ->
    -- Return hash of current state
    resume current_state_hash
```

### 3. Verification and Proofs

Content addressing enables built-in verification:

```tel
-- Verify data matches its hash
effect verify : (Hash<T>, T) -> <verification> Boolean

-- Generate merkle proofs for data
effect generate_proof : Hash<T> -> <merkle> MerkleProof<T>

-- Verify a merkle proof
effect verify_proof : MerkleProof<T> -> <verification> Boolean

-- Using verification
is_valid <- perform verify (token_id, token_data)

-- Generate proof of inclusion
proof <- perform generate_proof transaction_hash

-- Verify proof
is_included <- perform verify_proof proof
```

### 4. Content-Addressed Temporal Effects

Temporal effects work with content-addressed states:

```tel
-- Schedule an effect based on content-addressed trigger
effect at : Timestamp -> (() -> <e> Hash<Result>) -> <scheduler, e> Hash<Task>

-- Schedule based on state transition
effect after_state : Hash<State> -> (() -> <e> Hash<Result>) -> <scheduler, e> Hash<Task>

-- Schedule a content-addressed task
task_hash <- perform at tomorrow_midnight do
  -- This will run at the specified time
  settlement_result <- perform daily_settlement
  return settlement_result

-- Schedule task to run after a specific state is reached
task_hash <- perform after_state completed_state_hash do
  -- This will run after the state transition
  notification_result <- perform notify_completion
  return notification_result
```

### 5. Causal Relationships

Content addressing enables explicit causal relationships:

```tel
-- Define a causal relationship between content-addressed entities
effect link : LinkSpec -> <causality> Hash<Link>

-- Create a causal dependency
link_hash <- perform link {
  from: cause_hash,
  to: effect_hash,
  relationship: "caused_by"
}

-- Query causal graph
ancestors <- perform query
  from causal_graph
  start_at entity_hash
  traverse incoming "caused_by"
  collect nodes
```

## Practical Examples

### Example: Content-Addressed DEX Order Book

```tel
program LimitOrderBook
  -- Order placement flow
  flow place_order = do
    input
      trader : Hash<Account>
      market : Hash<Market>
      side : Side
      amount : Amount
      price : Price
      
    -- Create order data
    let order_data = {
      trader: trader,
      market: market,
      side: side,
      amount: amount,
      price: price,
      status: "Active",
      created_at: now()
    }
    
    -- Store and get content hash
    order_hash <- perform store order_data
    
    -- Update order book
    book_hash <- perform query
      from order_books
      where order_book.market == market
      select order_book
      
    -- Load order book
    book <- perform load book_hash
    
    -- Add order to book
    let updated_book = add_order(book, order_hash)
    
    -- Store updated book
    updated_book_hash <- perform store updated_book
    
    -- Create causal link
    perform link {
      from: book_hash,
      to: updated_book_hash,
      relationship: "updated_by"
    }
    
    -- Return order hash
    return order_hash
```

### Example: Content-Addressed Auction with Proof Generation

```tel
program VerifiableAuction
  flow initialize = do
    input
      seller : Hash<Account>
      item : Hash<Item>
      reserve_price : Amount
      duration : Duration
      
    -- Create auction data
    let auction_data = {
      seller: seller,
      item: item,
      reserve_price: reserve_price,
      start_time: now(),
      end_time: now() + duration,
      highest_bid: None,
      highest_bidder: None,
      status: "Active"
    }
    
    -- Store and get content hash
    auction_hash <- perform store auction_data
    
    -- Schedule auction end
    task_hash <- perform at auction_data.end_time do
      -- End the auction
      finalized_auction_hash <- perform end_auction auction_hash
      
      -- Generate inclusion proof for the final state
      proof <- perform generate_proof finalized_auction_hash
      
      -- Store the proof for verification
      proof_hash <- perform store proof
      
      -- Return finalized auction
      return finalized_auction_hash
      
    -- Return auction hash
    return auction_hash
```

## Benefits of Content-Addressed TEL

1. **Verifiability** - Any data or state can be verified against its claimed hash
2. **Traceability** - Complete history of state transitions is preserved and verifiable
3. **Reproducibility** - Computations can be reproduced with the same inputs
4. **Composability** - Content-addressed components can be safely composed
5. **Introspection** - The system can reason about its own data and state
6. **Interoperability** - Content addressing provides a universal way to reference data
7. **Determinism** - Execution is fully deterministic based on content-addressed inputs
8. **Integrity** - Data integrity is assured by the content addressing system

## Migration Guide

For users familiar with the current TEL syntax, here's a comparison guide:

| Current TEL | New TEL | Notes |
|-------------|---------|-------|
| `transfer(from, to, amount)` | `perform transfer { from: from, to: to, amount: amount }` | Effects are explicit with `perform` |
| `if condition { ... }` | `if condition then ...` | No braces, whitespace significant |
| Implicit states | `perform transition NewState` | Explicit state transitions as effects |
| Implicit control flow | `flow name = do ...` | Named, explicit flows with do notation |
| Ad-hoc concurrency | `perform spawn do ...` | Effect-based concurrency |
| `x.method()` | `x |> method` | Pipeline style for method chaining |
| Separate systems | Unified effect system | All operations share the same foundation |

## Workshop Discussion Questions

1. How do we achieve the right balance between abstraction and clarity?
2. What additional effect handlers would be valuable in the standard library?
3. How should we approach teaching this unified model to developers?
4. What performance optimizations are most important for the effect system?
5. How can we best leverage the unified design for formal verification?
6. How can we best utilize content addressing for cross-system interoperability?
7. What verification capabilities should we build into the standard library? 