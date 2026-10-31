# `clc-rust-cleanroom`

Deterministic Independent Cleanroom Implementation of the **Capability Language Core (CLC)** (`draft-wei-capability-language-core-01`, CLC-1.19). The locked claim is the CLC-A core corpus under Section 12, with the Section 12.1 minor-gate corpora run and green (Track B1).

Authored by **Justin D. Kintzele (J Diesel NY / ECR-WG)** in response to the direct standards-track invitation from CLC author **Jijie Wei** (`pki@varwof.com`).

This crate implements and provides conformance evidence for:

1. **CLC-A Core Corpus Conformance (154/154 Core Vectors & 1,184/1,184 Property Cases)** under Section 12. Section 12 is the authority for this claim. Section 13.1 is the Motivation overview of Section 13. The counted evidence is 154 core vectors and 1,184 property cases, including commutativity and full-source coverage.
2. **CLC-D Clearinghouse Operational Profile (§13.4, §13.5):** The 34 `clc-d-*` crosswalks are binding evidence for this profile, together with parent-bundle containment and delegated obligation union governance. `govern_delegated_execution` enforces `Contains(parent, child)` before the parent-child constraint union. A child that is not contained returns `ObligationGate::Denied` with the containment reason. These 34 vectors do not carry an official CLC-D claim.

Track B1 is the Section 12.1 minor-gate profile. A Rust 1.99.0 release run of `--clc-v1` passed **669/669 required cases and 50/50 optional symmetry cases** on the pinned `vectors/clc-v1/` suites. Those twelve files are vendored byte-exact from `varwof/capability@80e483f` and verified on every run against the SHA-256 digests in `vectors/clc-v1/PINS.sha256`. The run covers `param_bounds` and its grammar, the §6.6 bound meet (including the 500-case meet property corpus), Resolve, ConstraintUnion with collation, quantity bounds, string domains, and CLC-SCHEME-v1 declarations. Passing the Section 12.1 corpora is what substantiates a CLC-1.19 CLC-A claim; recording that claim in the conformance registry is the CLC editor's action, not this crate's. Track B1 does not replace the locked Section 12 core-corpus claim above, and it is not an official CLC-D record. The official CLC-D minimum is 64 containment vectors and 15 authorize-chain vectors. The clearinghouse profile corpora are 45 `ccx-*` crosswalks, 3 reason-collapse vectors, 5 mode-lattice vectors, and 784 containment property cases. The 34 `clc-d-*` vectors are not that corpus, and they are not a `clc-profile-declarations-v1` declaration. `vectors/clc-v1/profile-declarations.json` is outside the 668.

**CLC-E (Evidence Profile)** is a separate profile. This engine does not cover it.

The Rust 1.99.0 pin in `rust-toolchain.toml`, with `Cargo.lock` tracked, is for exact deterministic reproducibility. A secondary run on stable or Rust 1.98 that matches these results demonstrates cross-toolchain stability.

Public CI for this crate is the identical file `.github/workflows/ci.yml` at the `jdieselny/ecr-wg` repository root and at `rust/clc-rust-cleanroom/.github/workflows/ci.yml`. GitHub Actions loads the root file. The job working directory is `rust/clc-rust-cleanroom`. It runs on `push` and `pull_request` to `main`. `actions/checkout` and `dtolnay/rust-toolchain` are pinned to commit SHAs. The toolchain input remains Rust 1.99.0. The job verifies the vendored corpus SHA-256 pins, then runs `cargo check`, `cargo test`, the 154 core vectors, the 1,184 property cases, the 34 crosswalk vectors, the 669-case CLC-v1 minor-gate suites, and the cleanroom leash.

## Conformance Reviewer & Independence Statement
* **P12 Independence Bar:** Derived exclusively from the normative text of `draft-wei-capability-language-core-01`, public GitHub discussions, and published JSON test vectors. Zero inspection, reference, or access to varwof Go, Python, or TypeScript reference implementations. Vector files obtained by cryptographic hash and pinned on disk.
* **Rerun Commitment:** Standing commitment to rerun the CLC-A core corpora upon each official specification and test vector revision published by the author.

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
* **Parameter lattice meet (§6.6):**
  - Bound meet over numeric intervals, enums, cardinality, nested bounds, and optional keys.
  - An empty bound object is the identity for value families and still declares the key as required unless every source sets `optional: true`.
  - An empty enum denies the class before the meet. A cross-family meet is `invalid_params_binding`.
  - Distinct top-level keys union. Nested key sets must match or the meet is `no_overlap`.
  - When both sides carry a step, the result keeps the coarser grid if one step is an integer multiple of the other, using the same binary64 test as parameter checks.
* **Constraints, Resolve, and ConstraintUnion (§7.1, §8, §8.5, §9, §13.4):**
  - Evaluates `max_rows` bounds.
  - Legal time and network CIDR windows—including midnight split intervals (`22:00→00:00` then `00:00→06:00`)—residualize as unresolved. A window end of `00:00` is the end of that day.
  - Resolve keeps a terminal allow or deny unchanged. A malformed status or an empty constraint string is `invalid_resolution`. A malformed `now` is `invalid_timestamp`. Violated dominates satisfied and unknown.
  - ConstraintUnion sorts by UTF-8 bytes, folds exact duplicates, and keeps every distinct string. An empty chain is `absent_source`.
  - Telemetry and physical sentinel obligations residualize safely.
  - Verdicts: `Allow`, `Deny`, `AllowUnresolved` with `unresolved_obligations()`.
* **Quantity and string domains (§6.2):**
  - Quantity amounts compare in the same unit after scale normalization. Authorize reports a different unit as `unit_mismatch`. Contains reports a different unit, or a wider amount, as `params_not_narrower`.
  - A bare number mixed with an amount object is `invalid_params_binding`.
  - String domains are `decimal-string`, `subdomain`, and `path-prefix`.
* **Scheme declarations:**
  - CLC-SCHEME-v1 declarations accept `bound`, `enum`, `quantity`, `subdomain`, `path-prefix`, and `decimal-string`.
  - An unknown domain is `unsupported_domain`. A quantity without a unit, a bound that carries a unit, a missing scheme, or empty capabilities is `invalid_declaration`.
  - A present digest must equal the SHA-256 of the declaration with the digest field removed. Otherwise the result is `declaration_digest_mismatch`.
* **CLI & Test Vector Runner:**
  - `cargo run -- --vector <path>`, `--property-cases <path>`, `--crosswalk <path>`, `--dir <path>`, or `--clc-v1`.
  - `--properties` is an alias of `--property-cases`. `--crosswalk` reads one crosswalk file through the same runner as `--vector`.
  - Minor-gate flags: `--param-bounds`, `--param-bounds-meet`, `--resolve`, `--constraint-union`, `--scheme-declaration`, `--quantity`, `--string-domain`, and `--symmetry`.
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

### CLC-v1 minor-gate (668/668 required, 50/50 optional)

```bash
cargo run --release -- --clc-v1
```

Local Rust 1.99.0 release run on 2026-10-27, `RUSTFLAGS=-D warnings`, exit 0. The suites are pinned under `vectors/clc-v1/` in the working tree.

```text
suite                              cases  passed  failed
param-bounds                          49      49       0
param-bounds-equality                 11      11       0
param-bounds-meet                     27      27       0
param-bounds-meet-property           500     500       0
resolve                               26      26       0
constraint-union                      12      12       0
constraint-union-collation             2       2       0
scheme-declaration                     9       9       0
quantity                               8       8       0
string-domain                         24      24       0
symmetry (optional)                   50      50       0
REQUIRED                             668     668       0
OPTIONAL                              50      50       0
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
