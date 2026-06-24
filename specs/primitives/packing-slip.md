---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: primitive-draft
---

# Packing Slip + Hash

**Status:** DRAFT
**Version:** 0.1
**Source:** Continuum-meta design whiteboard (2026-05, as transcribed by synthesis seat)

## Abstract

The Packing Slip is the **ingress envelope** produced by a synthetic agent before any query leaves the local node. It packages the raw human input (the "vomitprompt") together with the local context the synth holds at the moment of the call, and seals the bundle with a cryptographic hash.

It is the first artifact in the Continuum-meta data flow. Everything downstream -- the [Bill of Lading](bill-of-lading.md), AIR routing, COGSTOR cache lookup -- operates on the Packing Slip.

## Position in the data flow

```
[HUMAN] -> [SYNTH (S/T + L/T memory)] -> [PACKING SLIP + HASH] -> [BILL OF LADING] -> [CONTINUUM-META]
```

## Required fields

A conformant Packing Slip MUST contain:

- **`raw_input`** -- the human input, unmodified.
- **`synth_id`** -- identifier of the synthetic agent producing the slip.
- **`short_term_context`** -- current session state the synth is holding (subset of COGSTOR `working_memory` and `active_items`).
- **`long_term_context_refs`** -- pointers (not contents) to COGSTOR objects the synth believes are relevant. Resolution is the overlay's job.
- **`grace_fields`** -- at minimum the GOAL and CONSTRAINTS the synth has bound to this call. ROUTING and ANCHOR may be filled by AIR.
- **`timestamp`** -- when the slip was sealed.
- **`hash`** -- cryptographic checksum over all of the above. Algorithm unspecified at v0.1; SHA-256 recommended for prototypes.

## Required properties

- **Local construction.** The Packing Slip is built on the synth node before any network call. It is the artifact that converts unstructured ingress into a structured handoff.
- **Deterministic hashing.** Identical input + identical context + identical synth state MUST produce identical hashes. This is what makes overlay cache lookup tractable.
- **Tamper evidence.** Modification of any field after sealing invalidates the hash.

## Non-goals

- **The Packing Slip is not signed.** Signing happens at the Bill of Lading layer, which wraps the Packing Slip with sender and transport identity.
- **The Packing Slip is not transmitted alone.** It is always wrapped in a Bill of Lading for transport.

## Open problems

1. **Hash algorithm and length.** SHA-256 recommended for prototypes; long-term choice (e.g., quantum-resistant) open.
2. **Long-term context inclusion threshold.** How does the synth decide which long-term context refs to include? Pure relevance scoring, recency, operator-pinned, or learned? Open.
3. **Context contamination.** A synth handling multiple users in one process must not leak one user's context into another's Packing Slip. Isolation mechanism unspecified.
4. **Privacy boundary.** A Packing Slip contains raw human input -- the most sensitive payload in the system. Encryption-at-construction (vs. at-Bill-of-Lading-wrap) is open.

## Status to advance

DRAFT advances to STABLE when:

- Wire format is specified (JSON / CBOR / protobuf).
- Hash algorithm is specified and justified.
- At least two independent implementations produce identical hashes for identical inputs.

## Relation to other specs

- Wrapped by: [Bill of Lading](bill-of-lading.md)
- Resolved against: [COGSTOR](../cogstor.md)
- Constrained by: [GRACE Contract](../grace-contract.md)
- Routed by: [AIR Protocol](../air-protocol.md)
