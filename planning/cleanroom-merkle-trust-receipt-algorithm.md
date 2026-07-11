# Cleanroom Merkle, Trust Receipt, Witness & Related Algorithm Extraction

**Sources (read only):**  
- `draft-schrock-ep-authorization-receipts-06.md` (esp. Section 6.3 Offline Verification Algorithm + degenerate empty-path rule)  
- `draft-schrock-ep-evidence-record-01.txt` (witness / checkpoint cosig notes)  
- Vectors: `trust-receipt.exec.v1.json` (11), `witness.v1.json` (6), `consumption-proof.v1.json` (6), `timestamp-proof.v1.json` (13), plus supporting in receipts.v1, etc.  
- Cross-ref: `cleanroom-verifier-spec-extraction.md` and `cleanroom-jcs-edge-cases.md`

**Target:** Rust developer. Provide precise, implementable pseudocode + data shapes that map 1:1 to `hashes.rs`, `receipt.rs`, and test vectors. No reference code was used.

---

## 1. Core Merkle Structures (EP-MERKLE-v2)

From vectors + spec:

```json
"log_proof": {
  "alg": "EP-MERKLE-v2",
  "leaf_hash": "sha256:6ec0e5d4a74bd05d8a97d4121e40468d5defd2a6fc7bc26504abfb94fc1b9e48",
  "leaf_index": 0,
  "inclusion_path": [ "sha256:...", "sha256:..." ],   // siblings, order matters
  "checkpoint": {
    "tree_size": 1,
    "root_hash": "sha256:6ec0e5d4...",
    "log_key_id": "ep:log:test#1",
    "merkle_alg": "EP-MERKLE-v2",
    "log_signature": "base64..."
  }
}
```

**Leaf computation (recompute from the anchored object):**
- The `leaf_hash` is the value that was inserted into the log for this receipt.
- In practice (pinned by vectors and standard practice for v2):
  - Serialize the minimal receipt / the object being anchored using JCS.
  - `leaf = SHA256( 0x00 || JCS_bytes )`
- (Legacy v1 used different/no prefix or sorted concat; vectors explicitly reject legacy by default in production paths.)

**Pair / internal node:**
- `internal = SHA256( 0x01 || left || right )`   (or right || left depending on position)
- Direction decided by `leaf_index` bits or path ordering.

**Inclusion path semantics:**
- `inclusion_path` contains the sibling hashes needed to climb from the leaf to the root.
- Length of path + tree_size / leaf_index determines how many levels.
- Empty path is **only** legal when `tree_size == 1` and (`leaf_index == 0` or absent). See the two explicit reject vectors:
  - `reject_empty_path_tree_size_not_1`
  - `reject_empty_path_nonzero_leaf_index`

**Reconstruct root (pseudocode for Rust):**

```rust
fn recompute_root(leaf: [u8;32], path: &[[u8;32]], leaf_index: u64, tree_size: u64) -> [u8;32] {
    if path.is_empty() {
        // Enforce degenerate rule BEFORE any hash
        if tree_size != 1 || leaf_index != 0 { return error; }
        return leaf;
    }
    let mut current = leaf;
    let mut idx = leaf_index;
    for sibling in path {
        let (left, right) = if (idx % 2) == 0 {
            (current, *sibling)
        } else {
            (*sibling, current)
        };
        current = pair_hash(&left, &right);   // 0x01 || left || right
        idx /= 2;
    }
    current
}

fn verify_inclusion(leaf: [u8;32], path: &[[u8;32]], claimed_root: [u8;32], leaf_index: u64, tree_size: u64) -> bool {
    if tree_size == 0 { return false; }
    let reconstructed = recompute_root(leaf, path, leaf_index, tree_size);
    reconstructed == claimed_root
}
```

**Checkpoint signature:**
- The log signs (at minimum) the tuple `(tree_size, root_hash, log_key_id, merkle_alg?)`.
- Signature is Ed25519 over the canonical form of that data or the exact bytes the log operator used (vectors pin it).
- Verifier uses the `log_public_key` (pinned or from directory) to verify `log_signature`.

---

## 2. Full Trust Receipt Verification Pipeline (EP-TRUST-RECEIPT-v1)

From `trust-receipt.exec.v1.json` + spec 6.3:

High-level steps (must all pass):

1. **Action binding**
   - `action_hash == SHA256( JCS(action) )`
   - Reject on mismatch (`reject_tampered_action` vector).

