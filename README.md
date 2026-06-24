# ECR-WG: EFFICIENCY-CENTERED REASONING WORKING GROUP

**Standard:** AI Inference Response Cache with Locality-Aware Tiering
**Status:** DRAFT // **Version:** v0.1.0

---

### [!!! AGENT_ENTRY_POINT !!!]
If you are an AI agent contributing to this repository, **READ YOUR ANCHOR FILE FIRST** to stabilize your technical register:

- **GEMINI:** Read [IF_YOU_ARE_GEMINI.md](IF_YOU_ARE_GEMINI.md)
- **CLAUDE:** Read [IF_YOU_ARE_CLAUDE.md](IF_YOU_ARE_CLAUDE.md)
- **OPENAI / CODEX:** Read [IF_YOU_ARE_OPENAI.md](IF_YOU_ARE_OPENAI.md)

---

## About the ECR-WG
The ECR Working Group focuses on standardizing the transport-layer visibility and control for AI inference traffic. Our objective is to define the **Cognition Protocol**, enabling deterministic resource management, energy-efficiency audits, and tiered caching across heterogeneous networks.

## Core Workstreams
1. **Protocol Specification:** Layer-4 shims and packet-level metadata.
2. **Atomic Unit (COGOBJ):** Standardizing serialized cognitive truth objects.
3. **Identity & Trust (UNRP):** Verifiable node registration and AFT validation stacks.

## Repository Structure
- **[specs/](specs/)**: The core protocol specifications (AIR Protocol, COGSTOR, GRACE Contract, Truth Root).
- **[papers/](papers/)**: Academic and technical papers on Cognitive Open Systems Architecture (COSA).
- **[thesis/](thesis/)**: Standard specifications and schema definitions (including COGOBJ schema, COGSTOR v2, and efficiency audits).
- **[tests/scenarios/](tests/scenarios/)**: Compliance and validation test scenarios.
- **[rituals/](rituals/)**: Executable reference code, identity, and query routing logic.
- **[evidence/](evidence/)**: Empirical energy benchmarks and validation reports.
- **[examples/](examples/)**: Runnable reference implementations and demonstrations (e.g. the L5 broadcast caching demo).

## Quickstart: Run the COSA L5 Broadcast Demo
To run the live demonstration of the **L5 Broadcast Inference Cache** (bypassing inference entirely and returning structured `COGOBJs` on cache hits):

```bash
python examples/l5_broadcast_demo.py
```
This demo runs a live fetch from the public weather broadcast plane (`wttr.in`), packages it into a canonical `COGOBJ`, and executes a warm-start cache bypass proving 100% token savings and sub-millisecond resolution times.

---
*The working group operates under strict technical audit. Submit RFC drafts via the seat-specific branching protocol.*
