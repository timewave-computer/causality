;; Multi-Party Transaction Example
;; Demonstrates atomic execution across multiple participants
;;
;; This example shows:
;; 1. Multi-party atomic swaps
;; 2. Conditional execution based on all parties' agreement
;; 3. Escrow-like functionality
;; 4. All-or-nothing transaction semantics

;; Create a transaction proposal that multiple parties must sign
(define create-proposal (lambda (proposer-id amount asset-type conditions)
  (alloc 'Proposal (list proposer-id amount asset-type conditions 'pending))))

;; Sign a proposal by a party
;; In practice, this would involve cryptographic signatures
(define sign-proposal (lambda (proposal signer-id signature)
  (let ((proposal-data (consume proposal)))
    (let ((existing-signatures (car (cddddr proposal-data))))
      (let ((new-signatures (list signer-id signature existing-signatures)))
        (let ((updated-data (list 
                             (car proposal-data)      ; proposer-id
                             (cadr proposal-data)     ; amount  
                             (caddr proposal-data)    ; asset-type
                             (cadddr proposal-data)   ; conditions
                             new-signatures)))        ; signatures
          (alloc 'Proposal updated-data)))))))

;; Check if all required parties have signed
(define all-parties-signed (lambda (proposal required-parties)
  (let ((proposal-data (consume proposal)))
    (let ((signatures (car (cddddr proposal-data))))
      ;; Simplified: check if we have enough signatures
      ;; In practice, would verify cryptographic signatures
      (if (>= (length signatures) required-parties)
          (pure (list #t (alloc 'Proposal proposal-data)))
          (pure (list #f (alloc 'Proposal proposal-data))))))))

;; Execute multi-party atomic swap
;; All parties contribute assets, all receive according to agreement
(define execute-multi-party-swap (lambda (proposals participants)
  ;; Collect all assets from participants
  (let ((total-contributions (collect-contributions proposals)))
    ;; Verify all conditions are met
    (bind (verify-all-conditions proposals)
          (lambda (conditions-met)
            (if conditions-met
                ;; Distribute assets according to agreement
                (distribute-assets total-contributions participants)
                ;; Return all assets to original owners
                (pure 'transaction-failed)))))))

;; Collect contributions from all participants
(define collect-contributions (lambda (proposals)
  ;; Simplified: sum all contributed amounts
  ;; In practice, would handle different asset types
  (let ((total 0))
    (pure total))))  ; Placeholder implementation

;; Verify all conditions for the multi-party transaction
(define verify-all-conditions (lambda (proposals)
  ;; Simplified: assume all conditions met
  ;; In practice, would check each condition
  (pure #t)))

;; Distribute assets according to the agreement
(define distribute-assets (lambda (total-assets participants)
  ;; Create new asset allocations for each participant
  (let ((assets-per-participant (/ total-assets (length participants))))
    (pure (map (lambda (participant)
                 (alloc 'Asset assets-per-participant))
               participants)))))

;; Map function implementation (simplified)
(define map (lambda (f list)
  (if (= list 'nil)
      (pure 'nil)
      (bind (f (car list))
            (lambda (result)
              (bind (map f (cdr list))
                    (lambda (rest)
                      (pure (cons result rest)))))))))

;; Example usage: Three-party atomic swap

;; 1. Alice proposes to contribute 100 TokenA
(define alice-proposal 
  (create-proposal 'alice 100 'TokenA 'none))

;; 2. Bob proposes to contribute 200 TokenB  
(define bob-proposal
  (create-proposal 'bob 200 'TokenB 'none))

;; 3. Charlie proposes to contribute 50 TokenC
(define charlie-proposal
  (create-proposal 'charlie 50 'TokenC 'none))

;; 4. Each party signs all proposals
(define alice-signed-proposals
  (bind alice-proposal
        (lambda (prop1)
          (bind (sign-proposal prop1 'alice 'alice-sig)
                (lambda (signed1)
                  (bind bob-proposal
                        (lambda (prop2)
                          (bind (sign-proposal prop2 'alice 'alice-sig)
                                (lambda (signed2)
                                  (bind charlie-proposal
                                        (lambda (prop3)
                                          (sign-proposal prop3 'alice 'alice-sig))))))))))))

;; 5. Verify all parties have signed (simplified)
(define all-signed
  (bind alice-signed-proposals
        (lambda (proposals)
          (all-parties-signed proposals 3))))

;; 6. Execute the multi-party transaction if all conditions met
(define transaction-result
  (bind all-signed
        (lambda (verification)
          (if (car verification)
              (let ((participants (list 'alice 'bob 'charlie)))
                (execute-multi-party-swap (list alice-proposal bob-proposal charlie-proposal) participants))
              (pure 'insufficient-signatures)))))

;; 7. The result shows successful multi-party atomic execution
;; All parties receive their allocated assets or transaction fails completely
(pure transaction-result) 