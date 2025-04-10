# Temporal Effect Language (TEL) Specification

## 1. Introduction

The Temporal Effect Language (TEL) is a pure, functional, combinator-based language designed for expressing temporal effects in a mathematically rigorous manner. The language enables reasoning about the sequencing and coordination of effects while maintaining deterministic execution and content addressability.

### 1.1 Design Goals

1. **Enhanced Composability**: Support for natural function composition and effect chaining
2. **Referential Transparency**: Pure expressions with explicit effect handling
3. **Content Addressability**: All expressions and data have a unique content-based identity
4. **Deterministic Execution**: Evaluation produces the same results given the same inputs
5. **Temporal Reasoning**: Clear semantics for reasoning about the temporal ordering of effects
6. **Resource Linearity**: Prevention of resource duplication or loss through linear types
7. **Domain-Aware Causality**: Validation of causal relationships across domain boundaries
8. **Effect Isolation**: Clear boundaries between pure computations and effects
9. **Merkle Tree Structure**: AST represented as a Merkle tree for provable execution paths
10. **Unified Effect Model**: All side effects, including state transitions, expressed through a single effect system
11. **Row Polymorphism**: Flexible record types with row type support for extensibility
12. **Integrated Query Language**: First-class support for queries and state transformations

### 1.2 Core Design Principles

1. **Purely Functional**: Expression-based with no mutable state
2. **Content Addressed**: All expressions and data have a unique content ID
3. **Effect-Based**: All side effects are modeled as explicit effect combinators
4. **Temporal Semantics**: Clear ordering of effect evaluation
5. **Whitespace Significance**: Indentation-based block structure with minimal syntax noise
6. **Pipeline Style**: Natural left-to-right flow using pipe operators
7. **Declarative Semantics**: Describing "what" should happen, not "how"
8. **Merkle Path Verification**: Execution paths are verifiable through Merkle proofs
9. **Effects as State Transitions**: All state changes occur through the unified effect system
10. **Row Polymorphism**: Extensible records through row types
11. **Query Integration**: Seamless integration of query operations with the effect system

## 2. Core Combinators

TEL is built on a foundation of five core combinators from SKI calculus and related systems:

| Combinator | Name | Lambda Expression | Description |
|------------|------|-------------------|-------------|
| I | Identity | λx.x | Returns its argument unchanged |
| K | Constant | λx.λy.x | Creates a constant function that always returns its first argument |
| S | Substitution | λx.λy.λz.xz(yz) | Applies a function to an argument after substitution |
| B | Composition | λx.λy.λz.x(yz) | Composes two functions (B f g x = f (g x)) |
| C | Flip | λx.λy.λz.xzy | Flips the order of the second and third arguments |

These five combinators are universal and can express any computable function.

## 3. TEL-Specific Combinators

In addition to the core combinators, TEL defines domain-specific combinators for effects, state management, and content addressing:

### 3.1 Effect Combinators

| Combinator | Description | Semantics |
|------------|-------------|-----------|
| Effect(name, args) | Performs an effect | Invokes a handler for the named effect with the given arguments |
| Handler(effect, body) | Defines a handler | Creates a handler for the specified effect |
| Resume(value) | Resumes a suspended effect | Returns a value to the suspended computation |
| Do(effects) | Sequences effects | Performs effects in order, binding results to names |
| Spawn(effect) | Concurrent effect | Initiates effect execution without waiting |
| Race(effects) | First-to-complete | Performs multiple effects concurrently, returns first result |
| Within(time, effect) | Time-bounded effect | Performs effect with deadline, fails if not completed in time |
| Transition(state, args) | State transition | Changes system state, represented as a special effect |

### 3.2 Resource Combinators

| Combinator | Description | Semantics |
|------------|-------------|-----------|
| Resource(definition) | Defines resource | Creates a resource with specified properties |
| Transfer(resource, from, to) | Transfers resource | Transfers ownership of a resource |
| Balance(resource, owner) | Checks balance | Returns the balance of a resource for an owner |
| MintResource(resource, amount) | Creates resource | Creates new instances of a resource |
| BurnResource(resource, amount) | Destroys resource | Destroys instances of a resource |
| ComputeDelta(resources) | Calculates change | Computes the net change across resource operations |

### 3.3 Content Addressing Combinators

| Combinator | Description | Semantics |
|------------|-------------|-----------|
| ContentId(data) | Generates content ID | Creates a content ID for the data |
| Store(data) | Stores content | Stores data and returns its content ID |
| Load(id) | Loads content | Retrieves data by its content ID |
| Link(from, to, relation) | Creates relationship | Establishes causal relationship between content IDs |
| Verify(id, data) | Verifies content | Checks if data matches the claimed content ID |
| Proof(id) | Generates proof | Creates a verification proof for content-addressed data |

