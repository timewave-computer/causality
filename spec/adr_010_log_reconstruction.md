# ADR-010: Committee as External Indexer Proxy and Unified Log Reconstruction

## Status

**Proposed**

# Context

In early versions of Causality, Committees acted as **simulated chain nodes** with complete control over the Domain. This included:

- Direct ingestion of simulated transactions.
- Custom indexing and fact extraction.
- Writing a **FactLog** directly to disk.
- Serving fact queries to programs from that log.

However, this model does not align with real chain deployments. Many production chains:

- **Do not support custom indexing natively**.
- **Expose limited RPC query surfaces**.
- **Have no native notion of program-specific facts**.

As a result, Committees will migrate from being **embedded simulated nodes** to **external indexer proxies** that:

- Connect to **real chain nodes** (full, archive, RPC).
- Observe Domain data using **pre-configured fact extraction rules**.
- Gossip these facts to Operators and optionally persist them.
- Expose fact queries using **either direct node queries** or derived local FactLogs.
- No longer own the canonical FactLog — Operators and external users can reconstruct FactLogs themselves by reapplying fact extraction rules to chain data.


# Decision

## Revised Committee Role

| Responsibility | Previous (Simulated) | New (Real chain Client) |
|---|---|---|
| Domain Control | Owned Domain data | Observes external Domain |
| Fact Extraction | Direct from internal data | Queries from external node, applies rules |
| Fact Storage | Writes direct FactLog | Optional — Operators can reconstruct |
| Fact Proofing | Internal proof | Uses real chain proofs (inclusion, headers) |
| Fact Gossip | Gossip to Operators | Same |
| Fact Queries | From local log | From node or reconstructed log |
| Transaction Acceptance | Accept program messages | Same |
| Domain State Queries | Serve direct state queries | Proxy to real node |


## Unified Log and FactLog Construction as On-Demand Process

### Previous
- Each Committee maintained its own FactLog.
- Replay relied directly on these.

### New
- FactLogs can be constructed by **any Operator or external user** with:
    - Direct chain access (RPC).
    - Fact extraction rules.
- Unified Logs for programs will still exist, but they will link to:
    - Facts provided by Committees (real-time or derived).
- This makes FactLogs a **derived product** rather than a source of truth.
- The unified log of a program links to the **FactIDs**, ensuring causal traceability — the **facts themselves** can be fetched later, even if the original Committee disappears.


## Actor Data Sharing Interface

To support **fact retrieval** from other actors who may have the data, a new **Data Discovery and Request Protocol** will be added.

### Actor Request Interface
Every actor exposes:
```rust
trait DataProvider {
    async fn query_fact(&self, fact_id: FactID) -> Option<Fact>;
    async fn query_log_segment(&self, segment_id: SegmentID) -> Option<LogSegment>;
    async fn query_fact_log_range(&self, domain_id: DomainID, time_range: (LamportTime, LamportTime)) -> Vec<Fact>;
}
```


## Fact Observation Pipeline (Revised)

| Step | Action |
|---|---|
| 1 | Committee connects to real chain node (full node, archive, RPC). |
| 2 | Committee loads fact extraction rules (TOML). |
| 3 | As blocks arrive, the User extracts matching facts. |
| 4 | Facts are signed by the User (proving it observed them). |
| 5 | Facts are gossiped to Operators. |
| 6 | Operators update their **local FactLog cache** (optional). |
| 7 | Operators and programs refer to facts only by `FactID`, not the full content. |
| 8 | External tools (dashboards, auditors) can reconstruct full FactLogs using the same rules. |


## Example Fact Extraction Rule (TOML)

```toml
[[observation]]
type = "PriceObservation"
path = "uniswapV3Pool:ETH/USDC.price"
proof = "inclusion"

[[observation]]
type = "DepositObservation"
path = "timeOperators.escrow.deposit"
proof = "inclusion"
```


## Example Fact (Gossiped)

```json
{
    "factID": "bafy123...",
    "Domain": "Ethereum",
    "factType": "PriceObservation",
    "factValue": {"ETH/USDC": 2900},
    "observedAt": 12345678,
    "proof": {
        "blockHeader": "...",
        "proofPath": ["0xabc", "0xdef"],
        "signedBy": "User.eth"
    }
}
```


## Simulation vs Production Config

| Environment | Domain Source | FactLog Source | Query Source |
|---|---|---|---|
| In-Memory Sim | Embedded mock Domain | Direct writes | Direct queries |
| Local Process Sim | Embedded mock Domain | Direct writes | Direct queries |
| Geo-Distributed | Real chain nodes | Derived from RPC | Queries via RPC or reconstructed log |


## New CLI/Config Options

### Committee Config Example (TOML)

```toml
[User]
Domain = "Ethereum"
mode = "Real"
rpcEndpoint = "https://mainnet.infura.io/v3/...-api-key"
factRules = "./fact_rules/ethereum.toml"
```


## New Public FactLog Construction Tool

To support dashboards and 3rd party tools, provide a CLI:

```bash
nix run .#factlog-reconstruct -- \
    --Domain Ethereum \
    --rpc https://mainnet.infura.io/v3/... \
    --rules ./fact_rules/ethereum.toml \
    --out ./ethereum_factlog.jsonl
```


## Visualizations and Dashboards

- Dashboards can consume reconstructed FactLogs alongside program Unified Logs.
- This allows visualization of:
    - Cross-domain causal flow.
    - Program execution progress.
    - Fact-to-effect causality Domains.


## Benefits

- Supports real chain deployments without requiring native indexing changes.  
- Separates fact observation from storage — Committees can remain lightweight.  
- Enables third-party fact and effect visualization.  
- Keeps FactLogs reconstructible independently of Committee storage.  
- Provides a consistent interface for fact discovery across all simulation and real modes.  
- Allows Causality to support any chain with reasonable RPC access.  


## Example Visualization Flow

1. Dashboard fetches program's Unified Log.
2. Dashboard queries FactIDs referenced in each effect.
3. Dashboard either:
    - Fetches facts from Operators (if cached).
    - Reconstructs facts from chain using fact rules.
4. Dashboard renders full causal DAG, including external facts.


## Architectural Implications

| Component | Change |
|---|---|
| Committee | No longer primary FactLog store (optional). |
| Operators | May store derived FactLogs. |
| Replay | Replay uses derived FactLogs if needed. |
| Visualization | Dashboards query facts directly or reconstruct them. |
| External Auditors | Can independently reconstruct FactLogs. |


## Invariant

- All replayable programs must have a complete causal trace linking to **FactIDs**.  
- All referenced facts must be either:
    - Present in Operator cache.
    - Reconstructible from chain data and fact rules.
- Programs can only observe external state through these facts — no direct RPC queries.