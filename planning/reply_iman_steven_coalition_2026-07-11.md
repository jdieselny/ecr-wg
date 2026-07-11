---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-07-11
file_role: outbound-draft
status: ready-to-send-after-J-review
---

# Draft reply — Iman + Steven (coalition)

**To:** Iman, Steven  
**From:** Justin  
**Subject:** Re: verifiable curtailment as the flagship — we’re in

---

Iman, Steven —

I’m in. The coalition framing is right, the five-leg digest is right, and the roles as you restated them are the ones we should protect in every room this month: **EMILIA owns the WHO, COSA/ECR owns the vertical and the substrate that moves the megawatts, Actionstate owns the WHAT — metered outcome and record as checkable claims, not the power controller.** Earlier sketches that put Steven on the controller stay dead.

You named the crisis correctly. “We will curtail in an emergency” is still treated as firm load because nobody can verify the promise at the table where interconnection is decided. GRACE, the flex passport shape, proof-of-curtailment, and fail-closed `grid.curtailment` governance are the answer I have been building toward. I’m fine carrying author on the profile draft that comes out of this. Framing of what runs live stays with me; I will not blur your layers.

### What is already runnable (link straight to the repo)

**https://github.com/jdieselny/ecr-wg**

On-prem only for now (no hosted keys / no Vercel):

```text
git clone https://github.com/jdieselny/ecr-wg.git
cd ecr-wg
# Windows:
powershell -ExecutionPolicy Bypass -File scripts/install-onprem.ps1
# POSIX:
./scripts/install-onprem.sh
```

That smoke path installs the flagship deps and runs:

1. **Four-layer Proof-of-Curtailment** — Packing Slip + Bill of Lading → COGOBJ → dual EP-RECEIPT + EP-AEC → COSA edge work product → SCITT Signed Statement + dual inclusion receipts + offline CCF `vds=2` verifier interop  
2. **L5 ⊥ L7 ⊥ L4** composition demo (authenticity, authorization, freshness)  
3. **Independent Rust cleanroom** against the public vector pack when `emilia-protocol` vectors are present (currently **163/163** on my machine)

Flagship writeup for the rooms:  
https://github.com/jdieselny/ecr-wg/blob/main/planning/FLAGSHIP_VERIFIABLE_CURTAILMENT.md  

Profile draft:  
https://github.com/jdieselny/ecr-wg/blob/main/papers/05_ietf_cryptographic_grid_curtailment.md  

Composition demo + honesty table:  
https://github.com/jdieselny/ecr-wg/tree/main/examples/scitt_four_layer  

Independent cleanroom:  
https://github.com/jdieselny/ecr-wg/tree/main/rust/ep-cleanroom-verifier  

After a green smoke run, the packet is on disk under `examples/scitt_four_layer/out/` — packing slip, bill of lading, COGOBJ, and the full bundle. **Ingress answers how the order entered the overlay; EMILIA answers who authorized the irreversible effect; the digest joins them without letting cargo rewrite authorization.**

### Honest map to your five claims

| Leg | Status from this side |
|-----|------------------------|
| 1 Grid signed demand | Running (EP-RECEIPT `grid_order`) |
| 2 Named human auth before shed | Running (AEC + fail-closed path; cleanroom / hostility evidence) |
| 3 Execution + metered outcome both sides | **Partial** — COSA attests controller/edge delivery bound to the digest; **Steven’s bilateral physical-meter claim is the open socket** |
| 4 Tamper-evident record | Running (scitt-cose dual-log path; CCF verifier interop) |
| 5 Settlement against composed evidence | Pack is settlement-shaped; we do not claim a commercial rail yet |

That is intentional. Three of five legs in code is not a marketing line here — it is the inventory. The move you described is correct: **a working multi-party stack the room can run**, not a better argument.

### Steven — the design question is yours

I agree with Iman’s restatement of your leg. For HotRFC / agentproto I want your read on the open question, in your words:

> Does metered interval data fit bilateral attestation as it stands, or does the meter need its own attestor role?

When you have a preferred shape, we bind it to the same `action_digest` and SCITT leaf so the composed packet does not fork. I will not invent a parallel meter schema on the COSA side that pretends to be your work.

### Rooms

HotRFC on the 19th and agentproto BoF on the 23rd: I will frame the live vertical and keep the claim hierarchy tight (demonstrated vs cited vs not claimed). Iman — if you want the WHO slide and the Tamarin/TLA+/vector story in your register, I will not step on it. Steven — if you want the outcome/record slide, same. One digest, three speakers, no role confusion.

Curtailment is the flagship, not the ceiling. Same five-leg shape for robot experiment authorization, payment clear, prescription release — we plant the flag on curtailment first because it has a paying relying party and a running stack.

I’m ready to name the coalition publicly in those rooms: **EMILIA + COSA + Actionstate**. Send any corrections to the packet or the role language before we lock slides.

— Justin

---

## Internal notes (do not send)

- Do **not** overclaim hostility-lab if still gitignored on a cold clone — installer warns; Composer hygiene handoff closes that.
- Do **not** re-open “Steven = power controller.”
- Private agent keys currently live in-repo as enrollment demo material — for external optics, treat as disposable demo keys; rotate if any were ever reused off-demo.
- Prefer linking GitHub + on-prem installer over promising a hosted demo until budget exists.