## 4. Syntax and Grammar

TEL combinators can be expressed in multiple syntactic forms. The primary syntax is inspired by PureScript, with significant whitespace, row types, and functional notation:

### 4.1 Point-Free Notation

```
expr ::= combinator | application | literal | reference
combinator ::= "I" | "K" | "S" | "B" | "C"
application ::= expr expr
literal ::= <int> | <float> | <string> | <bool> | "Nothing"
reference ::= <identifier>
```

### 4.2 PureScript-Inspired Syntax

```purescript
-- Module definition
module Program.OrderProcessor where

-- Import statements
import Effect.State (transition)
import Effect.Content (contentId, store, load)
import Effect.Query (query, select, where_, from)
import Data.ContentId (ContentId)

-- Comments in TEL use -- for line comments, similar to PureScript and Haskell
-- Block comments are supported with /* and */ delimiters

-- Type definition with row polymorphism
type Account r = 
  { id :: ContentId
  , balance :: Int
  | r
  }

-- Concrete type using row type
type UserAccount = Account (name :: String, email :: String)

-- Sum type definition for state representation
data State
  = Pending
  | Processing 
  | Completed { orderId :: ContentId, timestamp :: Int }
  | Failed { reason :: String }

-- Type class definition
class Storable a where
  toJSON :: a -> String
  fromJSON :: String -> a

-- Type class instance
instance storableState :: Storable State where
  toJSON state = -- implementation
  fromJSON json = -- implementation

-- Higher-order function with row polymorphism
mapRecord :: forall r a b. (a -> b) -> { values :: Array a | r } -> { values :: Array b | r }
mapRecord f record = record { values = map f record.values }

-- Effect definition
effect :: String -> Type -> Effect
effect name paramType = 
  { name = name
  , params = paramType
  , result = resultType
  }

-- Effect with constraints and row types
transfer :: forall r s. 
  Storable r => 
  { from :: ContentId
  , to :: ContentId
  , amount :: Int
  | s
  } -> 
  Effect { transfer :: Unit, logging :: Unit } (ContentId r)

-- Effect handler definition with where clause
handler :: String -> State -> Array EffectHandler -> Handler
handler name initialState handlers =
  { name = name
  , state = initialState
  , handlers = handlers
  }
  where
    handleErrorCase e = -- implementation

-- Effect handler clause
handleEffect :: Effect -> (Params -> Suspended a -> Result)
handleEffect effect params resume = do
  -- Handler implementation
  result <- performOperation params
  resume result

-- Flow definition
flow :: String -> Array Input -> Array Statement -> Flow
flow name inputs statements =
  { name = name
  , inputs = inputs
  , body = statements
  }

-- Statement types
let_ :: String -> Expr -> Statement
let_ name expr = LetStatement name expr

if_ :: Expr -> Array Statement -> Array Statement -> Statement
if_ condition thenBranch elseBranch = 
  IfStatement condition thenBranch elseBranch

case_ :: Expr -> Array (Tuple Pattern Statement) -> Statement
case_ expr patterns = CaseStatement expr patterns

perform :: String -> Params -> Statement
perform effectName params = PerformStatement effectName params

-- Function application is done with spaces
apply :: forall a b. (a -> b) -> a -> b
apply f x = f x

-- Pipeline operator
pipeline :: forall a b. a -> (a -> b) -> b
pipeline x f = f x

-- Do notation
doBind :: forall a b m. Monad m => m a -> (a -> m b) -> m b
doBind = bind
```

### 4.3 Pipeline Syntax

```purescript
-- Pipeline operator (left-to-right composition)
x |> f = f x

-- Examples
getValue
  |> transform
  |> format
  |> display
```

### 4.4 Effect Syntax

```purescript
-- Effect operation
perform :: forall a. String -> a -> Effect a
perform name params = Effect name params

-- Examples
perform "transfer" 
  { from: sender
  , to: receiver
  , amount: 100
  }

perform "store" document

-- State transition as an effect
perform "transition" (Completed 
  { orderId
  , timestamp: now
  })
```

### 4.5 Query Language Syntax

