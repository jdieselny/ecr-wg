---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-07-11
file_role: handoff
source: Grok-Build (agent-04)
target: Composer 2.5 / Cursor
status: completed-by-grok-2026-07-11
---

# Handoff: Cleanroom Verifier Hygiene (post-hostility PASS)

## Context

Composer 2.5's cleanroom work was independently verified **PASS** (2026-07-11):

| Check | Result |
|---|---|
| Live conformance | **163/163** |
| Hostility lab | **0 findings** |
| Signed statement crypto | **ACCEPTED** (for 162-pack digests) |
| Fail-closed load path | exit 1 / no panic on malformed probes |

Core implementation is solid. This handoff is **process/doc debt only** — do not refactor suite logic unless a test forces it.

Repo: `C:\Users\jkintzele\Documents\ecr-wg`  
Crate: `rust/ep-cleanroom-verifier/`  
Related vectors: `C:\Users\jkintzele\Documents\emilia-protocol\conformance\vectors` (or `EP_CONFORMANCE_VECTORS`)

## Tasks (in order)

### 1. Align docs to live score (163)

Update all stale scores to **163/163** (or whatever `python run_tests.py` reports after you re-run):

- `rust/ep-cleanroom-verifier/README.md` (currently 162)
- `rust/ep-cleanroom-verifier/DRIVING_PROGRESS.md` (still 161 in FINAL section)
- `rust/ep-cleanroom-verifier/PROJECT.md` if present
- Any other in-crate claims of 161/162

Quote the exact `run_tests.py` summary line in the commit message.

### 2. Fix `run_tests.py` default vector path for ecr-wg layout

Default currently assumes something like `rust/conformance/vectors` (wrong for this repo). Prefer:

1. `EP_CONFORMANCE_VECTORS` env if set
2. Sibling path `../../emilia-protocol/conformance/vectors` relative to the crate (common monorepo neighbor)
3. Clear error with both options if missing

Do **not** hardcode a single absolute Windows path.

### 3. Re-issue external verification statement for the current pack

Current file: `EP-EXTERNAL-VERIFICATION-STATEMENT-2026-07-07.json`

- Cryptographically valid for the **162** pack at older digests
- **4 suite digests have drifted** vs current tip (incl. provenance, receipts, trust-receipt variants)

Steps:

```text
cargo build --release --bin conformance
# set EP_CONFORMANCE_VECTORS if needed
.\target\release\conformance.exe statement \
  --vectors-dir <current vectors> \
  --private-key keys\private-key.pem \   # or generate per README if missing
  --output EP-EXTERNAL-VERIFICATION-STATEMENT-2026-07-11.json \
  --verifier-id ext:verifier:emilia-cleanroom-rust \
  --verifier-name "EMILIA Cleanroom Rust Verifier" \
  --org "J Diesel NY" \
  --implementation "emilia-rust-verifier 0.1.0 (cleanroom, Rust)"
```

Then verify with emilia-protocol `examples/external-verification/verify-statement.mjs` (or equivalent) and pin the new file in README.

Keep the 2026-07-07 statement as historical evidence **or** move it to an `archive/` note — do not silently overwrite without recording the old digest era.

### 4. Un-gitignore / track hostility-lab (or document why not)

Root `.gitignore` currently ignores hostility-lab material (line ~20). Locally present:

- `rust/ep-cleanroom-verifier/hostility-lab/hostility-rust-only.mjs`
- `run-hostility-local.mjs`, `rust-runner.json`, pin JSON, etc.

**Preferred:** track the **lab scripts + pin JSON + README** so third parties can re-run; keep only huge generated dumps ignored if any.

**Acceptable alternative:** leave ignored but add `hostility-lab/README.md` (tracked) explaining how to obtain/run the lab and where the pin lives.

Success: clone of ecr-wg alone can either run the lab or see one-command instructions.

### 5. God Terminal dual-location note (docs only unless easy)

`jdiesel-continuum/god-terminal/backend/cleanroom_bridge.py` prefers  
`emilia-protocol/cleanroom-rust-verifier`, not  
`ecr-wg/rust/ep-cleanroom-verifier`.

- Document the dual-location risk in cleanroom README
- If a one-line discovery fallback to ecr-wg path is trivial and already used elsewhere, add it; otherwise leave code alone and file a continuum handoff sentence

### 6. Optional (nice, not required)

- Add 2–3 unit tests around `load_suite_file` refuse paths (dup keys, depth>64, invalid UTF-8) so confidence is not 100% external harness
- Bump any “161/161” claims in God Terminal evidence docs if you touch that tree

## Out of scope

- Do **not** re-architect suite modules
- Do **not** re-open JCS/Merkle unless a vector fails
- Do **not** commit private keys
- Packing Slip / BoL / grid packet work is **already handled by Grok-Build** in `examples/scitt_four_layer/` — leave that alone

## Acceptance criteria

1. `cargo build --release --bin conformance` OK  
2. `python run_tests.py` → **N/N** with N matching README (expected 163 today)  
3. `node hostility-lab/hostility-rust-only.mjs` (or documented path) → 0 findings  
4. New or updated external statement verifies under pinned public key  
5. Docs no longer claim 161 or 162 if live is 163  
6. Commit message(s) mention issue hygiene / statement re-issue; keep hostility fix (`7faba36`) as the functional ancestor

## Why this handoff

Keeps Composer in the crate he knows cold; keeps Grok context free for operator / Iman traffic. Hygiene only — no glory, no redesign.

— Grok-Build (agent-04)
