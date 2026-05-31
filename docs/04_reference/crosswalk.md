---
review_state: AI-generated-user-reviewed-pending
agent: SEV (continuum-SEV seat, Claude Opus 4.8, via Claude Code)
registrant: Justin Kintzele
generated_at: 2026-05-31
file_role: reference
status: DRAFT
---

# Crosswalk: SO (Spiritual Offerings) <-> ECR-WG

**Purpose.** SO is a running human-to-human message channel (`DatacomWorkspace/relay`) developed alongside this standard. It is the first concrete v0 instance of several ECR-WG layers, at human scale. This document maps SO's vocabulary and artifacts to ECR-WG terms so neither effort drifts from the other, and so SO can serve as a conformance fixture.

It also resolves the one term that collides.

---

## Term crosswalk

| SO / North Star term | ECR-WG term | Notes |
|---|---|---|
| COGOBJ (compressed intent-diff, the transmitted unit) | **Packing Slip** | SO's COGOBJ maps to the Packing Slip envelope, NOT to the COGSTOR "Cognitive Object". See the collision note below. |
| COGSTOR (per-operator L1 store) | **COGSTOR** | Same name, same role. SO's `archive/` is a degenerate file-based COGSTOR. |
| CP (Cognition Protocol, the transport) | **AIR Protocol** (routing) + the Cognition/Intelligence flow names | SO's transport maps to AIR. ECR-WG splits the path into Cognition (ingress) and Intelligence (return). |
| AFT block (operator, synth, substrate, lineage, sig) | **AFT** (Attested Fact Trace) record + Bill of Lading `sender_signature` / `grace_attestation` | SO's attestation block is the human-scale form of the BoL signature fields. |
| Truth Root (grounding anchor) | **Truth Root** (layer) + **registrant** (anchor) | Aligned after the 2026-05-31 rename. |
| Operator (root of trust) | **registrant** | Aligned. |
| Surface-not-execute; human accept / reject | **GRACE** (CONSTRAINTS, EVIDENCE, operator gate) | SO's `check_inbox` enforces GRACE-like discipline at human scale. |
| Substrate (model + priors + lineage) | (unnamed in ECR-WG) | Candidate addition to ECR-WG terminology. |

### Collision resolved: COGOBJ vs Cognitive Object

"COGOBJ" in the SO lexicon is the **transmitted** unit (a compressed intent-diff). ECR-WG's COGSTOR "Cognitive Object" is the **stored** memory record (`agent_id`, `working_memory`, etc.). These are different objects. Resolution: SO's COGOBJ maps to the **Packing Slip**; the term "Cognitive Object" is reserved for the stored COGSTOR record. Do not use "COGOBJ" for the stored record.

---

## Artifact crosswalk (Relay implementation -> ECR-WG spec)

| Relay artifact | ECR-WG spec | Conformance level |
|---|---|---|
| Signed Relay message file | Bill of Lading wrapping a Packing Slip | v0: message body is the Packing Slip payload; the signed commit stands in for the BoL `sender_signature`. |
| `mailboxes/<op>/inbox` routing | AIR Protocol | v0 **degenerate**: person-addressed, not semantic-anycast. No vector match, no Zero-Match fallback. |
| `check_inbox` ritual (surface, decide, archive) | GRACE + COGSTOR Re-Absorption | Surfaces under GRACE discipline; archiving is a degenerate Re-Absorption. |
| Signed git commit | Truth Root attestation | v0: commit signature substitutes for a per-output enrollment-key signature against a registry. |
| `archive/` directory | COGSTOR | v0 file-based persistence, matching COGSTOR's "graceful degradation to local file I/O". |
| SO SANITIZATION doctrine | COGSTOR open-problem #3, Packing Slip open-problem #4 | SO answers the privacy-boundary open problems at human scale. |

---

## What SO proves, and what it does not

**Proves (running code):** that a GRACE-disciplined, attested, human-gated message channel with persistence works end to end. It is the first instance touching AIR, Truth Root, COGSTOR, and GRACE together, even if degenerately.

**Does not yet implement:** semantic anycast routing, per-output enrollment-key signatures against a real registry, the Truth Root registry itself, or cross-provider serialization. SO is to these specs what the Dima enrollment card is to the Truth Root spec: a dogfood fixture that shows the shape and exposes the gaps.
