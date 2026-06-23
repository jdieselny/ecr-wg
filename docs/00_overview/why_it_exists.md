# Why Continuum Exists

**Tier:** git-tracked.
**Status:** STUB — scheduled for future session.

## What this doc will contain

The origin story of Continuum, written as a narrative. This is the "why" doc, distinct from `what_is_continuum.md` (which is the "what").

Planned sections:

- **The yellow-arrow incident** — April 21, 2026. A customer deck was built with solid blue fill, text-based letters, and serif typography instead of the established standard. Quality regression visible to the user, invisible to the AI. The moment where Justin decided the memory-summary-driven approach had hit its ceiling.
- **What was tried before** — memory-only continuity, user-preferences, project-level instructions, and why each failed to solve the structural problem.
- **The design session** — the evening of April 22, 2026. Architecture choices, pushback on Gemini's three-root-directory proposal, the Speaker's Podium rule, the two-tier separation decision, the Mrs. Code role.
- **What we caught in real time** — Mrs. Code catching sanitization violations SEV introduced; the OOB sweep catching five more pre-existing violations; the near-miss with the rogue `.git` directory at the home directory level.
- **What this taught us** — principles that came out of the build that we'd apply again.

## Why this matters

The yellow-arrow incident is the single most important artifact of Continuum's origin. A future operator (or a future Justin who's forgotten) needs to know what problem this system actually solves. Without the story, Continuum looks like over-engineering. With the story, it looks like the right amount of engineering for the problem at hand.

## When this will be written

First draft: future session when Justin has time to tell the story properly and SEV can capture it faithfully. This is not a doc to rush.

## Related docs

- `what_is_continuum.md` — the conceptual overview (already written)
- `../05_evolution/lessons_learned/2026-04-21_yellow-arrow-incident.md` — the incident writeup (stub)
