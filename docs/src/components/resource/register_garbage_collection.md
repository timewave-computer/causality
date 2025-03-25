<!-- Garbage collection for registers -->
<!-- Original file: docs/src/register_garbage_collection.md -->

# Register Garbage Collection

This document describes the implementation and usage of the epoch-based register garbage collection system in Causality.

## Overview

The Causality system uses a one-time use register model, where registers are immutable and consumed when used in operations, creating new registers for state changes. This provides strong auditability and causal reasoning, but can lead to state explosion over time. 

To address this, we've implemented an epoch-based garbage collection system that:

1. Maintains a clean conceptual model with one-time use registers
2. Prevents state explosion by archiving and summarizing old registers
3. Preserves essential state information while reclaiming storage
4. Provides mechanisms for retrieving archived registers when needed

## Core Components

### Epoch Management

Registers are grouped into epochs, which typically correspond to a period of time or a block height range. The system maintains:

- Current epoch ID
- Epoch boundaries (mapping of epochs to block heights)
- Register tracking per epoch
- Archival policies

Epochs advance according to system-defined events, such as time periods or block height thresholds.

### Register Status Lifecycle

Each register goes through a lifecycle with these possible statuses:

- **Active**: Register is usable for operations
- **Consumed**: Register has been used in an operation and cannot be reused
- **Archived**: Register has been compressed and moved to archive storage
- **Summarized**: Register has been consolidated into a summary with other registers

### Garbage Collection Process

The garbage collection process operates at epoch boundaries and includes:

1. Identifying epochs eligible for collection (based on age)
2. Creating summaries of registers based on configurable strategies
3. Compressing and archiving original register data
4. Replacing full registers with smaller stubs containing essential information
5. Updating global indices to maintain system consistency

### Summarization Strategies

The system supports multiple summarization strategies:

- **Resource-based**: Group registers by resource type
- **Account-based**: Group registers by owner account
- **Type-based**: Group registers by register content type
- **Custom**: Define custom grouping functions

### Archive System

Archived registers are stored in a dedicated archive system that provides:

- Efficient compressed storage
- Data integrity verification
- On-demand retrieval
- Searchability by various criteria

## Configuration

The garbage collection system is configurable with these key parameters:

- `keepEpochs`: Number of recent epochs to keep fully accessible
- `pruneAfter`: Age (in epochs) at which garbage collection begins
- `summaryStrategy`: Strategy for creating register summaries
- `archiveLocation`: Where to store archived register data

## Usage

### Automatic Garbage Collection

The system will automatically trigger garbage collection for eligible epochs during normal operation. No manual intervention is typically required.

### Manual Management

For maintenance or debugging purposes, the `register_gc.sh` script provides utilities for:

- Viewing the current epoch and register status
- Manually triggering garbage collection
- Managing archival policies
- Retrieving registers from archives
- Monitoring storage usage

Example usage:

```bash
# View current status
./scripts/register_gc.sh status

# Run garbage collection on a specific epoch
./scripts/register_gc.sh gc 5

# Force garbage collection even if not eligible
./scripts/register_gc.sh gc 10 --force

# Change configuration
./scripts/register_gc.sh config PRUNE_AFTER 7

# List archived registers
./scripts/register_gc.sh archives

# Retrieve a register from archives
./scripts/register_gc.sh retrieve reg-archive-12345
```

## Implementation Details

The garbage collection system is implemented in the `TimeOperators.Core.Resource.EpochManager` module, with these key components:

- `EpochManager`: Manages epoch boundaries and tracks registers
- `ArchivalPolicy`: Configures garbage collection behavior
- `RegisterStatus`: Tracks register lifecycle states
- `SummaryStrategy`: Defines how to group and summarize registers
- `ArchiveSystem`: Handles storage and retrieval of archived registers

Integration with the rest of the system is handled through the `TimeOperators.Core.Resource.Integration` module, which provides:

- Initialization of the epoch manager
- Periodic garbage collection triggers
- Global state management

## Performance Considerations

The garbage collection system is designed to:

- Process registers in batches to manage memory usage
- Operate asynchronously to avoid blocking user operations
- Use efficient compression for archived registers
- Minimize on-Domain storage for summarized registers

## Security Considerations

The garbage collection system maintains security by:

- Preserving cryptographic verification of archived data
- Maintaining register hashes for integrity verification
- Keeping essential provenance information in summaries
- Supporting ZK-proof verification of register histories

## Conclusion

The epoch-based garbage collection system provides an ideal balance between conceptual clarity and practical efficiency. By treating registers as immutable and creating explicit state transitions, we gain strong auditability and causal reasoning. By implementing structured garbage collection at epoch boundaries, we prevent state explosion without sacrificing these properties. 