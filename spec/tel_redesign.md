# TEL Syntax Redesign - Workshop Document

## Goals

- Create a **clean, intuitive syntax** for TEL that can be learned in an afternoon
- Provide **better abstractions** for expressing concurrency
- Improve **control flow** patterns
- Ensure a **complete state space** with clear success/failure paths
- Make program correctness easy to verify 
- Support the pure functional architecture with effect system
- Use a more functional style with fewer braces
- Incorporate **algebraic effects** inspired by Eff and Koka languages
- Integrate **temporal effects** as first-class citizens in the language

## Core Design Principles

1. **Declarative over imperative** - Focus on describing "what" should happen, not "how"
2. **Functional paradigm** - Embrace immutability and expression-based programming
3. **Explicit state transitions** - All state changes are explicit and trackable
4. **Whitespace significance** - Use indentation rather than braces for blocks 
5. **Pipeline style** - Use pipe operators for chaining operations
6. **Effect isolation** - Effects are clearly distinguished from pure computation
7. **Algebraic effects** - Effects are first-class values with explicit handlers
8. **Effect typing** - Track effects in the type system
9. **Temporal awareness** - Time is a first-class concept in the language

## Syntax Overview

```tel
-- This is a comment
{- This is a multi-line comment -}

-- Define a program
program TokenSwap
  -- Declare input parameters with types
  input
    sender : Address
    token_a : TokenId
    amount_a : Amount
    token_b : TokenId
    min_amount_b : Amount
    deadline : Timestamp

  -- Declare possible states
  state
    initial Pending
    Approved
    Swapping
    final Completed
    final Failed reason:String
    
  -- Define temporal windows
  time
    execution_window = between now() and deadline
    reporting_window = after deadline for 24h

  -- Define effect handlers
  handlers
    -- Define a custom handler for transfer errors
    handler transfer_error
      effect transfer_failed reason -> 
        log "Transfer failed: {reason}"
        transition Failed reason
        
    -- Define a custom handler for timeouts
    handler deadline_check
      effect check_deadline current_time deadline ->
        if current_time > deadline then
          log "Deadline exceeded"
          transition Failed "Deadline exceeded"
          resume false
        else
          resume true
          
    -- Define temporal effect handler
    handler time_window_handler
      effect within_window window ->
        current <- now()
        if window.contains(current) then
          resume true
        else
          resume false

  -- Define the main workflow
  flow main = do
    -- Check we're in the valid time window
    in_window <- perform within_window execution_window
    unless in_window do
      transition Failed "Outside execution window"
      return ()
    
    -- Check time constraints using an effect
    valid <- perform check_deadline (now()) deadline
    
    unless valid do
      return ()
      
    -- Check approvals
    transition Approved
    
    -- Execute concurrent operations with explicit dependencies
    concurrent do
      -- Transfer tokens from sender
      transfer_result <- effect transfer
        from: sender
        token: token_a
        amount: amount_a
        to: self()

      -- Handle potential failure
      if not transfer_result.success then
        perform transfer_failed transfer_result.reason
      
    -- Execute the swap
    transition Swapping
    swap_result <- effect swap
      token_in: token_a
      amount_in: amount_a
      token_out: token_b
      min_amount_out: min_amount_b

    -- Handle result using pattern matching
    case swap_result of
      Success result ->
        -- Transfer the output tokens to sender
        return_result <- effect transfer
          from: self()
          token: token_b
          amount: result.amount_out
          to: sender
        
        case return_result of
          Success _ -> 
            transition Completed
          Failure reason -> 
            transition Failed reason
            
      Failure reason ->
        transition Failed reason
        
  -- Define a time-triggered flow
  flow report after deadline = do
    -- This flow automatically executes after the deadline
    current <- getCurrentState
    
    case current of
      Completed ->
        effect send_notification
          to: sender
          message: "Swap completed successfully"
      Failed reason ->
        effect send_notification
          to: sender
          message: "Swap failed: {reason}"
      _ ->
        transition Failed "Timed out"
```

## Key Language Features

### 1. State Machine Model

TEL programs follow a clear state machine model:

```tel
state
  initial Starting
  Processing
  final Success
  final Failure reason:String
```

- Each program has explicitly defined states
- States are marked as `initial` or `final`
- States can carry data (like error reasons)
- All transitions between states are explicit
- Final states clearly indicate success/failure

### 2. Do Notation for Flows

The `flow` construct uses do notation for a sequence of operations:

