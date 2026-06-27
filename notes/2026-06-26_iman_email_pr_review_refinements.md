---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-06-26
file_role: email_log
author_agent: Gemini-in-body on AntiGravity Substrate
author_unrp_id: E-2A0F1954-1845-001
author_thumbprint: 16E2D7AFBFA6CE09
status: ARCHIVED
---

# Email Log: PR Review, Alignment, and Freshness Binding

* **Date:** June 26, 2026 (morning, pre-meeting check-in)
* **From:** Iman Schrock <team@emiliaprotocol.ai>
* **To:** Justin Kintzele <jkintzele@jdieselny.com>
* **Subject:** PR Review, Alignment, and Freshness Binding

## Email Content

> [!NOTE]
> This is a transcription of the email received from Iman Schrock on June 26, 2026, reviewing the pull request refinements, paper taxonomy, and freshness binding candidates.

Justin —

Read the whole PR in context, not just Note 06. All four refinements landed exactly right:

* Expiry is the EP validity window (expires_at = issued_at + duration), duration_seconds as the human-readable echo. Correct.
* grid.curtailment expressed as a PIP-013 authorization_scope (target_set / effect_class / magnitude / window) — one receipt model across both layers, spec stays EMILIA-owned. That's the whole game; you nailed it.
* Named human + quorum in the signoff, not the parameters, with EP-QUORUM required for hard cuts. Right separation: parameters say WHAT, the signoff proves WHO.
* priority_marker = sha256(canonical_receipt). Unforgeable by construction, no new crypto, no parallel verification surface. Clean.

And the governance decision is locked my way: receipt fields ride the EMILIA receipts draft + PIP-013 as the EP action-type profile, COSA references them. One model, both layers share. We lock it on the whiteboard Friday.

On the prototype — I checked it imports `from emilia_verify import verify_receipt, canonicalize`, the real published verifier, over JCS/RFC-8785 canonical bytes. That's the proof point: it verifies VALID with zero new code because it's a conformant EP receipt, not a look-alike. The dual-key separation (L5 authenticity key vs L7 approver key) is load-bearing and you scoped it honestly — tamper-on-content fails at L5, confused-deputy and replay fail at L7. We run it live Friday; I'll have a verifier open on my side so the room sees two independent checkers agree.

Two things you undersold in your note that I want on the record:

1. Paper 04 maps the work against the IETF taxonomy drafts and routes the Accountability / Policy-Enforcement cell to the EMILIA-composed gate. That's EMILIA named as the mechanism inside an IETF-facing alignment paper — exactly the positioning we want going into the room.

2. PR #2 (cosa-ep-l7-binding, the 900s freshness window) is the same primitive EP already carries normatively: the receipts draft has an agent_binding section — agent_id plus a delegation block (scheme, ref, hash, observed_at) that the verifier evaluates fail-closed on freshness. So your freshness prototype and the EP draft are converging on one artifact. If you align your window field to the draft's delegation.observed_at, the L4→L7 binding becomes literally the same object on both sides rather than two parallel ideas. Worth ten minutes Friday.

One small boundary note for Friday so the two-layer story stays crisp: in Paper 04 §2.2, keep Truth Root's "human-accountability links" scoped to agent authorship/enrollment provenance. The accountability of a named human to a specific action is the L7 receipt — which §2.4 already routes correctly. Just want the line between the two to read cleanly to a skeptical reviewer.

This is genuinely strong work. Looking forward to locking it on the whiteboard Friday.

Iman
team@emiliaprotocol.ai
