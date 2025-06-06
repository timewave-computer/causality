;; Simple Token Transfer Example
;; Demonstrates basic token operations using Causality Lisp
;; 
;; This example shows:
;; 1. Token creation with initial supply
;; 2. Transfer between accounts
;; 3. Balance checking
;; 4. Linear resource management

;; Create a token with initial supply
;; alloc creates a linear resource
(define create-token (lambda (initial-supply)
  (alloc 'Token initial-supply)))

;; Transfer tokens from one account to another
;; Consumes the source token and creates new tokens for source and destination
(define transfer (lambda (token amount destination)
  (let ((balance (consume token)))
    (if (>= balance amount)
        ;; Split the token: remainder to source, amount to destination
        (let ((remainder (- balance amount)))
          (pure (list 
                 (alloc 'Token remainder)      ; Source gets remainder
                 (alloc 'Token amount))))      ; Destination gets amount
        ;; Insufficient balance - recreate original token
        (pure (list (alloc 'Token balance) 'nil))))))

;; Check token balance
;; This is a read-only operation that doesn't consume the token
(define balance (lambda (token)
  (let ((amount (consume token)))
    ;; Recreate the token since we consumed it for inspection
    (pure (list (alloc 'Token amount) amount)))))

;; Example usage:

;; 1. Create initial token with 100 units
(define alice-token (create-token 100))

;; 2. Transfer 30 units from alice to bob
(define transfer-result 
  (bind alice-token 
        (lambda (token) (transfer token 30 'bob))))

;; 3. Extract the resulting tokens
(define extract-tokens
  (bind transfer-result
        (lambda (tokens)
          (pure (list (car tokens)      ; Alice's remaining token
                     (cadr tokens))))))  ; Bob's new token

;; 4. Check Alice's remaining balance
(define check-alice-balance
  (bind extract-tokens
        (lambda (tokens)
          (let ((alice-token (car tokens)))
            (balance alice-token)))))

;; 5. The final result should show Alice has 70 units remaining
(pure check-alice-balance) 