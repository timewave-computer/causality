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

## Integrated Query System

TEL integrates powerful query capabilities directly into the language. This allows developers to naturally express queries over resources, events, facts, and causal execution graphs.

### Query Expression Syntax

Query expressions are first-class citizens in TEL:

```tel
-- Basic query expression 
let active_accounts = query
  from accounts
  where account.balance > 0
  select account.address
  
-- Query with temporal constraints
let recent_transfers = query
  from transfers
  where transfer.timestamp > now() - 24h
  select { from: transfer.from, to: transfer.to, amount: transfer.amount }
  
-- Query with ordering and limits
let top_accounts = query
  from accounts
  order_by account.balance desc
  limit 10
  select account
```

### Querying Different Data Sources

Queries can be performed on different types of data:

```tel
-- Query over resources
let available_tokens = query
  from tokens
  where token.status == Available
  select token
  
-- Query over events
let user_actions = query
  from events
  where event.type == "UserAction" && event.user == current_user
  within last 7d
  select event
  
-- Query over facts (assertions in the system)
let valid_assertions = query
  from facts
  where fact.status == Valid && fact.relates_to == entity_id
  select fact.assertion
```

### Querying Execution History and Causal Graphs

TEL provides unique capabilities to query execution flow and causal relationships:

```tel
-- Query the execution history
let previous_states = query
  from execution_history
  where execution.program == self()
  select execution.states
  
-- Query causal relationships
let dependent_operations = query
  from causal_graph
  where operation.id == target_id
  traverse outgoing edges
  depth 3
  select operation
  
-- Query execution path
let critical_path = query
  from execution_paths
  where path.end == current_state
  order_by path.duration asc
  limit 1
  select path.operations
```

### Temporal and Spatial Query Operators

Query expressions include operators specifically designed for temporal and spatial queries:

```tel
-- Temporal operators
let overlapping_events = query
  from events as e1, events as e2
  where e1.id != e2.id && e1 overlaps e2
  select { event1: e1, event2: e2 }
  
-- Sequential patterns
let suspicious_sequence = query
  from events
  match sequence(
    event(type == "Login", user: user),
    event(type == "PermissionChange", target: "Admin"),
    event(type == "SensitiveAccess")
  ) within 5m
  select { user: user }
```

### Aggregations and Transformations

Queries can include aggregations and transformations:

```tel
-- Aggregation query
let token_stats = query
  from transfers
  where transfer.token == token_id
  group_by transfer.date
  select {
    date: transfer.date,
    volume: sum(transfer.amount),
    count: count(),
    avg_amount: avg(transfer.amount)
  }
  
-- Query with transformations
let normalized_values = query
  from metrics
  select {
    metric: metric.name,
    value: normalize(metric.value, metric.min, metric.max)
  }
```

### Reactive Queries

Queries can be reactive, automatically updating when underlying data changes:

```tel
-- Reactive query that maintains an up-to-date view
let active_users = reactive query
  from sessions
  where session.status == Active
  select session.user
  
-- Query that triggers effects when results change
reactive query
  from balance_changes
  where balance_change.account == watched_account
  on_change do
    effect notify_balance_change
      account: watched_account
      new_balance: balance_change.new_balance
```

### Integration with Effect System

Queries seamlessly integrate with TEL's effect system:

```tel
-- Query that produces effects
let transfer_result = perform query_with_effects
  from transfer_requests
  where request.status == Pending
  select effect process_transfer
    from: request.from
    to: request.to
    amount: request.amount
    
-- Query that is handled by effect handlers
handler query_limiter
  effect expensive_query params ->
    if within_rate_limits then
      result <- execute_query(params)
      resume result
    else
      resume empty_result
```

### Causal Querying and Temporal Logic

TEL's query system includes powerful temporal logic capabilities:

```tel
-- Query using temporal logic operators
let correctness_check = query
  from execution_trace
  where eventually(state == Completed) 
    && always(balance >= 0)
    && never(unauthorized_access)
  select execution_trace.id
  
-- Complex causality query
let root_causes = query
  from causal_graph
  where effect.id == failure_id
  traverse incoming until(operation.type == "UserAction")
  select distinct operation
```

### Graph Traversal Queries

TEL includes specialized syntax for graph traversal:

```tel
-- Resource dependency graph query
let dependencies = query
  from resource_graph
  start_at resource_id
  traverse outgoing
  where edge.type == "Depends"
  collect nodes
  limit depth 5
  select node
  
-- State transition graph analysis
let cycles = query
  from state_graph
  detect cycles
  where cycle.length < 5
  select cycle
```

