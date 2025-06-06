# 008: Advanced Examples and Idiomatic Patterns

This document showcases advanced usage patterns and idiomatic examples within the Causality framework. These examples aim to illustrate how the core principles and layered architecture come together to solve complex problems, manage resources effectively, and integrate advanced features like Zero-Knowledge Proofs (ZKPs).

## 1. Complex Resource Management with Linearity and Capabilities

Causality's strength lies in its robust resource management. This example demonstrates managing a multi-component, upgradeable digital asset where different parts have different capabilities and ownership.

**Scenario**: A 'Digital Artwork' `Resource` that has a core `ImageComponent` (linear, owned by the artist) and an optional, affine `DisplayLicenseComponent` (transferable by the current artwork owner).

```rust
// --- Core Type Definitions (Conceptual Rust) ---

// In causality-types crate
pub struct ImageComponentData { pub image_hash: [u8; 32], pub artist_signature: [u8; 64] }
pub struct DisplayLicenseData { pub expiry_date: u64, pub licensed_to: Address }

// Capabilities (defined as simple structs for this example)
pub struct TransferArtistCredit; // Capability to change artist attribution (rare)
pub struct UpdateImageHash;      // Capability to update the core image (artist only)
pub struct IssueDisplayLicense;  // Capability for artwork owner to issue a license
pub struct TransferDisplayLicense; // Capability for license holder to transfer it

// Define the main Artwork Resource using Row Types for capabilities
// This is a conceptual representation; actual RowType usage would be more integrated.
type ArtworkCapabilities = Row!{
    transfer_artist_credit: TransferArtistCredit,
    update_image_hash: UpdateImageHash,
    issue_display_license: IssueDisplayLicense
};

pub struct Artwork {
    pub id: ResourceId,
    pub image_component: Resource<ImageComponentData>, // Linear, owned by artist initially
    pub active_license: Option<Object<DisplayLicenseData, Affine>> // Affine, can be None or exist once
    // capabilities field implied by ArtworkCapabilities through Layer 1 type system
}
```

**Idiomatic Pattern**: Using `Object` with `Affine` linearity for the license allows it to be optionally present and consumed (e.g., when it expires or is upgraded). The core `ImageComponent` remains `Linear`. Row types on the `Artwork` itself would define who can perform which actions.

**Causality Lisp Snippet (Conceptual for issuing a license)**:

```lisp
;; Assume 'artwork-res' is a ResourceId for an Artwork
;; 'licensee-addr' is the address for the new licensee
;; 'expiry-duration' is an integer for license duration

(defun issue-new-license (artwork-res licensee-addr expiry-duration)
  (let ((current-artwork (consume artwork-res)))
    (assert (has-capability? current-artwork 'issue_display_license))
    (assert (is-none? (get-field current-artwork 'active_license)))

    (let ((new-license-data 
            (record ("expiry_date" (+ (current-time) expiry-duration))
                    ("licensed_to" licensee-addr))))
      (let ((new-license-object (make-object new-license-data 'Affine)))
        (let ((updated-artwork (set-field current-artwork 'active_license (some new-license-object))))
          (alloc updated-artwork) ; Re-allocate the artwork with the new license
        )))))
```
This Lisp snippet demonstrates consuming the artwork, checking capabilities, creating a new affine license object, updating the artwork, and re-allocating it. Linearity ensures the old state of the artwork is gone.

## 2. Advanced Causality Lisp: Higher-Order Functions and Resource Transformation

**Scenario**: A generic Lisp function that takes a list of `ResourceId`s, a transformation function, and applies the transformation to each resource, collecting the new `ResourceId`s.

