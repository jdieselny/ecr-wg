# ep-cleanroom-verifier

**Independent cleanroom Rust implementation** of the EMILIA Protocol (EP) verifier.

Written strictly from the IETF drafts (`draft-schrock-ep-authorization-receipts`) and the public conformance vectors. **No reference implementation was used** — no reading or copying from `packages/verify`, `python-verify`, or any other EMILIA reference source.

**Canonical tree for this implementation:** `https://github.com/jdieselny/ecr-wg/tree/main/rust/ep-cleanroom-verifier`  
A historical sibling copy may exist under `emilia-protocol/cleanroom-rust-verifier` or God Terminal bridges that still discover that path — **prefer this ecr-wg tree** to avoid dual-location drift.

## Results (2026-07-13)

| Milestone | Status |
|-----------|--------|
| Conformance vectors | **193/193** pass |
| Canonicalization suite | **35/35** pass |
| Boundary vector `same_party_evidence_presented_as_independent` | **REJECTED** (as required) |
| Hostility lab (type-confusion + raw parser) | **0 findings** (see `hostility-lab/`) |
| Signed external statement (current pack) | `EP-EXTERNAL-VERIFICATION-STATEMENT-2026-07-13.json` |
| Historical statements (archive) | `archive/EP-EXTERNAL-VERIFICATION-STATEMENT-2026-07-13-d4fff94.json`, `archive/EP-EXTERNAL-VERIFICATION-STATEMENT-2026-07-11.json`, `archive/EP-EXTERNAL-VERIFICATION-STATEMENT-2026-07-07.json` |
| Procedure | `EP-CONFORMANCE-RUN-OWN-IMPLEMENTATION-v1` |
| Verifier identity | `ext:verifier:emilia-cleanroom-rust` |
| Pinned public key | `keys/public.key` |

The signed statement was produced **from this Rust binary** (`conformance statement`), not from a Python or Node wrapper. Third parties can verify it with the harness in `emilia-protocol/examples/external-verification/verify-statement.mjs`.

## Build

```bash
cd rust/ep-cleanroom-verifier
cargo build --release --bin conformance
```

## Run conformance (193 vectors)

Point at the public vector pack (clone `emilia-protocol` beside `ecr-wg`, or set `EP_CONFORMANCE_VECTORS`):

```bash
# auto-discovers ../emilia-protocol/conformance/vectors when present
python run_tests.py

# or explicit:
# Windows PowerShell
$env:EP_CONFORMANCE_VECTORS = "C:\path\to\emilia-protocol\conformance\vectors"
python run_tests.py

# Or invoke the binary directly on one suite:
.\target\release\conformance.exe C:\path\to\conformance\vectors\boundary.v1.json
```

Expected: `E2E Conformance Summary: 193/193 vectors passed.`

## Hostility lab

```bash
# requires emilia-protocol checkout (bundle + vectors) and a built binary
node hostility-lab/hostility-rust-only.mjs
# or:
node hostility-lab/run-hostility-local.mjs
```

See [`hostility-lab/README.md`](hostility-lab/README.md). Pin artifact: `hostility-lab/rust-cleanroom-jdieselny.v1.json`.

## Issue a signed external verification statement

```bash
# Private key is local-only (gitignored). Generate once if missing:
# node path/to/emilia-protocol/examples/external-verification/generate-key.mjs --out keys

cargo build --release --bin conformance

.\target\release\conformance.exe statement \
  --vectors-dir path\to\emilia-protocol\conformance\vectors \
  --private-key keys\private-key.pem \
  --output EP-EXTERNAL-VERIFICATION-STATEMENT-YYYY-MM-DD.json \
  --verifier-id ext:verifier:emilia-cleanroom-rust \
  --verifier-name "EMILIA Cleanroom Rust Verifier" \
  --org "J Diesel NY" \
  --implementation "emilia-rust-verifier 0.1.0 (cleanroom, Rust)"
```

**Never commit `keys/private-key.pem`.** Only `keys/public.key` is tracked.

## CLI modes

```
conformance <vectors.json>              # run one suite, emit JSON results
conformance verify --suite ...          # single-document verify
conformance canonicalize --input ...    # JCS canonicalize
conformance statement --vectors-dir ... # sign EP-EXTERNAL-VERIFICATION-STATEMENT-v1
conformance version
```

## Cleanroom rules

- Only IETF drafts + raw conformance vectors as specification input.
- Never read `packages/verify` or other reference implementations.
- Fail-closed on all reject vectors; boundary cases must reject for the right reason.
- Malformed suite files → non-zero exit + `REFUSE:` on stderr (no panic).

## Layout

```
Cargo.toml
src/
  lib.rs
  canonical.rs      # JCS + parse gate
  crypto.rs         # Ed25519, P-256, RSA
  external_statement.rs
  merkle.rs
  suites/           # per-suite runners (17 suites)
  bin/conformance.rs
run_tests.py        # E2E harness (193 vectors; path auto-discovery)
hostility-lab/      # differential hostility + pin
archive/            # historical external statements
keys/public.key     # pinned verifier public key (private key gitignored)
```
