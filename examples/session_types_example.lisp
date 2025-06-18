;; Session Types Example for Causality Lisp
;; Demonstrates the session types integration as part of Phase 4: Causality Lisp Integration
;;
;; This example shows:
;; 1. Session type declarations with dual protocols
;; 2. Session usage with with-session contexts
;; 3. Session operations (send, receive, select, case)
;; 4. Type-safe communication protocols

;; Define a simple payment protocol with client and server roles
(def-session PaymentProtocol
  (client !Amount ?Receipt End)
  (server ?Amount !Receipt End))

;; Client-side payment implementation
(define handle-payment-client (lambda (amount)
  (with-session PaymentProtocol.client
    (do
      ;; Send payment amount to server
      (session-send channel amount)
      
      ;; Receive receipt from server
      (session-recv channel)))))

;; Server-side payment implementation  
(define handle-payment-server (lambda ()
  (with-session PaymentProtocol.server
    (do
      ;; Receive payment amount from client
      (let ((amount (session-recv channel)))
        ;; Process payment and generate receipt
        (let ((receipt (process-payment amount)))
          ;; Send receipt back to client
          (session-send channel receipt)))))))

;; Example with choice operations - negotiation protocol
(def-session NegotiationProtocol
  (proposer !Offer (?Counter !Accept End))
  (acceptor ?Offer (!Counter ?Accept End)))

;; Proposer implementation with choices
(define make-proposal (lambda (initial-offer)
  (with-session NegotiationProtocol.proposer
    (do
      ;; Send initial offer
      (session-send channel initial-offer)
      
      ;; Handle response using session-case
      (session-case channel
        (Counter (lambda (counter-offer)
          (session-send channel (accept-counter counter-offer))))
        (Accept (lambda (acceptance)
          (finalize-deal acceptance))))))))

;; Acceptor implementation  
(define handle-proposal (lambda ()
  (with-session NegotiationProtocol.acceptor
    (do
      ;; Receive initial offer
      (let ((offer (session-recv channel)))
        ;; Decide whether to counter or accept
        (if (acceptable-offer offer)
            ;; Accept the offer
            (session-select channel "Accept")
            ;; Make counter-offer
            (do
              (session-select channel "Counter")
              (session-send channel (make-counter-offer offer))
              (session-recv channel))))))))

;; Multi-party session example - escrow protocol
(def-session EscrowProtocol
  (buyer !Item ?Quote !Payment ?Confirmation End)
  (seller ?Item !Quote ?Payment !Delivery End)
  (escrow ?Payment !Payment ?Confirmation !Delivery End))

;; Buyer role in escrow
(define buyer-escrow (lambda (item payment)
  (with-session EscrowProtocol.buyer
    (do
      ;; Send item request
      (session-send channel item)
      
      ;; Receive quote
      (let ((quote (session-recv channel)))
        ;; Send payment to escrow
        (session-send channel payment)
        
        ;; Wait for confirmation
        (session-recv channel))))))

;; Example usage demonstrating session types integration
(define example-usage (lambda ()
  ;; Simple payment
  (let ((client-result (handle-payment-client 100)))
    ;; Negotiation
    (let ((proposal-result (make-proposal "Initial offer")))
      ;; Escrow transaction
      (buyer-escrow "Ticket" 50)))))

;; This example demonstrates how session types provide:
;; 1. Type-safe communication protocols
;; 2. Automatic duality checking (client/server protocols are compatible)
;; 3. Deadlock freedom (well-typed programs cannot deadlock)
;; 4. Protocol compliance (operations must follow declared session types)
;; 5. Integration with Layer 1 linear lambda calculus
;;
;; Session types form the third pillar of Layer 2 alongside Effects and Intents,
;; enabling complex distributed protocol orchestration with compile-time safety. 