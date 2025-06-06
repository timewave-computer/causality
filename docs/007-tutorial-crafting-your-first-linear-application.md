# 007: Tutorial - Building a Linear Resource Application

Welcome to your first hands-on experience with Causality! This tutorial will guide you through creating a simple linear application using Causality Lisp. We'll focus on the core principles of linearity and how they manifest in code, managing a unique digital asset: an Event Ticket.

## 1. Prerequisites

Before you begin, please ensure you have set up your development environment as described in [006: Environment Setup and First Build](./006-environment-setup-and-first-build.md). This will provide you with the necessary tools to (conceptually) compile and run the Lisp code we'll be writing.

## 2. The Scenario: Digital Event Tickets

Imagine we want to create a system for managing digital tickets for an event. Each ticket must be unique and should only be usable once. Furthermore, a ticket can only be owned by one person at a time. This is a perfect scenario for a linear application:

-   **Uniqueness and Scarcity**: Each ticket is a distinct resource.
-   **Linearity**: A ticket cannot be duplicated. When it's transferred, the old owner no longer possesses it. When it's redeemed, it's consumed and cannot be used again.

We'll use Causality Lisp (Layer 1) to define the logic for issuing, transferring, and redeeming these tickets.

## 3. Designing the Ticket Resource

In Causality Lisp, we can represent our ticket using a `record`. A record is a collection of named fields. For our ticket, we'll want to store:

-   `event_name`: The name of the event (e.g., "Causality Con 2025").
-   `ticket_id`: A unique identifier for this specific ticket.
-   `owner`: The identifier of the current ticket holder (e.g., a public key or username).
-   `status`: The current status of the ticket (e.g., "active", "redeemed").

Here's how a ticket might look as a Lisp `record` value:

```lisp
(record ("event_name" "Causality Con 2025")
        ("ticket_id" "TICKET-001")
        ("owner" "alice_pk")
        ("status" "active"))
```

## 4. Writing Causality Lisp Code

Let's define some functions to manage our tickets. Remember, in Causality, operations that modify linear resources typically involve `consume`-ing the old state and `alloc`-ating a new state.

### 4.1. Issuing a New Ticket

To issue a new ticket, we'll create a new record with the ticket's details and use the `alloc` primitive to bring it into existence as a linear resource. The `alloc` primitive returns a `ResourceId` which is a handle to this newly created resource.

```lisp
(defun issue-ticket (event-name ticket-id initial-owner)
  (let ((new-ticket-data 
          (record ("event_name" event-name)
                  ("ticket_id" ticket-id)
                  ("owner" initial-owner)
                  ("status" "active"))))
    (alloc new-ticket-data) ; Allocates the ticket data as a new linear resource
    ))

;; Example of issuing a ticket:
;; (let ((ticket1-id (issue-ticket "Causality Con 2025" "TICKET-001" "alice_pk")))
;;   ticket1-id ; This would be the ResourceId of the new ticket
;; ) 
```

### 4.2. Transferring a Ticket

To transfer a ticket, the current owner must provide their `ResourceId` for the ticket. We'll `consume` this resource (which makes the old state unavailable), update the `owner` field, and then `alloc` the modified ticket data as a new resource. This new resource will have a new `ResourceId`.

```lisp
(defun transfer-ticket (ticket-resource-id new-owner)
  (let ((current-ticket-data (consume ticket-resource-id)))
    ;; Ensure the ticket is active before transferring
    (assert (string=? (get-field current-ticket-data "status") "active"))

    (let ((transferred-ticket-data 
            (update-field current-ticket-data "owner" new-owner)))
      (alloc transferred-ticket-data) ; Allocates the updated ticket data
    )))

;; Example of transferring ticket1-id (from previous example) to bob_pk:
;; (let ((new-ticket2-id (transfer-ticket ticket1-id "bob_pk")))
;;   new-ticket2-id ; This is the ResourceId of the ticket now owned by bob_pk
;; ) 
```
**Key Point**: After `(consume ticket-resource-id)`, the resource associated with `ticket-resource-id` is no longer valid. A new resource (and `ResourceId`) is created by `(alloc transferred-ticket-data)`.

