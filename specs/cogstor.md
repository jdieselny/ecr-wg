---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: spec-draft
---

# COGSTOR Specification

**Status:** DRAFT
**Version:** 0.1
**Source:** Continuum-meta `architecture/cogstor_persistence_proposal.md` (2026-05-21)

## Abstract

COGSTOR (Cognitive Object Storage) is the persistence substrate of the Continuum-meta stack. It stores the working memory of synthetic agents -- short-term session state and long-term learned patterns -- so that the cognitive network retains what would otherwise be re-sent through the model on every call.

COGSTOR is the load-bearing mechanism for the savings reported in [NETL Exhibit D](../evidence/netl-energy-v0.4.md). Without persistent cognitive state, Stateless Redundant Execution is unavoidable.

## Object model

A COGSTOR object is a serializable record holding:

- `agent_id` -- stable identifier of the synthetic agent.
- `session_timestamp` -- when this object was last written.
- `active_items` -- current working context.
- `learned_patterns` -- long-term preferences and corrections.
- `working_memory` -- in-flight task state.

## Required properties

A conformant COGSTOR implementation MUST provide:

- **Pointer-file deduplication.** Two objects sharing content reference the same underlying bytes.
- **Differential snapshots.** Successive writes for the same `agent_id` store deltas, not full copies.
- **Edge caching.** Reads served from the closest cache to the requesting node.
- **TTL with operator override.** Default eviction policy with explicit pin support.
- **Graceful degradation.** If the storage layer is unavailable, the system MUST fall back to a slower path (e.g., local file I/O) rather than fail the execution.
- **Re-Absorption on egress.** When a cognitive result returns from any source -- overlay cache or cold datacenter call -- the result MUST be written to the local COGSTOR substrate *before* it is delivered to the human endpoint. The network is the system of record; the human endpoint is volatile. Without Re-Absorption, every datacenter response is a one-shot UDP-style leak: energy spent, intelligence lost, retention zero. With it, the network retains what the human cannot.

## Optional properties

- **Session history.** Append-only log of prior sessions for audit, post-mortem, and pattern analysis.
- **Cross-operator namespacing.** Multiple operators sharing a backend without cross-contamination.
- **Durable snapshot persistence** beyond cache TTL.

## Reference implementation: Redis backend

The Continuum-meta substrate proposes a Redis 7.0+ reference implementation. Full design lives in `continuum-meta/architecture/cogstor_persistence_proposal.md`, including:

- Redis Hash schema (`continuum:cogstor:{agent_id}`).
- Redis Streams for session history.
- A five-phase migration path from file-based to Redis-backed COGSTOR.
- Dual-write transition, fallback-to-file resilience, and operator-scoped namespaces.

Memcached is documented as a simpler alternative for deployments without session history requirements.

**The reference implementation is non-normative.** Any backend that meets the required properties conforms.

## Open problems

1. **Cross-provider serialization.** COGSTOR objects today encode provider-specific context shapes. A portable wire format is unspecified.
2. **Cache invalidation under model upgrades.** When the upstream model version changes, what fraction of cached cognitive state remains valid? Mechanism unspecified.
3. **Privacy boundary.** A COGSTOR shared across operators is a potential leak channel. The trust boundary between `agent_id`, operator, and tenant is unspecified.
4. **Conformance test suite.** None exists. Required for the spec to advance from DRAFT.

## Status to advance

DRAFT advances to STABLE when:

- Wire format is specified.
- At least two independent implementations interoperate.
- Conformance tests pass on both.