### Provenance and Data Lineage

Queries can track data provenance and lineage:

```tel
-- Track data lineage
let data_origins = query
  from data_lineage
  where data.id == target_data_id
  traverse incoming
  until node.type == "Source"
  select node
  
-- Provenance query
let transaction_provenance = query
  from provenance_graph
  where entity.id == transaction_id
  select {
    initiator: first(incoming(entity).where(type == "Initiator")),
    approvers: collect(incoming(entity).where(type == "Approver")),
    timestamp: entity.creation_time
  }
```

### Cross-Flow Analysis

Queries can analyze patterns across multiple workflows:

```tel
-- Detect patterns across different flows
let common_patterns = query
  from execution_flows as f1, execution_flows as f2
  where f1.id != f2.id
    && similar_sequence(f1.operations, f2.operations) > 0.8
  select {
    flow1: f1.id,
    flow2: f2.id,
    similarity: similarity_score(f1.operations, f2.operations)
  }
  
-- Find flows affecting the same resources
let resource_conflicts = query
  from flows
  where any(flow.resources).intersects(target_resources)
  select flow
```

### Query Composition

Queries can be composed and reused:

```tel
-- Define a reusable query component
let active_status = query_fragment
  where status == Active
  
-- Reference in another query
let active_items = query
  from items
  include active_status
  select item
  
-- Query composition
let complex_report = query
  with user_summaries as (
    from users
    select { id: user.id, activity: count(user.actions) }
  ),
  active_resources as (
    from resources
    where resource.status == Active
    select resource
  )
  from user_summaries, active_resources
  where user_summaries.id == active_resources.owner
  select { user: user_summaries, resources: collect(active_resources) }
```

## Expanded Query Domains

The integrated query system extends beyond basic resources and events to encompass the full range of system aspects:

### Blockchain and Consensus Data

```tel
-- Query blockchain data
let recent_blocks = query
  from blocks
  where block.height > current_height - 100
  order_by block.height desc
  select { 
    height: block.height,
    hash: block.hash,
    timestamp: block.timestamp,
    tx_count: count(block.transactions)
  }
  
-- Query transaction inclusion
let tx_status = query
  from blocks
  where contains(block.transactions, tx_hash)
  select {
    height: block.height,
    confirmations: current_height - block.height,
    block_time: block.timestamp
  }

-- Consensus state queries
let validator_set = query
  from validators
  where validator.active == true
  order_by validator.voting_power desc
  select validator
```

### System Metadata and Registry

```tel
-- Query available contract interfaces
let swap_interfaces = query
  from contract_registry
  where implements(contract, "SwapInterface")
  select { 
    address: contract.address, 
    version: contract.version,
    methods: contract.methods.where(visibility == Public)
  }
  
-- Query system component versions
let component_versions = query
  from system_registry
  select {
    component: component.name,
    version: component.version,
    status: component.status
  }
  
-- Query capability permissions
let user_capabilities = query
  from capabilities
  where capability.owner == user_id
  select capability
```

### Cryptographic Proofs and Verification

```tel
-- Query and verify zero-knowledge proofs
let valid_proofs = query
  from proofs
  where proof.status == Unverified && verify_proof(proof)
  select proof
  
-- Query merkle inclusion proofs
let inclusion_proof = query
  from merkle_tree
  where merkle_tree.root == root_hash
  select generate_proof(merkle_tree, leaf_data)
  
-- Query signature validations
let valid_signatures = query
  from signatures
  where verify_signature(signature, message, public_key)
  select {
    signer: public_key_to_address(public_key),
    timestamp: signature.timestamp
  }
```

### Network Topology and Routing

```tel
-- Query network structure
let network_paths = query
  from network_graph
  where shortest_path(source_node, target_node)
  select {
    path: path_nodes,
    hops: count(path_nodes) - 1,
    estimated_latency: sum(path_edges.latency)
  }
  
-- Query node connectivity
let peer_connections = query
  from network_nodes
  where node.status == Online
  traverse outgoing edges
  where edge.type == "Peer"
  select {
    node: node.id,
    peers: count(edges),
    regions: distinct(connected_nodes.region)
  }
```

### Resource Pricing and Cost Estimation

