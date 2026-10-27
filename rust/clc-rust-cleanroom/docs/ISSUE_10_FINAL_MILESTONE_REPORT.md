# Issue 10 final milestone report

**Status:** unattested draft. Unsent. This file is not a GitHub comment and it has not been posted to `varwof/capability#10`.

Replace `COMMIT_SHA` with the commit that contains this hygiene diff, then paste only the comment below. Post it after that commit is on the public default branch of `jdieselny/ecr-wg`. This working tree has no new commit. `HEAD` is still `4c175c5146a97347cf364d9aa8b0c3b05e3f7cac`.

Do not commit `GROK_HANDOFF_WO138.md`. That handoff spells the vendor tokens the cleanroom leash rejects in source, workflows, and profile documents. The leash does not scan files whose names start with `GROK_HANDOFF`.

## What this diff is

Parent pin: `4c175c5146a97347cf364d9aa8b0c3b05e3f7cac`.

The runner gained two CLI names. `--properties` is an alias of `--property-cases`. `--crosswalk` reads one crosswalk file through the same `run_vector_file` path as `--vector`. Lattice, grammar, and evaluation are unchanged.

GitHub Actions reads workflows from the repository root. The executable workflow is `/.github/workflows/ci.yml` with working directory `rust/clc-rust-cleanroom`. The crate-local copy is `rust/clc-rust-cleanroom/.github/workflows/ci.yml`. The steps match. The crate-local file is not loaded by GitHub.

## Local verification (2026-10-27)

Rust `1.99.0` (`rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`). `RUSTFLAGS=-D warnings`. Working directory `rust/clc-rust-cleanroom`. No `warning:` lines in the log.

| Step | Result |
| --- | --- |
| `#![forbid(unsafe_code)]` in `src/lib.rs` and `src/main.rs` | present |
| Cleanroom leash, before this draft existed | exit 0, 20 files, vendor strings absent |
| Cleanroom leash, including this draft | exit 0, 21 files, vendor strings absent |
| `cargo check --all-targets --verbose` | exit 0 |
| `cargo test --verbose` | exit 0. Library: 44 passed, 0 failed. Binary harness: 0 tests. Doc-tests: 0 tests. |
| `cargo run --release -- --vector vectors/clc-a-core-vectors.json` | exit 0. 154 passed, 0 failed |
| `cargo run --release -- --properties vectors/property-cases.json` | exit 0. 1184 passed, 0 failed. Commutativity verified: 1,184 / 1,184. Full-source coverage verified: 1,184 / 1,184. |
| `cargo run --release -- --crosswalk vectors/clc-d-crosswalk-vectors.json` | exit 0. 34 passed, 0 failed |

The property runner's commutativity check compares the forward and reverse order. A passing 1,184 / 1,184 line means both orders agreed on every case. This sitting did not re-run stable or Rust 1.98.

The workflow file is in the tree. GitHub Actions has not run it. It becomes active when the root workflow is on `main`.

## Paste-ready comment

Implementation report for the CLC cleanroom in `jdieselny/ecr-wg`, directory `rust/clc-rust-cleanroom`.

Commit: `COMMIT_SHA`

Toolchain: Rust 1.99.0, pinned in `rust-toolchain.toml`, with `Cargo.lock` tracked. The pin is for exact deterministic reproducibility. A secondary run on stable or Rust 1.98 that matches these results demonstrates cross-toolchain stability.

Verified on that commit under `RUSTFLAGS="-D warnings"`, with `#![forbid(unsafe_code)]` in `src/lib.rs` and `src/main.rs`:

- 44/44 unit tests.
- 154/154 core vectors. Section 12 is the authority for CLC-A core conformance. Section 13.1 is the Motivation overview of Section 13.
- 1,184/1,184 property cases, with forward/reverse commutativity and full-source coverage.
- 34/34 `clc-d-*` crosswalk vectors. These are binding evidence for the Clearinghouse Operational Profile (§13.4, §13.5).

The full official CLC-D corpus is 45 `ccx-*` crosswalks, 64 containment vectors, 15 authorize-chain vectors, 3 reason-collapse vectors, and 784 property cases. The 34 `clc-d-*` vectors are binding clearinghouse operational profile evidence. They are not a replacement for that separate 45 `ccx-*` suite.

CLC-E (Evidence Profile) is a separate profile. This engine does not cover it.

Public CI is `.github/workflows/ci.yml`. It runs on `push` and `pull_request` to `main`, on `ubuntu-latest`, with the Rust 1.99.0 toolchain (`dtolnay/rust-toolchain@master`, components `clippy` and `rustfmt`) and `actions/checkout@v4`. The job working directory is `rust/clc-rust-cleanroom`. It runs `cargo check --all-targets --verbose` under `RUSTFLAGS="-D warnings"`, `cargo test --verbose`, the 154 core vectors, the 1,184 property cases, the 34 crosswalk vectors, and the cleanroom leash.

Issue #4 remains open. Implementation and credit remain with Justin D. Kintzele.
