---
aft: AI-generated-user-reviewed-pending
agent: Dima (Continuum-meta principal architect, Claude Opus 4.7)
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: spec-rfc-stage
---

# AIR Protocol Specification

**Status:** RFC-STAGE
**Version:** 0.0 (problem statement only)

## Abstract

AIR (Advanced Intelligent Routing) is the proposed semantic anycast layer of the Continuum-meta stack. It routes cognitive queries to the node most likely to satisfy them at the lowest marginal compute cost. It is intentionally **not IP-based**.

**No implementation exists.** This document states the problem and constrains the design space. The Working Group is invited to design it.

## Problem statement

In a Stateless Redundant Execution deployment, every query traverses the same path: client to model endpoint, full context as prefill, every time. There is no notion of "this query is similar to one I just answered" at the network layer; deduplication, if it happens, happens inside the provider.

For a Continuum-meta deployment to capture the savings shown in [NETL Exhibit D](../evidence/netl-energy-v0.4.md), the network MUST route a query to a node that already holds relevant cached cognitive state (in [COGSTOR](cogstor.md)) **before** it falls back to a cold model call.

## Required behavior (target)

1. **Semantic match.** Routing decisions are informed by vector similarity between the incoming query and the cognitive state held at candidate nodes.
2. **Capacity awareness.** Routing accounts for token density and load at candidate nodes -- a perfect cache hit on an overloaded node may be slower than a near-hit on an idle one.
3. **Zero-Match fallback.** If no node holds usable state, the query drops to a cold model call. This is the bypass path; **minimizing it is the protocol's purpose.**
4. **GRACE-aware.** Routing decisions carry the GRACE ROUTING field of the request, not the underlying transport's destination.

## Design constraints (from substrate whiteboard, 2026-05)

The design whiteboard sketches the operational flow as:

```
HUMAN -> SYNTH (memory) -> Packing Slip + Hash -> BILL OF LADING -> CONTINUUM-META
                                                                          |
                                                                          v
                              [Highly Structured | Full Context | ZERO MATCH]
```

AIR is the layer that resolves which Continuum-meta node receives the Bill of Lading and which return-layer path it takes. The Bill of Lading and Packing Slip + Hash are design-phase artifacts captured on the whiteboard; their wire format is intentionally open at this revision.

## Open problems

1. **Vector index distribution.** Where does the semantic index live? Per-node, gossip-replicated, centralized, hierarchical? Each choice has a different failure mode.
2. **Bill of Lading wire format.** JSON-LD, CBOR, protobuf -- open.
3. **Packing Slip + Hash semantics.** Whiteboard-defined as the deterministic envelope around a query. Cryptographic binding to the originating synthetic agent is presumed but unspecified.
4. **Backpressure.** What happens when multiple nodes claim they can satisfy a request? Tiebreak unspecified.
5. **Adversarial routing.** A malicious node could claim high cache density to attract queries and harvest context. Mitigation unspecified.
6. **Interaction with the Truth Root.** Should routing decisions verify the requesting agent's signature before considering the request? Open.
7. **Regional sharding / blast-radii topology.** Whether the overlay is one global plane or sharded by cultural, regulatory, or corporate boundary is open and contingent on engineering need. Captured as exploratory thinking in [DESIGN_NOTES.md](../DESIGN_NOTES.md). The WG does not pre-commit to a topology.

## Status to advance

RFC-STAGE advances to DRAFT when:

- A wire format for the Bill of Lading is proposed.
- A routing decision algorithm is specified and benchmarked against a baseline (e.g., round-robin or shortest-path).
- At least one prototype implementation routes a non-trivial workload.

The WG explicitly invites infrastructure architects with **anycast, CDN, and content-aware routing** experience to lead this track.
