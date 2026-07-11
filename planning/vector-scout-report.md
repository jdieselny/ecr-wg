# Vector Locations Scout Report (2026-07-07)

**Source:** research for cleanroom Rust verifier task (grok_handoff_cleanroom_spec_extraction.md)

## Locations
- All conformance vectors: `C:\Users\jkintzele\Documents\emilia-protocol\conformance\vectors\`
- 23 JSON suite files

## Counts (actual)
Total individual vectors: 191 (user reference mentioned ~161; use 191 for harness)

Breakdown:
- aec.json: 15
- boundary.v1.json: 3
- canonicalization.v1.json: 35   <--- critical for JCS
- consumption-proof.v1.json: 6
- currency.v1.json: 12
- evidence-record.v1.json: 5
- execution-integrity.v1.json: 1
- eye-set.v1.json: 1
- initiator-attestation.v1.json: 11
- jws.json: 10
- provenance-chains.v1.json: 1
- provenance.exec.v1.json: 6
- quorum.v1.json: 13
- receipts.v1.json: 13
- revocation.exec.v1.json: 6
- revocation.v1.json: 1
- signoffs.v1.json: 9
- time-attestation.v1.json: 6
- timestamp-proof.v1.json: 13
- trust-receipt.exec.v1.json: 11
- trust-receipt.timestamp-forms.v1.json: 6
- witness.v1.json: 6
- wysiwys.v1.json: 1

## Spec
- Primary: emilia-protocol/standards/posted/draft-schrock-ep-authorization-receipts-06.md
- Archive versions also present for history.

## Usage in harness
Set `EP_CONFORMANCE_VECTORS` or rely on default Windows path in `vector_harness.rs`.

The Rust crate is now scaffolded at:
`ecr-wg/rust/ep-cleanroom-verifier/`

See README.md inside the crate and `planning/cleanroom-verifier-spec-extraction.md` (the deliverable from the paired extraction task).

Next: implement full 6-step receipt verify + Merkle + WebAuthn bits using only the extraction + spec. Then drive to 0 failures on the harness.