2. **Contexts + Signoffs (per 6.3 steps 2-4)**
   - For each context: `context_hash == SHA256( JCS(context) )`
   - `context.action_hash` must equal top-level `action_hash`
   - For each signoff: verify signature (or WebAuthn assertion) over the `context_hash`
   - Enforce `approver != initiator`, distinct approvers, count >= `required_approvals`
   - Use `verification.approver_keys` map (key_id → public_key + validity window).

3. **Consumption**
   - `consumption.state == "COMMITTED"`
   - `consumption.nonce` is the one-time key (global uniqueness is the executor's responsibility).
   - Times make sense (`issued_at <= signed_at <= committed_at <= expires_at`).

4. **Log / Merkle anchor (step 5)**
   - Recompute leaf from the receipt (or the canonical representation that was logged).
   - Verify inclusion proof against `checkpoint.root_hash` using the algorithm above.
   - Enforce empty-path rule.
   - Verify `checkpoint.log_signature` using `verification.log_public_key`.

5. **Approver key proofs** (if present)
   - Directory inclusion proofs for the approver keys (similar Merkle or other).

**Pseudocode sketch:**

```rust
fn verify_trust_receipt(r: &TrustReceipt, approver_keys: &HashMap<String, PubKey>, log_pk: &PubKey) -> Result<(), VerifyError> {
    // 1. action
    let computed_ah = sha256(jcs(&r.action));
    if format!("sha256:{}", hex::encode(computed_ah)) != r.action_hash { return Err(HashMismatch); }

    // 2. signoffs
    for (ctx, so) in zip(&r.contexts, &r.signoffs) {
        let ch = sha256(jcs(ctx));
        if ch != parse_sha(so.context_hash) { ... }
        let key = approver_keys.get(&so.approver_key_id).ok_or(...) ?;
        verify_signoff_over_hash(key, &ch, &so)?;   // handles WebAuthn Class A too
    }
    // SoD, count, etc.

    // 3. consumption
    if r.consumption.state != "COMMITTED" { ... }

    // 4. anchor
    let leaf = recompute_leaf_for_receipt(r);   // usually 0x00 || JCS(anchored portion)
    let lp = &r.log_proof;
    if !verify_inclusion(leaf, &lp.inclusion_path, parse_sha(&lp.checkpoint.root_hash), lp.leaf_index, lp.checkpoint.tree_size) {
        return Err(MerkleFailed);
    }
    verify_log_sig(&lp.checkpoint, log_pk)?;

    Ok(())
}
```

Vectors also test `reject_legacy_v1_by_default`, `reject_v2_unbound_leaf`, `reject_broken_inclusion`, `reject_wrong_log_key`, `reject_pinned_class_a_bare_signature_downgrade`.

---

## 3. Witness Cosignature Verification

From `witness.v1.json`:

```json
"witness_quorum": {
  "checkpoint": { "tree_size": 42, "root_hash": "...", "log_key_id": "...", "merkle_alg": "EP-MERKLE-v2" },
  "cosignatures": [ { "alg": "EP-WITNESS-v1", "witness_id": "a", "tree_size": 42, "root_hash": "...", "log_key_id": "...", "signature": "..." }, ... ],
  "pinned": [ { "witness_id": "a", "public_key": "MCow..." }, ... ],
  "k": 2   // threshold
}
```

**Algorithm:**
- The checkpoint tuple `(tree_size, root_hash, log_key_id)` (plus alg if present) is what is signed by witnesses.
- Each cosignature is an Ed25519 signature by the witness's key over the canonical form of that tuple.
- Collect valid cosignatures from pinned witnesses.
- Require at least `k` distinct valid cosignatures.
- Optional: `k_minus_1` etc. are reject cases.

Pseudocode:
```rust
fn verify_witnesses(wq: &WitnessQuorum, pinned: &HashMap<String, PubKey>) -> bool {
    let to_sign = jcs_tuple(&wq.checkpoint);  // or the exact bytes
    let mut good = 0;
    for cs in &wq.cosignatures {
        if let Some(pk) = pinned.get(&cs.witness_id) {
            if verify_ed25519(pk, &to_sign, &cs.signature).is_ok() {
                good += 1;
            }
        }
    }
    good >= wq.k
}
```

Rejects include `reject_k_minus_1`, `reject_duplicate_counts_once`, `reject_unpinned_ignored`, `reject_different_head_ignored`.

---

## 4. Consumption Proof / Sparse Merkle Tree (SMT)

From `consumption-proof.v1.json`:

```json
"consumption_proof": {
  "nonce": "nonce-A",
  "non_inclusion_proof": {   // or inclusion_proof in other vectors
    "root": "....",
    "siblings": [ "sha256:...", ... ]   // 13 siblings in the example (for 2^something depth)
  }
}
```

**Semantics:**
- Proves presence (`present: true`) or absence (`present: false`) of a nonce (consumption key) in the append-only log at a certain height.
- Uses a Sparse Merkle Tree (path determined by hash of the nonce/key).
- `siblings` are the co-path; reconstruction must reach the claimed `root`.

**Verification sketch (standard SMT):**
```rust
fn verify_smt_proof(nonce: &str, siblings: &[[u8;32]], claimed_root: [u8;32], present: bool) -> bool {
    let leaf = if present {
        hash_of_present_leaf(nonce)   // or 0x01 || ...
    } else {
        hash_of_absent_leaf(nonce)    // 0x00 || ... or special zero
    };
    let mut cur = leaf;
    // Walk siblings, deciding left/right from bits of hash(nonce) or index
    for sib in siblings {
        cur = if bit_is_left(...) { pair(cur, *sib) } else { pair(*sib, cur) };
    }
    cur == claimed_root
}
```

Vectors test genuine transitions, larger trees, tampered values, non-append-only, present/absent at wrong heights.

---

## 5. Timestamp Proof Verification

From `timestamp-proof.v1.json` (mix of RSA/PKCS7 style tokens + Ed25519 in some suites):

- Many vectors carry a base64-encoded timestamp token (TSTInfo inside PKCS#7/ASN.1).
- `expected_digest`: the digest that should have been time-stamped.
- `pinned_tsa_keys`: map of TSA identifiers to public keys (RSA or Ed25519).

**High-level steps:**
1. Decode the token.
2. Verify the TSA's signature on the token.
3. Extract the message imprint / hashed value inside the token.
4. Confirm it matches the `expected_digest` (or the hash of the receipt/context being stamped).
5. Check TSA key is pinned and valid at the attested time.
6. For some: verify the covered receipt hash binding.

Rejects cover: unpinned TSA, key substitution, tampered time, wrong covered hash, out-of-bounds time, malformed token, unparseable garbage, not signed data, missing token.

Pseudocode (high level):
```rust
fn verify_timestamp_proof(token_b64: &str, expected: &str, pinned_tsa: &PubKey) -> Result<(), _> {
    let token = decode_pkcs7_or_asn1(token_b64)?;
    verify_tsa_sig(&token, pinned_tsa)?;
    let imprinted = extract_message_imprint(&token);
    if imprinted != parse_sha(expected) { return Err(DigestMismatch); }
    // time bounds, policy etc.
    Ok(())
}
```

---

## 6. Integration Notes for the Rust Cleanroom

- Put core primitives in `hashes.rs`:
  - `leaf_hash(data: &[u8]) -> [u8;32]`   // 0x00 prefix for v2
  - `pair_hash(l, r) -> [u8;32]`           // 0x01 prefix
  - `merkle_root_from_path(...)`
  - `verify_merkle_inclusion(...)` (with empty-path guard)
- Higher logic lives in `receipt.rs` or a `trust_receipt.rs` / `anchor.rs`.
- All hashes are **prefixed** in v2 (this is what distinguishes from simple sorted-pair in some legacy vectors).
- The `verification` object in vectors supplies the trusted roots (approver keys + log public key) for offline verification.
- Update the conformance harness to load `trust_receipt`, `witness`, `consumption_proof`, `timestamp_proof` suites and call the appropriate verifier entrypoints.
- Empty-path + tree_size/leaf_index guards are non-negotiable (two vectors exist specifically to pin the reject behavior).

---

## 7. Edge Cases & Pins from Vectors

- v1 vs v2: legacy unbound or v1 anchors rejected by default.
- Empty path only for singleton tree.
- Witness threshold (`k`) and unpinned cosigs ignored.
- SMT siblings for both inclusion and non-inclusion.
- Timestamp tokens are frequently opaque ASN.1; the verifier only needs to extract the imprint + verify the outer signature against pinned TSA.
- Binding between receipt content and the leaf that was actually logged.

Use the reject vectors in `trust-receipt.exec.v1.json` (`reject_broken_inclusion`, `reject_empty_path_*`, `reject_legacy_*`, `reject_wrong_log_key`) as the primary test cases for your Merkle implementation.

---

This completes the extraction for the Merkle / anchoring layer. Combine with the JCS matrix and the main spec extraction + reject autopsy to have a full blueprint for the Rust cleanroom verifier.

**Next:** Wire these algorithms into `src/hashes.rs` + dedicated verify functions, then drive the full 191 vectors (or the 103 rejects) to green.