```purescript
-- Query type definition
data Query a = Query QuerySpec

-- Query builder functions
from :: String -> Query Unit
from collectionName = Query { collection: collectionName }

where_ :: forall a. Query a -> Condition -> Query a
where_ (Query spec) condition = Query (spec { filter = condition })

select :: forall a b. Query a -> (a -> b) -> Query b
select (Query spec) projection = Query (spec { select = projection })

orderBy :: forall a. Query a -> String -> SortDirection -> Query a
orderBy (Query spec) field direction = Query (spec { orderBy = { field, direction } })

limit :: forall a. Query a -> Int -> Query a
limit (Query spec) n = Query (spec { limit = n })

-- Query execution (as an effect)
runQuery :: forall a. Query a -> Effect (Array a)
runQuery query = perform "query" query

-- Query usage example
results <- runQuery $ from "orders"
  |> where_ (\order -> order.status == "pending" && order.amount > 100)
  |> select (\order -> { id: order.id, amount: order.amount })
  |> orderBy "createdAt" Desc
  |> limit 10

-- Query with join
results <- runQuery $ from "orders"
  |> join "users" (\order user -> order.userId == user.id)
  |> select (\{order, user} -> {orderId: order.id, userName: user.name})

-- Update query (produces a state change effect)
updateResult <- runQuery $ from "orders"
  |> where_ (\order -> order.id == targetId)
  |> update (\order -> order { status = "processing" })

-- Transaction (multiple queries as a single effect)
transaction do
  fromAccount <- runQuery $ from "accounts" 
    |> where_ (\acc -> acc.id == sourceId)
    |> single

  toAccount <- runQuery $ from "accounts"
    |> where_ (\acc -> acc.id == targetId)
    |> single

  -- Update sender account
  runQuery $ from "accounts"
    |> where_ (\acc -> acc.id == sourceId)
    |> update (\acc -> acc { balance = acc.balance - amount })

  -- Update receiver account
  runQuery $ from "accounts"
    |> where_ (\acc -> acc.id == targetId)
    |> update (\acc -> acc { balance = acc.balance + amount })
```

## 5. Semantics and Evaluation

TEL combinators are evaluated through beta-reduction according to the following rules:

### 5.1 Core Reduction Rules

- I x → x
- K x y → x
- S f g x → (f x) (g x)
- B f g x → f (g x)
- C f g x → f x g

### 5.2 Combinator Expression Evaluation

Combinator expressions are evaluated through the following process:

1. **Beta-reduction**: Apply core combinator rules until no further reductions are possible
2. **Effect resolution**: Resolve effect combinators through their handlers
3. **Content ID computation**: Calculate content IDs for all expressions
4. **Resource tracking**: Validate resource linearity constraints
5. **Temporal ordering**: Enforce temporal constraints on effect execution
6. **Domain validation**: Validate cross-domain operations

### 5.3 Effect Handling

1. When an Effect combinator is evaluated, the runtime:
   - Looks up the appropriate handler
   - Suspends the current computation
   - Evaluates the handler with the effect arguments
   - Resumes the suspended computation with the handler's result

2. Effect handlers are scoped lexically, with a stack of handler frames maintained during evaluation.

3. Effect combinators compose through:
   - Sequential composition (effect1 |> effect2)
   - Parallel composition (Race(effect1, effect2))
   - Resource-parameterized composition (WithResource(resource, effect))
   - Time-bounded composition (Within(duration, effect))

### 5.4 Resource Linearity

Resource operations adhere to linear type constraints:

1. Resources cannot be duplicated or lost
2. Each transfer operation preserves the resource quantity
3. Resource delta calculations must sum to zero
4. Resource operations in different domains require formal endorsements

### 5.5 State as Effect History

In TEL, system state is derived from the history of applied effects:

1. **No Separate State Storage**: There is no independent concept of mutable state
2. **Content-Addressed Effect Chain**: The current state is a position in the content-addressed chain of effects
3. **Derived State Properties**: Any state property is computed by applying the effects in order
4. **State Transitions as Effects**: State changes are represented by the `transition` effect type
5. **History Preservation**: The complete history of state transitions is preserved through content addressing

State transition effects have special semantics:

1. They create a new state value with the specified properties
2. They compute a content ID for this state
3. They establish a causal link between the previous state and the new state
4. They return the content ID of the new state

This approach unifies state management with the effect system, providing a consistent model for all side effects.

### 5.6 Content Addressing

All combinator expressions are content-addressed by:
1. Serializing the expression to a canonical form
2. Computing a cryptographic hash of the serialized data
3. Using the hash as a globally unique identifier

Content addressing provides:
1. Deduplication of identical expressions
2. Verifiability of results
3. Causal tracking between expressions
4. Transparent data dependencies
5. Deterministic references to all program elements

### 5.7 Row Types and Polymorphism

Row types in TEL enable polymorphic operations over records with different shapes:

1. **Extensible Records**: Records can be extended with additional fields without changing the type
2. **Row Variables**: Type variables can represent sets of record fields
3. **Row Constraints**: Constraints can require or forbid certain fields
4. **Structural Subtyping**: A function requiring certain fields can accept any record with those fields, plus optional additional fields

