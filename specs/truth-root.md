---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: spec-rfc-stage
---

# Truth Root Specification

**Status:** IMPLEMENTED (v1.0)
**Version:** 1.0

## Abstract

The Truth Root is the proposed attestation layer of the Continuum-meta stack. It provides **cryptographic provenance** for every synthetic output: a signed, append-only record binding each result to the synthetic agent that produced it, and binding that agent to its human accountability chain.

**It is provenance, not prevention.** A signed liar is still a liar. The point is that they are a *named* liar, and the lie is auditable.

## Problem statement

Generative outputs today are unattributed. When a synthetic agent fabricates, the fabrication propagates with no durable link to its originator, no signed record of the input that produced it, and no chain back to a human accountable party. The downstream cost -- wasted human verification, compounding misinformation, legal ambiguity -- is borne by everyone except the agent operator who produced the output.

The Truth Root closes the attribution loop.

## Required behavior (target)

1. **Per-output signature.** Every synthetic output carries a cryptographic signature bound to:
   - The synthetic agent's enrollment key.
   - The hash of the input context that produced it.
   - The timestamp and the GRACE EVIDENCE field of the call.
2. **Human accountability chain.** Each enrollment key is bound to a human (or organizational) registrant. The binding is public.
3. **Append-only registry.** Enrollments and revocations are recorded in a public, tamper-evident log.
4. **Open verification.** Anyone may verify a signature against the registry without permission or fee.

## Implementation Details (v1.0)

- **Signature Scheme:** Ed25519 Native Cryptography (via Iman's protocol).
- **Format:** Thumbprints are represented as the SPKI DER Base64 representation of the agent's public key (e.g. `MCowBQYDK2Vw...`).
- **Registry Technology:** Append-only plain-text markdown cards stored in the `enrollments/` directory, anchored by the cryptographic signature of the git commit author (the human registrant).
- **Verification:** Natively verifiable by the COSA external verifier using standard `cryptography.hazmat.primitives.asymmetric.ed25519`.

## Non-goals (explicit and load-bearing)

The Truth Root DOES NOT:

- **Prevent hallucinations.** Models hallucinate; the Store records *who* hallucinated.
- **Adjudicate truth.** The Store records provenance, not correctness.
- **Gatekeep enrollment.** Any party may enroll; revocation is on the registrant's accountability, not on a gatekeeper's preference.

This non-goal list is load-bearing. Earlier framings used "blocks hallucinations" as a marketing claim; that claim is **rejected as overclaim and is not part of this specification**.

## Future Considerations

1. **Revocation.** How fast can a compromised key be revoked, and what is the verifier's window of trust? (Currently managed via git commit revocation).
2. **Key rotation.** Enrollment keys are long-lived; signing keys probably should not be. Rotation protocol unspecified.
3. **Cross-jurisdiction enrollment.** Who registers an autonomous agent operating across borders? Open.
4. **Synthetic-to-synthetic delegation.** If agent A signs an output that agent B then incorporates, what does B's signature attest to? Layered provenance unspecified.
5. **Performance.** Per-output signing at scale is non-trivial. Batched and Merkle-tree approaches unevaluated.
