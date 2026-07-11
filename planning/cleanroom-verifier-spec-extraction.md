# Cleanroom Verifier Specification Extraction: EP Authorization Receipts

**Source Spec:** draft-schrock-ep-authorization-receipts-00 (EMILIA Protocol)
**Target Audience:** Rust developer implementing a cleanroom verifier from spec only
**Integrity Constraint:** This document is derived exclusively from the IETF draft text, vector schemas, and RFC references. No reference implementation source was consulted.
**Date:** 2026-07-07
**Purpose:** Provide an exhaustive, implementation-ready blueprint for a Rust crate that verifies EP receipts and conformance vectors without any dependency on existing EMILIA code.

---

## 1. Overview of the Protocol

EP (EMILIA Protocol) authorization receipts provide cryptographic evidence that a named, accountable human (or quorum) approved one exact high-risk action before execution.

Key invariants (machine-checked):
- ConsumeOnce: nonce used at most once globally.
- BindingMatch: signoff only satisfies the exact context/action it signed.
- TerminalIrreversibility: no exit from terminal states (COMMITTED, DENIED, EXPIRED).
- SelfApprovalImpossible: initiator never approves their own action; approvers pairwise distinct.
- NoBypassWrite: COMMITTED only after full verification sequence.

The receipt is offline-verifiable using only:
- The receipt JSON
- Approver public key(s) (pinned or via directory inclusion proof)
- Trusted log checkpoint (signed tree head)

No live calls to operators or logs required for core verification.

---

## 2. Core Data Structures

### 2.1 Action Object

Minimal JSON (exact fields may vary by action_type, but these are normative):

```json
{
  "ep_version": "1.0",
  "action_type": "string (e.g. \"wire.release\", \"grid.curtailment\")",
  "target": { "system": "string", "resource": "string" },
  "parameters": { "object": "any serializable params" },
  "initiator": "ep:entity:...",
  "policy_id": "ep:policy:...",
  "requested_at": "RFC3339 timestamp"
}
```

**Action Hash:**
- Serialize the Action Object using RFC 8785 JCS (JSON Canonicalization Scheme).
- SHA-256 digest of the canonical bytes.
- Verifier MUST recompute and compare; mismatch → reject.

Sensitive values MAY be replaced by salted hashes (e.g. "beneficiary_account_hash": "sha256:...") provided the executor can recompute them for binding.

### 2.2 Authorization Context

One per required approver:

```json
{
  "ep_version": "1.0",
  "context_type": "ep.signoff.v1",
  "action_hash": "sha256:...",
  "policy_id": "ep:policy:...",
  "policy_hash": "sha256:...",
  "initiator": "ep:entity:...",
  "approver": "ep:approver:...",
  "approver_index": "integer",
  "required_approvals": "integer (for m-of-n)",
  "nonce": "base64url (≥128 bits CSPRNG, globally unique)",
  "issued_at": "RFC3339",
  "expires_at": "RFC3339",
  "prev_receipt_hash": "sha256:..."
}
```

**Rules:**
- `nonce` is the consumption key (G3).
- `policy_hash` commits to exact evaluated policy version.
- `prev_receipt_hash` chains to issuing log's last receipt.
- Context is JCS-canonicalized → context_hash = SHA-256(canonical bytes).
- Approver signs the context_hash (not the full context).

### 2.3 Signoff

```json
{
  "context_hash": "sha256:...",
  "signature": "base64url",
  "key_class": "A" | "B" | "C",
  "approver_key_id": "string",
  "signed_at": "RFC3339",
  "webauthn": { "authenticator_data": "b64u", "client_data_json": "b64u" }  // for Class A
}
```

### 2.4 Trust Receipt (Terminal Artifact)