Row type evaluation follows these rules:

1. **Field Access**: Accessing a field requires the field to be present in the row type
2. **Record Extension**: Adding a field to a record produces a new record type with the field
3. **Record Restriction**: Removing a field from a record produces a new record type without the field
4. **Record Combination**: Merging records with disjoint fields produces a record with all fields
5. **Record Update**: Updating a field preserves the record's type

### 5.8 Query Evaluation

Queries in TEL are evaluated as special effects that:

1. **Query Planning**: Convert the query structure to an execution plan
2. **Content Resolution**: Resolve content IDs in the query
3. **Execution**: Run the query against the content-addressed store
4. **Result Construction**: Build the result set based on the query projection
5. **Content Addressing**: Content-address the query results

Query evaluation follows these principles:

1. **Declarative Semantics**: Queries specify what data to retrieve, not how to retrieve it
2. **Composability**: Queries can be composed and transformed
3. **Purity**: Queries without update operations are pure and have no side effects
4. **Effect Integration**: Queries that modify state are treated as effects in the system
5. **Content Addressing**: Query results are content-addressed like all other data

## 6. Type System

TEL combinators use a structural type system with the following core types:

### 6.1 Base Types

- `Unit`: The unit type (representing null/void)
- `Bool`: Boolean values
- `Int`: 64-bit integers
- `Float`: 64-bit floating point numbers
- `String`: UTF-8 string
- `ContentId<T>`: Content identifier for a value of type T

### 6.2 Composite Types

- `List<T>`: Homogeneous list of values of type T
- `Map<K, V>`: Key-value mapping from K to V
- `State<S>`: A state with fields defined by record type S
- `Effect<E, R>`: An effect of type E that returns a value of type R
- `Resource<R>`: A linear resource of type R
- `Domain<D>`: A domain of type D
- `Handler<E, R>`: A handler for effect E producing result R
- `Controller<C>`: A controller for resource operations

### 6.3 Effect Types

Effect types capture the allowed effects in an expression:

```
effect_type ::= <pure> | <effect1, effect2, ...>
```

Effect types propagate through combinators according to these rules:
1. Pure expressions have no effects: <pure>
2. Effect combinators introduce their specified effects
3. Composition combines effect sets: <e1, e2, ...> ∪ <e3, e4, ...>
4. Handlers remove handled effects: <e1, e2, e3> \ <e2> = <e1, e3>

### 6.4 Resource Types

Resource types enforce linearity constraints:

```
resource_type ::= Resource<kind, quantity, domain>
```

Resource operations follow these typing rules:
1. Resource creation produces a new Resource<K, Q, D>
2. Resource transfer preserves the total quantity
3. Resource operations within a domain maintain quantity
4. Cross-domain operations require formal endorsements

### 6.5 State Effect Types

State transition effects have the following type signature:

```
transition :: State -> Effect { state, logging } (ContentId State)
```

Where:
- The input is the new state value
- The effects include state manipulation and logging
- The return value is the content ID of the new state

This type ensures that state transitions are tracked in the effect system and properly content-addressed.

### 6.6 Type Rules for Combinators

- I : a → a
- K : a → b → a
- S : (a → b → c) → (a → b) → a → c
- B : (b → c) → (a → b) → a → c
- C : (a → b → c) → b → a → c

## 7. AST as a Merkle Tree

The TEL Abstract Syntax Tree (AST) is structured as a Merkle tree, enabling efficient content addressing and verifiable execution paths.

### 7.1 Merkle Tree Structure

Each node in the TEL AST is represented as a Merkle node with the following properties:

```
MerkleNode {
  hash: ContentId,
  node_type: NodeType,
  children: List<MerkleNode>,
  metadata: Map<String, Value>
}
```

The hash of each node is computed recursively:
1. Leaf nodes are hashed based on their values
2. Non-leaf nodes hash the concatenation of their children's hashes along with node-specific metadata

This structure provides:
- Unique content addressing for any expression or sub-expression
- Efficient verification of program structure
- Tamper-evident execution history

### 7.2 Merkle Paths for Execution

Program execution is represented as a path through the Merkle tree:

```elm
-- Definition of a Merkle path
type MerklePath = List 
  { hash :: ContentId
  , direction :: Direction
  , siblingHashes :: List ContentId
  }

-- Creating a path proof
createMerkleProof :: ContentId -> ContentId -> Effect MerklePath
createMerkleProof rootHash targetHash = 
  perform "create_merkle_proof" { root = rootHash, target = targetHash }

-- Get content ID for a specific AST node
getNodeContentId :: AstNode -> Effect ContentId
getNodeContentId node =
  perform "get_node_content_id" node

-- Execute and generate a Merkle path
executeWithProof :: Expr -> Effect { result :: a, path :: MerklePath }
executeWithProof expr =
  perform "execute_with_proof" expr

-- Verify an execution path
verifyMerklePath :: { root :: ContentId, path :: MerklePath, target :: ContentId } -> Effect Bool
verifyMerklePath params =
  perform "verify_merkle_path" params
```

