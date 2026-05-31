---
aft: AI-generated-user-reviewed-pending
agent: Dima (Continuum-meta principal architect, Claude Opus 4.7)
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: governance
---

# ECR-WG Charter

## Purpose

Standardize the cognitive networking primitives -- execution contract, storage, routing, and provenance -- required to operate generative compute at planetary scale without the compounding prefill, energy, and trust costs of **Stateless Redundant Execution (SRE)**.

## Scope

**In scope:**
- The four standards tracks in [ARCHITECTURE.md](ARCHITECTURE.md): GRACE, COGSTOR, AIR Protocol, Truth Root.
- Reference implementations and conformance benchmarks.
- Evidence: replication and extension of NETL energy measurements across providers, scenarios, and seats.

**Out of scope:**
- Model training, fine-tuning, RLHF.
- Application-layer agent frameworks.
- Provider-specific optimization that is not portable.

## Decision process

**Running code and rough consensus** (after RFC 7282).

- A proposal advances when a working implementation exists and no sustained objection is on the record.
- A **sustained objection** is a stated technical concern, in writing, that the proposal author has not refuted.
- The chair calls consensus. Disputed calls go to a 7-day public comment window, then the chair calls again.

## Membership

Open. To participate:

1. Read the relevant spec.
2. Open an issue or PR.
3. Sign your commits.

There is no application. There are no fees. There is no gatekeeping body other than the technical merit of contributions.

## Initial chair

Justin Kintzele (acting), pending first WG meeting.

## Working seats

The ECR-WG is currently operated as a three-node loop:

- **Direction** -- operator authority over scope and brand.
- **Architecture** -- principal architect, system orchestration, integrity gates.
- **Synthesis** -- context translation between human, architect, and external audiences.

This is a working configuration, not a governance structure. Any of the three roles may be filled by humans or by signed synthetic agents conformant with the Truth Root.

## Anti-capture provisions

- **Apache License 2.0 with patent grant.** Implementers cannot sue contributors over standardized mechanisms.
- **All specs versioned and signed.** A vendor cannot quietly fork the standard and call it the same name.
- **No paid tiers. No private spec branches. No member-only mailing lists.**
- **Substantive decisions on the public record.** Off-list lobbying does not count toward consensus.

## Amendment

This charter amends by rough consensus, same as any other spec.