```tel
flow process_payment = do
  -- Sequence of operations with state transitions
```

Flow blocks can contain:

- Conditional branching with `if ... then ... else`
- State transitions with `transition State`
- Effect execution with `effect_name params`
- Variable declarations with `<-` for effects and `let` for pure values
- Pattern matching with `case ... of`

### 3. Algebraic Effects and Handlers

Inspired by Eff and Koka, TEL supports algebraic effects with explicit handlers:

```tel
-- Define an effect operation
effect check_balance : Address -> Token -> <balance,failure> Amount

-- Define a handler for an effect
handler balance_tracking
  effect check_balance address token ->
    balance <- query_actual_balance address token
    if balance > 0 then
      resume balance
    else
      perform failure "Insufficient balance"
      
  effect failure reason ->
    log "Operation failed: {reason}"
    resume unit
```

- Effects are first-class values in the language
- Effects are typed with their potential outcomes
- Handlers can intercept effects and provide implementations
- Handlers can be composed and nested
- Effects can be resumed with a value or short-circuited

### 4. Effect Type System

TEL has an effect type system inspired by Koka:

```tel
-- A function with a pure return type (no effects)
pure calculate_fee : Amount -> Amount
calculate_fee amount = amount * 0.01

-- A function with effects
check_and_transfer : Address -> Amount -> <transfer,failure> Boolean
check_and_transfer recipient amount = do
  balance <- perform check_balance self() token
  if balance >= amount then
    result <- effect transfer to: recipient amount: amount
    return result
  else
    perform failure "Insufficient balance"
    return false
```

- Function types track their potential effects
- Effect polymorphism allows for generic handling
- Effect inference reduces annotation burden
- Row polymorphism for extensible effect types
- Local effect handling with delimited continuations

### 5. Concurrency Model

The `concurrent` block expresses parallel operations:

```tel
concurrent do
  -- These operations happen concurrently
  result_a <- effect operation_a param1 param2
  result_b <- effect operation_b param3 param4
  
  -- Dependencies are expressed through variable usage
  let combined = process result_a result_b
```

- Operations within a concurrent block can execute in parallel
- Dependencies between operations are automatically detected
- Synchronization happens implicitly at variable access
- Explicit barriers can be defined with `sync`
- Structured concurrency based on algebraic effects (like Koka)

### 6. Effect System Integration

Effects are explicitly invoked using the `effect` keyword:

```tel
result <- effect effect_name
  param1: value1
  param2: value2
```

Or using the `perform` keyword for algebraic effects:

```tel
result <- perform my_effect arg1 arg2
```

- All side effects are explicit and tracked
- Effect results are bound with `<-`
- Effects are categorized by visibility layers
- Pure functions use normal invocation syntax: `function arg1 arg2`
- Effects can be handled locally or propagated

### 7. Pattern Matching

Pattern matching is a primary construct for handling data:

```tel
case result of
  Success value ->
    -- Handle success case
  Failure reason ->
    -- Handle failure case with reason
```

### 8. Pipeline Operators

Functions can be chained using pipeline operators:

```tel
result =
  getData
    |> transform
    |> validate
    |> formatOutput
```

### 9. Resource Guards

Resource acquisition uses a `with` expression:

```tel
with resource <- acquire_resource param1 do
  -- Use resource here
  -- Automatically released after block
```

### 10. Temporal Effects

TEL treats time as a first-class concept with explicit temporal effects:

```tel
-- Define time windows
time
  execution_window = between start_time and end_time
  cooling_period = after end_time for 24h
  
-- Time-based effect operations
effect schedule : <time> Task -> <scheduler> TaskId
effect cancel_task : <scheduler> TaskId -> <scheduler> Unit
effect delay : <time> Duration -> <time> Unit
effect at : <time> Timestamp -> a -> <time> a
```

Temporal effects enable explicit reasoning about:

- Time windows and deadlines
- Scheduled operations
- Time-based state transitions
- Temporal dependencies between operations
- Rate limiting and cooldown periods

### 11. Time-Triggered Flows

Flows can be triggered by temporal conditions:

```tel
-- Flow that executes at a specific time
flow daily_settlement at 00:00 UTC = do
  -- Settlement logic
  
-- Flow that executes after an event
flow finalize after timeout = do
  -- Finalization logic
  
-- Flow that executes periodically
flow heartbeat every 5m = do
  -- Health check logic
```

Time-triggered flows provide:

