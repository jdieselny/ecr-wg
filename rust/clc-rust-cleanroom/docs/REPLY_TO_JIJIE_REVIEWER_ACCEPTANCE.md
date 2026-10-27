# Reply to Jijie Wei — reviewer acceptance

**Status:** unattested draft. Unsent. Do not send this email.

Replace `COMMIT_SHA` with the commit that pins the two workflow files, locks the Option 3(a) wording, and adds this draft. Send only after that commit is on the public default branch of `jdieselny/ecr-wg`, and after the Issue #10 description line in `docs/ISSUE_10_HEADER_PATCH.md` has been prepended.

The recorded evidence she already verified stays at `8db495ac68f9d5164952e8aa24426990f3a3d9e1`, public run `40503426566`, registry row `reg-2026-001` in `CONFORMANCE.md` at `2e53271`. This reply does not claim that run used the pinned action SHAs. Those pins are in `COMMIT_SHA`.

**To:** Jijie Wei \<pki@varwof.com\>
**Subject:** Re: CLC-D containment gate and clearinghouse profile note (clc-rust-cleanroom @ 8db495a)

## Message

Hi Jijie,

I accept the provisional Conformance Reviewer role, limited to CLC-A.

In scope: the CLC-A core corpus, `vectors.json` (154) and `property-cases.json` (1,184), including verdict, reason code, and the unresolved channel.

Not in this appointment: the CLC-1.19 minor-gate semantics, CLC-D, CLC-E, and governance votes. Provisional means the scope extends to the 1.19 corpora when I have run them, and to CLC-D when 64 containment vectors and 15 authorize-chain vectors pass. I will not review my own implementation. `reg-2026-001` remains your editor bootstrap row.

I am taking option 3(a). The wording in `README.md`, `Cargo.toml`, and this note is:

**CLC-A Core Corpus Conformance (154/154 Core Vectors & 1,184/1,184 Property Cases)** under Section 12.

I am not advertising full CLC-1.19 CLC-A conformance. Section 12.1's minor gate (`param_bounds` grammar and §6.6 meet, Resolve, ConstraintUnion, and §6.2 canonicalization) stays on a later milestone. The official CLC-D corpus stays on a later milestone. The 34 `clc-d-*` vectors remain binding evidence for the clearinghouse operational profile (§13.4, §13.5). They do not carry a CLC-D claim. Issue #4 remains open.

**P12 Independence Bar:** Derived exclusively from the normative text of `draft-wei-capability-language-core-01`, public GitHub discussions, and published JSON test vectors. Zero inspection, reference, or access to varwof Go, Python, or TypeScript reference implementations. Vector files obtained by cryptographic hash and pinned on disk.

**Rerun Commitment:** Standing commitment to rerun the CLC-A core corpora upon each official specification and test vector revision published by the author.

The committed blobs at `8db495ac68f9d5164952e8aa24426990f3a3d9e1` match the corpus hashes you recorded for `1f3cbdf`:

- `vectors/clc-a-core-vectors.json`: `3c1a5672d6f107444903f06c51a71ce92451f1b45ca92c5a84a345b0de5a5bbd`
- `vectors/property-cases.json`: `f00ed1f49299293ac9fff1e10287b9642aa49d6dacf25b2b4a1be2d7103b2f07`

`COMMIT_SHA` does not change the engine. It pins the actions, makes the two workflow copies byte-identical, and locks the claim text. Local rerun of that tree on 2026-10-27, Rust 1.99.0 (`rustc 1.99.0 (b940084d7 2026-09-28)`), `RUSTFLAGS=-D warnings`. I did not run stable or 1.96.1.

| Corpus | Cases | Passed | Failed | Command |
| --- | ---: | ---: | ---: | --- |
| Unit tests | 44 | 44 | 0 | `cargo test --verbose` |
| CLC-A core vectors | 154 | 154 | 0 | `cargo run --release -- --vector vectors/clc-a-core-vectors.json` |
| Property cases | 1,184 | 1,184 | 0 | `cargo run --release -- --properties vectors/property-cases.json` |
| `clc-d-*` crosswalks (binding evidence, not a CLC-D claim) | 34 | 34 | 0 | `cargo run --release -- --crosswalk vectors/clc-d-crosswalk-vectors.json` |

Property cases also reported commutativity 1,184/1,184 and full-source coverage 1,184/1,184. `cargo check --all-targets --verbose` exited 0. `#![forbid(unsafe_code)]` remains in `src/lib.rs` and `src/main.rs`.

Not run, and not claimed: param-bounds 49, param-bounds-meet 27, meet-property 500, param-bounds-equality 11, resolve 26, constraint-union 12, constraint-union-collation 2, scheme-declaration 9, quantity 8, string-domain 24, symmetry 50, containment 64, authorize-chain 15, crosswalk 45, reason-collapse 3, mode-lattice 5, containment property 784.

CI pins in `COMMIT_SHA`:

- `actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683` (v4.2.2)
- `dtolnay/rust-toolchain@683b588c487a69d01f6c6ddf40f036ff682cc2c8` (master, 2026-10-16), with toolchain input `1.99.0`
- `.github/workflows/ci.yml` and `rust/clc-rust-cleanroom/.github/workflows/ci.yml` are the same file. GitHub Actions loads the root copy. The job working directory is `rust/clc-rust-cleanroom`.

I will prepend this line to the top of the Issue #10 description:

> **Verified Milestone Update:** See [Final Implementation Report & Passing Public CI](https://github.com/varwof/capability/issues/10#issuecomment-6441841957) (Commit `8db495a`, Rust 1.99.0, 44 unit tests, 154 core vectors, 1,184 property cases, 34 crosswalks).

Best regards,

Justin D. Kintzele
J Diesel NY / ECR-WG
