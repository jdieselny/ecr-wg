---
review_state: AI-generated-user-reviewed-pending
agent: SEV (continuum-SEV seat, Claude Opus 4.8, via Claude Code)
registrant: Justin Kintzele
generated_at: 2026-05-31
file_role: reference
status: DRAFT // decisions gated 2026-05-31
---

# Trust Layer Terminology

**Purpose.** Define the trust and truth vocabulary for the attestation layer. These terms are integral to the stack and were previously undocumented or duplicated. Operator decisions of 2026-05-31 are applied below.

This document is reference, not normative behavior. Normative behavior lives in the layer spec it points to.

---

## Resolved decisions (2026-05-31)

1. **"Public Trust Store" and "Truth Store" are both retired.** The attestation layer is named **Truth Root**. (The spec file `specs/public-trust-store.md` and all cross-references migrate to `specs/truth-root.md`; migration checklist at the bottom.)
2. **AFT public expansion is "Attested Fact Trace."** The acronym AFT is retained throughout. The informal working expansion is retained by the team and is not for publication.
3. **The frontmatter `aft:` field is renamed `review_state:`** so the AFT token belongs to the protocol alone. (Repo-wide frontmatter migration pending.)

---

## Definitions

### Truth Root
The name of the attestation layer, and the anchor concept it is rooted in. A provenance chain is **grounded** when it terminates at an accountable **registrant** (below). Operationally, reality is what chains to a registrant in the Truth Root. Truth Root is both the cryptographic root of trust and the accountability root.

### Registrant
The accountable human or organization a provenance chain terminates at. Singular per party. An agent enrolls under a registrant. This is the **anchor**; "Truth Root" is the layer that holds and verifies anchors. (Naming the anchor "registrant" rather than also calling it "Truth Root" is deliberate: it removes the anchor-versus-layer ambiguity.)

### AFT (Attested Fact Trace)
The per-output attestation record, and the property that a claim carries one. An AFT record binds:
- the claim or output hash,
- the producing agent's enrollment key,
- the input-context hash,
- the timestamp and the GRACE EVIDENCE field.

An **AFT check** walks an AFT record back to its registrant through the Truth Root. A claim that cannot be walked to a registrant is ungrounded: rumor, stale, or hallucinated.

---

## Relationships (one breath)

An agent enrolls under a **registrant**. Its outputs carry **AFT** records. AFT records are recorded in and verifiable against the **Truth Root**. An AFT check grounds a claim by chaining its AFT record back to a registrant.

---

## Crosswalk

**To the SO (Spiritual Offerings) channel** (`DatacomWorkspace/relay`), the human-scale v0 prototype:

| ecr-wg term | SO equivalent |
|---|---|
| Truth Root (layer) | signed-commit history + public key registry |
| Registrant | operator key / root of trust |
| AFT record | SO attestation block (operator, synth, substrate, lineage, sig) |

---

## Migration checklist (pending git-init + operator go)

- [ ] Rename `specs/public-trust-store.md` to `specs/truth-root.md`; update its title and abstract.
- [ ] Update cross-references in `ARCHITECTURE.md`, `specs/primitives/bill-of-lading.md`, `specs/air-protocol.md`, and `README.md`.
- [ ] Replace "Public Trust Store" / "Trust Store" usages with "Truth Root" repo-wide.
- [ ] Set AFT expansion to "Attested Fact Trace" wherever AFT is first expanded.
- [ ] Rename frontmatter `aft:` to `review_state:` across all files.
- [ ] Define or remove `UNRP` (referenced in README and the agent anchor, currently undefined).
