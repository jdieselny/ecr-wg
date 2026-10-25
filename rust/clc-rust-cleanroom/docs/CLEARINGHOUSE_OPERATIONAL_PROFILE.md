# Clearinghouse Operational Profile

**Status:** design note for the clearinghouse question (varwof/capability issue 4).
**Profile id:** `clearinghouse->clc-v1`
**Language:** `draft-wei-capability-language-core-01` (CLC-1.19)
**Engine:** `clc-rust-cleanroom`, `evaluator::evaluate_containment` and `evaluator::evaluate_containment_any`
**Workorder:** WO-20261025-134

This profile binds an external clearinghouse token to a CLC grant pair, runs the CLC-D containment gate, and hands residual constraints to an out-of-band governor. It does not add a constraint type, a reason code, or a verdict to the carrier-neutral core.

## 1. Cleanroom

The relation below is taken from the normative containment clauses, from lattice narrowing (child at or inside parent), and from the vectors already in this crate (`vectors/ccx-iicp-001.json`, `vectors/clc-d-crosswalk-vectors.json`). No Go, Python, or TypeScript implementation was read for this profile. `vectors/ccx-iicp-001.json` is unchanged. `clc-d-power-narrow-001` repeats that 10 kW / 7200 s / groups `[1,2,3]` narrowing under a distinct id.

## 2. What Contains decides

`Contains(parent, child)` is true when every operation the child would allow, the parent would allow. The check is three layers. The first failure wins.

| Layer | Check | Code |
| --- | --- | --- |
| 1. Grant validity | Identifier grammar and parameter binding | The grammar or binding code, unchanged |
| 2. Identifier coverage | Namespace, then segment depth and a trailing `*` | `different_namespace`, otherwise `child_exceeds_parent` |
| 3. Parameter narrowing | Key closure, upper bounds, enums, `param_bounds` | `params_not_narrower`, unless layer 1 already failed |

Namespace is `vendor/product-vMAJOR` plus the first action segment. The only v1 wildcard is a trailing segment `*`. It matches one or more following segments on a boundary. A literal parent does not cover a wildcard child. A broader parent wildcard covers a longer child wildcard. Identical wildcards cover each other. A shorter child wildcard does not.

Parameter rules used by this engine:

- A parent with omitted params, or params `{}`, is unconstrained. The child may declare keys.
- Once the parent declares any key, the child key set (the union of `params` keys and `param_bounds` keys) equals the parent key set. An added key and a dropped key both fail.
- A number in `params` is an upper bound. The child value is less than or equal to the parent value. Lower bounds live in `param_bounds.min`.
- Strings and booleans match exactly. A boolean is not the number 1. JSON types are not coerced.
- An array is a set. Inclusion is subset-or-equal, so equal sets contain, and member order does not matter. An empty child array under a non-empty parent array is vacuous inclusion. A parent empty array fails as `params_not_narrower`.
- Nested objects keep the same keys and recurse.
- `param_bounds`: child min is present and greater than or equal to parent min; child max is present and less than or equal to parent max; child step is an integer multiple of parent step (`k · step_parent = step_child`). Step 3 under step 1 narrows. Step 3 under step 2 does not. Step 1 under step 2 does not, because the finer grid admits points the parent excludes. Omitting a parent min, max, or step widens.
- `optional: true` on the child, when the parent bound is required, widens.
- An empty bound object `{}` is key closure only. The child may introduce a family under that key.
- The same key in both `params` and `param_bounds` is `invalid_params_binding` at layer 1.

A parent bundle contains the child when any one member does (`evaluate_containment_any`). If none do, the reason is the denial of the first parent id in UTF-8 byte order. An empty bundle is `child_exceeds_parent`. The child side of a vector is one capability, the first entry of `child_source.capabilities`.

## 3. What Contains does not decide

Constraints are outside the relation (§13.4.4). Folding time windows, CIDRs, or sentinels into Contains would make the relation depend on a component the core leaves unresolved, and it would change which delegations succeed. This profile keeps that split:

- `clc-d-time-leak-001`: child window `17:00–19:00` against parent `08:00–18:00` still contains.
- `clc-d-cidr-escape-001`: `10.1.0.0/24` against parent `10.0.0.0/16` still contains.
- `clc-d-sentinel-dropped-001`: a child that drops `emilia/telemetry-v1:redundant_power_path` still contains.

The execution sentinel, not Contains, refuses those leaks. See section 6.

`delegation_mode_not_narrower` is a binding-profile pre-check. Contains does not emit it. A deployment that adds delegation mode runs that check before the gate and keeps the code beside the CLC-D codes.

A filesystem path, URL, or shell-looking string is a `BoundaryNotice` citing the parameter-risk draft. `clc-d-complex-domain-001` contains. The notice travels with the result. It is not a denial.

`empty_bound_denies_class` stays the Authorize, Entail, and Intersect code for a declared empty bound. Contains reports a parent empty array as `params_not_narrower`.

## 4. Carrier shorthand collapse

Issue 4 needs a clearinghouse to explain its own denial vocabulary without inventing a second CLC-D relation. Shorthands collapse many-to-one onto the stable codes. A carrier that still wants its own word records it next to the CLC code, in this corpus as `carrier_shorthand`. It does not replace `expect.reason`.

| Carrier shorthand | CLC-D result |
| --- | --- |
| `identifier_not_covered` | `child_exceeds_parent` |
| `params_exceed_parent` | `params_not_narrower` |
| `undeclared_param_key` | `params_not_narrower` |
| `step_not_multiple` | `params_not_narrower` when the multiple test fails. Step 3 under step 1 contains. |
| `different_namespace` | `different_namespace` |
| `time_window_not_contained` | Not a Contains code. Union axis, section 6. |
| `cidr_not_contained` | Not a Contains code. Union axis, section 6. |
| `complex_domain_risk` | `BoundaryNotice`. Contains may still be true. |
| `delegation_mode_not_narrower` | Profile pre-check. Contains does not emit it. |

