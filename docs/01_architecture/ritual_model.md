# Ritual Model (deep dive)

**Tier:** git-tracked.
**Status:** STUB — overview in `system_overview.md`, ritual files themselves live in `continuum/rituals/`.

## What this doc will contain

The formal specification of how rituals work and compose. Topics:

- **Ritual anatomy.** The standard structure of a ritual file: trigger phrase, purpose, inputs, process, outputs, anti-patterns, related rituals.
- **Composition rules.** Which rituals can be nested, which should never be. Example: Task Invocation → Deliverable Checks → OOB Audit compose naturally. POST is always atomic and cannot be nested.
- **Trigger phrase design.** Why the current phrases were chosen (imperative verb first, memorable in a hurry, low collision risk). Rules for adding new triggers.
- **When to add a ritual vs inline rule.** Rituals have overhead; not every rule needs to be a ritual. Decision criteria.
- **Ritual versioning.** How to update a ritual without breaking muscle memory. When a trigger phrase should change (rarely) vs when only the underlying process changes (commonly).
- **The seven existing rituals.** Detailed walkthrough of POST, Session Open, Session Close, Task Invocation, Problem Frame, Deliverable Checks, OOB Audit, Calibration — purpose, when to use, what can go wrong.

## Why this doc matters

Rituals are the behavioral API of Continuum. Without a spec, they drift over time and lose consistency. The spec is how we keep the muscle memory reliable.

## When this will be written

Two triggers would prompt writing this: (1) adding the 9th ritual (four is a pattern, seven is a system, nine means a spec is overdue), or (2) the first significant ritual revision that needs a decision about whether to break existing muscle memory.

## Related docs

- `system_overview.md` — high-level ritual overview
- `../04_reference/ritual_index.md` — the lookup table of rituals (stub)
- `../../rituals/` — the actual ritual files
