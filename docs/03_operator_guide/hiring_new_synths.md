# Hiring New Synths

**Tier:** git-tracked.
**Status:** STUB — Session 2 build target. Content will be authored alongside the first real hire (Fred).

## What this doc will contain

The end-to-end procedure for adding a new persona to Continuum. Topics:

- **When to hire.** Signals that SEV is being stretched across incompatible domains and a dedicated persona would produce better output. Counter-signals: adding personas for vanity or for domains too narrow to justify a card.
- **The hire_persona.md ritual.** What the invocation looks like, what seed inputs are required (LinkedIn PDFs, role description, voice samples, domain bounds), what SEV produces.
- **The onboarding review.** How Justin reviews a freshly-authored persona card before going live with it. Specific checks: voice is distinctive from SEV, scope is clear, hard rules are concrete not vague, forbidden patterns are named.
- **The first session under a new persona.** Explicit calibration on turn 1 ("that's too formal," "that's not how Fred would say this," etc.). How to iterate the card based on early sessions.
- **Collaborator cards.** Related to persona hiring but distinct: cards for real colleagues (Tomy, Divit, Alex, etc.) that give SEV context for drafting emails to them, anticipating their reactions, framing work for their consumption. Lives in `continuum-local/collaborators/` (never tracked, contains personal context).
- **Retirement.** When a persona stops earning its keep. Archive process.

## Why this doc matters

The Continuum vision assumes multiple synthetic workers over time. Without a clean hiring process, each new persona becomes an ad-hoc adventure. The hire ritual is what turns adventures into a repeatable capability.

## When this will be written

Session 2, alongside actually hiring Fred. The hire ritual and this doc are co-authored; the ritual file goes in `continuum/rituals/hire_persona.md` and the operator-facing companion is this doc.

## Related docs

- `../01_architecture/persona_model.md` — the formal persona spec (stub)
- `continuum/rituals/hire_persona.md` — the ritual file (to be authored Session 2)
- `../04_reference/aft_source_types.md` — source classification for persona cards (stub)