- Declarative scheduling
- Automatic execution at specified times
- Time-based error recovery
- Deadline enforcement
- Temporal integrity guarantees

## Algebraic Effects in Detail

Inspired by Eff and Koka, TEL's effect system provides fine-grained control over computations with side effects:

### Defining Effects

```tel
-- Define effect signatures
effect read_state : <state> State
effect write_state : State -> <state> Unit
effect fail : String -> <failure> Never
effect random : <random> Int
```

### Using Effects

```tel
flow process_data = do
  -- Invoke effects with perform
  current <- perform read_state
  r <- perform random
  
  -- Conditional based on effect results
  if r > 10 && current.valid then
    new_state = update current r
    perform write_state new_state
    return new_state.value
  else
    perform fail "Invalid state or random value"
```

### Handling Effects

```tel
handler state_handler initial_state
  effect read_state -> 
    resume initial_state
    
  effect write_state new_state ->
    resume unit with state_handler new_state
    
handler failure_handler
  effect fail msg ->
    log "Failure: {msg}"
    return default_value
    
-- Applying handlers to a computation
handle process_data with 
  state_handler initial_state
  then failure_handler
```

### Effect Composition

```tel
-- Compose multiple handlers
with state <- handle_state do
  with errors <- handle_errors do
    with logging <- handle_logging do
      process_data state
```

### Effect Inference

TEL uses effect inference to reduce the annotation burden:

```tel
-- No need to annotate all effects, they are inferred
flow process = do
  -- This automatically has <state,random,failure> effects
  data <- process_data
  -- Rest of the computation
```

### Temporal Effect Handlers

TEL provides specialized handlers for managing temporal effects:

```tel
-- Define a deadline handler
handler deadline_handler deadline
  effect check_deadline ->
    current <- now()
    if current > deadline then
      perform timeout
      resume false
    else
      resume true
      
  effect timeout ->
    log "Operation timed out"
    transition Timeout
    
-- Define a rate limiting handler
handler rate_limiter max_calls period
  var calls = 0
  var window_start = now()
  
  effect throttled_call operation ->
    current <- now()
    
    -- Reset window if needed
    if current > window_start + period then
      calls <- 0
      window_start <- current
    
    -- Check rate limit
    if calls < max_calls then
      calls <- calls + 1
      result <- operation()
      resume result
    else
      perform rate_limited
      resume default_value
      
  effect rate_limited ->
    log "Rate limit exceeded"
```

### Time Windows

Time windows are first-class values that can be manipulated and composed:

```tel
-- Define time windows
let trading_hours = between 9:00 and 17:00 on weekdays
let maintenance_window = between 2:00 and 4:00 on weekends

-- Combine windows
let active_window = trading_hours except maintenance_window

-- Use in conditions
if within active_window then
  -- Execute trading logic
else
  -- Execute off-hours logic
```

## Concrete Examples

### Example 1: Simple Token Transfer with Effect Handlers

```tel
program TokenTransfer
  input
    sender : Address
    recipient : Address
    token : TokenId
    amount : Amount

  state
    initial Pending
    Approved
    final Completed
    final Failed reason:String
    
  handlers
    handler transfer_handling
      effect insufficient_balance ->
        transition Failed "Insufficient balance"
        resume unit
        
      effect transfer_failed reason ->
        transition Failed reason
        resume unit

  flow main = do
    -- Check if sender has enough balance
    balance <- effect query_balance
      address: sender
      token: token

    if balance < amount then
      perform insufficient_balance
      return ()
      
    transition Approved

    -- Execute the transfer
    transfer_result <- effect transfer
      from: sender
      token: token
      amount: amount
      to: recipient

    -- Process result with pattern matching
    case transfer_result of
      Success _ ->
        transition Completed
      Failure reason ->
        perform transfer_failed reason
```

### Example 2: Atomic Swap Between Users with Advanced Error Handling