```tel
-- Query resource pricing
let operation_costs = query
  from cost_model
  where operation in ["Transfer", "Swap", "Mint"]
  select {
    operation: operation,
    base_cost: cost_model.base_fee,
    unit_cost: cost_model.unit_fee,
    estimated_total: estimate_cost(operation, params)
  }
  
-- Query historical price data
let token_price_history = query
  from price_feed
  where price_feed.token == token_id
  during last 7d
  group_by day(price_feed.timestamp)
  select {
    date: day(price_feed.timestamp),
    open: first(price_feed.price),
    close: last(price_feed.price),
    high: max(price_feed.price),
    low: min(price_feed.price),
    volume: sum(price_feed.volume)
  }
```

### Governance and Protocol Parameters

```tel
-- Query governance proposals
let active_proposals = query
  from proposals
  where proposal.status == Active
  select {
    id: proposal.id,
    title: proposal.title,
    proposer: proposal.creator,
    votes_yes: sum(proposal.votes.where(type == Yes).power),
    votes_no: sum(proposal.votes.where(type == No).power),
    ends_at: proposal.voting_ends_at
  }
  
-- Query protocol parameters
let current_params = query
  from protocol_parameters
  where parameter.active == true
  select {
    name: parameter.name,
    value: parameter.value,
    last_updated: parameter.update_time,
    min_value: parameter.constraints.min,
    max_value: parameter.constraints.max
  }
```

### Cross-Chain and Bridge Data

```tel
-- Query cross-chain transfers
let pending_transfers = query
  from cross_chain_transfers
  where transfer.status == Pending && transfer.destination_chain == "Ethereum"
  select transfer
  
-- Query token mappings across chains
let token_mappings = query
  from token_registry
  join bridge_mappings on token_registry.id == bridge_mappings.native_token
  select {
    native_token: token_registry.symbol,
    external_chain: bridge_mappings.chain,
    external_address: bridge_mappings.address,
    mapping_type: bridge_mappings.type
  }
```

### Simulation and Hypothetical Execution

```tel
-- Query simulation results
let simulated_swaps = query
  from simulation_results
  where simulation.operation == "Swap" && 
        simulation.params.token_in == token_a &&
        simulation.params.token_out == token_b
  order_by simulation.params.amount_in
  select {
    amount_in: simulation.params.amount_in,
    amount_out: simulation.result.amount_out,
    price_impact: simulation.result.price_impact,
    route: simulation.result.route
  }
  
-- Query state transition simulations
let possible_outcomes = query
  from state_simulations
  where simulation.start_state == current_state
  traverse outgoing transitions
  depth 3
  select {
    path: path_transitions,
    end_state: path.end,
    probability: calculate_probability(path),
    value: calculate_value(path.end)
  }
```

### Temporal and Spatial Indexing

```tel
-- Geospatial queries
let nearby_validators = query
  from validator_nodes
  where geo_distance(validator_nodes.location, user_location) < 1000km
  order_by geo_distance(validator_nodes.location, user_location)
  select validator_nodes
  
-- Combined temporal and spatial queries
let regional_activity = query
  from transactions
  where transaction.timestamp within last 24h
  group_by transaction.region, hour(transaction.timestamp)
  select {
    region: transaction.region,
    hour: hour(transaction.timestamp),
    tx_count: count(),
    volume: sum(transaction.amount)
  }
```

### Storage and State Queries

```tel
-- Query state history
let account_history = query
  from state_history
  where state_history.key == account_key
  order_by state_history.block_height desc
  limit 10
  select {
    height: state_history.block_height,
    value: state_history.value,
    changed_by: state_history.transaction_hash
  }
  
-- Query storage tiers
let tiered_storage = query
  from storage_allocation
  group_by storage_allocation.tier
  select {
    tier: storage_allocation.tier,
    used_space: sum(storage_allocation.size),
    item_count: count(),
    cost_per_byte: storage_allocation.tier_cost
  }
```

### Machine Learning and Analytics

```tel
-- Query prediction models
let price_prediction = query
  from ml_models
  where ml_models.type == "PricePrediction" && ml_models.token == token_id
  select predict(ml_models, {
    window: 7d,
    features: ["volume", "volatility", "market_trend"]
  })
  
-- Query anomaly detection
let anomalies = query
  from transaction_patterns
  where detect_anomaly(transaction_patterns, 
    { sensitivity: 0.8, baseline_period: 30d })
  select {
    pattern: transaction_patterns,
    anomaly_score: anomaly_score(transaction_patterns),
    similar_patterns: find_similar(transaction_patterns, 3)
  }
```

### Integration with External Data Sources

