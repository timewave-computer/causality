# ADR-005: Invocation Model

## Status

**Accepted - Updated 2025-03-07**


## Context

The Causality system requires a **formal invocation model** to describe:

- How users trigger program execution.
- How programs communicate with each other.
- How external facts (like cross-domain deposits) enter the system.
- How responses flow back to users after a program completes an action.

This invocation model must:
- Maintain **strong causal traceability**.
- Be fully **auditable and replayable**.
- Be **domain-agnostic** (work across heterogeneous domains).
- Enforce **separation between actors and programs** — programs should never directly trust or communicate with off-domain actors.
- Integrate with the **account program** model, where each actor owns a **single gateway program** that intermediates all asset and message flows.


## Decision

### Core Principle: Account Programs as Invocation Gateways

- **Users do not directly communicate with programs.**
- Each user owns exactly **one account program**, which:
    - Holds all assets for that user across domains.
    - Mediates all **actor-initiated messages**.
    - Receives all **program responses** intended for the user.
    - Records all inbound and outbound messages in a **per-resource effect log**.
- Programs only trust messages from **other programs** (never directly from actors), and rely on **account programs** for actor-to-program and program-to-actor communication.


## Invocation Flow Overview

### 1. Actor Initiates Action

- User signs a message proposing an action (deposit, withdrawal, invocation, observation).
- The message is submitted to the Causality network.
- The message is applied as an **effect** to the actor's **account program**.
- The account program records the message in its **outbox**, causally linking it to prior effects.


### 2. Account Program Dispatches Message

- The account program evaluates the message type:
    - **Deposit:** Transfers assets to the target program.
    - **Withdrawal:** Pulls assets back from a program.
    - **Invocation:** Sends a cross-program invocation message.
- Each action is recorded in the account program's **per-resource effect log**.


### 3. Target Program Receives Invocation

- The target program receives the invocation as a proposed **effect**.
- The effect includes:
    - The calling account program's ID.
    - The target function (entrypoint).
    - Arguments.
    - Time map snapshot (what facts were known at invocation time).

- The program applies the effect and logs it into its **own effect log**, causally linking it to prior effects.


### 4. Program Generates Optional Response

- If the program produces a result for the User, it sends a **SendCallback** effect to the originating account program.
- This callback is applied to the account program's **inbox**, causally linking it to the prior invocation.


### 5. User Retrieves Response

- Users poll their account program to read their **inbox**, retrieving all received callbacks in causal order.
- This completes the invocation lifecycle.


## Message Types

| Message Type | Origin | Destination | Purpose |
|---|---|---|---|
| Deposit | User | Account Program | Transfer assets into a program |
| Withdraw | User | Account Program | Retrieve assets from a program |
| Invoke | User | Account Program | Call a program function |
| Transfer | Account Program | Program | Transfer assets to a program |
| SendCallback | Program | Account Program | Return results to User |
| ReceiveCallback | Account Program | User (via inbox query) | Retrieve program responses |
| Watch | Account Program | Committee | Observe external deposit or event |


## Account Program State

Each account program tracks:

```rust
struct AccountProgramState {
    balances: HashMap<(domainId, Asset), Amount>,
    inbox: Vec<ReceivedMessage>,
    outbox: Vec<SentMessage>,
    effect_dag: EffectDAG,
}
```

- **Balances:** Current asset holdings across all domains.
- **Inbox:** Messages received from programs (e.g., callbacks).
- **Outbox:** Messages sent to programs.
- **EffectDAG:** Full causal history of all applied effects.


## Security and Provenance Guarantees

This invocation model guarantees:

- Programs only talk to programs — programs never need to trust off-domain Users directly.  
- All actor actions are signed and logged via account program effects.  
- Programs can verify the **full provenance** of any incoming message by querying the sender's account program log.  
- All communication produces permanent, auditable log entries.


## Actor-Program Separation Invariant

| Communication Type | Mediated By |
|---|---|
| Actor to Program | Account Program Outbox |
| Program to Actor | Account Program Inbox |
| Program to Program | Direct (via invocation effects) |


## External Consistency via Time Map Snapshots

- Each cross-program message references a **time map snapshot**, proving which external facts were known at the time the message was generated.
- If external facts change before an effect applies, the preconditions are re-validated before the effect is accepted.


## Examples

### Deposit Flow

**User -> AccountProgram -> TargetProgram**

1. User submits:

```toml
type = "deposit"
resource = "USDC"
amount = 100
destination = "TradeProgram"
```

2. Account program creates a `Transfer` effect, moving USDC to TradeProgram.
3. TradeProgram applies the effect, updating its internal balances.


### Cross-Program Invocation

**User -> AccountProgram -> TargetProgram -> AccountProgram -> User**

1. User submits:

```toml
type = "invoke"
target = "SettlementProgram"
entrypoint = "finalizeTrade"
arguments = ["order123"]
```

2. Account program packages this into:

```rust
Effect::Invoke {
    target_program: String::from("SettlementProgram"),
    entrypoint: String::from("finalizeTrade"),
    arguments: vec![String::from("order123")],
    observed_facts: vec![/* ... */],
}
```

3. SettlementProgram applies the effect.
4. SettlementProgram generates a result:

```toml
type = "callback"
target = "actor123"
payload = { result = "Trade Settled" }
```

5. Account program logs the callback in its inbox.


### User Polling Inbox

User queries:

```http
GET /account/{UserID}/inbox
```

Response:

```json
[
    { "source": "SettlementProgram", "payload": { "result": "Trade Settled" } }
]
```


## Summary Flow Diagram

## Summary Flow Diagram

```
+-----------------+
|    User         |
| (proposes)      |
+-----------------+
        |
        v
+--------------------+
| Account Program    |
| (owned by user)    |
+--------------------+
        |
        v
+------------------+
| Target Program   |
+------------------+
        |
        v
+--------------------+
| Account Program    |
| (for responses)    |
+--------------------+
        |
        v
+-----------------+
|     User        |
| (polls inbox)   |
+-----------------+
```

## Replay and Auditability

- Every message and response is recorded as a **causal effect** in an **append-only log**.
- Each effect has:
    - Full causal ancestry.
    - Cryptographic signature (proving origin).
    - Content hash (proving integrity).
- Replay re-applies effects in order, guaranteeing deterministic reconstruction of program state.


## Relationship to Other Parts of the System

| Component | Role in Invocation Model |
|---|---|
| Account Program | GateUser and message queue for each user |
| Effect Pipeline | Processes all proposed effects |
| Resource Logs | Record every effect per resource |
| Time Map | Provides external consistency for observed facts |
| Fact Logs | Track external events that trigger actor messages (e.g., deposits) |
| Unified Log | Combined view of all applied effects, facts, and events |


## Benefits

- Fully auditable and replayable.  
- No direct actor-program trust.  
- Clear separation of concerns (actors only control account programs).  
- Built-in support for external consistency (time map snapshots).  
- Actor policies (rate limits, multi-sig) enforced at account level.  

