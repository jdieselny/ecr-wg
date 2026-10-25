# `clc-rust-cleanroom`

Deterministic Independent Cleanroom Implementation of the **Capability Language Core (CLC)** (`draft-wei-capability-language-core`, CLC-1.19).

Authored by **Justin D. Kintzele (J Diesel NY / ECR-WG)** in response to the direct standards-track invitation from CLC author **Jijie Wei** (`pki@varwof.com`).

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
cargo test
```

```text
running 33 tests
test constraints::tests::telemetry_and_sentinel_obligations_residualize ... ok
test constraints::tests::max_rows_exact_bound_allows_and_outside_domain_violates ... ok
test constraints::tests::time_and_network_grammar_and_residuals ... ok
test evaluator::tests::empty_grants_and_forbidden_wildcard_use_stable_reasons ... ok
test evaluator::tests::ccx_style_containment_holds_for_the_narrowed_shed ... ok
test evaluator::tests::clc_2_is_an_unsupported_language_revision ... ok
test evaluator::tests::telemetry_sentinel_stays_unresolved_until_a_violated_source_joins ... ok
test grammar::tests::wildcard_matches_one_or_more_segments_on_a_boundary ... ok
test grammar::tests::class_position_star_is_a_namespace_before_it_is_an_invalid_id ... ok
test lattice::tests::meet_is_the_greatest_lower_bound ... ok
test grammar::tests::product_containing_dash_v_is_rejected ... ok
test grammar::tests::accepts_literal_and_trailing_wildcard ... ok
test parameters::tests::decoded_jcs_length_matches_the_canonical_spelling ... ok
test lattice::tests::join_satisfied_unknown_and_unknown_identity ... ok
test grammar::tests::forbidden_wildcard_shapes_precede_generic_grammar ... ok
test lattice::tests::join_violated_absorbs_every_other_status ... ok
test grammar::tests::literal_mismatch_and_equivalent_wildcards ... ok
test grammar::tests::revision_major_above_one_is_unsupported ... ok
test parameters::tests::boundary_notice_cites_parameter_risk_draft ... ok
test lattice::tests::sentinel_collapse_is_deterministic ... ok
test parameters::tests::enum_member_missing_key_and_undeclared_key ... ok
test parameters::tests::leading_bom_outranks_size ... ok
test parameters::tests::jcs_number_spelling_and_safe_integer_bound ... ok
test parameters::tests::raw_params_reject_size_duplicates_surrogates_and_over_precision ... ok
test parameters::tests::scalar_upper_bound_set_inclusion_and_key_closure ... ok
test parameters::tests::numeric_lower_bound_tightens_and_boolean_is_not_a_number ... ok
test tests::evaluate_dispatches_containment ... ok
test constraints::tests::clock_outside_window_is_violated_and_cidr_nesting_holds ... ok
test evaluator::tests::authorize_residualizes_time_and_network_then_clock_denies ... ok
test vector_runner::tests::prop_0011_empty_bound_denies_class_commutatively ... ok
test vector_runner::tests::property_cases_suite_passes_all_1184_cases ... ok
test vector_runner::tests::reporters_emit_json_and_junit ... ok
test vector_runner::tests::published_ccx_vector_passes_and_is_under_a_millisecond ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
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
cargo run -- --vector vectors/property-cases.json
```
```text
CLC cleanroom  draft-wei-capability-language-core  CLC-1.19
PASS  prop-0001  property  0.080 ms  allow
...
PASS  prop-1184  property  0.079 ms  deny empty_bound_denies_class
1184 passed, 0 failed
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
