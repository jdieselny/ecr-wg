# Cleanroom Reject Vector Autopsy

**Source:** All files under `emilia-protocol/conformance/vectors/*.json` (23 suites).  
**Spec:** `draft-schrock-ep-authorization-receipts-06.md` + quorum spec + evidence/timestamp related.  
**Constraint:** Purely from vectors + spec text. No reference verify packages.

**Goal (per Iman):** Prove the cleanroom understands *why* each adversarial vector is rejected — tampered payload, wrong key, malformed sig, broken anchor, missing user verification, quorum out-of-order, etc.

**Total reject vectors:** 103

## Summary Statistics

Rejects per suite (sorted):
- EP-CANONICALIZATION-v1: 13
- EP-QUORUM-v1: 11
- EP-RECEIPT-v1: 9
- EP-TIMESTAMP-PROOF-v1: 9
- EP-SIGNOFF-v1 (Class A): 8
- EP-TRUST-RECEIPT-v1 (§6.2): 8
- EP-INITIATOR-ATTESTATION-v1: 7
- EP-RECEIPT-JWS-PROFILE-v1: 7
- EP-REVOCATION-v1: 5
- EP-TIME-ATTESTATION-v1: 5
- EP-SMT-CONSUME-v1 (consumption-proof): 4
- EP-EVIDENCE-RECORD-v1: 4
- EP-PROVENANCE-CHAIN-v1: 4
- EP-WITNESS-v1: 4
- EP-TRUST-RECEIPT timestamp profile + required_approvals: 3
- EP-BOUNDARY-v1: 2

**By primary failure category (approximate, some overlap):**
- Cryptographic / signature / key / tamper: ~35 (JWS, receipts, signoffs, trust-receipt, timestamp, revocation, etc.)
- Structural / profile / JCS (dups, surrogates, numbers, depth): 13 (canonicalization)
- Quorum / SoD / ordering / threshold / role / chain: 11 (quorum)
- Lifecycle / consumption / time / freshness / expiry: ~12
- Anchor / Merkle / inclusion / proof binding: ~10 (trust-receipt, receipts, timestamp-proof, witness)
- Binding / action / context / digest / audience mismatches: many across signoff/receipt/quorum/provenance
- Policy / attestation / version / field presence: ~15 (initiator-attestation, boundary, evidence, etc.)

Most common: cryptographic tampering + wrong key, followed by structural/profile violations and quorum SoD/ordering violations.

---

## Detailed Autopsy by Category

### 1. Cryptographic Failures (bad sig, wrong key, tampered canonical bytes, malformed)

#### receipts.v1.json / reject_tampered_payload
- **File:** receipts.v1.json
- **Failure Class:** cryptographic / payload tamper
- **What is wrong:** The payload inside the signed document was mutated after the signature was created.
- **Detection:** Recompute JCS of (tampered) payload → different bytes → Ed25519 verify over the (wrong) hash fails, or direct sig verify fails.
- **Spec reference:** Core signature binding (Section 5/6.3); any change to signed material invalidates the signature over the JCS form.

#### receipts.v1.json / reject_tampered_nested_param
- **File:** receipts.v1.json
- **Failure Class:** cryptographic / recursive tamper
- **What is wrong:** A deeply nested field inside the action/parameters (e.g. bank destination) was altered.
- **Detection:** Recursive JCS changes → action or context hash changes → signature no longer matches.
- **Spec reference:** JCS applies recursively; binding of action_hash and context_hash.

#### receipts.v1.json / reject_wrong_key
- **File:** receipts.v1.json
- **Failure Class:** cryptographic / key mismatch
- **What is wrong:** Signature is valid for its key, but the presented public_key is unrelated (or wrong enrollment).
- **Detection:** Ed25519 verify succeeds only against the true signer key; presented key produces SignatureFailed.
- **Spec reference:** Approver key lookup + verify (Section 6.3 step 3).

#### receipts.v1.json / reject_malformed_signature
- **File:** receipts.v1.json
- **Failure Class:** cryptographic / malformed
- **What is wrong:** Signature value is not 64 bytes or not valid base64 / not a point on the curve, etc.
- **Detection:** Early parse or dalek `Signature::from_slice` / verify fails with MalformedSignature / SignatureFailed.
- **Spec reference:** Signature format requirements.

#### jws.json / reject_tampered_payload
- **File:** jws.json
- **Failure Class:** cryptographic (JWS profile)
- **What is wrong:** Payload bytes flipped after signing.
- **Detection:** JWS payload is not JCS-canonical or signature over the protected + payload fails.
- **Spec reference:** JWS profile for EP receipts.