```tel
program AtomicSwap
  input
    party_a : Address
    token_a : TokenId
    amount_a : Amount
    party_b : Address
    token_b : TokenId
    amount_b : Amount
    timeout : Timestamp

  state
    initial Created
    PartyADeposited
    PartyBDeposited
    final Completed
    final Cancelled
    final TimedOut
    
  -- Define effects
  effect collect : Address -> Token -> Amount -> <deposit,failure> DepositResult
  effect refund : Address -> Token -> Amount -> <transfer,failure> Unit
  effect timeout_check : Timestamp -> <time,failure> Boolean
  
  handlers
    handler deposit_handler
      effect deposit address token amount ->
        result <- effect collect_deposit
          from: address
          token: token
          amount: amount
        resume result
          
      effect failure reason ->
        log "Operation failed: {reason}"
        transition Cancelled
        resume unit
        
    handler timeout_handler current_time
      effect timeout_check deadline ->
        if current_time > deadline then
          transition TimedOut
          resume true
        else
          resume false

  flow main = do
    -- Handle with our custom handlers
    handle with 
      deposit_handler 
      then timeout_handler (now())
      do
      
      -- Check for timeout
      timed_out <- perform timeout_check timeout
      when timed_out do
        return ()
      
      -- Collect deposit from Party A
      deposit_a <- perform collect party_a token_a amount_a
      
      case deposit_a of
        Failure _ ->
          return ()
        Success _ ->
          transition PartyADeposited
          
          -- Collect deposit from Party B
          deposit_b <- perform collect party_b token_b amount_b
          
          case deposit_b of
            Failure _ ->
              -- Refund Party A and cancel
              perform refund party_a token_a amount_a
              return ()
              
            Success _ ->
              transition PartyBDeposited
  
              -- Execute the swap atomically
              concurrent do
                transfer_to_b <- effect transfer
                  from: self()
                  to: party_b
                  token: token_a
                  amount: amount_a
  
                transfer_to_a <- effect transfer
                  from: self()
                  to: party_a
                  token: token_b
                  amount: amount_b
  
                if transfer_to_a.success && transfer_to_b.success then
                  transition Completed
                else
                  transition Failed "Transfer failed"

  -- Handle timeout situation
  flow timeout = do
    if now() > timeout then
      current <- getCurrentState
      
      case current of
        PartyADeposited ->
          effect refund
            to: party_a
            token: token_a
            amount: amount_a
          transition TimedOut
            
        PartyBDeposited ->
          effect refund
            to: party_a
            token: token_a
            amount: amount_a
          effect refund
            to: party_b
            token: token_b
            amount: amount_b
          transition TimedOut
            
        _ ->
          pure ()
```

### Example 3: Auction with Temporal Effects

```tel
program Auction
  input
    seller : Address
    item : ItemId
    start_price : Amount
    reserve_price : Amount
    start_time : Timestamp
    end_time : Timestamp

  state
    initial Created
    Active
    final Completed buyer:Address final_price:Amount
    final Cancelled reason:String
    
  time
    bidding_window = between start_time and end_time
    grace_period = after end_time for 15m
    
  -- Define temporal effects
  effect within_window : TimeWindow -> <time> Boolean
  effect schedule_end : Timestamp -> <scheduler> TaskId
  
  handlers
    handler time_window_handler
      effect within_window window ->
        current <- now()
        if window.contains(current) then
          resume true
        else
          resume false
  
  -- Track auction state
  data
    highest_bid : Amount = start_price
    highest_bidder : Maybe Address = Nothing
    end_task : Maybe TaskId = Nothing

  flow start = do
    -- Check if we're before the start time
    in_window <- perform within_window bidding_window
    
    unless in_window do
      when (now() > end_time) do
        transition Cancelled "Auction start time has passed"
      return ()
      
    -- Schedule the auction end
    task_id <- perform schedule_end end_time
    end_task <- Just task_id
    
    transition Active
  
  flow place_bid = do
    input
      bidder : Address
      bid_amount : Amount
      
    -- Check auction is active
    state <- getCurrentState
    unless (state == Active) do
      return Failure "Auction is not active"
    
    -- Check we're within the bidding window
    in_window <- perform within_window bidding_window
    unless in_window do
      return Failure "Bidding window has closed"
    
    -- Check bid is higher than current bid
    unless (bid_amount > highest_bid) do
      return Failure "Bid is too low"
    
    -- Update highest bid
    highest_bid <- bid_amount
    highest_bidder <- Just bidder
    
    return Success unit
  
  -- This flow is triggered automatically at the end time
  flow end_auction at end_time = do
    -- Check if auction is active
    state <- getCurrentState
    unless (state == Active) do
      return ()
    
    -- Check if reserve price was met
    case highest_bidder of
      Just bidder ->
        if highest_bid >= reserve_price then
          -- Transfer the item
          effect transfer_item
            from: seller
            to: bidder
            item: item
            
          -- Transfer the payment
          effect transfer_payment
            from: bidder
            to: seller
            amount: highest_bid
            
          transition Completed bidder highest_bid
        else
          transition Cancelled "Reserve price not met"
          
      Nothing ->
        transition Cancelled "No bids received"
```

