# Track B1 report — Section 12.1 minor-gate corpora (CLC-1.19 CLC-A)

**Status:** local verification of an unattested draft. Not posted to any issue.

This report covers the Section 12.1 minor gate on top of the recorded Section 12
core-corpus milestone. It adds no new semantic surface to the core evaluator: it
is the parameter lattice meet, Resolve, ConstraintUnion collation, quantity
bounds, string domains, and CLC-SCHEME-v1 declarations that an advertised
CLC-1.19 implementation is required to provide.

## Corpus provenance

Twelve files under `vectors/clc-v1/` are vendored byte-exact from
`varwof/capability@80e483f` (upstream `main`, after the merge of `jdieselny`
PR #11). Each file's SHA-256 digest is recorded in
`vectors/clc-v1/PINS.sha256` and re-verified on every CI run by
`scripts/verify_vector_pins.py`. The check fails on a missing file, an altered
file, or an unpinned `.json` file in that directory.

Byte-exactness was confirmed against the upstream commit with git blob
comparison, not by inspection of the rendered content.

## Local verification

Rust `1.99.0` (`rustc 1.99.0 (b940084d7 2026-09-28)`,
`cargo 1.99.0 (5f94df478 2026-08-27)`), `RUSTFLAGS=-D warnings`,
`cargo check --all-targets` exit 0.

Command: `cargo run --release -- --clc-v1`

| Suite | Cases | Passed | Failed | Required |
| --- | --- | --- | --- | --- |
| param-bounds | 49 | 49 | 0 | yes |
| param-bounds-equality | 11 | 11 | 0 | yes |
| param-bounds-meet | 27 | 27 | 0 | yes |
| param-bounds-meet-property | 500 | 500 | 0 | yes |
| resolve | 27 | 27 | 0 | yes |
| constraint-union | 12 | 12 | 0 | yes |
| constraint-union-collation | 2 | 2 | 0 | yes |
| scheme-declaration | 9 | 9 | 0 | yes |
| quantity | 8 | 8 | 0 | yes |
| string-domain | 24 | 24 | 0 | yes |
| symmetry (optional) | 50 | 50 | 0 | no |
| **REQUIRED** | **669** | **669** | **0** | |
| **OPTIONAL** | **50** | **50** | **0** | |

`resolve` is 27 rather than 26 because `resolve-027` — the IICP crosswalk case
contributed through PR #11 — is part of the published corpus. The earlier
668-case tally predated that merge.

Prior milestones remain green on the same tree: 44/44 unit tests, 154/154 core
vectors, 1,184/1,184 property cases with commutativity and full-source coverage,
34/34 `clc-d-*` crosswalk vectors.

## Why these numbers are not self-graded

A suite that reports only passes is not evidence. Three controls were run:

1. **Expectation mutation, one case per suite family.** A wrong expectation was
   injected into `param-bounds`, `param-bounds-meet`, `resolve`,
   `constraint-union`, `scheme-declaration`, `quantity`, and `string-domain`.
   Each run exited 1 and named the failing case, for example
   `FAIL pb-001 param-bounds verdict allow != deny` and
   `FAIL resolve-027 resolve expected allow/time:violated; actual
   deny/time:violated`. The files were then restored and re-verified against the
   upstream blob hashes.
2. **Pin verification tamper test.** Flipping a single byte in
   `quantity-vectors.json` produced
   `altered quantity-vectors.json` with both digests and exit 1.
3. **Cleanroom leash probe.** The leash excludes `vectors/clc-v1` because that
   directory is third-party prose covered by the hash pins. Injecting a
   forbidden vendor token into `src/error.rs` and into a non-pinned crosswalk
   vector both still failed the leash, so the exclusion is not a hole.

## Scope

- **Claimed:** the Section 12.1 minor-gate corpora pass on the pinned target.
  Together with the recorded Section 12 core-corpus run, this is the evidence
  base for a CLC-1.19 CLC-A claim. The registry row is the CLC editor's to
  record.
- **Not claimed:** the official CLC-D corpus (64 containment, 15
  authorize-chain, 45 `ccx-*` crosswalks, 3 reason-collapse, 5 mode-lattice,
  784 containment property cases). The 34 `clc-d-*` vectors remain
  binding/profile evidence and do not carry that claim.
- **Not covered:** CLC-E, a separate profile.

## Runner commands

```
cargo run --release -- --clc-v1                                        # 669 required + 50 optional
cargo run --release -- --vector vectors/clc-a-core-vectors.json        # 154
cargo run --release -- --properties vectors/property-cases.json        # 1,184
cargo run --release -- --crosswalk vectors/clc-d-crosswalk-vectors.json # 34
python3 scripts/verify_vector_pins.py                                  # 12 pins
```