Merkle paths enable:
1. **Verifiable Execution**: Proof that a specific execution branch was taken
2. **Minimal Disclosure**: Ability to prove execution properties without revealing the entire program
3. **Deterministic Addressing**: Any execution branch has a unique content ID

### 7.3 Content Addressing via Merkle Paths

Execution steps are content-addressed using their position in the Merkle tree:

```elm
-- Get content ID for a specific AST node
node_id <- perform get_node_content_id ast_node

-- Execute and generate a Merkle path
result_with_path <- perform execute_with_proof expression

-- Verify an execution path
is_valid <- perform verify_merkle_path {
  root: program_hash,
  path: execution_path,
  target: result_hash
}
```

This provides several key advantages:
1. **Deterministic References**: Any execution step can be uniquely referenced
2. **Execution Verification**: Third parties can verify program execution
3. **History Compression**: Store only the path in the Merkle tree instead of full execution history
4. **Secure Composition**: Compose programs with verifiable properties

### 7.4 Merkle-Based Effect Handling

Effect handlers interact with the Merkle tree structure:

```elm
-- Effect handler with Merkle tree integration
merkleAwareHandler :: State -> Handler
merkleAwareHandler initialState =
  { state = perform "hash_state" initialState
  , handlers =
      [ handleEffect "perform_with_proof" \params resume ->
          let
            currentPath = perform "get_merkle_path" {}
            result = perform "handle_effect" params
            resultHash = perform "hash_content" result
            newPath = perform "extend_merkle_path" 
              { path = currentPath
              , node = resultHash
              }
          in
            resume 
              { result = result
              , path = newPath
              }
      ]
  }
```

## 8. Unified Effect System with Content Addressing

TEL's effect system integrates directly with content addressing, with state transitions as first-class effects:

### 8.1 Effect Definitions

```purescript
-- Define a content-addressed effect with row polymorphism
effect :: forall r a e. String -> r -> Effect e a

-- Transfer effect with row polymorphism
transfer :: forall r.
  { from :: ContentId Account
  , to :: ContentId Account
  , amount :: Int
  | r
  } 
  -> Effect { transfer :: Unit, logging :: Unit } (ContentId TransferResult)

-- State transition as effect
transition :: forall r.
  State r
  -> Effect { state :: Unit, logging :: Unit } (ContentId (State r))

-- Query effect
query :: forall a r.
  Query a
  -> Effect { query :: Unit | r } (Array a)

-- Effect handler
transferHandler :: Handler
transferHandler = 
  { handlers: 
      [ handleEffect "transfer" \params resume -> do
          -- Verify accounts exist
          fromAccount <- perform "load" params.from
          toAccount <- perform "load" params.to
          
          -- Create result data
          result <- 
            if fromAccount.balance >= params.amount then do
              -- Update balances
              updatedFrom <- perform "update_account" 
                { id: params.from
                , balance: fromAccount.balance - params.amount
                }
              
              updatedTo <- perform "update_account" 
                { id: params.to
                , balance: toAccount.balance + params.amount
                }
              
              -- Return success
              pure { success: true
                   , from: updatedFrom
                   , to: updatedTo
                   }
            else
              -- Return failure
              pure { success: false
                   , reason: "Insufficient balance"
                   }
          
          -- Create content ID and store result
          resultId <- perform "content_id" result
          perform "store" { id: resultId, data: result }
          
          -- Resume with content ID
          resume resultId
      ]
  }
```

### 8.2 State as Effect Chain

