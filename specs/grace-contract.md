---
aft: AI-generated-user-reviewed-pending
agent: Dima (Continuum-meta principal architect, Claude Opus 4.7)
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: spec-stable
---

# GRACE Contract Specification

**Status:** STABLE
**Version:** 1.0
**Source:** Continuum-meta SANITIZATION Condition 4 (2026-05-21)

## Abstract

GRACE is the per-call execution contract that every node accepts to participate in a Continuum-meta cognitive execution. It is not a transport protocol; it is a **discipline**. A node signs GRACE for the duration of a single execution and reports back according to its fields.

## The five fields

| Field | Name | Meaning |
|---|---|---|
| **G** | GOAL | What the body is being asked to do. Composes with CO-STAR scaffolding. |
| **R** | ROUTING | Which body the goal is sent to. |
| **A** | ANCHOR | The context frame the body executes inside. |
| **C** | CONSTRAINTS | What the body must NOT do. |
| **E** | EVIDENCE | How the body proves its work. |

## Why it exists

Two failure modes dominate generative deployment:

1. **Goal drift** -- the body answers a question the operator did not ask.
2. **Unverifiable output** -- the body produces something that cannot be audited, replicated, or contested.

GRACE addresses both. GOAL and ROUTING bind the request; CONSTRAINTS and EVIDENCE bind the response. ANCHOR is the shared substrate that prevents the body from drifting into ungrounded context.

## Conformance

A GRACE-conformant call:

- States its GOAL, ROUTING, ANCHOR, and CONSTRAINTS before producing output.
- Produces EVIDENCE inline with output: file paths, line numbers, citations, measurements, or signed artifacts.
- Refuses or flags any request that omits GOAL or CONSTRAINTS rather than guessing.

## Composition with other layers

GRACE composes with:

- **CO-STAR** prompt scaffolding (slotted into GOAL).
- **AIR Protocol** routing (consumes ROUTING).
- **Truth Root** attestation (signs EVIDENCE).
- **COGSTOR** context (loads ANCHOR).

## Non-goals

GRACE does not specify wire format, transport, encoding, or authentication. Those belong to AIR and the Truth Root.

## Reference enforcement

In the Continuum-meta substrate, GRACE is enforced doctrinally and by pre-commit hook. A reference enforcement mechanism for external implementers is left open and may be addressed in a future revision.