```tel
-- Query oracle data
let asset_prices = query
  from oracles
  where oracle.feed_type == "Price" && oracle.assets.contains(requested_assets)
  select {
    asset: oracle.asset,
    price: oracle.latest_value,
    timestamp: oracle.latest_update,
    confidence: oracle.confidence_score
  }
  
-- Query external API data
let weather_data = query
  from external_apis
  where external_apis.provider == "WeatherService" && 
        external_apis.location == user_location
  select external_apis.latest_data
```

## Unified Query Composition

These specialized domain queries can be combined and composed with the core query capabilities:

```tel
-- Complex cross-domain query
let liquid_staking_analysis = query
  -- Start with governance parameters
  with staking_params as (
    from protocol_parameters
    where parameter.category == "Staking"
    select parameter
  ),
  -- Include validator data
  active_validators as (
    from validators
    where validator.active == true
    select validator
  ),
  -- Include price data
  token_price as (
    from price_feed
    where price_feed.token == staking_token_id
    order_by price_feed.timestamp desc
    limit 1
    select price_feed
  ),
  -- Include user positions
  user_positions as (
    from staking_positions
    where position.owner == user_id
    select position
  )
  -- Join and analyze
  from staking_params, active_validators, token_price, user_positions
  select {
    apy: calculate_apy(staking_params, active_validators),
    total_staked: sum(active_validators.staked_amount),
    user_stake: sum(user_positions.amount),
    value_usd: sum(user_positions.amount) * token_price.price,
    reward_frequency: staking_params.where(name == "reward_frequency").value
  }
```

The query capabilities shown across these domains demonstrate how TEL can provide a unified query interface across the entire system, eliminating the need for separate query languages while maintaining type safety, effect tracking, and integration with the core language features.

## Integration with Time-Based Features

TEL's query system integrates seamlessly with its temporal features:

```tel
-- Query events within a time window
time_window trading_hours = between 9:00 and 17:00

let trading_activity = query
  from transactions
  during trading_hours
  select transaction
  
-- Query that respects temporal barriers
let validated_operations = query
  from operations
  where operation.status == Completed
    && operation.completion_time before time_barrier
  select operation
  
-- Temporal slicing of data
let hourly_stats = query
  from event_stream
  slice by hours
  select {
    hour: slice.start,
    count: count(events),
    volume: sum(event.amount)
  }
```

### Benefits of Integrated Query Capabilities

1. **Unified Programming Model** - No context switching between query and execution languages
2. **Type Safety** - Queries are statically typed and checked like any other TEL code
3. **Effect Integration** - Queries can produce and be controlled by effects
4. **Temporal Awareness** - Native understanding of time in queries
5. **Causal Reasoning** - Direct access to execution causality and dependencies
6. **Performance** - Query optimization built into the language runtime
7. **Composability** - Queries compose with other language features
8. **Safety** - Controlled access to resources through effect system

## Read-Write Query Operations

The TEL query system supports not only reading data but also modifying it through a unified syntax, merging query and data manipulation capabilities into a single language construct:

### Data Modification Operations

```tel
-- Insert new data
effect insert into accounts
  values: {
    id: generate_id(),
    owner: user_id,
    balance: initial_amount,
    created_at: now()
  }
  returning account_id

-- Update existing data
effect update accounts
  where account.owner == user_id
  set {
    balance: account.balance + amount,
    last_modified: now()
  }
  returning account.balance

-- Delete data
effect delete from inactive_sessions
  where session.last_active < now() - 7d
  returning count()
```

### Transactional Query Operations

Queries can be executed within transactions, maintaining ACID properties:

```tel
-- Transactional query operations
effect transaction do
  -- Read current balance
  source_balance <- query
    from accounts
    where account.id == source_id
    select account.balance
    
  -- Ensure sufficient balance
  if source_balance < amount then
    abort "Insufficient balance"
    
  -- Update source account
  effect update accounts
    where account.id == source_id
    set balance: account.balance - amount
    
  -- Update destination account  
  effect update accounts
    where account.id == destination_id
    set balance: account.balance + amount
```

### Conditional Data Modifications

```tel
-- Conditional update with complex logic
effect update orders
  where order.id == order_id
  when order.status == Pending
  set {
    status: Confirmed,
    confirmation_time: now()
  }
  else when order.status == Processing
  set {
    status: Completed,
    completion_time: now()
  }
  else
  fail "Invalid state transition"
```

### Bulk Operations

