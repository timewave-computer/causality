;; Minimal Token Example
;; Uses only basic primitives that are currently implemented
;; 
;; This demonstrates:
;; 1. Creating a resource (token)
;; 2. Basic effect sequencing with pure and bind

;; Simple token creation - creates a resource with value 100
(pure (alloc 100))

;; Note: This is the minimal example that should work with the current implementation.
;; More advanced features like transfers, balance checking, and function definitions
;; will be added as the compiler supports more language constructs. 