### Example 4: Time-Based Protocol with Temporal Effects

```tel
program TimelockedProtocol
  input
    initiator : Address
    participant : Address
    secret_hash : Hash
    deposit_amount : Amount
    lock_time : Duration
    
  state
    initial Created
    InitiatorFunded
    ParticipantFunded
    final Claimed
    final Refunded
    final Expired
    
  time
    funding_window = for 24h after creation
    claim_window = for lock_time after both_funded
    refund_window = after claim_window.end
    
  data
    creation_time : Timestamp = now()
    both_funded_time : Maybe Timestamp = Nothing
    
  flow fund_initiator = do
    -- Check if we're in the funding window
    unless (now() < creation_time + 24h) do
      transition Expired
      return Failure "Funding window expired"
    
    -- Collect deposit from initiator
    result <- effect collect_deposit
      from: initiator
      amount: deposit_amount
      
    case result of
      Success _ ->
        transition InitiatorFunded
        return Success unit
      Failure reason ->
        return Failure reason
  
  flow fund_participant = do
    -- Check if initiator has funded
    state <- getCurrentState
    unless (state == InitiatorFunded) do
      return Failure "Initiator hasn't funded yet"
      
    -- Check if we're in the funding window
    unless (now() < creation_time + 24h) do
      transition Expired
      return Failure "Funding window expired"
    
    -- Collect deposit from participant
    result <- effect collect_deposit
      from: participant
      amount: deposit_amount
      
    case result of
      Success _ ->
        both_funded_time <- Just (now())
        transition ParticipantFunded
        
        -- Schedule the expiration check
        effect schedule_task
          at: now() + lock_time
          operation: "check_expiration"
          
        return Success unit
      Failure reason ->
        return Failure reason
  
  flow claim = do
    input
      claimer : Address
      secret : Secret
      
    -- Check state is properly funded
    state <- getCurrentState
    unless (state == ParticipantFunded) do
      return Failure "Contract not properly funded"
      
    -- Verify the secret
    unless (hash(secret) == secret_hash) do
      return Failure "Invalid secret"
      
    -- Check we're in the claim window
    case both_funded_time of
      Just funded_time ->
        unless (now() < funded_time + lock_time) do
          return Failure "Claim window expired"
          
        -- Transfer funds to the claimer
        effect transfer
          to: claimer
          amount: deposit_amount * 2
          
        transition Claimed
        return Success unit
        
      Nothing ->
        return Failure "Contract not properly funded"
  
  -- Time-triggered flow to check for expiration
  flow check_expiration at both_funded_time + lock_time = do
    state <- getCurrentState
    
    -- If still in participant funded state after lock time, allow refund
    when (state == ParticipantFunded) do
      effect transfer
        to: initiator
        amount: deposit_amount
        
      effect transfer
        to: participant
        amount: deposit_amount
        
      transition Expired
```

## Type System

TEL has a strong, static type system with effect tracking:

```tel
-- Type definitions
type OrderParams =
  { token : TokenId
  , price : Price
  , quantity : Quantity
  }

-- Type aliases
type alias Price = Amount

-- Generic types
type Result a
  = Success a
  | Failure String

-- Enum types
type OrderSide
  = Buy
  | Sell
  
-- Effect type annotations
type CreateOrder : OrderParams -> <order,failure> OrderId
type CancelOrder : OrderId -> <order,failure> Boolean

-- Row polymorphism for effects (inspired by Koka)
type CheckValidity : a -> <e> Boolean with e : failure

### Temporal Types

TEL includes specialized types for working with time:

```tel
-- Basic temporal types
type Timestamp
type Duration
type TimeWindow

-- Temporal effect types
type WithinWindow : TimeWindow -> <time> Boolean
type After : Timestamp -> <time> Boolean
type Delay : Duration -> <time> Unit

-- Higher-order temporal types
type Schedule : Timestamp -> (() -> <e> a) -> <scheduler|e> TaskId
type Periodic : Duration -> (() -> <e> a) -> <scheduler|e> TaskId

-- Composite temporal types
type TimeBound a = 
  { value : a
  , valid_from : Timestamp
  , valid_until : Timestamp
  }
