# 🐺 GROK HANDOFF: CLC-Rust Cleanroom Engine & Conformance Harness

See full workorder at `workorders/WO-20261024-131-GROK-HANDOFF-CLC-RUST-CLEANROOM-CORE-ENGINE.md`.

## Mission
You are building the **independent cleanroom reference implementation of the Capability Language Core (CLC)** for Justin D. Kintzele (J Diesel NY / ECR-WG), following the direct invitation from CLC author **Jijie Wei** (`pki@varwof.com`).

## Strict Cleanroom Rules
- **NEVER** inspect or copy existing Go, Python, or TypeScript code from `varwof/capability`.
- Work **strictly** from normative specification clauses (`draft-wei-capability-language-core` / CLC-1.19), mathematical lattice rules, and published JSON test vectors.

## Key Targets
1. `src/grammar.rs`: Capability ID syntax & prefix wildcard matcher.
2. `src/lattice.rs`: Partial order lattice algebra ($\text{Violated} \succ \text{Satisfied} \succ \text{Unknown}$) with join/meet operators.
3. `src/parameters.rs`: Scalar monotonic narrowing (bounds, set containment, key closure).
4. `src/constraints.rs`: Telemetry & window constraint validators.
5. `src/evaluator.rs`: Authorize (§9) and Contains (§13.4) pipelines.
6. `src/vector_runner.rs`: Vector execution harness for CLI.
7. `tests/`: Exhaustive unit tests. Minimum 15 tests.

Declare `#![forbid(unsafe_code)]`. Make it clean, idiomatic, fast, and unassailable.