Layer 1 codes that pass through unchanged include `unsupported_wildcard`, `invalid_capability_id`, `missing_capability_id`, `invalid_params_binding`, `invalid_params_null`, `invalid_params_number`, and `invalid_params_size`.

At a boundary that must not reveal policy shape, an implementation may present `child_exceeds_parent` to the caller as `capability_not_authorized`. It must not present `params_not_narrower` as that code. This crate returns the stable code on the containment result.

## 5. Token to capability identifier

A clearinghouse token is carrier metadata. It is not a capability identifier.

| Carrier field | CLC role |
| --- | --- |
| Workorder id (`wo-example-curtail-082`) | Ingress label. Stored on the receipt. Not parsed as an id. |
| Parent grant id (`grant-example-curtail-001`) | Name of the parent grant inside the clearinghouse. Not a capability id. |
| Capability `id` | The only string Contains parses. Shape `vendor/product-vMAJOR:action[:segment]…` with an optional trailing `:*`. |
| `parent_grant_id` on the child | Join key back to the parent grant record. |

The published power crosswalk is the pattern:

- Parent capability `example/power-v1:curtail:*`, grant id `grant-example-curtail-001`.
- Child capability `example/power-v1:curtail:ups-unit-01`, workorder `wo-example-curtail-082`, `parent_grant_id` pointing at that grant.
- Parameters narrow: `max_shed_kw` 10 to 2.5, `duration_sec` 7200 to 900, `circuit_groups` `[1,2,3]` to `[2]`.

A database clearinghouse token maps the same way: the token names a workorder, and the capability id is `std/database-v1:query:SELECT` (or a longer target) under a parent `std/database-v1:query:*`. Scope strings that are not that grammar fail at layer 1. This profile does not add a second parser for OAuth scope strings or for IICP paths. The clearinghouse translates, then CLC judges the translation.

`example/` identifiers in the vectors are fixtures. A production registration uses the vendor and product the clearinghouse actually issues.

## 6. `allow_unresolved` and the governor

Authorize returns `AllowUnresolved` when a covering grant allows and its recognized-but-unevaluated constraints are non-empty. In this core those residuals are legal time windows, network CIDRs, and telemetry or physical sentinels. `max_rows` is evaluated in Authorize. Other constraint schemes follow the core rule already implemented: unknown schemes deny, while telemetry and sentinel forms residualize.

The governor is outside the crate. Grid sensors, smart meters, and Watchtower discharge residuals. They do not extend the CLC type system.

On a delegated execution the governor builds the obligation set as the union of the parent grant's constraints and the child grant's constraints, then Resolve:

- A child window that sticks out past the parent window does not delete the parent window. The clock is judged against the parent obligation as well as the child obligation. A midnight-wrapping window is the pair of segments the core already accepts (`22:00→00:00` followed by `00:00→06:00`). That arithmetic belongs to the clock check inside Resolve, not to Contains.
- A child CIDR outside the parent prefix does not delete the parent prefix. The source address has to satisfy the parent block. Bitwise prefix checks stay in the governor's network check.
- A sentinel the child dropped is still in the union. `Unknown` does not actuate. `Violated` denies with `{type}:violated`. `Satisfied` from the sensor, meter, or Watchtower clears that one residual.

Actuation waits until every residual in the union is `Satisfied`. `Allow` with an empty residual list needs no governor round-trip. `Deny` never reaches the governor.

This is why a wider child constraint can still pass Contains. The parent's obligation remains in force at execution. The core stays carrier-neutral: it does not learn a grid-sensor type or a meter type.

## 7. Lifecycle

1. **Ingress.** The clearinghouse presents one child capability and one or more parent capabilities, plus the workorder id and the parent grant id. Those two ids are logged. They are not capability grammar.
2. **Profile pre-check.** If the deployment defines delegation mode, it runs that lattice here. A failure is `delegation_mode_not_narrower` and stops the handoff. Deployments without that field skip this step.
3. **CLC-D containment gate.** `evaluate_containment` for one parent, or `evaluate_containment_any` for a bundle. False stops the delegation. The stable reason is the one recorded on the receipt. Boundary notices are attached and do not by themselves stop the gate.
4. **Execution sentinel.** At use, Authorize the operation against the child grant. `Deny` stops. `Allow` may actuate. `AllowUnresolved` hands the residual union (parent constraints and child constraints) to the governor. Resolve collapses each obligation. Actuation requires every obligation `Satisfied`.
5. **SCITT / EP attestation.** An external receipt binds the workorder id, the parent and child capability ids, the containment boolean and reason (or the notices), the Authorize verdict, and the residual resolutions. This crate does not mint that receipt and does not define the COSE or SCITT bytes. The statement's CLC inputs are the values above, so a verifier can re-run Contains and Authorize without the clearinghouse's private vocabulary.

## 8. Corpus

`vectors/clc-d-crosswalk-vectors.json` is a JSON array of `contain-crosswalk` objects. Each object has `parent_source.capabilities` and `child_source.capabilities`. `expect.contains` is required for a negative case. `expect.reason`, when present, is the stable CLC code. `carrier_shorthand` is documentation for the runner to ignore.

The directory suite runs every JSON file under `vectors/`. This file is part of that suite. `ccx-iicp-001` remains the single published baseline vector in its own file.
