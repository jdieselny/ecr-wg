---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: architecture
---

# Cognitive Open Systems Architecture (COSA) Reference Layout

> **Thesis.** Break the direct path between *unstructured human ingress* and *unconstrained datacenter compute*. Every cognitive query that traverses that path without an overlay is a query that pays full prefill cost and returns nothing to the network. The layers below exist to interpose structure.

## Premise

**Stateless Redundant Execution (SRE)** (the dominant pattern in generative deployment today) sends the full context to the model on every call because the network has no memory of prior calls. As context grows, per-call prefill cost grows. As users grow, both multiply. The crisis is not model size; it is **redundant prefill at planetary scale**.

The COSA architecture asserts that cognitive workloads need a *stack*, not a model. The layers below are not peers, and they are not a knock-off of OSI; they are an **asymmetric execution contract over three functional substrates**.

## The stack

```
  +------------------------------------------------------------------+
  |  GRACE Contract  (per-call execution discipline)                 |  STABLE
  +------------------------------------------------------------------+
  |  AIR Protocol     |  COGSTOR             |  Truth Root           |
  |  semantic         |  cognitive object    |  cryptographic        |
  |  anycast routing  |  storage, diff       |  provenance &         |
  |                   |  snapshots,          |  attestation          |
  |                   |  pointer-file        |                       |
  |                   |  deduplication       |                       |
  |  DRAFT            |  DRAFT               |  DRAFT                |
  +------------------------------------------------------------------+
```

**GRACE sits *across* the three substrates** as the contract every node signs to participate in any single execution. It is not a transport layer. It is a discipline.

## Operational data flow

```
  [HUMAN] --> [SYNTH (S/T + L/T memory)] --> [Packing Slip + Hash]
                                                       |
                                                       v
                                              [BILL OF LADING]
                                                       |
                                                       v
                                              [COSA OVERLAY]
                                                       |
                            +--------------------------+
                            |
                            v
  [INTELLIGENCE RETURN LAYER] <-- Highly Structured | Full Context | ZERO MATCH
```

**Read as:** a human query is shaped by the synthetic agent's short- and long-term memory into a structured handoff (Packing Slip + Hash), wrapped in a signed Bill of Lading, and routed into the COSA overlay. The overlay checks its caches first; only on **Zero Match** is the query allowed to drop to a cold AI datacenter call.

**The conditional is strict:** `check overlay -> if Zero Match -> then route to AI datacenter`. The default path is the overlay. The datacenter is the bypass, not the baseline.

**The Zero Match gate is where the savings come from.** Every query the overlay can answer is a query that does not hit prefill capacity. Every query that *does* hit the datacenter writes its result back into COGSTOR on return (see [specs/cogstor.md](specs/cogstor.md) §Re-Absorption), so the same query never costs full prefill twice.

*The Bill of Lading, Packing Slip + Hash, and Zero Match gate are design-phase constraints. Their wire formats are intentionally unspecified at this revision; see the AIR Protocol and Truth Root specs for the open problems they pose.*

## Layer roles

### GRACE Contract (STABLE)
Per-call discipline a node accepts to participate in a execution. Five fields: GOAL, ROUTING, ANCHOR, CONSTRAINTS, EVIDENCE. See [specs/grace-contract.md](specs/grace-contract.md).

### AIR Protocol (DRAFT)
Semantic anycast: queries route to the node most likely to satisfy them at lowest cost, ranked by cache density, vector proximity, and capacity. Not IP-based. See [specs/air-protocol.md](specs/air-protocol.md).

### COGSTOR (DRAFT)
Cognitive object storage. Differential snapshots, pointer-file deduplication, edge caching. The mechanism by which the overlay remembers context. See [specs/cogstor.md](specs/cogstor.md).

### Truth Root (DRAFT)
Cryptographic provenance for every synthetic output. Binds each result to the synthetic agent that produced it and the human accountability chain behind that agent. **Provenance, not prevention**: a signed liar is still a liar, but they are a *named* one. See [specs/truth-root.md](specs/truth-root.md).

## What this stack is not

*   **Not an OSI replacement.** OSI is layered transport. This is a cognitive execution contract sitting over functional substrates. Comparing them is a category error.
*   **Not a model.** No weights, no training, no inference engine. Provider-agnostic by construction.
*   **Not finished.** Three of four layers are pre-stable.

## What the evidence currently supports

See [evidence/netl-energy-v0.4.md](evidence/netl-energy-v0.4.md) for current energy performance metrics.
