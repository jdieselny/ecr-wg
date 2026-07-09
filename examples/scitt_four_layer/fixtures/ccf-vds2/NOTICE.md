# Fixture provenance: real CCF SCITT receipt (vds=2)

These bytes are the **append-only frozen vector**
`test-vectors/v1/valid-ccf-vds2/` from
[action-state-group/scitt-cose](https://github.com/action-state-group/scitt-cose)
(Apache-2.0), captured **2026-06-26** against **scitt-ccf-ledger v7.0.6**
(VIRTUAL mode).

| File | Role |
|---|---|
| `statement.cose` | pyscitt `did:x509` Signed Statement (ES256) |
| `receipt.cose` | CCF-issued COSE Receipt (`vds=2` / ccf.v1, ES384) |
| `log-key.pub` | CCF service public key used to verify the receipt |
| `issuer-key.pub` | Statement issuer public key |
| `payload.bin` | Opaque statement payload |
| `expected.json` | Expected verifier outcomes (incl. reconstructed root) |

They are **not** our Proof-of-Curtailment statement. They prove that the same
`scitt-cose` `verify_receipt` path used in this demo accepts a **real**
Microsoft CCF receipt (VDS ≠ RFC9162). Live registration of *our* bundle
requires a running CCF node (`SCITT_CCF_URL`); see the demo README.