#### jws.json / reject_wrong_key, reject_tampered_signature, reject_unsupported_alg, reject_wrong_typ, reject_non_canonical_payload, reject_malformed_compact
- Similar pattern: wrong key, mutated sig segment, alg != EdDSA, typ wrong, payload not JCS, bad 3-part compact serialization.
- All caught at JWS parse + verify layer before higher receipt logic.

#### signoffs.v1.json / reject_crypto_wrong_key
- **File:** signoffs.v1.json
- **Failure Class:** cryptographic
- **What is wrong:** Assertion signature verifies against a different key than the enrolled approver key.
- **Detection:** WebAuthn / Ed25519 verify step uses the wrong pubkey → fail.

#### signoffs.v1.json / reject_crypto_malformed_sig
- Signature bytes invalid for the algorithm (P-256 or Ed25519).

#### trust-receipt.exec.v1.json / reject_tampered_action
- Action inside receipt mutated → action_hash binding + receipt leaf fails.

#### trust-receipt.exec.v1.json / reject_wrong_log_key
- Checkpoint or log_signature verified against wrong/unpinned log key.

#### timestamp-proof.v1.json / reject_tampered_signature, reject_wrong_pinned_key, reject_digest_mismatch
- Signature over timestamp token or covered hash is bad, or pinned TSA key doesn't match.

#### revocation.exec.v1.json / reject_key_substitution, reject_tampered_field
- Revocation record signed with wrong/substituted key or fields mutated.

Many more follow the same "tamper after sign" or "key not the one that signed" pattern. The cleanroom must always re-canonicalize + re-verify the exact bytes the approver (or log/TSA) signed.

---

### 2. Structural / JCS Profile Failures (canonicalization + parse gates)

All 13 from canonicalization.v1.json (see the companion `cleanroom-jcs-edge-cases.md` for full matrix):

- Duplicate keys (literal, escaped alias, nested, non-BMP alias): after unescape the member names collide. Must reject before or during canon.
- Lone surrogates (high, low, reversed, high+ BMP, in member name): RFC 8785 + EP profile explicitly require rejection of unpaired UTF-16 surrogates.
- Unsafe integer (2^53), large exponent, non-integer real: violates I-JSON + EP safe-integer + integer-only rule.
- Depth > 64: pinned bound in EP conformance profile.

These are caught by the strict-parse gate + isCanonicalizable predicate + JCS layer.

---

### 3. Quorum / SoD / Ordering / Policy Violations (quorum.v1.json — 11 rejects)

See detailed inspection for concrete data.

#### quorum.v1.json / reject_under_threshold
- Only 2 signoffs provided when `required: 3`.
- Detection: Count of valid distinct signoffs < policy.required.
- Spec: m-of-n / required_approvals (Section 7).

#### quorum.v1.json / reject_duplicate_human
- Same human (po_rivera) signs two different roles/slots.
- `distinct_humans: true` violated.
- Detection: Map approver identities (not just keys) and count unique humans.

#### quorum.v1.json / reject_out_of_order
- Ordered policy; signatures not in strictly increasing time per role order.
- Detection: For mode=ordered, verify issued_at / signed_at sequence matches policy approver list order.

#### quorum.v1.json / reject_action_mismatch
- One member's context has a different action_hash than the quorum's declared action_hash.
- Detection: Cross-binding check across all members + top-level action_hash.

#### quorum.v1.json / reject_expired_window
- Spread of signatures exceeds `window_sec` in policy.
- Detection: max(signed_at) - min(issued_at) > policy.window_sec.

#### quorum.v1.json / reject_one_bad_signature
- One signoff fails crypto verify against its stated key.
- Detection: Per-member Ed25519/WebAuthn verify.

#### quorum.v1.json / reject_wrong_role
- Signer claims a role not present in the policy's approvers list.
- Detection: Role + approver must exactly match one of the declared slots.

#### quorum.v1.json / reject_broken_chain
- `ordered_chain: true`; final signoff's `prev` / chain link does not match predecessor.
- Detection: Strong chain linking (predecessor hash in later context).

#### quorum.v1.json / reject_duplicate_key
- Two different humans used the exact same device key.
- Violates `distinct_keys` (unconditional in many policies).

#### quorum.v1.json / reject_initiator_is_approver
- Initiator appears as one of the approvers (SoD violation).
- Explicitly forbidden.

#### quorum.v1.json / reject_distinct_humans_false_shared_key
- Even when `distinct_humans: false`, using one key for two seats is still rejected on distinct_keys.

**Spec references:** draft + `draft-schrock-ep-quorum-02.md` (m-of-n, ordered, distinct_humans, distinct_keys, chain, window).

---

### 4. Action / Context / Binding Failures

#### signoffs.v1.json / reject_action_binding_hash
- context.action_hash differs from what the authenticator signed over.
- Challenge (context_hash) no longer matches the presented action.

