# ep-cleanroom-verifier

**Independent cleanroom Rust implementation** of the EMILIA Protocol (EP) verifier.

Written strictly from the IETF drafts (`draft-schrock-ep-authorization-receipts`) and the public conformance vectors. **No reference implementation was used** — no reading or copying from `packages/verify`, `python-verify`, or any other EMILIA reference source.

## Results (2026-07-07)

| Milestone | Status |
|-----------|--------|
| Conformance vectors | **162/162** pass |
| Canonicalization suite | **35/35** pass |
| Boundary vector `same_party_evidence_presented_as_independent` | **REJECTED** (as required) |
| Signed external statement | `EP-EXTERNAL-VERIFICATION-STATEMENT-2026-07-07.json` |
| Procedure | `EP-CONFORMANCE-RUN-OWN-IMPLEMENTATION-v1` |
| Verifier identity | `ext:verifier:emilia-cleanroom-rust` |
| Pinned public key | `keys/public.key` |

The signed statement was produced **from this Rust binary** (`conformance statement`), not from a Python or Node wrapper. Third parties can verify it with the harness in `emilia-protocol/examples/external-verification/verify-statement.mjs`.

## Build

```bash
cd rust/ep-cleanroom-verifier
cargo build --bin conformance
```

## Run conformance (162 vectors)

Point at the public vector pack (clone `emilia-protocol` or set your own path):

```bash
# Windows
python run_tests.py

# Or invoke the binary directly on one suite:
.\target\debug\conformance.exe C:\path\to\conformance\vectors\boundary.v1.json
```

Expected: `162/162 vectors passed`.

## Issue a signed external verification statement

```bash
# Generate a keypair (once; keep private-key.pem secret)
node ../../path/to/emilia-protocol/examples/external-verification/generate-key.mjs --out keys

cargo build --bin conformance

.\target\debug\conformance.exe statement \
  --vectors-dir C:\path\to\emilia-protocol\conformance\vectors \
  --private-key keys\private-key.pem \
  --output EP-EXTERNAL-VERIFICATION-STATEMENT.json \
  --verifier-id ext:verifier:emilia-cleanroom-rust \
  --verifier-name "EMILIA Cleanroom Rust Verifier" \
  --org "J Diesel NY" \
  --implementation "emilia-rust-verifier 0.1.0 (cleanroom, Rust)"
```

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

## Layout

```
Cargo.toml
src/
  lib.rs
  canonical.rs      # JCS + parse gate
  crypto.rs         # Ed25519, P-256, RSA
  external_statement.rs  # EP-EXTERNAL-VERIFICATION-STATEMENT-v1 signing
  merkle.rs
  suites/           # per-suite runners (16 suites)
  bin/conformance.rs
run_tests.py        # E2E harness (162 vectors)
```