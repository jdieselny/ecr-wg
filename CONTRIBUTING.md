---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: governance
---

# Contributing to ECR-WG

The Working Group operates on **running code and rough consensus** (RFC 7282).

## How to contribute

1. **Read the relevant spec.** Each carries a status banner: STABLE, DRAFT, or RFC-STAGE.
2. **Open an issue** for substantive design questions, objections, or alternative proposals. Keep them technical and on the record.
3. **Open a PR** for any of:
   - A working implementation of an RFC-STAGE or DRAFT spec.
   - A conformance test.
   - A replication or extension of [evidence/netl-energy-v0.4.md](evidence/netl-energy-v0.4.md) under a new model, scenario, or seat.
   - A specification change with a working justification.
4. **Sign your commits.** Provenance is the point.

## What advances a spec

| From | To | Required |
|---|---|---|
| RFC-STAGE | DRAFT | Mechanism specified; one prototype implementation |
| DRAFT | STABLE | Wire format specified; two independent interoperating implementations; conformance suite |

## What does not advance a spec

- Marketing claims without measurement.
- Proposals that introduce undefined terms.
- Changes that break Apache 2.0 portability.
- Changes that require closed-source dependencies.

## Sustained objection

If your PR is rejected and you believe the rejection is not technically grounded, file a **sustained objection**: a written technical concern that the PR author or chair has not refuted. Sustained objections block consensus until they are resolved on the record.

## Code of conduct

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Short version: argue the work, not the person.

## Replication is contribution

The single most valuable contribution to this WG today is **independent replication of NETL Exhibit D** across additional models, scenarios, and seats. The evidence base is N=5 on one scenario with one model family across two seats. We need N=500, multi-provider, multi-scenario. If you can run benchmarks, run benchmarks.
