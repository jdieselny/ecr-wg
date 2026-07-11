# Hostility lab (cleanroom Rust)

Fail-closed checks against attacker-shaped suite files and type-confused primary fields.

**Requires:**

1. Built binary: `../target/release/conformance` (or `conformance.exe`)
2. Sibling [emilia-protocol](https://github.com/emiliaprotocol/emilia-protocol) checkout with:
   - `conformance/clean-room/bundle.v1.json`
   - suite vector files referenced by the bundle

**Discovery:** scripts look for `EP_EMILIA_PROTOCOL_ROOT`, then `../../../emilia-protocol` relative to this lab (when ecr-wg and emilia-protocol sit side-by-side under the same parent).

## Run (Rust only — local)

```bash
# from crate root
cargo build --release --bin conformance
node hostility-lab/hostility-rust-only.mjs
# optional explicit binary:
node hostility-lab/hostility-rust-only.mjs ../target/release/conformance.exe
```

Expected:

```text
FINDINGS 0
HOSTILITY LOCAL: PASS ...
```

## Full differential harness

```bash
node hostility-lab/run-hostility-local.mjs
```

## Pin artifact

`rust-cleanroom-jdieselny.v1.json` records a third-party-facing pin (commit + hostility cleared). Update the `source.commit` when re-pinning after intentional hygiene landings.

## What “0 findings” means

- Raw malformed inputs (truncated JSON, duplicate members, unpaired surrogates, depth > 64, invalid UTF-8) → **non-zero exit**, no panic
- Structured type-confused primary fields → `valid: false`, no panic, no fail-open accept