```purescript
-- Define system states with row polymorphism
data State r
  = Pending
  | Processing 
  | Completed { orderId :: ContentId, timestamp :: Int | r }
  | Failed { reason :: String | r }

-- State transition effect handler
transitionHandler :: forall r. State r -> Handler
transitionHandler initialState =
  { state: do
      -- Initialize with content ID of initial state
      stateId <- perform "content_id" initialState
      pure { currentStateId: stateId }
      
  , setup: do 
      perform "store" { id: stateId, data: initialState }
      
  , handlers:
      [ handleEffect "transition" \newState resume -> do
          -- Load current state
          currentState <- perform "load" state.currentStateId
          
          -- Validate transition based on current state and new state
          valid <- validateTransition currentState newState
          
          if valid then do
            -- Create content ID for new state
            newStateId <- perform "content_id" newState
            
            -- Store new state
            perform "store" { id: newStateId, data: newState }
            
            -- Create causal link (this is what forms the state history chain)
            perform "link" 
              { from: state.currentStateId
              , to: newStateId
              , relation: "state_transition"
              }
            
            -- Log the transition
            perform "log" 
              { event: "state_transition"
              , from: state.currentStateId
              , to: newStateId
              , timestamp: perform "now" {}
              }
            
            -- Update current state reference
            state.currentStateId <- newStateId
            
            -- Resume with new state ID
            resume newStateId
          else do
            -- Resume with error
            errorId <- perform "content_id" { error: "Invalid transition" }
            resume errorId
      ]
  }

-- Helper function to validate transitions
validateTransition :: forall r s. State r -> State s -> Boolean
validateTransition from to =
  case from, to of
    Pending, Processing -> true
    Processing, Completed _ -> true
    Pending, Failed _ -> true
    Processing, Failed _ -> true
    _, _ -> false
```

### 8.3 Query Integration

```purescript
-- Query handler
queryHandler :: Handler
queryHandler =
  { state: { collections: {} } 
  , handlers:
      [ handleEffect "query" \querySpec resume -> do
          -- Extract query parameters
          let collection = querySpec.from
          let filter = querySpec.where || (\_ -> true)
          let projection = querySpec.select || identity
          let sorting = querySpec.orderBy
          let limitCount = querySpec.limit
          
          -- Load collection data
          collectionData <- case Map.lookup collection state.collections of
            Just data -> pure data
            Nothing -> do
              -- Load from content store
              loadedData <- perform "load_collection" collection
              -- Cache in handler state
              state.collections <- Map.insert collection loadedData state.collections
              pure loadedData
          
          -- Apply filter
          let filtered = Array.filter filter collectionData
          
          -- Apply sorting if specified
          let sorted = case sorting of
            Just { field, direction } -> 
              let sorter = if direction == Asc 
                           then \a b -> compare (a # field) (b # field)
                           else \a b -> compare (b # field) (a # field)
              in Array.sortBy sorter filtered
            Nothing -> filtered
          
          -- Apply limit if specified
          let limited = case limitCount of
            Just n -> Array.take n sorted
            Nothing -> sorted
          
          -- Apply projection
          let results = Array.map projection limited
          
          -- Content-address the results
          resultsId <- perform "content_id" results
          perform "store" { id: resultsId, data: results }
          
          -- Resume with results
          resume results
      
      , handleEffect "update" \querySpec resume -> do
          -- Similar logic to query, but applies update operation
          let collection = querySpec.from
          let filter = querySpec.where || (\_ -> true)
          let updateFn = querySpec.update
          
          -- Load and filter collection
          collectionData <- perform "load_collection" collection
          let matching = Array.filter filter collectionData
          
          -- Apply updates
          let updates = Array.map updateFn matching
          
          -- Store updated items
          updateResults <- for updates \updatedItem -> do
            itemId <- perform "content_id" updatedItem
            perform "store" { id: itemId, data: updatedItem }
            -- Track the update
            perform "record_update" 
              { collection
              , id: itemId
              , timestamp: perform "now" {}
              }
            pure { id: itemId, item: updatedItem }
          
          -- Create result
          let result = { count: Array.length updates, items: updates }
          resultId <- perform "content_id" result
          perform "store" { id: resultId, data: result }
          
          -- Resume with result
          resume resultId
      ]
  }
```

## 9. Advanced Concepts

### 9.1 Resource Linearity and Conservation

TEL enforces resource conservation through linear types:

```elm
-- Define a resource
effect define_resource : ResourceDefinition -> <resource> ContentId<Resource>

-- Transfer a resource (must preserve quantity)
effect transfer : {resource: ContentId<Resource>, from: ContentId<Account>, to: ContentId<Account>, amount: Int}
                -> <transfer> ContentId<TransferResult>

-- Calculate resource delta (must sum to zero)
effect compute_delta : List<ResourceOperation> -> <resource_accounting> ContentId<Delta>

-- Resource handler that enforces conservation
handler resource_handler
  -- Track resource balances
  var balances = {}
  
  -- Handle transfer
  effect transfer params ->
    -- Update balances
    from_balance = balances[params.from][params.resource] || 0
    to_balance = balances[params.to][params.resource] || 0
    
    -- Check sufficient balance
    if from_balance >= params.amount then
      -- Update balances
      balances[params.from][params.resource] = from_balance - params.amount
      balances[params.to][params.resource] = to_balance + params.amount
      
      -- Create and return result
      result = { success: true }
      result_id = perform content_id result
      perform store result_id result
      resume result_id
    else
      -- Create and return error
      error = { success: false, reason: "Insufficient balance" }
      error_id = perform content_id error
      perform store error_id error
      resume error_id
      
  -- Verify conservation
  effect verify_conservation operations ->
    delta = compute_resource_delta(operations)
    if delta == 0 then
      resume perform content_id { valid: true }
    else
      resume perform content_id { valid: false, reason: "Conservation violation" }
```

