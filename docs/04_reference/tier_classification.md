# Tier Classification

**Tier:** git-tracked.
**Status:** STUB — formal contract at `/SANITIZATION.md`. This doc will be the practical lookup table.

## What this doc will contain

A decision table: given a specific content type, which tier does it belong in?

Structure: a large table with columns for Content Type, Tier, Rationale, Example. Covers:

- Persona voice rules (tracked)
- Persona active state (local)
- Brand colors / logo / typography (tracked)
- Brand asset files / logo PNGs / .pptx templates (local, because binaries with potential IP)
- Product category rules (tracked)
- Product-specific SKU names for publicly-marketed products (tracked)
- Product internal codenames (local)
- Open product questions pending internal answer (local)
- Competitor public specs with source citations (tracked if carefully framed, local otherwise)
- Competitor pricing / win-loss / deal-level intel (local)
- North Star aspirational missions (tracked)
- In-flight roadmap with customer names (local)
- Calibration findings generic (tracked)
- Calibration findings with customer / deal specifics (local)
- Session scratchpads (always local)
- Collaborator cards (local)

Plus gray-area cases:
- Hashed or anonymized customer data (local — even anonymization isn't reliable)
- Public-record customer wins where Datacom already has a case study (tracked OK if referencing the public case study)
- Dated public statements from a competitor (tracked OK with date and source)

## Why this doc matters

The `/SANITIZATION.md` contract at repo root states the rules in principle. This doc states them in practice: "here is your specific content, here is where it goes." Reduces judgment calls to lookups.

## When this will be written

After enough gray-area decisions have accumulated that a lookup table is more useful than case-by-case reasoning. Rough threshold: ~20 documented tier decisions across the team.

## Related docs

- `/SANITIZATION.md` (at repo root) — the formal contract
- `../01_architecture/two_tier_model.md` — the deep architectural doc (stub)
- `../03_operator_guide/sanitization_review.md` — the audit process (stub)
