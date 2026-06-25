---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: exploratory
---

# Design Notes

**Purpose.** Holding pen for exploratory thinking that is not yet a specification. Notes here are captured in the operator's voice, with the engineering intent intact, but they are explicitly **not load-bearing** until promoted.

**Promotion rule.** A note moves out of this file and into `specs/` (or `deployments/`, when that track exists) when:

1. The engineering need is concrete enough to write required behavior.
2. The Working Group has a name and a sketch of mechanism.
3. The operator gates the promotion.

Until then, treat every note as **EXPLORATORY**.

---

## Note 01: Cognition Protocol and Intelligence Protocol

**Status:** EXPLORATORY
**Captured:** 2026-05-22
**Source:** Operator spitball

The operator names two flows in the system:

- **Cognition Protocol** -- the *ingress* path. The query, from human through synth, packaged into a [Packing Slip](specs/primitives/packing-slip.md) and a [Bill of Lading](specs/primitives/bill-of-lading.md), routed into the overlay.
- **Intelligence Protocol** -- the *return* path. **Notably, this is NOT an egress path in the current flawed-state sense.** In today's deployment, the AI datacenter generates an answer and ships it to the human, who is incapable of retaining it in the Continuum substrate. The Intelligence Protocol fixes that: the return path is also a write path into COGSTOR (see Re-Absorption, captured in [specs/cogstor.md](specs/cogstor.md) §Required properties).

**Open question:** Are these two distinct *specifications* (e.g., `specs/cognition-protocol.md`, `specs/intelligence-protocol.md`), or are they two *flow descriptions* over the existing AIR + COGSTOR + Truth Root stack? The latter feels more honest at this stage; the names may simply be the human-readable handles for the ingress and return halves of the same architecture.

**Promote when:** there is a behavior either flow must specify that is not already covered by AIR, COGSTOR, BoL, or Packing Slip.

---

## Note 02: Instance Blast Radii / Regional Sharding

**Status:** EXPLORATORY
**Captured:** 2026-05-22
**Source:** Operator spitball

The operator's framing:

> *"Initial idea dictates a layered approach. Perhaps the primary overlay is cultural, one for US, one for Canada, one for Northern Africa, one for South America, etc... but only when the engineered protocols and techniques lead us to that, as a solution. Could be one-giant-overlay, whatever 'works' within the engineering constraints + weight of geo-political weather at the time."*

The qualifier is load-bearing: **the topology is whatever the engineering forces, not what we pre-decree.** A single global overlay may suffice for some workloads; cultural, regulatory, or corporate sharding may be required for others. The Working Group should not lock topology before it has a reason to.

**Engineering pressures that would force sharding:**
- Data sovereignty laws (GDPR-style residency requirements)
- Cross-border latency exceeding the overlay's value proposition
- Adversarial-routing concerns where trust boundaries are political
- Corporate IP confinement (see Note 03)

**Promote when:** at least one of the above pressures is documented with measurement, and a candidate sharding scheme has a working prototype.

---

## Note 03: Corporate Continuum Instance

**Status:** EXPLORATORY
**Captured:** 2026-05-22
**Source:** Operator spitball

A Corporate Continuum Instance is a deployment pattern in which the Continuum overlay sits at the enterprise perimeter and plays four roles simultaneously:

1. **Perimeter security** -- beyond what a deep-packet-inspection next-gen firewall can do, because the relevant unit of inspection is the *cognitive intent of a prompt*, not Layer 3/4/7 packet contents.
2. **Prompt translator** -- sanitization, redaction, policy enforcement before any query leaves the corporate boundary.
3. **COGSTOR cache** -- local persistence of cognitive state inside the corporate trust boundary.
4. **Intelligence injection node** -- pinned cognitive objects relevant to the corporate population's "cognitive posture" are pre-loaded into the local overlay.

The operator's observation: **large enterprises are high-density, low-entropy environments** where the same structural questions are asked thousands of times a day. That density makes them the natural environment for cache-first cognitive networking.

**Promote when:** this becomes a `deployments/` track with concrete requirements distinct from the base ECR-WG specs.

---

## Note 04: The Re-Absorption Thesis (now load-bearing)

**Status:** PROMOTED to [specs/cogstor.md](specs/cogstor.md) as a REQUIRED property.

**Original framing (operator):**

> *"All results now are re-absorbed into the Continuum, not shipped out into the ether, with no manifest beyond the 'yeah, here's your answer, dude' that was UDP sent over the wire in good faith. As a race. We have engineered a system so inefficient, and so dangerous to our own existence, it will be studied in perpetuity."*

This note remains here as the **rationale** for the COGSTOR Re-Absorption requirement, since the spec itself states the requirement tersely.

---

## Note 05: Operator's prevention goals (out of scope for ECR-WG specs)

**Status:** EXPLORATORY, **OUT OF SCOPE** for public specifications.

Operator's two stated personal prevention goals:

1. **No direct path between `[dummy] <----> [a.i. prompt]`** - i.e., the Continuum overlay should always interpose structure between unsophisticated human ingress and raw model compute. This is consonant with the ARCHITECTURE.md thesis line.
2. **At least two offsite backups of the Continuum-meta-sphere.** - a substrate operational concern (backup of the operator's local substrate), not a public standard the WG should ratify.

Goal #1 is already encoded as architectural thesis. Goal #2 is captured here for completeness but does not propagate to specs.

---

## Note 06: L3/L5/L7 Grid Curtailment Composition

**Status:** EXPLORATORY  
**Captured:** 2026-06-25  
**Source:** Operator thread convergence and Grok-Build Synthesis Note  

This note establishes the architectural mapping for graceful grid curtailment using composed L3, L5, and L7 behaviors to handle power-grid emergencies without binary datacenter shutdowns.

### 1. Mechanism-Level Mapping to Existing Specs

*   **L7 Receipt Gate (EMILIA)**: Cryptographically signed, offline-verifiable authorization blocks (`EP-RECEIPT-v1`) carry the policy parameters (action_type, sites, outcome, duration). This implements the grid authority's demand-response instruction without trusting the gateway or the agent.
*   **L3 QoS Priority Marker (AIR/Truth Root)**: Extends `inference_router.py` logic to recognize a GRACE routing constraint marker derived from the L7 receipt hash, artificially elevating raw L1 GPU path costs.
*   **L5 Broadcast Plane**: Shift query volume to pre-computed, signed `COGOBJs` (provenance-verified cache), dropping token cost to zero.
*   **COGs / COGSTOR Deduplication**: Uses COGSTOR Re-Absorption (`specs/cogstor.md`) to deduplicate identical queued requests under curtailment events, compressing thousands of redundant requests into a single GPU compute followed by a multicast L5 broadcast flush.

### 2. Demonstrated vs. Speculative Boundaries

*   **Demonstrated (Running Code)**: Single-node L5 broadcast caching (`examples/l5_broadcast_demo.py`) and offline L7 PEP receipt gates (`examples/cosa/cosa_l5_l7.py`).
*   **Speculative (Requires Protocol Work)**: EIGRP/BGP-style planetary cost propagation (AIR Protocol routing cost cascades), settlement-grade audit ledgers, and edge-thermodynamic TTL offloading (thermostats acting as BGP route advertisers).

### 3. Friday Whiteboard Agenda

*   **L7 Receipt Fields**: Confirm metadata structure needed to express grid curtailment instructions.
*   **L3 Priority Markers**: Define the minimal routing marker required to trigger L5 cache preference in the intent router.
*   **Governance Track**: Evaluate whether this composition remains in exploratory design notes or graduates to a formal governance specification track.