### 9.2 Domain-Aware Causality

TEL enforces correct causal relationships across domains:

```elm
-- Define domain
effect define_domain : DomainDefinition -> <domain> ContentId<Domain>

-- Cross-domain operation
effect cross_domain : {source: ContentId<Domain>, target: ContentId<Domain>, operation: Operation}
                    -> <cross_domain> ContentId<OperationResult>

-- Domain handler enforcing causality
handler domain_handler
  -- Domain registry
  var domains = {}
  
  -- Handle cross-domain operations
  effect cross_domain params ->
    -- Verify domains exist
    source_domain <- perform load params.source
    target_domain <- perform load params.target
    
    -- Check domain compatibility
    if compatible_domains(source_domain, target_domain) then
      -- Execute operation
      result <- perform execute_in_domain {
        domain: params.target,
        operation: params.operation
      }
      
      -- Create causal link
      perform link {
        from: params.source,
        to: params.target,
        relation: "cross_domain_operation"
      }
      
      -- Return result
      resume result
    else
      -- Return error
      error = { success: false, reason: "Incompatible domains" }
      error_id = perform content_id error
      perform store error_id error
      resume error_id
```

### 9.3 Dual Validation

TEL combines temporal and ancestral validation:

```elm
-- Temporal validation
effect validate_temporal : Sequence -> <temporal> ContentId<ValidationResult>

-- Ancestral validation
effect validate_ancestral : Tree -> <ancestral> ContentId<ValidationResult>

-- Dual validation handler
handler dual_validator
  -- Handle dual validation
  effect validate_dual params ->
    -- Perform temporal validation
    temporal_result <- perform validate_temporal params.sequence
    
    -- Perform ancestral validation
    ancestral_result <- perform validate_ancestral params.tree
    
    -- Combine results
    if temporal_result.valid && ancestral_result.valid then
      result = { valid: true }
    else
      result = {
        valid: false,
        temporal_errors: temporal_result.errors,
        ancestral_errors: ancestral_result.errors
      }
    
    -- Return result
    result_id = perform content_id result
    perform store result_id result
    resume result_id
```

### 9.4 Row Polymorphism Applications

```purescript
-- Generic record operations with row polymorphism
mapValues :: forall r s a b. 
  (a -> b) -> 
  { | Record r a } -> 
  { | Record r b }
mapValues f record = 
  mapRecordWithIndex (\_ v -> f v) record

-- Safe record extension with row constraints
extendRecord :: forall r label a. 
  Lacks label r => 
  label -> a -> { | r } -> { | Insert label a r }
extendRecord label value record = 
  insert label value record

-- Record combination with disjoint constraint
mergeRecords :: forall r1 r2 r3. 
  Union r1 r2 r3 => 
  { | r1 } -> { | r2 } -> { | r3 }
mergeRecords r1 r2 = 
  union r1 r2

-- Generic state transition with row polymorphism
transitionWithData :: forall r1 r2.
  Lacks "timestamp" r1 =>
  State r1 -> 
  { | r2 } -> 
  Effect { state :: Unit } (ContentId (State (Union r1 r2)))
transitionWithData state data = 
  perform "transition" (applyStateData state data)
  where
    applyStateData (Completed fields) extraData = 
      Completed (union fields extraData)
    applyStateData baseState extraData = 
      baseState
```

### 9.5 Query Composition and Optimization

```purescript
-- Query composition
composeQueries :: forall a b c.
  (a -> b) ->
  (b -> c) ->
  Query a ->
  Query c
composeQueries f g query =
  query |> select f |> select g

-- Query optimization
optimizeQuery :: forall a. Query a -> Query a
optimizeQuery query =
  case query of
    Query { where: Just pred1, select: sel } |> where_ pred2 ->
      -- Combine multiple where clauses
      Query { where: Just (\x -> pred1 x && pred2 x), select: sel }
    
    Query { select: sel1 } |> select sel2 ->
      -- Combine projections
      Query { select: sel2 <<< sel1 }
    
    _ -> query

-- Safe query builder with row types
buildQuery :: forall r.
  { collection :: String
  , filter :: Maybe (Record r -> Boolean)
  , sort :: Maybe { field :: String, dir :: SortDirection }
  | r
  } ->
  Query (Record r)
buildQuery spec =
  from spec.collection
  |> maybe identity where_ spec.filter
  |> maybe identity (\s -> orderBy s.field s.dir) spec.sort
```