### 4.3. Redeeming a Ticket

Redeeming a ticket means it's been used and should no longer be active or transferable. We can model this by consuming the ticket and changing its status to "redeemed".

```lisp
(defun redeem-ticket (ticket-resource-id)
  (let ((current-ticket-data (consume ticket-resource-id)))
    (assert (string=? (get-field current-ticket-data "status") "active"))

    (let ((redeemed-ticket-data 
            (update-field current-ticket-data "status" "redeemed")))
      ;; Optionally, we could alloc this 'redeemed' state if we need a record of it.
      ;; Or, if redeeming truly means it's gone, we might not alloc a new state,
      ;; effectively burning the ticket. For this tutorial, let's alloc its redeemed state.
      (alloc redeemed-ticket-data)
      (unit) ; Return unit to signify completion
    )))

;; Example of redeeming ticket2-id (owned by bob_pk):
;; (redeem-ticket new-ticket2-id)
```

## 5. Core Lisp Primitives Used

In this tutorial, we've used several core Causality Lisp constructs:

-   `(defun name (params...) body...)`: Defines a function.
-   `(let ((var value)...) body...)`: Binds variables to values locally.
-   `(record (field-name value)...)`: Creates a record data structure.
-   `(alloc data)`: Allocates `data` as a new linear resource, returning its `ResourceId`.
-   `(consume resource-id)`: Consumes the linear resource pointed to by `resource-id`, returning its data. The resource is no longer valid.
-   `(get-field record-data field-name)`: Retrieves the value of a field from a record.
-   `(update-field record-data field-name new-value)`: Returns a *new* record with the specified field updated. (Records are immutable; this creates a new version).
-   `(assert condition)`: Halts execution if the condition is false.
-   `(string=? str1 str2)`: Compares two strings for equality.
-   `(unit)`: Represents a void or empty value, often used to signify completion of an action.

These primitives form the building blocks for defining resource transformations and logic in Layer 1.

## 6. Running Your Lisp Code (Conceptual)

In a complete Causality application:

1.  Your Lisp code (like the functions `issue-ticket`, `transfer-ticket`) would be defined and stored, likely identified by their own content-hashes (`ExprId`).
2.  These functions would be invoked as part of Layer 2 `Effect`s or `Intent`s. For example, an `Intent` to "Transfer Ticket" would reference the `transfer-ticket` Lisp `ExprId` and provide the necessary `ResourceId` and new owner details as inputs.
3.  The Causality Lisp compiler would translate your Lisp code into Layer 0 Typed Register Machine instructions.
4.  The Layer 0 VM would execute these instructions, performing the `alloc` and `consume` operations on actual resources.

The environment setup in `006` provides tools like `cargo build` and `cargo test` which compile and test the Rust components that implement this Lisp interpreter, compiler, and the Layer 0 VM.

## 7. Conclusion

Congratulations! You've designed and written your first linear application logic in Causality Lisp. You've seen how to:

-   Model real-world assets as linear resources.
-   Use Lisp `record`s to structure data.
-   Employ `alloc` and `consume` to manage the lifecycle of linear resources, ensuring correctness.
-   Define functions to encapsulate resource logic.

This simple ticket example demonstrates the power of linearity for building robust and verifiable systems. From here, you can explore more complex data structures, advanced Lisp patterns, and how these Layer 1 constructs integrate with Layer 2 Effects and Intents for full-fledged application development.

Refer back to [004: Layer 1 - Structured Types and Causality Lisp](./004-layer-1-structured-types-and-causality-lisp.md) and [007: Causality Lisp Language Specification](./007-causality-lisp-language-specification.md) for more details on Layer 1 and the Lisp language.
