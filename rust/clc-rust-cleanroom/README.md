# `clc-rust-cleanroom`

Deterministic Independent Cleanroom Implementation of the **Capability Language Core (CLC)** (`draft-wei-capability-language-core`, CLC-1.19).

Authored by **Justin D. Kintzele (J Diesel NY / ECR-WG)** in response to the direct standards-track invitation from CLC author **Jijie Wei** (`pki@varwof.com`).

This crate implements and provides conformance evidence for:

1. **CLC-A Core Conformance Profile (§13.1):** 154/154 Core Vectors, 1,184/1,184 Property Cases (including commutativity and full-source coverage).
2. **CLC-D Clearinghouse Operational Profile (§13.4, §13.5):** 34/34 Crosswalk Vectors, parent-bundle containment, and delegated obligation union governance. `govern_delegated_execution` enforces `Contains(parent, child)` before the parent-child constraint union. A child that is not contained returns `ObligationGate::Denied` with the containment reason.

Extended §6.6 `param_bounds` Meet and full §12.1 general resolution are scoped to follow-on milestone profiles. The 34 crosswalk vectors are this crate's clearinghouse binding evidence. They are not a substitute for the official CLC-D corpus or for a `clc-profile-declarations-v1` declaration.

---

## 1. Cleanroom Mandate & Independence Invariant
This crate was built under strict cleanroom isolation to satisfy the project's **P12 independence bar**:
- Decides strictly from normative `draft-wei-capability-language-core` (CLC-1.19), mathematical lattice algebra, and JSON vectors.
- **Zero** inspection, scraping, or referencing of existing Go (`varwof/register`), Python, or TypeScript (`varwof/aic-capability-demo`) reference implementations across the `varwof` organization.
- Enforces `#![forbid(unsafe_code)]` across both `lib.rs` and `main.rs`.
- Passes `cargo check`, `cargo build`, and `cargo test` with `-D warnings`.

---

## 2. Specification Capabilities

* **Grammar (§3, §13.2):**
  - Canonical identifier grammar: `namespace/capability-v1:action:target`.
  - Trailing wildcard prefix matching (`*`), strict segment boundary validation.
  - Forbidden shapes (such as `SEL*` and `queryEXTRA`) deterministically fail as `unsupported_wildcard` before generic ID parse errors.
* **Partial Order Lattice Algebra (§8.5):**
  - Strict partial order: $\text{Violated} \succ \text{Satisfied} \succ \text{Unknown}$.
  - Join ($\bigvee$) is the most restrictive status.
  - A violated out-of-band sentinel collapses the decision deterministically to `deny` with `{type}:violated`.
* **Scalar Parameter Narrowing & P11 Invariants (§6.2, §6.5, §7, §9.1, §13.4.3):**
  - Symmetric key closure: Child parameters cannot introduce undeclared keys.
  - Plain numbers evaluated as upper bounds ($2.5 \le 10$).
  - `param_bounds` raises min, lowers max, and requires child step to be an integer multiple of parent step.
  - P11 Empty-Bound Denial: A declared parameter value of `[]` or `{}` is an explicit empty bound ($\emptyset$) and denies the class with `empty_bound_denies_class` commutatively across all intersection sources, while absent parameters or `{}` maintain the §7 Rule 6 identity.
  - Complex parameter domains (filesystem paths, URLs, shell commands) trigger an explicit boundary notice citing `draft-correctover-gg-mcp-parameter-risk`.
* **CLC-D Containment Crosswalks & Parent Bundles (§13.4, §13.5):**
  - Evaluates monotonic delegation ($\text{Child} \sqsubseteq \text{Parent}$) via `evaluate_containment` and `evaluate_containment_any`.
  - Layered verification: grant validity, identifier coverage (`different_namespace`, `child_exceeds_parent`), and parameter narrowing (`params_not_narrower`).
  - Step grids: Child step must be an integer multiple of parent step ($k \cdot \text{step}_P = \text{step}_C$).
  - Execution Union Axis: Time windows, network CIDRs, and physical sentinels remain on the execution union axis evaluated by the operational governor rather than prematurely failing the containment gate.
  - Complete operational lifecycle documented in [`docs/CLEARINGHOUSE_OPERATIONAL_PROFILE.md`](docs/CLEARINGHOUSE_OPERATIONAL_PROFILE.md).
* **Constraints & Decision Pipeline (§8, §9, §13.4):**
  - Evaluates `max_rows` bounds.
  - Legal time and network CIDR windows—including midnight split intervals (`22:00→00:00` then `00:00→06:00`)—residualize as unresolved.
  - Telemetry and physical sentinel obligations residualize safely.
  - Verdicts: `Allow`, `Deny`, `AllowUnresolved` with `unresolved_obligations()`.
* **CLI & Test Vector Runner:**
  - `cargo run -- --vector <path>`, `--property-cases <path>`, or `--dir <path>`.
  - Structured output reports: `--report text|json|junit`.
  - Exits with non-zero code on test failure.

---

## 3. Conformance & Benchmark Verification

### Unit Test Suite
```bash
cargo test --lib
```

Library unit tests pass under `#![forbid(unsafe_code)]` and `RUSTFLAGS=-D warnings`. The delegated-execution tests deny a child identifier the parent does not cover (`child_exceeds_parent`) and parameters that are not narrower (`params_not_narrower`) even when the child grant would authorize the operation on its own.

```text
running 44 tests
...
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

### Core Vector Suite (154/154)
```bash
cargo run -- --vector vectors/clc-a-core-vectors.json
```
```text
CLC cleanroom  draft-wei-capability-language-core  CLC-1.19
PASS  syntax-001  syntax  0.302 ms  valid
...
PASS  params-070  entail  0.046 ms  allow
154 passed, 0 failed
```

### Property Cases Suite (1,184/1,184)
```bash
cargo run -- --property-cases vectors/property-cases.json
```
```text
CLC cleanroom  draft-wei-capability-language-core  CLC-1.19
PASS  prop-0001  property  0.080 ms  allow
...
PASS  prop-1184  property  0.079 ms  deny empty_bound_denies_class
1184 passed, 0 failed
Commutativity verified: 1,184 / 1,184
Full-source coverage verified: 1,184 / 1,184
```

### CLC-D Crosswalk Suite (34/34)
```bash
cargo run -- --vector vectors/clc-d-crosswalk-vectors.json
```
```text
CLC cleanroom  draft-wei-capability-language-core  CLC-1.19
PASS  clc-d-power-narrow-001  contain-crosswalk  0.080 ms  contains=true
...
PASS  clc-d-bundle-none-001  contain-crosswalk  0.053 ms  contains=false different_namespace
34 passed, 0 failed
```

### Published CCX Vector Execution
```bash
cargo run -- --vector vectors/ccx-iicp-001.json
```
```text
CLC cleanroom  draft-wei-capability-language-core  CLC-1.19
PASS  ccx-iicp-001  contain-crosswalk  0.760 ms  contains=true
1 passed, 0 failed
```

---

## 4. License & Governance
- **License:** Apache-2.0
- **Contributions:** Inbound = Outbound with DCO (Developer Certificate of Origin) sign-off.
- **Standards:** Conforms to IETF BCP 78/79 rules.
