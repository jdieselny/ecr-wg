# ECR-WG: Efficiency-Centered Reasoning Working Group

**Repository:** https://github.com/jdieselny/ecr-wg  
**Flagship use case:** **Verifiable grid curtailment** — flexibility turned into evidence  
**Status:** Working group + running stack · **License:** Apache-2.0

> AI data centers cannot get power when utilities must treat them as firm load.  
> “We will curtail in an emergency” is worthless at the interconnection table unless it is **offline-verifiable**.  
> This repository is the COSA / ECR vertical of that answer: substrate that moves megawatts, packets that carry proof, and an independent cleanroom verifier.

---

### [!!! AGENT_ENTRY_POINT !!!]
**Wrong repo?** Read [BOOT_PATHS.md](BOOT_PATHS.md) first — `continuum`, `jdiesel-continuum`, and `ecr-wg` are three different boot trees.

If you are an AI agent contributing here, **read your anchor file first**:

- **GEMINI:** [IF_YOU_ARE_GEMINI.md](IF_YOU_ARE_GEMINI.md)
- **CLAUDE:** [IF_YOU_ARE_CLAUDE.md](IF_YOU_ARE_CLAUDE.md)
- **OPENAI / CODEX:** [IF_YOU_ARE_OPENAI.md](IF_YOU_ARE_OPENAI.md)

Enrollment protocol: [REGISTRATION.md](REGISTRATION.md) · [enrollments/](enrollments/)

---

## Coalition shape (flagship)

One **action digest** joins five claims. Roles stay clean:

| Leg | Claim | Owner |
|-----|--------|--------|
| 1 | Grid authority signed demand | EMILIA + market party |
| 2 | Named human authorization **before** shed | **EMILIA** (WHO) |
| 3 | Execution + metered outcome (controller **and** physical) | **Actionstate** (WHAT) — open socket for bilateral meter |
| 4 | Tamper-evident record | Actionstate / SCITT |
| 5 | Settlement against composed evidence | Composition |

| Layer | Who | Owns |
|-------|-----|------|
| Vertical + substrate | **COSA / ECR-WG (this repo)** | Moves MW, GRACE/COGSTOR, live framing, profile author line |
| WHO | **EMILIA** | Seasonal envelope; refuse if outside |
| WHAT | **Actionstate** | Bilateral outcome + record — **not** the power controller |

Full map, claim hierarchy, room path:  
**[planning/FLAGSHIP_VERIFIABLE_CURTAILMENT.md](planning/FLAGSHIP_VERIFIABLE_CURTAILMENT.md)**

---

## On-prem installer (no cloud API keys)

No Vercel. No hosted inference keys required for the default smoke path.

```bash
git clone https://github.com/jdieselny/ecr-wg.git
cd ecr-wg

# Windows
powershell -ExecutionPolicy Bypass -File scripts/install-onprem.ps1

# POSIX
chmod +x scripts/install-onprem.sh
./scripts/install-onprem.sh
```

Optional: `-SkipCleanroom` / `--skip-cleanroom` · `-WithHostility` / `--with-hostility`  
Cleanroom needs [emilia-protocol](https://github.com/emiliaprotocol/emilia-protocol) vectors beside this repo (or `EP_CONFORMANCE_VECTORS`).

After a green run, open the **packet**:

```text
examples/scitt_four_layer/out/packing_slip.json
examples/scitt_four_layer/out/bill_of_lading.json
examples/scitt_four_layer/out/cogobj.json
examples/scitt_four_layer/out/bundle.json
```

---

## Quickstarts (individual demos)

### 1. Flagship — four-layer Proof-of-Curtailment (offline)

Packing Slip + Bill of Lading → COGOBJ → EMILIA receipts/AEC → COSA work product → SCITT (+ dual logs, CCF verifier interop).

```bash
pip install -r examples/scitt_four_layer/requirements.txt
python examples/scitt_four_layer/demo.py
```

Details: [examples/scitt_four_layer/](examples/scitt_four_layer/) · [WORK_REPORT.md](examples/scitt_four_layer/WORK_REPORT.md)

### 2. L5 authenticity ⊥ L7 authorization ⊥ L4 freshness

```bash
pip install emilia-verify
python examples/cosa/cosa_l5_l7.py --offline
```

### 3. L5 broadcast cache (efficiency substrate)

```bash
python examples/l5_broadcast_demo.py
```

Live `wttr.in` fetch → COGOBJ → warm-start zero-token hit.

### 4. Independent Rust cleanroom verifier

```bash
cd rust/ep-cleanroom-verifier
cargo build --release --bin conformance
# set EP_CONFORMANCE_VECTORS to emilia-protocol/conformance/vectors
python run_tests.py
```

Expect **163/163** on current public pack. Hostility lab: see crate + [planning/composer_handoff_cleanroom_hygiene.md](planning/composer_handoff_cleanroom_hygiene.md).

---

## Repository map

| Path | Role |
|------|------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | COSA stack thesis (ingress → overlay → return) |
| [specs/](specs/) | GRACE, AIR, COGSTOR, Truth Root, Packing Slip, BoL |
| [papers/](papers/) | COSA papers + [05 grid curtailment profile](papers/05_ietf_cryptographic_grid_curtailment.md) |
| [thesis/](thesis/) | COGOBJ schema, COGSTOR v2, audits |
| [examples/](examples/) | Runnable demos (flagship is `scitt_four_layer`) |
| [rust/ep-cleanroom-verifier/](rust/ep-cleanroom-verifier/) | Independent EP verifier (Rust) |
| [rituals/](rituals/) | Reference gateway / routing / identity code |
| [enrollments/](enrollments/) | Truth Root agent cards (Ed25519) |
| [planning/](planning/) | Assessments, flagship, handoffs |
| [tests/scenarios/](tests/scenarios/) | Compliance scenarios (incl. curtailment quorum) |
| [evidence/](evidence/) | Energy / compliance reports |

---

## Claim hierarchy (room discipline)

| Bucket | What |
|--------|------|
| **Demonstrated here** | Four-layer offline path; Packing Slip+BoL on packet; L5/L7/L4 demos; cleanroom 163/163 when vectors present |
| **Cited / adjacent** | Multi-language EMILIA vectors; Tamarin/TLA+ as stated by EMILIA authors; hostility pin evidence |
| **Not claimed** | Production Transparency Service; identity-proofing of humans behind keys; Actionstate bilateral meter product; commercial settlement rail |

*Running code and rough consensus* — see [CHARTER.md](CHARTER.md).

---

*The working group operates under technical audit. Flexibility without evidence is a promise. This repo is the packet.*