```json
{
  "receipt_id": "ep:receipt:...",
  "action": { /* full Action Object */ },
  "action_hash": "sha256:...",
  "contexts": [ /* array of Authorization Contexts */ ],
  "signoffs": [ /* array of Signoffs */ ],
  "consumption": {
    "nonce": "b64u",
    "state": "COMMITTED" | "DENIED" | "EXPIRED",
    "committed_at": "RFC3339"
  },
  "log_proof": {
    "leaf_index": "int",
    "inclusion_path": ["sha256:...", ...],
    "checkpoint": {
      "tree_size": "int",
      "root_hash": "sha256:...",
      "log_signature": "b64u",
      "log_key_id": "string"
    }
  },
  "approver_key_proofs": [ /* directory inclusions */ ]
}
```

### 2.5 Merkle Tree Structures

- Leaf hash (v2): SHA-256( 0x00 || canonical_payload )
- Internal pair (v2): SHA-256( 0x01 || left || right )
- Inclusion proof: array of {hash, position: "left" | "right"}

Verification walks from leaf, applying pairs, must equal checkpoint root_hash.

Legacy v1 uses simple sorted concat without prefixes.

### 2.6 Conformance Test Vector

From vectors (example structure inferred from usage):

```json
{
  "id": "string",
  "document": { /* Action or Receipt or other */ },
  "public_key": "base64url SPKI or raw",
  "signature": { "value": "b64u", "algorithm": "Ed25519" | "ES256" },
  "expect": { "valid": true | false }
}
```

For receipts: additional fields for contexts, signoffs, etc.

Vectors cover:
- receipts.v1.json
- signoffs.v1.json
- quorum.v1.json
- revocation.*.json
- time-attestation.v1.json
- trust-receipt.*.json
- provenance.*.json
- evidence-record.v1.json
- canonicalization.v1.json
- boundary.v1.json
- currency.v1.json
- initiator-attestation.v1.json
- consumption-proof.v1.json
- witness.v1.json
- timestamp-proof.v1.json
- aec.json
- eye-set.v1.json
- execution-integrity.v1.json
- jws.json
- (others to reach 161 vectors)

---

## 3. Verification Algorithm (Step-by-Step, Exhaustive)

A cleanroom verifier MUST implement exactly these steps (and no more). All steps are offline.

### 3.1 Common Primitives

1. **JCS Canonicalization (RFC 8785)**
   - MUST use the exact algorithm from RFC 8785.
   - Sort object keys lexicographically (UTF-8 code points).
   - No whitespace.
   - Numbers as integers when safe; reject non-integer floats where integers required.
   - Strings: escape per JSON, no unnecessary escapes.
   - Implement or use a verified JCS library (but verify output against spec test vectors).

2. **Hashing**
   - SHA-256 over bytes.
   - Prefixes for Merkle: 0x00 for leaves, 0x01 for internals (v2).

3. **Ed25519 / ES256 Verification**
   - Use native crypto (ed25519 crate or ring for Rust).
   - For Class A (WebAuthn): validate authenticatorData, clientDataJSON.challenge == context_hash, UV flag set.
   - Signature is over the context_hash (or action_hash for some cases).

### 3.2 Receipt Verification Flow

For a presented Trust Receipt:

1. **Action Binding**
   - Canonicalize the embedded `action` object.
   - Compute SHA-256 → must equal `action_hash`.
   - Reject if mismatch.

2. **Context Validation (per context)**
   - Canonicalize context.
   - Compute context_hash.
   - Must commit to the action_hash and policy_hash.
   - `approver != initiator`.
   - `approver_index` unique across contexts.
   - `required_approvals` satisfied by count of valid signoffs.
   - `nonce` unique (track globally in consuming system).
   - `issued_at` <= `signed_at` <= `committed_at` <= `expires_at`.
   - `prev_receipt_hash` (if present) chains correctly.

3. **Signoff Verification (per signoff)**
   - Verify signature over `context_hash`.
   - Lookup approver_key by `approver_key_id` or via `approver_key_proofs`.
   - Validate key validity window contains `issued_at`.
   - For Class A: full WebAuthn validation (challenge, UV, authenticatorData).
   - Reject on any signature failure or key mismatch.