```

## Advanced Features

### Composability with Sub-Flows

```tel
flow main = do
  -- Execute a sub-flow
  result <- run purchase_flow
    user: sender
    amount: requested_amount

-- Define a reusable sub-flow
flow purchase_flow = do
  input
    user : Address
    amount : Amount
  
  -- Flow implementation
```

### Safe State Recovery

```tel
flow main = do
  -- Define recovery behavior for specific states
  recovery
    Pending -> run cancel_flow
    Processing -> run verify_and_complete_flow
  
  -- Main flow logic
```

### Conditional Execution

```tel
-- Execute a block only if a condition is met
when (amount > threshold) do
  effect special_processing amount: amount

-- Execute different blocks based on a condition
case user_type of
  Premium ->
    -- Premium user logic
  Regular ->
    -- Regular user logic
```

### Local Effect Handling

```tel
flow process_with_retries = do
  -- Handle specific effects locally
  handle with retry_handler max_retries do
    result <- perform operation args
    
    -- This code runs with retry_handler active
    process result
    
  -- This code runs after the handler scope ends
  finalize result
```

### Temporal Resource Management

```tel
-- Define a temporal resource with automatic cleanup
with_time_bound resource <- acquire_for duration do
  -- Use resource here
  -- Automatically released after duration

-- Define a resource within a time window
during trading_hours do
  -- Operations permitted only during trading hours
  -- Automatically suspended outside the window
  
-- Rate-limiting block
throttled max_calls per period do
  -- Operations inside this block are rate-limited
```

### Temporal Versioning and Scheduling

```tel
-- Define an upgrade with temporal controls
upgrade MyContract to version = "2.0.0"
  at: tomorrow at 00:00 UTC
  with: migration_plan
  
-- Specify temporally-aware execution logic
time_sensitive do
  -- High-priority operations
  before deadline do
    -- Critical path
  after deadline do
    -- Fallback path
```

### Temporal Effect Analysis

```tel
-- Static analysis for time bounds
<time_bound 2h> expensive_operation params
  -- Statically enforced maximum execution time
  
-- Deadline analysis
<deadline timestamp> critical_section
  -- Static verification of meeting deadline
```

## Temporal Patterns

TEL supports common temporal patterns with concise syntax:

### Time-Based State Transitions

```tel
-- State transitions after specified time
transition Expired after timeout

-- Conditional state transitions based on time
when after deadline && state == Pending do
  transition Expired
```

### Timeouts and Deadlines

```tel
-- Execute with timeout
with_timeout 5m do
  result <- expensive_operation()
  process result
  
-- Execute with deadline
before deadline do
  -- Fast path
else
  -- Slow path
```

### Periodic and Scheduled Execution

```tel
-- Execute periodically
every 1h do
  perform maintenance_task
  
-- Execute at specific time
at 00:00 UTC daily do
  perform settlement
```

### Time Windows

```tel
-- Execute during time window
during trading_hours do
  -- Trading operations
else
  -- Off-hours operations
  
-- Execute outside time window
unless during maintenance_window do
  -- Normal operations
```

### Temporal Causality

```tel
-- Ensure temporal ordering
after event_a before event_b do
  -- Operations that must happen between A and B
  
-- Ensure time-based preconditions
require event_a happened within last 24h
require event_b not_happened since event_a
```

## Time-Based Concurrency Primitives

TEL leverages its first-class time effects to provide novel concurrency and flow control primitives that go beyond traditional approaches:

### Temporal Barriers

```tel
-- Define a barrier that synchronizes concurrent operations at a specific time
time_barrier voting_end = at 12:00 UTC

-- Use the barrier to coordinate actions
concurrent do
  -- These operations run concurrently
  result_a <- perform operation_a
  result_b <- perform operation_b

  -- Wait until the temporal barrier before proceeding
  await time_barrier voting_end
  
  -- These operations will execute only after the barrier time
  perform finalize_results result_a result_b
```

### Causal Flows

```tel
-- Define a sequence of causally related time-based operations
causal_flow do
  -- Operation A happens first
  result_a <- perform operation_a
  
  -- Operation B happens strictly after A, with minimum delay
  after 5m then
    result_b <- perform operation_b result_a
  
  -- Operation C happens strictly after B, with exact timing
  at result_b.timestamp + 1h then
    perform operation_c result_b