```tel
-- Batch update with aggregation
effect update user_statistics
  join (
    from transactions
    where transaction.user == user_id
    group_by transaction.type
    select {
      type: transaction.type, 
      count: count(),
      volume: sum(transaction.amount)
    }
  ) as stats on user_statistics.type == stats.type
  set {
    transaction_count: user_statistics.transaction_count + stats.count,
    transaction_volume: user_statistics.transaction_volume + stats.volume,
    last_updated: now()
  }
```

### Effect Integration for Writes

Write operations are treated as effects, providing the same safety guarantees and tracking as other effects in TEL:

```tel
-- Explicit effect handling for data modification
handler storage_handler
  effect insert into collection values ->
    validate_schema(collection, values)
    if has_permission(current_user, collection, "write") then
      result <- perform_insert(collection, values)
      resume result
    else
      perform permission_denied
      resume failure "Permission denied"
      
  effect update collection where predicate set values ->
    if has_permission(current_user, collection, "write") then
      result <- perform_update(collection, predicate, values)
      resume result
    else
      perform permission_denied
      resume failure "Permission denied"
```

### Temporal Data Management

Write operations can be temporally aware:

```tel
-- Temporal data versioning
effect insert into accounts
  values: new_account
  valid_from: now()
  valid_until: indefinite
  
-- Time travel updates
effect update historical_prices
  where price.asset == asset_id && price.timestamp == timestamp
  set value: corrected_value
  record_correction: true
  
-- Scheduled modifications
effect update parameters
  set interest_rate: new_rate
  effective_from: tomorrow_at_midnight
```

### Query-Driven State Transitions

State transitions can be triggered by query results:

```tel
-- Query-based state transition
effect query_transition
  from accounts
  where account.balance < min_balance && account.status == Active
  select account
  transition account.status to Frozen
  notify account.owner
  
-- Complex state update based on aggregation
effect aggregate_transition
  from transactions
  group_by transactions.account
  having sum(transaction.amount) > suspicious_threshold
  select account_id
  transition account_status to UnderReview
```

### Write Operations with Temporal Causality

```tel
-- Causal writes with dependency tracking
effect causal_update accounts
  where account.id == target_id
  depends_on [tx1, tx2, tx3]
  set balance: calculate_balance(account.balance, [tx1, tx2, tx3])
  
-- Concurrent conflict resolution
effect concurrent_update resource
  where resource.id == resource_id
  set value: new_value
  on_conflict (current_value, attempting_value) ->
    resolve_with: merge_strategy(current_value, attempting_value)
```

### Multi-System Atomic Writes

```tel
-- Cross-system atomic operations
effect atomic_cross_system
  -- Update local database
  update local_accounts
    where account.id == account_id
    set balance: account.balance - amount
    
  -- Update external system via API
  external update_ledger
    account: account_id
    amount: -amount
    
  -- Ensure atomicity across both systems
  atomicity: all_or_nothing
  compensation: revert_all
```

### Write Queries with Access Control

```tel
-- Write operations with capability-based access control
with capability <- acquire_capability "write:accounts" do
  effect update accounts
    where account.id == account_id
    set balance: account.balance + amount
    
-- Role-based write protection
effect update sensitive_data
  where data.id == data_id
  set content: new_content
  requires role: "admin"
```

### Integration with Other TEL Features

Write queries seamlessly integrate with other language features:

```tel
-- Algebraic effects for data modification
handler transactional_handler
  effect begin_transaction ->
    tx_id <- create_transaction()
    resume tx_id
    
  effect commit_transaction tx_id ->
    success <- commit(tx_id)
    resume success
    
  effect rollback_transaction tx_id ->
    perform_rollback(tx_id)
    resume unit
    
-- Combining with temporal effects
time_window maintenance_window = between 01:00 and 03:00
  
flow perform_maintenance = do
  during maintenance_window do
    effect update system_state
      set status: Maintenance
      
    effect delete from temp_data
      where data.created_at < now() - 30d
      
    effect update system_state
      set status: Active
```

## Benefits of Integrated Read-Write Queries

1. **Unified Data Interface** - Single language construct for both reading and modifying data
2. **Type Safety** - Strong typing for both queries and modifications
3. **Transactional Safety** - Built-in transaction support with rollback capabilities
4. **Effect Tracking** - Write operations are tracked like any other effect
5. **Temporal Awareness** - Time-based modifications and scheduling
6. **Causality Tracking** - Dependencies and ordering of modifications
7. **Composition** - Write operations compose with other language features
8. **Access Control** - Fine-grained permission management for modifications

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