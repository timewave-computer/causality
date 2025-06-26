# Session Types Example (Lisp)

## Overview

This example demonstrates **session types** using Causality Lisp. It shows how to use session types for communication protocols, ensuring type-safe interactions between different parties.

## What it demonstrates

- **Session Type Protocols**: Defining communication protocols with types
- **Type-Safe Communication**: Ensuring protocol compliance at compile time
- **Multi-Party Sessions**: Coordinating multiple participants
- **Protocol Validation**: Checking that implementations follow protocols

## Key Concepts

1. **Session Types**: Types that describe communication protocols
2. **Protocol Compliance**: Ensuring implementations follow the specified protocol
3. **Type-Safe Communication**: Preventing protocol violations at compile time
4. **Multi-Party Coordination**: Managing complex communication patterns

## How to run

```bash
# From the session-types directory
cd /Users/hxrts/projects/timewave/reverse-causality
# Remove comments first (Lisp parser doesn't handle ;; comments yet)
grep -v "^;;" examples/lisp-examples/session-types/session_types_example.lisp | grep -v "^$" > /tmp/clean_session.lisp
cargo run --bin causality -- compile --input /tmp/clean_session.lisp --output /tmp/session_types.out
cat /tmp/session_types.out
```

## Expected Output

The compilation will produce register machine instructions that implement the session type protocol with proper type checking and communication safety.

## Architecture Notes

This demonstrates how **session types** provide compile-time guarantees for communication protocols in **Layer 2** that compile down to efficient **Layer 0** execution.