4. **Policy / Quorum / SoD**
   - Count distinct valid approvers >= `required_approvals`.
   - All approvers distinct from each other and from initiator.
   - For m-of-n: verify ordering if policy requires (see Section 7).
   - Policy hash must match evaluated policy.

5. **Merkle / Log Proof**
   - Recompute leaf hash from receipt (using inclusion of the receipt leaf).
   - Walk the `inclusion_path` using the pair function.
   - Must equal `checkpoint.root_hash`.
   - Verify `checkpoint.log_signature` over the checkpoint using trusted log public key.
   - Verify `approver_key_proofs` against trusted directory root or pinned keys.

6. **Consumption**
   - The `consumption.nonce` must not have been seen before.
   - State must be "COMMITTED".
   - Reject on replay, expiry, or wrong state.

### 3.3 Vector Verification

For a test vector JSON:
- If it contains "document" + "public_key" + "signature":
  - If document looks like receipt/action/context: run full receipt or context verification.
  - Compute validity.
- Compare computed .valid against `expect.valid`.
- For Merkle-specific vectors: test verifyMerkleAnchor independently.
- For canonicalization vectors: test JCS output exactly matches expected canonical form.

All 161 vectors must pass for conformance.

---

## 4. Reject Conditions (Exhaustive List)

Implementations MUST produce specific error/reject reasons. Here are the classes:

### 4.1 Tampered / Binding Failures
- Action hash mismatch (tampered Action Object).
- Context does not bind to action_hash or policy_hash.
- Signature does not verify over context_hash (wrong key, bad sig, malformed).
- Merkle proof does not reach checkpoint root.
- Checkpoint signature invalid or from untrusted log key.
- `prev_receipt_hash` does not chain to prior leaf.

### 4.2 Policy / Quorum / SoD Violations
- Fewer than `required_approvals` valid distinct signoffs.
- Initiator appears as approver.
- Duplicate approvers in m-of-n.
- Quorum ordering violation (if policy enforces sequence).
- Policy hash does not match any known/evaluated policy at `issued_at`.

### 4.3 Temporal / Consumption
- `issued_at` > `signed_at` or `committed_at` > `expires_at`.
- Nonce already consumed (replay).
- Receipt presented after `expires_at`.
- Nonce not unique globally.

### 4.4 Key / Directory
- Approver key not found or not valid at `issued_at`.
- Key class not permitted for this policy.
- Directory inclusion proof fails for the approver key.
- WebAuthn validation fails (wrong challenge, no UV, bad authenticatorData, etc.).

### 4.5 Malformed Data
- Missing required fields.
- Invalid base64url, sha256 prefixes, timestamps.
- JCS canonicalization fails (non-canonicalizable values).
- Version mismatch (ep_version not supported).
- Nonce too short (<128 bits entropy).

### 4.6 Confused-Deputy / Cross-Binding
- Receipt authorizes a different action than presented to executor.
- Digest bindings between layers broken (e.g. action_hash in receipt != action_hash in context).

---

## 5. Edge Cases & Spec Ambiguities to Resolve in Implementation

