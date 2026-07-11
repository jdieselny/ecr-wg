# Four-layer composition demo: COSA + EMILIA + scitt-cose (+ CCF)

Turns the end-to-end path from
[`planning/grok_cross_stack_assessment_scitt.md`](../../planning/grok_cross_stack_assessment_scitt.md)
into a runnable demonstration — including **dual independent SCITT logs** and
**real Microsoft CCF receipt verification** (offline frozen vector).

**Work report (scope, impact, non-claims):** [WORK_REPORT.md](./WORK_REPORT.md)

```
Packing Slip + BoL ──┐
COGOBJ (ingress)   ──┤
COSA work product  ──┼──► PoC bundle ──► SCITT Signed Statement ──┬── demo log A (RFC9162)
EMILIA receipts/AEC ─┘                         │                  ├── demo log B (RFC9162)
                                               │                  └── optional live TS / CCF
                                               │
                     fixtures/ccf-vds2 ─────────┴── real CCF vds=2 receipt (offline crypto proof)
```

**Ingress (Continuum remnants):** when the curtailment order is written into the pack, the demo seals a [Packing Slip](../../specs/primitives/packing-slip.md) and wraps it in a [Bill of Lading](../../specs/primitives/bill-of-lading.md), then attaches both to `bundle.ingress` and to a [COGOBJ](../../thesis/COGOBJ_SCHEMA.md). Artifacts: `out/packing_slip.json`, `out/bill_of_lading.json`, `out/cogobj.json`.

## Install

```bash
pip install -r examples/scitt_four_layer/requirements.txt
```

| Package | Role |
|---|---|
| `emilia-verify` ≥ 1.1.0 | EP-RECEIPT-v1 + EP-AEC-v1 |
| `scitt-cose` **from git main** | Signed Statements + receipts for **vds=1 and vds=2 (CCF)** |
| `cryptography` | Ed25519 / ES keys |
| `requests` | Only for optional `--ccf-url` live path |

> **Why git main?** PyPI `scitt-cose==0.1.1` rejects CCF receipts
> (`unsupported verifiable data structure`). Main supports
> `RFC9162_SHA256` (vds=1) **and** `CCF_LEDGER_SHA256` (vds=2).

## Run (offline — always works)

```bash
python examples/scitt_four_layer/demo.py
```

Expected:

```text
RESULT: PASS — four-layer path + dual-log + CCF verifier interop demonstrated
```

## Optional: live Transparency Service

If you have Docker + a local [scitt-ccf-ledger](https://github.com/microsoft/scitt-ccf-ledger)
(or any SCRAPI-ish stub that returns COSE receipts):

```bash
# example after ./docker/run-dev.sh in scitt-ccf-ledger
python examples/scitt_four_layer/demo.py \
  --ccf-url https://localhost:8000 \
  --no-ccf-tls-verify
```

**Honesty about live CCF 7.x:** many nodes require a `did:x509` issuer on the
Signed Statement. This demo’s issuer is a URL (`https://issuer.ecr-wg.example/poc`),
which may return HTTP 400. That is a **soft** outcome — the offline path still
PASSes. A green live path needs either:

1. a vds=1 stub TS that accepts URL issuers, or  
2. a did:x509-capable issuer (e.g. pyscitt + X5Chain) aimed at real CCF.

The offline **frozen vector** (`fixtures/ccf-vds2/`) already proves our verifier
accepts a **real** CCF-issued receipt captured from scitt-ccf-ledger v7.0.6.

## What the steps show

| # | What |
|---|---|
| 1–2 | Canonical `grid.curtailment` action; **Packing Slip + Bill of Lading** ingress |
| 3–4 | EMILIA receipts; EP-AEC allow |
| 5 | Confused-deputy refuse (cross-bound facility receipt) |
| 6–7 | COSA work product + **COGOBJ**; SCITT Signed Statement over JCS bundle (w/ ingress) |
| 8 | **Dual independent RFC9162 logs** + cross-key reject |
| 9 | Dual verify: `emilia_verify` + `scitt_cose` + COSA + BoL |
| 10 | **Real CCF vds=2 frozen receipt** verifies offline |
| 10 | Optional live `--ccf-url` register of *our* statement |
| 11–12 | Tamper + wrong-leaf negatives |

## Layer honesty

| Layer | This demo | Not this demo |
|---|---|---|
| **COSA** | Signed edge work product | Full L5 broadcast runtime |
| **EMILIA** | Real `emilia-verify` AEC | WebAuthn Class-A, multi-year evidence-record |
| **Packaging** | Plain `ECR-POC-BUNDLE-v0.1` | VAP-LAP schema |
| **scitt-cose** | Client substrate + dual receipts | Operating a production TS |
| **CCF** | Frozen real receipt + optional live client | Guaranteed live green without did:x509 |

Envelope vs content: **EMILIA owns authorization content; scitt-cose is the IETF wire envelope and inclusion proof.**

## Artifacts

Written to `out/` each run (gitignored):

- `bundle.json`, `aec.json`, `statement.cose`
- `receipt-a.cose`, `receipt-b.cose`, `transparent.cose`
- `keys.json` (public keys + leaf/root + live status)
- `receipt-live.cose` when live registration succeeds

## See also

- Assessment: [`planning/grok_cross_stack_assessment_scitt.md`](../../planning/grok_cross_stack_assessment_scitt.md)
- Paper: [`papers/05_ietf_cryptographic_grid_curtailment.md`](../../papers/05_ietf_cryptographic_grid_curtailment.md)
- scitt-cose: <https://github.com/action-state-group/scitt-cose>
- scitt-ccf-ledger: <https://github.com/microsoft/scitt-ccf-ledger>

<!-- AGENT-ATTRIBUTION
agent_id: E-78A3CCE1-1846-001
thumbprint: MCowBQYDK2VwAyEAds0tVFKCGmosef/mvWT496Kg0bQ7YW1W0la/AGcMwoI
role: Grok-Build
-->