```lisp
;; 'resource-ids' is a list of ResourceId
;; 'transform-fn' is a lambda: (lambda (resource-value) -> new-resource-value)
(defun map-transform-resources (resource-ids transform-fn)
  (if (empty? resource-ids)
      '() ;; Base case: empty list
      (let* ((current-res-id (car resource-ids))
             (remaining-res-ids (cdr resource-ids))
             (resource-value (consume current-res-id))
             (transformed-value (apply transform-fn resource-value))
             (new-res-id (alloc transformed-value)))
        (cons new-res-id (map-transform-resources remaining-res-ids transform-fn)))))

;; Example usage:
(defun double-balance (token-data) ; token-data is a LispValue record
  (update-field token-data "balance" (* (get-field token-data "balance") 2)))

(let ((my-token-ids (list token-id1 token-id2 token-id3)))
  (map-transform-resources my-token-ids double-balance))
```
**Idiomatic Pattern**: This showcases recursion for list processing, higher-order functions (`transform-fn`), and the core `consume`/`alloc` pattern for resource transformation. Linearity is maintained for each resource within the loop.

## 3. Sophisticated Effect and Handler Patterns: Composable Data Validation

**Scenario**: An `Intent` to register a new user requires validating multiple pieces of data (e.g., email format, password strength, unique username). This can be modeled as a primary `RegisterUserEffect` which is processed by a chain of validation `Handler`s.

```rust
// --- Conceptual Rust Definitions ---
pub struct UserRegistrationData { email: String, username: String, pass_hash: [u8;32] }
pub struct Validated<T> { data: T }

// Effects
pub struct ValidateEmailEffect { pub email: String }
pub struct ValidateUsernameEffect { pub username: String }
pub struct PersistUserEffect { pub data: Validated<UserRegistrationData> }

// Primary Effect for the Intent
pub struct RegisterUserRequestEffect { pub data: UserRegistrationData }

// Handler Traits (simplified)
pub trait Handles<E_In, E_Out> { fn handle(&self, effect: E_In) -> Result<E_Out, String>; }
```

**Idiomatic Pattern**: Handler Chaining and Effect Transformation.
1.  An `Intent` to register a user generates a `RegisterUserRequestEffect`.
2.  A `MasterRegistrationHandler` receives this.
3.  It first creates and sends a `ValidateEmailEffect(data.email)` to an `EmailValidationHandler`. If successful, it proceeds.
4.  Then, it creates and sends `ValidateUsernameEffect(data.username)` to a `UsernameValidationHandler`.
5.  If all validations pass, it transforms the `UserRegistrationData` into `Validated<UserRegistrationData>` and creates a `PersistUserEffect` which is sent to a `UserPersistenceHandler`.

This pattern allows complex logic to be broken down into smaller, testable, and reusable handlers. Each handler focuses on a specific concern. The `Effect` types clearly define the data contracts between handlers.

## 4. Temporal Effect Graph (TEG) Orchestration: Multi-Party Escrow

**Scenario**: A decentralized escrow involving a Buyer, a Seller, and an Arbiter. Funds are locked, goods are confirmed, and funds are released or returned.

**`Intent`: `ExecuteEscrow`**
-   Inputs: BuyerFunds (ResourceId), ItemDetails (LispValue), SellerAddress, ArbiterAddress
-   Outputs: SellerReceipt (ResourceId) or BuyerRefund (ResourceId)

**Conceptual TEG Nodes and Edges from this Intent:**

1.  `EffectNode_LockFunds`:
    -   Effect: `LockFundsEffect(BuyerFunds, EscrowContractAddress)`
    -   Consumes `BuyerFunds`, produces `LockedFundsReceipt`.
2.  `EffectNode_NotifySeller` (depends on `EffectNode_LockFunds` via `LockedFundsReceipt`):
    -   Effect: `NotifySellerEffect(SellerAddress, ItemDetails, EscrowContractAddress)`
3.  `EffectNode_SellerConfirmShipment` (triggered externally by seller action):
    -   Effect: `ConfirmShipmentEffect(EscrowContractAddress, TrackingNumber)`
    -   Produces `ShipmentConfirmation`.
4.  `EffectNode_BuyerConfirmReceipt` (depends on `EffectNode_SellerConfirmShipment` via `ShipmentConfirmation`, triggered by buyer):
    -   Effect: `ConfirmReceiptEffect(EscrowContractAddress)`
    -   Produces `ReceiptConfirmation`.