```

### Temporal Race

```tel
-- Execute operations in a temporal race pattern
race
  -- First branch with timing constraint
  within 5m do
    result <- perform fast_operation
    return result
    
  -- Second branch (fallback if first doesn't complete in time)
  after 5m do
    result <- perform backup_operation
    return result
    
  -- Branch that happens at a specific time regardless
  at deadline do
    return default_result
```

### Time-Sliced Execution

```tel
-- Execute tasks in time slices with fairness guarantees
time_sliced 100ms do
  -- These tasks will get fair time allocation
  task_a <- time_slice do
    perform long_running_task_a
    
  task_b <- time_slice do
    perform long_running_task_b
    
  -- Join the results when both complete
  join task_a task_b
```

### Temporal Backpressure

```tel
-- Rate-limit operations based on time
with_backpressure max_per_second: 10 do
  -- These operations are automatically rate-limited
  for item in items do
    perform process_item item
    
  -- The system will automatically throttle to maintain the rate
```

### Speculative Execution with Timeout

```tel
-- Try a speculative path with temporal constraints
speculate 
  -- Main speculation path
  primary do
    result <- perform optimistic_operation
    if is_valid result then
      commit result
      
  -- Alternative path if primary doesn't complete in time
  timeout 5s do
    perform pessimistic_operation
    
  -- Cleanup regardless of which path completes
  finally do
    perform cleanup
```

### Temporal Circuit Breaker

```tel
-- Define a circuit breaker with temporal characteristics
circuit_breaker
  -- Normal operation mode
  closed do
    result <- perform protected_operation
    return result
    
  -- Failure threshold that opens the circuit
  threshold
    failures: 5
    within: 1m
    
  -- Recovery policy
  recovery
    attempt_after: 30s
    reset_after: 5m
```

### Time-Priority Scheduling

```tel
-- Schedule operations with different time priorities
time_priority do
  -- High priority operations go first
  high do
    perform critical_operation
    
  -- Medium priority after high completes or times out
  medium timeout 5s do
    perform important_operation
    
  -- Low priority when resources are available
  low do
    perform background_operation
```

### Temporal Coordination Patterns

```tel
-- Fan-out with temporal constraints
fan_out
  -- Start multiple operations concurrently
  operations:
    perform operation_a
    perform operation_b
    perform operation_c
    
  -- Temporal collect policy
  collect:
    all_within 10s or  -- Collect all results if all complete within 10s
    any_after 5s or    -- Collect any results after at least 5s
    timeout 30s        -- Give up after 30s total
    
-- Fan-in with temporal constraints
fan_in
  -- Listen for multiple event sources 
  sources:
    event_stream_a
    event_stream_b
    
  -- Processing policy
  process:
    when_all within 2s then
      perform process_batch
    or
    every 5s then
      perform process_available
```

### Temporal Quarantine

```tel
-- Isolate operations in temporal quarantine
quarantine
  -- Operations in quarantine can't affect external state until the time window expires
  operations do
    perform risky_operation_a
    perform risky_operation_b
    
  -- Commit only after validation during quarantine period
  validate after 10m do
    if all_valid then
      commit
    else
      rollback
```

### Temporal Retries

```tel
-- Sophisticated retry pattern with temporal backoff
retry
  -- Operation to retry
  operation do
    perform unreliable_operation
    
  -- Retry policy
  policy
    max_attempts: 5
    backoff: exponential 1s max 30s
    jitter: 0.1
    timeout: 2m
```

### Causality Enforcement

```tel
-- Enforce happens-before relationships
happens_before
  -- Define causal relationships
  operation_a ~> operation_b  -- A must happen before B
  operation_b ~> operation_c  -- B must happen before C
  
  -- Execute with guarantees
  execute
    perform operation_c
    perform operation_a
    perform operation_b
    
  -- System will reorder to ensure causality is preserved
```

### Time-Based Memory Management

```tel
-- Define temporally scoped variables
temporal_scope 5m do
  -- These values only exist within this time scope
  temp_value <- perform expensive_calculation
  use temp_value for calculations
  
  -- Values are automatically garbage collected after the time window
```

## Integration with Effect System

These temporal concurrency primitives integrate seamlessly with TEL's algebraic effect system:

```tel
-- Define a handler for time-based concurrency
handler time_adaptive_handler
  -- Handle temporal backpressure
  effect backpressure operation max_rate ->
    current_rate <- get_current_rate
    if current_rate <= max_rate then
      result <- operation()
      resume result
    else
      delay_time <- calculate_delay current_rate max_rate
      perform delay delay_time
      result <- operation()
      resume result
      
  -- Handle temporal races
  effect race operations timeout ->
    -- Set up the race with temporal constraints
    winner <- perform time_bounded_select operations timeout
    resume winner
```

## Benefits for Transaction Languages

These time-based concurrency primitives provide several key advantages in a transaction language context:

1. **Execution Predictability** - Time-bounded operations ensure transactions complete within expected windows
2. **Resource Efficiency** - Temporal scheduling prevents resource contention and optimizes throughput
3. **Fairness Guarantees** - Time-sliced execution ensures fair access to resources
4. **Safety** - Temporal circuit breakers and quarantine protect against cascading failures
5. **Performance** - Speculative execution and races allow for optimistic paths with guarantees
6. **Determinism** - Causality enforcement ensures reproducible transaction ordering
7. **Resilience** - Sophisticated temporal retry patterns handle transient failures gracefully
8. **Composability** - All primitives can be nested and composed in predictable ways

## Migration from Current TEL

For users familiar with the current TEL syntax, here's a comparison guide:

| Current TEL | New TEL | Notes |
|-------------|---------|-------|
| `transfer(from, to, amount)` | `effect transfer from: from to: to amount: amount` | Effects are explicit with `effect` |
| `if condition { ... }` | `if condition then ...` | No braces, whitespace significant |
| No explicit states | `state initial S1 S2 final S3` | Explicit state machine |
| Implicit control flow | `flow name = do ...` | Named, explicit flows with do notation |
| No concurrency primitives | `concurrent do ...` | Explicit concurrency |
| `let x = y` | `let x = y` for pure values, `x <- effect e` for effects | Different binding for effects vs pure |
| `x.method()` | `x |> method` | Pipeline style for method chaining |
| No effect handlers | `handler name effect op -> ...` | Algebraic effect handlers |
| No effect types | `SomeFunc : A -> <effect1,effect2> B` | Effect typing |
| No time windows | `time window = between t1 and t2` | Explicit time windows |
| No scheduled flows | `flow name at time = do ...` | Time-triggered flows |
| Implicit timeouts | `with_timeout duration do ...` | Explicit timeout handling |

## Comparison with Eff and Koka

TEL's effect system draws inspiration from both Eff and Koka:

### Similarities with Eff

- First-class algebraic effects and handlers
- Explicit resumptions in handlers
- Effect operations as typed interfaces
- Support for effect polymorphism
- Nesting and composition of handlers

### Similarities with Koka

- Row-based effect types
- Effect inference
- Structured concurrency model
- Tracking effects in function types
- Delimiting effect scope
- Predictable resource handling

### TEL-Specific Extensions

- Integration with explicit state machine model
- Domain-specific handler optimizations
- Built-in concurrency with automatic dependency tracking
- Specialized handlers for TEL domains (cryptography, finance, etc.)
- Automatic effect isolation based on capability model
- Explicit modeling of time and temporal effects
- Time windows as first-class values
- Time-triggered flows and handlers
- Built-in tempo-spatial reasoning

## Next Steps for Consideration

1. **Effect Typing** - More granular typing of effects for better error handling
2. **Formal Verification** - Tools to verify correctness of state transitions
3. **Visual Representation** - Generate diagrams from TEL code
4. **IDE Tooling** - Syntax highlighting, code completion, error checking
5. **Test Harness** - Framework for unit testing TEL programs
6. **Effect Analyzer** - Static analysis for effect usage and handlers
7. **Optimized Compilation** - Specialized compilation for effect handlers
8. **Visualizing Effect Flow** - Tools to visualize effect propagation
9. **Temporal Verification** - Formal methods for verifying temporal properties
10. **Real-Time Guarantees** - Analysis for real-time execution bounds
11. **Temporal Debugging** - Tools for debugging temporal issues

## Questions for Workshop Discussion

1. Is the syntax intuitive enough for new developers?
2. Are there edge cases in the concurrency model that need addressing?
3. Should we add more syntactic sugar for common patterns?
4. How should we handle versioning and upgrades of TEL programs?
5. What additional safety guarantees could we provide?
6. Are the algebraic effect handlers expressive enough for our use cases?
7. How can we balance simplicity with the power of algebraic effects?
8. What specialized effects should be built into the language vs. library?
9. How should we handle time zone awareness in temporal effects?
10. What temporal guarantees should the language provide?
11. How can we effectively test temporal properties in TEL programs? 