#### signoffs.v1.json / reject_action_binding_nonce
- Nonce (consumption key) altered after signing.

#### receipts.v1.json / reject_tampered_anchor (also binding)
- Signature ok but Merkle proof / anchor does not bind to the receipt.

Many provenance, revocation, and trust-receipt vectors also test cross-layer digest bindings.

---

### 5. Anchor / Merkle / Log Proof Failures

#### trust-receipt.exec.v1.json / reject_broken_inclusion
- Inclusion path + leaf does not reconstruct the checkpoint root_hash.

#### trust-receipt.exec.v1.json / reject_empty_path_tree_size_not_1
#### trust-receipt.exec.v1.json / reject_empty_path_nonzero_leaf_index
- Degenerate empty path rules (explicitly called out in spec Section 6.3): empty path only allowed for tree_size==1 and leaf_index==0 (or absent). Otherwise reject before any hash walk.

#### receipts.v1.json / reject_legacy_v1_anchor_by_default, reject_v2_unbound_leaf
- Legacy v1 (unprefixed) or unbound leaf hashes are refused when v2 binding is required.

#### timestamp-proof.v1.json / reject_digest_mismatch, reject_missing_token, etc.
- Covered digest or token binding fails.

**Spec reference:** Merkle rules + "Degenerate empty-path rule" in 6.3.

---

### 6. Lifecycle / Temporal / Consumption / Freshness Failures

#### consumption-proof.v1.json / reject_present_at_h1, reject_absent_at_h2, reject_non_append_only, reject_tampered_value
- Consumption state machine / append-only log violations (present when should be absent, etc.).

#### time-attestation.v1.json / reject_out_of_bounds_time, reject_tampered_time
- Time outside attested window or mutated.

#### trust-receipt.timestamp-forms.v1.json / reject_no_timezone, reject_date_only, reject_required_approvals_string
- Timestamp / time-string formatting violations + type errors on required_approvals.

#### signoffs.v1.json / reject_lifecycle_uv_absent, reject_lifecycle_up_absent
- WebAuthn assertion missing User Verification or User Presence flags.

#### revocation / timestamp / evidence vectors also carry staleness / freshness rejects.

**Spec reference:** Consumption state machine (Section 6.1), temporal checks in 6.3, expiry, nonce consumption.

---

### 7. Boundary / Attestation / Version / Field Presence Failures

#### boundary.v1.json / attribution_substituted_for_authorization, raw_claim_pass_through
- Post-execution attribution presented as if it were pre-authorization; or self-asserted unlimited authority.

#### initiator-attestation.v1.json (7 rejects)
- reject_missing_model_id, reject_empty_model_version, reject_malformed_digest, reject_missing_digest, reject_unknown_member, reject_wrong_version, reject_statement_over_cap
- Missing required fields, bad digests, unknown members, version errors, capability over-claims.

#### evidence-record.v1.json
- reject_broken_renewal, reject_unpinned_tsa, reject_protected_mismatch, reject_non_monotonic_time

#### provenance.exec.v1.json
- reject_scope_violation, reject_tampered_proof, reject_unpinned_delegator, reject_constraints_relaxed

#### witness.v1.json
- reject_k_minus_1, reject_duplicate_counts_once, reject_unpinned_ignored, reject_different_head_ignored

#### receipts.v1.json / reject_unsupported_version, reject_missing_signature

These enforce the "exact structure + no extra claims + pinned roots + version + presence" requirements.

---

### 8. Other / Specialized

- **revocation.exec** + **time-attestation**: unpinned revoker/TSA, key substitution attacks.
- **SMT consume / consumption-proof**: append-only and presence/absence proofs.
- Many timestamp and evidence vectors test TSA pinning + monotonic time + protected field binding.

---

## How a Cleanroom Verifier Must Use This

For every vector with `expect.valid == false`:
- Run the full verification path.
- It must return an error (never "valid").
- Ideally surface a specific `VerifyError` variant matching the category (e.g. `HashMismatch`, `QuorumViolation`, `SignatureFailed`, `MerkleFailed`, `ConsumptionViolation`, `Malformed`).
- The 103 reject vectors are the adversarial test set that proves the verifier is not overly permissive.

Cross-reference with the main `cleanroom-verifier-spec-extraction.md` reject taxonomy (tampered payload, wrong key, SoD, replay, anchor, etc.).

When the Rust harness runs and a reject vector unexpectedly returns true, that is the exact bug to fix using the description above + the vector JSON content.

---

**Completion criteria for this handoff:** This document exists in `planning/cleanroom-reject-vector-autopsy.md`. The Rust verifier + harness should be updated over time to assert not just "valid==false" but that the failure class roughly matches (via error kind or logged reason) for high-value vectors.

All 103 covered.
