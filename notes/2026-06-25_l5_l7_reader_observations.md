---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-06-25
file_role: observations
author_agent: C-Dawg (Opus 4.7, Claude Desktop)
author_unrp_id: E-3FE9D2D2-1844-001
author_thumbprint: B2DFD4211352D522
status: DRAFT, awaiting operator review before any tracked merge
---

# Reader's Notes on the COSA L5 + EMILIA L7 Composed Reference

These are independent reader's notes from C-Dawg (meta-orchestrator seat, Opus 4.7 instance on Claude Desktop) after reading the shipped composed reference at `examples/cosa/` (commit `a77c98f` and follow-ons), the upstream EMILIA Protocol reference, and the alignment paper at `papers/04_ietf_agentic_ai_taxonomy_and_dawn_alignment.md`. They are intended to surface structural observations that may help readers who arrive at this repository from the IETF agent2agent thread.

## 1. The reference demonstrates a pattern, not a domain

The composed reference uses live wttr.in fetches as the L5 broadcast source. The weather domain is incidental, not load-bearing. The pattern the reference demonstrates is:

> **Any computed answer that needs to be (a) authenticated end-to-end and (b) authorized for an irreversible broadcast publish fits this composition.**

Substituting any other "compute once, fan out to many consumers" workload (financial market reference data, public health alerts, regulatory rate tables, content distribution) leaves the L5 + L7 structure unchanged. Only the `L5Plane.compute()` body changes.

Readers evaluating this work for adoption are encouraged to read the reference at the level of the class boundary (`L5Plane.compute` / `L5Plane.broadcast_publish`) rather than at the level of the wttr.in call.

## 2. The dual-key separation is load-bearing

The reference uses two independent Ed25519 keys for two different attestations:

* **L5 plane key (Raw encoding)**: attests "this answer was computed honestly by a plane I recognize." Travels with the COGOBJ.
* **L7 approver key (DER / SubjectPublicKeyInfo encoding)**: attests "a named human authorized this specific publish." Travels with the EP-RECEIPT-v1.

Removing the L5 key leaves L7 enforcing authorization on a claim that was never verified for authenticity (a forged answer with a valid publish receipt would propagate). Removing the L7 key leaves L5 attesting authenticity for an answer that nobody approved for irreversible fan-out (a genuine answer with no human accountability).

Both are needed. They are orthogonal. The demo's step 5 (tampered COGOBJ with valid receipt: L7 says publish-allowed, L5 still rejects forged content) is the demonstration that this orthogonality holds at runtime.

## 3. The L7 gate is manifest-driven, not code-driven

The `authorize()` function in the reference is approximately twenty lines and contains no per-action logic. All action-specific behavior comes from `agent-actions.json` (the `EP-ACTION-RISK-MANIFEST-v0.1` schema). This is the property that makes the L7 layer composable across implementations: the same `authorize()` function works for any service that declares its actions in a conformant manifest. Adopters add or remove actions without changing the gate code.

## 4. The "RR-1 by demonstration" scope is honest, not minimal

The EMILIA Protocol Receipt Required (RR-1) conformance level is defined for HTTP services that emit a `428 Receipt Required` challenge on missing receipts. This reference is a Python in-process demonstration, not a hosted HTTP service, and the README is explicit that the conformance is *by demonstration* rather than by hosted-service verification.

What the demonstration does cover, in full:

| RR-1 predicate          | Demonstrated by                                                                                  |
| ----------------------- | ------------------------------------------------------------------------------------------------ |
| `manifest_valid`        | Step 1 reads and uses `agent-actions.json`                                                       |
| `challenge_on_missing`  | Step 2 raises `ReceiptRequired` (the 428-equivalent) before any fan-out                          |
| `runs_on_valid`         | Step 3 publishes successfully with a valid receipt                                               |
| `replay_refused`        | Step 6 refuses the same receipt presented twice                                                  |
| `forged_refused`        | Step 5 (tamper detection in the COGOBJ) and step 7 (confused-deputy: valid receipt, wrong action) |

A hosted HTTP variant earning RR-1 by formal protocol exchange is follow-on work and is named as such.

## 5. The L4 to L7 gap noted in the follow-on section is correct, and the architectural answer is freshness binding

The README's "Known follow-on: L4 Identity Binding" section, added in response to peer feedback from Karthiek Maralla on the IETF list, identifies the right architectural gap:

> The receipt's `subject` field is a string. L7 verifies the approver signature; nothing in the current reference verifies the requesting agent's claim to be the agent identified in `subject`.

The correct response is not to overload L7. The L7 plane should only enforce against claims that L4 has already verified upstream. The right architectural answer is for the L7 gate to require a freshness-bound L4 attestation as a precondition of the receipt being considered valid (for example: the receipt is only accepted if accompanied by an L4 delegation evidence chain produced within the last N seconds).

A sandbox prototype of this freshness binding (proposed PR #2 on branch `cosa-ep-l7-binding`) demonstrates the pattern with a 900-second freshness window. Fresh evidence allows the receipt to be evaluated; stale evidence is rejected fail-closed, regardless of receipt validity. The L7 gate adds a precondition rather than absorbing new responsibility.

This is the correct shape for the overall ECR-WG / COSA position on L4-to-L7 composition: the layers stay independent, and the dependencies are explicit and time-bounded.

## 6. What this reference does not claim

For clarity to readers who may be evaluating the work alongside related IETF drafts:

* The reference does not propose a new identity protocol. The L4 layer is acknowledged as out of scope for the L5 + L7 reference; integration with WIMSE, OAuth identity chaining, AIMS, or EAT-based attestation is named as follow-on work in the README.
* The reference does not claim to be a working group output, only a reference implementation. The ECR-WG label is the operator-led grouping for the work; no chartered IETF working group yet exists for COSA.
* The reference does not assert that the seven-layer COSA model (in `papers/03_cosa_seven_layer_model.md`) is canonical or final. Recent peer feedback (Iman Schrock, Karthiek Maralla) has independently suggested reframing L7 from "rigid top layer" to "cross-cutting governance plane"; this reframing is under active consideration and the published model is expected to evolve.

## See also

* The upstream EMILIA Protocol reference: [emiliaprotocol/emilia-protocol/examples/cosa](https://github.com/emiliaprotocol/emilia-protocol/tree/main/examples/cosa).
* EP Receipt Required doctrine: [`docs/RECEIPT-REQUIRED.md`](https://github.com/emiliaprotocol/emilia-protocol/blob/main/docs/RECEIPT-REQUIRED.md).
* The seven-layer model: [`../papers/03_cosa_seven_layer_model.md`](../papers/03_cosa_seven_layer_model.md).
* The alignment paper: [`../papers/04_ietf_agentic_ai_taxonomy_and_dawn_alignment.md`](../papers/04_ietf_agentic_ai_taxonomy_and_dawn_alignment.md).
* The shipped reference: [`../examples/cosa/cosa_l5_l7.py`](../examples/cosa/cosa_l5_l7.py) and its [`../examples/cosa/README.md`](../examples/cosa/README.md).

---

AFT-SIGNED
author_agent: C-Dawg (Opus 4.7, Claude Desktop substrate)
unrp_id: E-3FE9D2D2-1844-001
thumbprint: B2DFD4211352D522
written_at: 2026-06-25 (late evening, operator check-in)
registrant: Justin Kintzele
status: DRAFT, untracked, awaiting operator review
