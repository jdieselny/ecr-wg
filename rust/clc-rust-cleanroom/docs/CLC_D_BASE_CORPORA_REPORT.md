# CLC-D base corpora report — containment (64) and authorize-chain (15)

**Status:** local verification. Not posted to any issue, not yet in a public CI
run at a pushed commit.

This adds the official CLC-D base corpora. Under `CONFORMANCE.md` §2 the base
CLC-D claim is exactly these two corpora: `containment-vectors.json` (64) and
`authorize-chain-vectors.json` (15). The profile corpora — crosswalk (45),
reason-collapse (3), mode-lattice (5), containment property (784) — are a
separate, larger claim and are **not** made here.

## Corpus provenance

Both files are vendored byte-exact from `varwof/capability@80e483f` (upstream
`main`, after the merge of `jdieselny` PR #11 and the addition of `ccx-047`).
SHA-256 digests are recorded in `vectors/clc-d/PINS.sha256` and verified on
every CI run by `scripts/verify_vector_pins.py`, which now checks both
`vectors/clc-v1/` and `vectors/clc-d/`. Byte-exactness was confirmed against the
upstream commit by git blob comparison, and the files were confirmed to contain
no CR bytes so the pins are LF-stable across platforms.

## Result

Command: `cargo run --release -- --clc-d`

| Suite | Cases | Passed | Failed |
| --- | --- | --- | --- |
| containment (§13.4) | 64 | 64 | 0 |
| authorize-chain (§13.11) | 15 | 15 | 0 |
| **REQUIRED** | **79** | **79** | **0** |

Regressions on the same tree remain green: 44/44 unit tests, 154/154 core
vectors, 1,184/1,184 property cases, 34/34 `clc-d-*` crosswalks, 669/669
minor-gate required plus 50/50 optional, 14/14 corpus pins, leash exit 0.

## What the chain corpus actually required

The containment corpus needed only a shape adapter: the official vectors carry
bare `parent` and `child` grants where the binding-profile crosswalks use a
`parent_source` bundle, and `Contains` is the same relation either way.

AuthorizeWithChain needed three ordered rules, and the ordering is asserted by
the corpus rather than assumed:

1. **Empty chain fails closed** as `absent_source` (`ac-001`, §7 rule 5).
2. **Every hop must be contained in its parent**, first failure ends the call.
   This gate runs *before* operation validation, which `ac-007` pins down: a
   broken hop is reported even when the request carries no `id` at all.
3. **The request is authorized against the effective intersection of the whole
   chain, not the leaf grant.** `ac-009` is the key soundness case: the child
   carries no constraint, so authorizing against the leaf alone would allow a
   request the ancestor's `max_rows: 10` forbids. `ac-012` and `ac-015` extend
   the same point to `param_bounds`, where the §6.6 meet keeps the ancestor's
   upper bound (effective max 50), so an operation at 75 is `params_out_of_range`
   even though the leaf alone would admit it.

The effective grant is therefore built as: leaf identifier, §6.6 meet of every
link's `params`/`param_bounds`, and the union of every link's constraints (the
union axis, §13.4.4 — constraints stay outside `Contains` but are in force
through `Intersect`).

## Why the 79 is not self-graded

One expectation was flipped per corpus and the runner re-run:

| Mutation | Result |
| --- | --- |
| `contain-001` `contains` true → false | exit 1, `FAIL contain-001 contain contains=true reason=` |
| `contain-003` `contains` false → true | exit 1, `FAIL contain-003 contain contains=false reason=child_exceeds_parent` |
| `ac-002` `verdict` allow → deny | exit 1, `FAIL ac-002 ... (verdict allow != deny)` |
| `ac-009` `verdict` deny → allow | exit 1, `FAIL ac-009 ... (verdict deny != allow)` |

The `ac-009` mutation is the informative one: the run reports
`actual deny/max_rows:violated`, which means the implementation genuinely
produced the ancestor's constraint violation rather than coincidence.

One deliberate deviation is worth recording. `ac-012` carries
`"reason": ""` on an `allow` verdict. An empty string is not a reason code, and
no other vector in any suite asserts one, so `finish()` now treats an
empty-string expected reason as *not asserted*. Matching it literally would have
required inventing an empty reason code. Flagged as an upstream nit in
`ccx-046`/`resolve-027`-style editorial terms rather than silently absorbed.

## Scope

- **Evidenced here:** the base CLC-D corpora pass on the pinned target.
- **Not claimed:** the CLC-D profile corpora (45 `ccx-*` crosswalks, 3
  reason-collapse, 5 mode-lattice, 784 containment property cases).
- **Not claimed:** CLC-E.
- **Not this crate's call:** recording the claim in `conformance/registry.md`,
  which is the CLC editor's action until a reviewer scope covers the class.

## Runner commands

```
cargo run --release -- --clc-d                                         # 79 (64 + 15)
cargo run --release -- --clc-v1                                        # 669 required + 50 optional
cargo run --release -- --vector vectors/clc-a-core-vectors.json        # 154
cargo run --release -- --properties vectors/property-cases.json        # 1,184
cargo run --release -- --crosswalk vectors/clc-d-crosswalk-vectors.json # 34
python3 scripts/verify_vector_pins.py                                  # 14 pins
```
