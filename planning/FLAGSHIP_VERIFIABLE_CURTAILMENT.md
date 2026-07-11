---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-07-11
file_role: flagship
status: working
---

# Flagship: Verifiable Curtailment (coalition packet)

**Repository:** https://github.com/jdieselny/ecr-wg  
**On-prem smoke (no cloud keys):**

```bash
# Windows
powershell -ExecutionPolicy Bypass -File scripts/install-onprem.ps1

# POSIX
./scripts/install-onprem.sh
```

This is the **worked example** the accountability composition has been missing: a high-stakes use case with a running multi-party path, not a better slide.

---

## One digest, five claims

| # | Claim | Owner | Status in this repo |
|---|--------|--------|---------------------|
| 1 | Grid authority’s signed curtailment demand | EMILIA + market party | **Running** — EP-RECEIPT-v1 `grid_order` |
| 2 | Named human authorization **before** shed | EMILIA (WHO / seasonal envelope) | **Running** — AEC + fail-closed; cleanroom + hostility evidence |
| 3 | Execution + **metered outcome** (controller *and* physical) | **Actionstate (WHAT)** | **Partial** — COSA edge work product (controller side); **Steven’s bilateral meter leg is the open socket** |
| 4 | Tamper-evident record | Actionstate / SCITT | **Running** — scitt-cose Signed Statement + dual RFC9162 (+ CCF vds=2 verify) |
| 5 | Settlement only against composed evidence | Composition | **Pack ready** — bundle + digests; commercial settlement product not claimed |

**Change one bound field → verification fails.** That is the property the interconnection table does not have today when someone says “we’ll curtail in an emergency.”

---

## Roles (do not blur)

| Party | Layer | Owns |
|-------|--------|------|
| **J / ECR-WG / COSA** | Vertical + execution substrate | Moves megawatts, GRACE/COGSTOR/overlay, frames the live story, author on profile draft |
| **Iman / EMILIA** | WHO | Human authorization envelope; gate refuses orders outside it |
| **Steven / Actionstate** | WHAT | Bilateral outcome + record (controller-reported vs physically-measured as **two claims**); log anchor — **not** the power controller |

---

## What runs offline today (default)

```
HUMAN / ISO order
  → Packing Slip + Hash          (ingress cargo)
  → Bill of Lading               (signed transport)
  → COGOBJ                       (same digests on cognitive packet)
  → EP-RECEIPT ×2 + EP-AEC       (WHO, fail-closed on confused deputy)
  → COSA work product            (edge shed telemetry, action-bound)
  → SCITT COSE_Sign1 + dual logs (tamper-evident envelope)
```

| Command | What it proves |
|---------|----------------|
| `python examples/scitt_four_layer/demo.py` | Full flagship path + negatives + CCF verifier interop |
| `python examples/cosa/cosa_l5_l7.py --offline` | L5 authenticity ⊥ L7 authorization ⊥ L4 freshness |
| `python examples/l5_broadcast_demo.py` | Prefill bypass / zero-token warm cache (efficiency substrate) |
| Cleanroom via installer or `rust/ep-cleanroom-verifier` | Independent Rust verifier on public vectors (163/163) |

Artifacts after flagship run: `examples/scitt_four_layer/out/`  
→ `packing_slip.json`, `bill_of_lading.json`, `cogobj.json`, `bundle.json`, COSE objects.

---

## Proof stack (honest claim hierarchy)

**Demonstrated in this repo / adjacent cleanroom:**

- Runnable four-layer composition (COSA × EMILIA × scitt-cose)
- Independent Rust cleanroom: **163/163** conformance vectors
- Hostility-cleared pin (local lab; track via Composer hygiene handoff)
- Ingress envelope (Packing Slip + BoL) on the written packet

**Cited from EMILIA / coalition proof work (not re-proven inside ecr-wg alone):**

- Multi-language vector agreement
- Tamarin / TLA+ model results as stated by EMILIA authors

**Not claimed:**

- Production Transparency Service operation
- Courtroom identity-proofing of the human behind an enrolled key
- Steven’s bilateral physical meter attestor (open design question he owns)
- Paying settlement rail

---

## Room path (HotRFC / agentproto)

**Logistics (honest):** HotRFC slots are on the order of ~4 minutes. Steven currently holds an awarded slot; J is **not** on-site (upstate NY) and remote attendance is not assumed. The public asset is the **cloneable stack**, not a physical mic.

1. **Name the coalition** — EMILIA + COSA + Actionstate.  
2. **Show the packet** — point at the repo + on-prem installer; open `bundle.json` / COGOBJ; flip one field; show refuse.  
3. **Assign the open socket** — Steven: interval data in bilateral attestation as-is, or dedicated meter attestor role?  
4. **Do not blur roles** — J owns vertical framing *in writing / repo*; Iman owns WHO at the table; Steven owns WHAT (and the short mic if that is the slot that exists).

---

## Related paths

| Path | Role |
|------|------|
| `papers/05_ietf_cryptographic_grid_curtailment.md` | Profile draft |
| `examples/scitt_four_layer/` | Flagship runnable |
| `rust/ep-cleanroom-verifier/` | Independent verifier |
| `planning/composer_handoff_cleanroom_hygiene.md` | Doc/statement hygiene |
| `planning/grok_cross_stack_assessment_scitt.md` | Dimensional assessment |
| `specs/primitives/packing-slip.md` + `bill-of-lading.md` | Ingress primitives |
| `thesis/COGOBJ_SCHEMA.md` | Packet schema (v2.1 ingress) |

---

*Flexibility without evidence is a promise. This repository turns flexibility into a packet the room can run.*
