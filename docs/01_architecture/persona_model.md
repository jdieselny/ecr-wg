# Persona Model (deep dive)

**Tier:** git-tracked.
**Status:** STUB — concept covered in `system_overview.md`. Full specification pending Session 2 when we hire Fred.

## What this doc will contain

The formal specification of how personas work in Continuum. Topics:

- **Persona card anatomy.** What goes in `persona.md` (voice, scope, hard rules, rituals, signatures, forbidden patterns). What goes in `active_state.md` (in-flight work). Standard section headers.
- **The Speaker's Podium rule, formally.** Why only one persona at a time. What happens at the boundary when Justin invokes a different persona. Session-level vs turn-level switching.
- **Hiring a new persona.** The `hire_persona.md` ritual (to be authored in Session 2). Seed inputs: LinkedIn PDFs, role description, voice samples, first-draft hard rules. Review process before a persona goes live.
- **Retiring a persona.** How to archive without deleting. Why retired personas still exist in the repo (learning).
- **Persona versioning.** Personas evolve over time. How to track changes to a persona's voice without losing the old version.
- **Cross-persona rules.** Rules that apply regardless of which persona is loaded (sanitization contract, AFT classification, commit workflow).

## Why this doc matters

Continuum's value scales with the number of well-tuned personas. The persona model needs to be clear enough that hiring a new synth is a known process, not an invention.

## When this will be written

Session 2, alongside hiring Fred. The act of writing the hire ritual while actually hiring a new persona is the natural way to get this right.

## Related docs

- `system_overview.md` — high-level Speaker's Podium overview
- `../03_operator_guide/hiring_new_synths.md` — the operational guide (stub, Session 2)