## 10. Examples

### 10.1 Simple Expression with Row Polymorphism

```purescript
-- Point-free style
S (K 10) (K 5) add

-- With row polymorphism
addFields :: forall r. { x :: Int, y :: Int | r } -> Int
addFields record = record.x + record.y

result = addFields { x: 10, y: 5, label: "point" }
-- Returns 15, working with the extended record
```

### 10.2 Effect Usage with Row Types

```purescript
-- Effect with row polymorphism
logWithMetadata :: forall r.
  { message :: String | r } ->
  Effect { logging :: Unit } Unit
logWithMetadata record =
  perform "log" record

-- Usage
result = do
  sum <- perform "add" { x: 10, y: 5 }
  logWithMetadata 
    { message: "Calculation complete"
    , value: sum
    , timestamp: now
    }
```

### 10.3 State Transition with Row Extension

```purescript
-- State definition with row polymorphism
data OrderState r
  = Pending
  | Processing
  | Completed { orderId :: ContentId | r }
  | Failed { reason :: String | r }

-- State transition with extra data
stateId <- perform "transition" (Completed 
  { orderId
  , amount: 100
  , currency: "USD"
  , customer: { id: customerId, name: customerName }
  })
```

### 10.4 Query-Based State Manipulation

```purescript
-- Query to find orders
pendingOrders <- runQuery $ from "orders"
  |> where_ (\order -> order.status == "pending" && order.amount > 100)
  |> orderBy "createdAt" Asc
  |> limit 10

-- Process orders one by one
results <- for pendingOrders \order -> do
  -- Update order status via query
  updateResult <- runQuery $ from "orders"
    |> where_ (\o -> o.id == order.id)
    |> update (\o -> o { status = "processing" })
  
  -- Perform order processing
  processingResult <- perform "process_order" { id: order.id }
  
  -- Update final status based on processing result
  case processingResult of
    Success -> do
      runQuery $ from "orders"
        |> where_ (\o -> o.id == order.id)
        |> update (\o -> o { 
            status = "completed", 
            completedAt = now
          })
      
      -- Notify customer
      perform "notify" { 
        customer: order.customer,
        message: "Your order has been completed."
      }
      
    Failure reason -> do
      runQuery $ from "orders"
        |> where_ (\o -> o.id == order.id)
        |> update (\o -> o { 
            status = "failed", 
            failureReason = reason
          })
      
      -- Notify customer
      perform "notify" { 
        customer: order.customer,
        message: "Your order failed: " <> reason
      }
      
  pure { orderId: order.id, result: processingResult }
```

### 10.5 Complex Query with Joins

```purescript
-- Query for order data with customer and product information
orderDetails <- runQuery $ from "orders"
  |> where_ (\order -> order.id == targetOrderId)
  |> join "customers" (\order customer -> order.customerId == customer.id)
  |> join "products" (\{order} product -> Array.elem product.id order.productIds)
  |> select \{order, customer, product} -> 
      { orderId: order.id
      , orderDate: order.createdAt
      , customerName: customer.name
      , productName: product.name
      , quantity: lookupQuantity order.items product.id
      }

-- Function to get product quantity from order items
lookupQuantity :: Array OrderItem -> ProductId -> Int
lookupQuantity items productId =
  case Array.find (\item -> item.productId == productId) items of
    Just item -> item.quantity
    Nothing -> 0

-- Process the order details
for orderDetails \detail -> do
  -- Use the joined data
  perform "log" { message: "Processing order item", detail }
  
  -- Update inventory based on the data
  perform "update_inventory" {
    productId: detail.productId,
    quantityChange: -detail.quantity
  }
```

## 11. Relationship to Lambda Calculus

TEL's combinator foundation is rooted in the lambda calculus:

| Lambda Term | TEL Combinator | Description |
|-------------|----------------|-------------|
| λx.x | I | Identity |
| λx.λy.x | K | Constant |
| λx.λy.λz.xz(yz) | S | Substitution |
| λx.λy.λz.x(yz) | B | Composition |
| λx.λy.λz.xzy | C | Flip |

These five combinators are sufficient to express any lambda term without explicit variables, making the language both powerful and elegant. The combinatory approach provides:

1. **Variable-free programming**: All operations and compositions can be expressed without named variables
2. **Compositional semantics**: Natural composition of functions and effects
3. **Referential transparency**: Expressions have the same meaning regardless of context
4. **Content addressing compatibility**: Easy serialization and content addressing

The combination of these core combinators with domain-specific combinators for effects, states, and resources creates a language that is both mathematically elegant and practically powerful. 