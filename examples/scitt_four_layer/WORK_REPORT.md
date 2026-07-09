# Work Report: Four-Layer Composition (COSA × EMILIA × scitt-cose)

**Date:** 2026-07-09  
**Repo:** [jdieselny/ecr-wg](https://github.com/jdieselny/ecr-wg)  
**Path:** `examples/scitt_four_layer/`  
**Status:** Implemented, offline-green, pushed to `main`  
**Authors:** ECR-WG (human operator + Grok-Build seat E-78A3CCE1-1846-001)

---

## 1. Why this exists

ECR-WG’s curtailment / accountable-compute story needed more than three named stacks on a whiteboard. It needed a **runnable vertical** that a skeptical reader can re-execute:

1. **Was it authorized?** — named parties, exact action, fail-closed composition  
2. **Was delivery attested at the edge?** — signed work product bound to the same action  
3. **Can a third party check the pack without trusting the operator’s logs?** — SCITT envelope + inclusion receipts  

This directory is that vertical. It does **not** invent new cryptography. It composes existing work:

| Layer | Upstream | Role here |
|---|---|---|
| COSA / ECR-WG | this repo | Edge shed telemetry work product |
| EMILIA | `emilia-verify` (EP-RECEIPT-v1, EP-AEC-v1) | Authorization content + confused-deputy defense |
| SCITT client | [action-state-group/scitt-cose](https://github.com/action-state-group/scitt-cose) | Signed Statement + COSE Receipts (RFC9162 + CCF vds=2) |
| CCF (interop only) | frozen vector from scitt-cose / scitt-ccf-ledger v7.0.6 | Proves verifier accepts real Microsoft-style receipts |

Related writeups:

- Assessment: [`planning/grok_cross_stack_assessment_scitt.md`](../../planning/grok_cross_stack_assessment_scitt.md)  
- Profile paper: [`papers/05_ietf_cryptographic_grid_curtailment.md`](../../papers/05_ietf_cryptographic_grid_curtailment.md) §2.1 / §2.1.1  

---

## 2. What was built

```
examples/scitt_four_layer/
  demo.py                 # end-to-end offline composition + negatives
  ccf_client.py           # optional live SCRAPI/CCF registration client
  requirements.txt        # emilia-verify + scitt-cose (git main for vds=2)
  README.md               # how to run + honesty table
  WORK_REPORT.md          # this document
  fixtures/ccf-vds2/      # real CCF receipt frozen vector (Apache-2.0 upstream)
  out/                    # ephemeral run artifacts (gitignored)
```

### Demonstrated properties (default offline run)

| # | Property | Mechanism |
|---|---|---|
| 1 | Canonical curtailment action | `action_digest = SHA-256(JCS(action))` |
| 2 | Dual human authorization | Grid + facility EP-RECEIPT-v1, digest in *signed* claim |
| 3 | Compound authorization | EP-AEC `grid_order AND facility_ack` |
| 4 | Confused-deputy refuse | Cross-bound facility receipt → AEC fail-closed |
| 5 | Edge evidence | COSA work product, action-bound |
| 6 | SCITT envelope | COSE_Sign1 Signed Statement over JCS(bundle) |
| 7 | Multi-log inclusion | Two independent RFC9162 receipts; cross-key reject |
| 8 | Dual offline verify | `emilia_verify` + `scitt_cose` + COSA |
| 9 | CCF verifier interop | Real `vds=2` frozen receipt verifies |
| 10 | Live TS | Optional `--ccf-url` (skipped without a running log) |
| 11–12 | Negatives | Tampered statement / wrong leaf refuse |

Typical footer:

```text
RESULT: PASS — four-layer path + dual-log + CCF verifier interop demonstrated
```

---

## 3. What this is *not* (boundaries)

Keeping these explicit is part of the contribution:

- **Not a Transparency Service.** Demo logs use scitt-cose primitives in-process. Hosting a production log (CCF, other) is operational work.  
- **Not a claim that our curtailment statement is registered in Microsoft’s production ledger.** Step 9 verifies a *prior* CCF capture to prove verifier compatibility.  
- **Not identity proofing.** Keys prove enrolled parties signed; courtroom identity of the human behind a key is out of scope (same shared gap as the wider stack).  
- **Not VAP-LAP.** Packaging is a plain `ECR-POC-BUNDLE-v0.1` JSON object; domain Evidence Pack formats remain optional.  
- **Envelope ≠ content.** A verifying SCITT statement does not authorize curtailment; the embedded EMILIA AEC does.

---

## 4. Why it may matter (impact, not hype)

1. **Closes the “SCITT is hand-waved” gap** in the earlier three-stack assessment by naming and exercising a real, neutral client substrate (scitt-cose).  
2. **Preserves EMILIA’s load-bearing role** for authorization while showing clean composition with IETF-shaped transparency artifacts.  
3. **Shows multi-implementer posture:** EMILIA verify, scitt-cose (Python + its own Go story upstream), and a real CCF receipt vector — without forcing a single vendor path.  
4. **Is reproducible:** clone, `pip install -r requirements.txt`, run `demo.py`, no network required for the default path.  
5. **Is citable in the curtailment profile** (`papers/05_…` §2.1.1 Implemented Path) so IETF/program-facing text is not only aspirational.

This is intermediate infrastructure: useful to people already building receipts, logs, and grid/agent accountability — not a consumer product launch.

---

## 5. How to re-run

From repository root:

```bash
pip install -r examples/scitt_four_layer/requirements.txt
python examples/scitt_four_layer/demo.py
```

Optional live registration (requires a running SCRAPI/CCF-compatible endpoint):

```bash
python examples/scitt_four_layer/demo.py --ccf-url https://localhost:8000 --no-ccf-tls-verify
```

Note: PyPI `scitt-cose==0.1.1` is RFC9162-only. The requirements file pins **git main** so CCF `vds=2` verification works.

---

## 6. Suggested reading order for external reviewers

1. This report (scope + claims).  
2. `README.md` in this directory (run + honesty table).  
3. `demo.py` output (or re-run).  
4. `planning/grok_cross_stack_assessment_scitt.md` (architecture).  
5. Upstream [scitt-cose](https://github.com/action-state-group/scitt-cose) README (substrate boundaries).  

---

## 7. Open follow-ons (not blocking)

- Live CCF 7.x registration of *our* statement may need a `did:x509` issuer profile.  
- Production TS ops, witnessing, and key lifecycle.  
- Multi-year EMILIA evidence-record renewal inside the same pack.  
- Identity-proofing / enrollment strength of approvers.

---

*Report prepared for archive and external pointer use. Claims are limited to what the offline demo and frozen fixtures actually show.*
