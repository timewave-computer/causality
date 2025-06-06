;; DEX Swap Example
;; Demonstrates token swapping between two different token types
;;
;; This example shows:
;; 1. Liquidity pool management
;; 2. Token swapping mechanics
;; 3. Price calculation based on constant product formula
;; 4. Atomic swap execution

;; Create a liquidity pool with two token types
;; A pool contains reserves of both Token A and Token B
(define create-pool (lambda (token-a-amount token-b-amount)
  (pure (list 
         (alloc 'TokenA token-a-amount)     ; Reserve A
         (alloc 'TokenB token-b-amount)))))  ; Reserve B

;; Simple constant product AMM formula: x * y = k
;; For swap of amount_in of Token A for Token B:
;; amount_out = (reserve_b * amount_in) / (reserve_a + amount_in)
(define calculate-swap-output (lambda (reserve-a reserve-b amount-in)
  (let ((numerator (* reserve-b amount-in)))
    (let ((denominator (+ reserve-a amount-in)))
      (pure (/ numerator denominator))))))

;; Execute a swap: Trade Token A for Token B
;; Takes pool reserves and input token, returns updated pool and output token
(define swap-a-for-b (lambda (pool-a pool-b input-token)
  (let ((reserve-a (consume pool-a)))
    (let ((reserve-b (consume pool-b)))
      (let ((amount-in (consume input-token)))
        ;; Calculate output amount using AMM formula
        (bind (calculate-swap-output reserve-a reserve-b amount-in)
              (lambda (amount-out)
                (let ((new-reserve-a (+ reserve-a amount-in)))
                  (let ((new-reserve-b (- reserve-b amount-out)))
                    ;; Return updated pool and output token
                    (pure (list
                           (alloc 'TokenA new-reserve-a)    ; Updated reserve A
                           (alloc 'TokenB new-reserve-b)    ; Updated reserve B
                           (alloc 'TokenB amount-out))))))))))) ; Output token

;; Example usage:

;; 1. Create initial liquidity pool with 1000 TokenA and 2000 TokenB
(define initial-pool (create-pool 1000 2000))

;; 2. User wants to swap 100 TokenA for TokenB
(define user-input (alloc 'TokenA 100))

;; 3. Extract pool reserves
(define pool-reserves 
  (bind initial-pool
        (lambda (pool)
          (pure (list (car pool) (cadr pool))))))

;; 4. Execute the swap
(define swap-result
  (bind pool-reserves
        (lambda (reserves)
          (let ((pool-a (car reserves)))
            (let ((pool-b (cadr reserves)))
              (swap-a-for-b pool-a pool-b user-input))))))

;; 5. The result contains: [new-pool-a, new-pool-b, output-token-b]
;; Expected: ~181 TokenB (calculated from constant product formula)
(pure swap-result) 