- **Policy Hash vs Policy ID**: Signature binds to `policy_hash`, not just ID. Implementations must store/evaluate the exact hashed policy.
- **Class C (Operator-custodied)**: Must be labeled and treated as lower assurance. New code SHOULD avoid.
- **Directory Authority**: If operator-operated, MUST have second-party attestation in `approver_key_proofs`. Verifier MUST downgrade if only operator-signed.
- **Offline vs Freshness**: Offline verification does NOT prove current validity (post-issuance revocation or log split-view). Must be documented clearly.
- **m-of-n Ordering**: Policies MAY require ordered approvals. Verify `approver_index` sequence if present.
- **Salted Hashes in Action**: Executor must be able to recompute for binding verification.
- **WebAuthn Challenge**: Must exactly equal context_hash (no extra encoding).
- **Log Checkpoint Inside Receipt**: Enables offline inclusion proof. Verifier walks inclusion_path.
- **Nonce Global Uniqueness**: Consuming system (verifying executor) is responsible for tracking seen nonces. Protocol assumes it.
- **JCS Edge Cases**: Numbers, unicode in keys, empty objects/arrays, duplicate keys (must reject).
- **Versioning**: "ep_version": "1.0" — reject unknown versions.
- **Quorum in Signoffs**: For hard cuts, require EP-QUORUM structure (m-of-n distinct humans).
- **Consumption State Machine**: Explicit states; only COMMITTED allows execution.

---

## 6. Conformance Test Vector Handling

- Vectors are per-suite JSON arrays of objects with "id", "document" or equivalent, "public_key", "signature", "expect": {"valid": bool}.
- Run verifier on each; result must match expect.valid.
- Separate vectors test sub-components (Merkle, JCS, WebAuthn, quorum, revocation, etc.).
- Total 161 vectors across ~20 suites (receipts, signoffs, quorum, revocation, time-attestation, trust-receipt, provenance, evidence-record, canonicalization, boundary, currency, initiator-attestation, consumption-proof, witness, timestamp-proof, aec, eye-set, execution-integrity, jws, etc.).

---

## 7. Rust Implementation Notes / Scaffolding Hints

(For the Rust developer)

**Crate Structure Suggestion:**
```
cleanroom-ep-verifier/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── jcs.rs          // RFC 8785 implementation or wrapper + tests
│   ├── hashes.rs       // sha256, leaf_hash, pair_hash, merkle_verify
│   ├── ed25519.rs      // signature verification + WebAuthn parsing
│   ├── receipt.rs      // structs + verify_receipt()
│   ├── context.rs
│   ├── signoff.rs
│   ├── merkle.rs
│   ├── consumption.rs
│   ├── policy.rs       // m-of-n, SoD
│   ├── error.rs        // exhaustive Reject reasons
│   └── vector.rs       // conformance vector runner
├── tests/
│   └── conformance.rs  // loads all 161 vectors, asserts
└── benches/
```

**Dependencies (zero external crypto if possible, or vetted):**
- `serde`, `serde_json` for parsing (with strict mode).
- `sha2` or ring for SHA-256.
- `ed25519-dalek` or ring for Ed25519.
- `base64` (url-safe, no padding).
- For WebAuthn: parse authenticatorData / clientDataJSON manually or minimal crate.

**Strict Requirements:**
- Every reject must map to a specific error variant.
- No panics on malformed input; all paths return Err.
- JCS must be bit-for-bit identical to spec test vectors.
- Test against the 161 vectors from emilia-protocol/conformance/vectors/.

**Key Algorithms to Implement First:**
1. JCS canonicalize (test against canonicalization.v1.json vectors).
2. Action hash + context hash computation.
3. Ed25519 verify + WebAuthn checks.
4. Merkle inclusion (both v1/v2).
5. Full receipt validation + consumption tracking (use a simple in-memory seen-nonces set for tests).

---

## 8. References & Further Reading (Spec Only)

- RFC 8785: JSON Canonicalization Scheme (JCS)
- RFC 2119 / 8174: Key words
- WebAuthn spec for Class A
- The source draft-schrock-ep-authorization-receipts-00 for full normative text
- Conformance vectors for schema and edge-case examples

This extraction is the complete blueprint. Implement against this document + the vectors + the RFCs. Any deviation requires justification against the spec text.

---

**End of Extraction Document**

<!-- AGENT-SIGNATURE
agent_id: E-78A3CCE1-1846-001
thumbprint: MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI=
role: Grok-Build (Grok 4.3 Build TUI)
date: 2026-07-07
-->