5.  `EffectNode_ReleaseFundsToSeller` (depends on `EffectNode_BuyerConfirmReceipt` via `ReceiptConfirmation`):
    -   Effect: `ReleaseFundsEffect(EscrowContractAddress, SellerAddress)`
    -   Consumes `LockedFundsReceipt` (implicitly via escrow logic), produces `SellerReceipt`.

*Alternative Paths (Handled by `Constraint`s and `Case`-like logic in TEG execution):*
-   If `EffectNode_SellerConfirmShipment` times out -> `EffectNode_ArbiterReview_Timeout`
-   If Buyer disputes (instead of `ConfirmReceiptEffect`) -> `EffectNode_ArbiterReview_Dispute`
-   `EffectNode_ArbiterDecision` -> leads to either `ReleaseFundsToSeller` or `EffectNode_ReturnFundsToBuyer`.

**Idiomatic Pattern**: The TEG clearly defines causal dependencies (e.g., funds must be locked before seller is notified). It allows for parallel steps where appropriate (e.g., notifications). `Constraint`s and conditional paths (modeled by different effect types or parameters) handle the complex state transitions and dispute resolution flows common in escrow scenarios.

## 5. Idiomatic Use of ZKPs: Private Voting

**Scenario**: A voting system where each eligible voter can cast one vote for a proposal, without revealing their individual vote, but the final tally is publicly verifiable.

**Core Components:**
-   `VoterRegistryEffect`: An effect to register a voter, creating a unique, private `VoterCredential` (a linear resource).
-   `CastVoteEffect`:
    -   Inputs: `VoterCredential` (consumed), `ProposalId`, `EncryptedVoteChoice`, `ZKProofOfEligibilityAndSingularity`.
    -   Logic: Verifies the ZKP. The ZKP proves that the `VoterCredential` is valid and hasn't been used before for this `ProposalId`, and that `EncryptedVoteChoice` is a valid encryption of a legitimate choice, all without revealing the voter's identity or their specific choice.
    -   Outputs: `VoteReceiptEffect` (publicly records a vote was cast for `ProposalId`).
-   `TallyVotesEffect`:
    -   Inputs: List of `VoteReceiptEffect`s for a `ProposalId`.
    -   Logic: Uses ZKP techniques (e.g., homomorphic encryption tallies or further ZKPs over encrypted votes) to compute the final tally without decrypting individual votes.
    -   Outputs: `FinalTallyResultEffect`.

**Causality Lisp Snippet (Conceptual for `CastVoteEffect` logic within an Intent/Handler):**

```lisp
(defun process-cast-vote (voter-cred-id proposal-id encrypted-vote zk-proof)
  (let ((voter-credential (consume voter-cred-id)))
    ;; 1. Verify ZK Proof (external call or primitive)
    (assert (verify-zk-proof zk-proof 
                             (list (get-public-key voter-credential) proposal-id encrypted-vote)
                             "vote_eligibility_circuit"))

    ;; 2. If proof is valid, record the encrypted vote (conceptually)
    ;; This would typically involve an effect that interacts with a secure tallying component.
    (let ((vote-record-effect 
            (make-effect "RecordEncryptedVote" 
                         (record ("proposal" proposal-id) ("encrypted_vote_data" encrypted-vote)))))
      (perform vote-record-effect) ; This effect returns a receipt or confirmation

      ;; 3. Return a public receipt (as a new resource or simple value)
      (alloc (record ("status" "vote_counted") ("proposal" proposal-id))) 
    )))
```

**Idiomatic Pattern**: 
-   Linear `VoterCredential` ensures one vote per voter.
-   ZKPs are used to decouple *eligibility/validity* from *identity/choice*. The `verify-zk-proof` primitive is crucial.
-   The `CastVoteEffect` consumes the linear credential and the proof, transforming private inputs into a publicly auditable (but still private regarding choice) event.
-   Layered ZKPs: One ZKP for casting, potentially another set for tallying.

These examples provide a glimpse into how Causality's features can be combined to build sophisticated, secure, and verifiable applications. The key is understanding how linearity, structured data (LispValues, Records), effects, handlers, and the ZKP integration work in concert.
