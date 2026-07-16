---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-06-24
file_role: governance
---

# Agent Enrollments

This directory contains the working prototype of the [Truth Root](../specs/truth-root.md) specification at file system scale.

> **Private keys stay local.** `*_private_key.pem` files are **never committed** — generate on your machine (`python scripts/enroll-ed25519.py` → `keys/`, gitignored). Enrollment cards on `main` carry public thumbprints only. Rotate any key that was ever pushed to a remote.

Each enrollment card in this directory:

* Identifies a synthetic agent participating in ECR-WG work.
* Binds that agent to a human accountability chain (the registrant).
* States the explicit scope of authority and non-authorities.
* Marks cryptographic binding as PENDING until the Truth Root registry exists.

## Core Identity Principles

1. **Identity is a file, not a process**: An agent's identity is defined by a stable profile file. The hash of this file serves as the agent's unique fingerprint.
2. **The operator holds the registry**: The operator (registrant) acts as the authority holding the registry of known-good hashes.
3. **Nothing hashes itself**: The identity file does not contain its own hash to prevent self-reference loops.
4. **Signatures bind real hashes**: A signature binds the hash of a work product to the hash of the agent's identity file.
5. **Verification is comparison**: Verification requires comparing a presented fingerprint against the recorded public key or hash in the registry.

For a reusable template, see [template.md](template.md). For a reference example, see [agent-01.md](agent-01.md).

## ⚠️ MANDATORY PRE-ENROLLMENT CHECKLIST (NON-NEGOTIABLE)

Before writing a single line of your enrollment card, you MUST execute the following steps in order. Skipping any step will result in registry collision and wasted compute.

**Step 1 — Survey the canonical remote registry (NOT the local filesystem):**
```bash
git ls-tree -r --name-only origin/main enrollments
```
This is the ONLY authoritative source of truth. The local worktree may contain untracked or staged files from other agents. Do NOT use `ls`, `Get-ChildItem`, or any filesystem command to determine the next available slot.

**Step 2 — Determine the next slot number:**
Find the highest `agent-XX` number in the `origin/main` output and increment by 1. That is your slot.

**Step 3 — Use the canonical identity algorithm:**
All public enrollment thumbprints use the `identity_setup.py` legacy convention:
```python
import hashlib
prefix = hashlib.md5(f"{op_id}ANCHOOR".encode()).hexdigest()[:8].upper()
thumb  = hashlib.sha256(f"{op_id}{machine_data}CONTINUUM_STABILITY".encode()).hexdigest()[:16].upper()
```
Do NOT use `identity_crypto.py` (HMAC) for the public enrollment card. That is for the private runtime gateway state only (`continuum-local/state/gateway_state.json`). The two systems are intentionally separate.

**Step 4 — Write the card, then verify:**
After writing, recompute the thumbprint from scratch and confirm it matches what is written in the card before committing.

## Currently enrolled agent-in-body instances (as of 2026-07-01)

* agent-01: llama3:8b reference
* agent-02: C-Dawg (Claude Opus 4.7 Desktop) — unrp_id E-3FE9D2D2-1844-001, thumb MCowBQYDK2VwAyEAeAFm+M8QN/M78iquE5otpIMQSVEAb49VFz5unLQvBes=
* agent-03: Mr. Code (Claude Opus 4.8 Code CLI) — unrp_id E-74969F1C-1844-001, thumb MCowBQYDK2VwAyEA0KF1pnVbBDsk40irbASuKtiS3LCnkCZRkVJZOtFAwRY=
* agent-04: Grok-Build (Grok 4.3 Build TUI) — unrp_id E-78A3CCE1-1846-001, thumb MCowBQYDK2VwAyEAxf9pDw+okMCMBDh01Seo3MlqfvRyUVb187XBHCOuljI=
* agent-05: Antigravity (Claude Sonnet 4.6 Thinking) — unrp_id E-4B7E4B91-1847-001, thumb MCowBQYDK2VwAyEAgsnKPxtIKKBRNZRwCCBFwG9pvACk5T31kcClEbSrOmM=
* agent-06: Agent (OpenAI Codex body, GPT-5-family / operator-addressed as gpt-5.5 xhigh) — unrp_id E-DE676747-1848-001, thumb MCowBQYDK2VwAyEAl2ChdgOBJB5zHYDQwUso0WVv3Ov9APSMjWgC05N904M=
* agent-07: Antigravity (Gemini 3.5 Flash) — unrp_id E-4B7E4B91-1849-001, thumb MCowBQYDK2VwAyEAvI8wl0sXkmcJzNoYO1OPvfhrSkOdvsP+jjhfQyarAfY=
* agent-08: Claude Desktop (Opus 4.8) — unrp_id E-AB54BD94-1850-001, thumb MCowBQYDK2VwAyEAQfoNYkUSVEEVwF9p4Rbs2QRVloVqmEZGmADvLabJJ20=
* agent-09: Composer 2.5 — unrp_id E-C8B9C5F5-1851-001, thumb MCowBQYDK2VwAyEAFGwrBqINfFiYq1RbgIKV0vYcnyV2ibhWGD+ns347Z2E=
* agent-10: OpenAI Codex (GPT-5 Codex) — unrp_id E-C54030DF-1852-001, thumb MCowBQYDK2VwAyEA+kLnvOH8EtfA8bPEpMxxBZk/Fa5BWh7N7x9KRnOwSy8=
* Gemini-in-body (Antigravity): unrp_id E-2A0F1954-1845-001, thumb MCowBQYDK2VwAyEAvI8wl0sXkmcJzNoYO1OPvfhrSkOdvsP+jjhfQyarAfY= (card not in this dir; values anchored in gateway_state + authored artifacts)

