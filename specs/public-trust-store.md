---
aft: AI-generated-user-reviewed-pending
agent: Dima (Continuum-meta principal architect, Claude Opus 4.7)
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: spec-rfc-stage
---

# Public Trust Store Specification

**Status:** RFC-STAGE
**Version:** 0.0 (problem statement only)

## Abstract

The Public Trust Store is the proposed attestation layer of the Continuum-meta stack. It provides **cryptographic provenance** for every synthetic output: a signed, append-only record binding each result to the synthetic agent that produced it, and binding that agent to its human accountability chain.

**It is provenance, not prevention.** A signed liar is still a liar. The point is that they are a *named* liar, and the lie is auditable.

## Problem statement

Generative outputs today are unattributed. When a synthetic agent fabricates, the fabrication propagates with no durable link to its originator, no signed record of the input that produced it, and no chain back to a human accountable party. The downstream cost -- wasted human verification, compounding misinformation, legal ambiguity -- is borne by everyone except the agent operator who produced the output.

The Trust Store closes the attribution loop.

## Required behavior (target)

1. **Per-output signature.** Every synthetic output carries a cryptographic signature bound to:
   - The synthetic agent's enrollment key.
   - The hash of the input context that produced it.
   - The timestamp and the GRACE EVIDENCE field of the call.
2. **Human accountability chain.** Each enrollment key is bound to a human (or organizational) registrant. The binding is public.
3. **Append-only registry.** Enrollments and revocations are recorded in a public, tamper-evident log.
4. **Open verification.** Anyone may verify a signature against the registry without permission or fee.

## Design constraints (from substrate whiteboard, 2026-05)

- The whiteboard refers to **"Packing Slip + Hash"** as the deterministic envelope around a query, and to per-agent enrollment as a precondition for participation in the overlay. The Trust Store is where those enrollments live.
- Apache License 2.0: no implementer may be excluded from the verification path.

## Non-goals (explicit and load-bearing)

The Trust Store DOES NOT:

- **Prevent hallucinations.** Models hallucinate; the Store records *who* hallucinated.
- **Adjudicate truth.** The Store records provenance, not correctness.
- **Gatekeep enrollment.** Any party may enroll; revocation is on the registrant's accountability, not on a gatekeeper's preference.

This non-goal list is load-bearing. Earlier framings used "blocks hallucinations" as a marketing claim; that claim is **rejected as overclaim and is not part of this specification**.

## Open problems

1. **Registry technology.** Certificate Transparency-style log, blockchain, federated CA, sigstore-style -- open.
2. **Revocation.** How fast can a compromised key be revoked, and what is the verifier's window of trust?
3. **Key rotation.** Enrollment keys are long-lived; signing keys probably should not be. Rotation protocol unspecified.
4. **Cross-jurisdiction enrollment.** Who registers an autonomous agent operating across borders? Open.
5. **Synthetic-to-synthetic delegation.** If agent A signs an output that agent B then incorporates, what does B's signature attest to? Layered provenance unspecified.
6. **Performance.** Per-output signing at scale is non-trivial. Batched and Merkle-tree approaches unevaluated.

## Status to advance

RFC-STAGE advances to DRAFT when:

- A signature scheme is proposed (algorithm, key sizes, format).
- A registry technology is chosen and justified against the requirement list.
- At least one prototype signs and verifies a non-trivial workload.

The WG explicitly invites **cryptographers, PKI engineers, and supply-chain provenance practitioners** (sigstore, in-toto, SLSA) to lead this track.
