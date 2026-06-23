# Ritual Index

**Tier:** git-tracked.
**Status:** SEED — populated with v1.2.1 rituals. Add new rituals here as they're authored.

Quick-reference lookup for every ritual in Continuum. The source files live in `continuum/rituals/`. This index exists for scanning.

## Session lifecycle rituals

| Ritual | Trigger | File | Purpose |
|---|---|---|---|
| POST | "Run POST" | `post.md` | Verify workspace state before load |
| Session Open | "Load Continuum" | `session_open.md` | Read persona, rules, state, roadmap |
| Session Close | "Commit this" | `session_close.md` | Propose tiered changeset for review |

## Task rituals

| Ritual | Trigger | File | Purpose |
|---|---|---|---|
| Task Invocation | Persona/Task/Output format | `task_invocation.md` | Scoped work with clear deliverable |
| Problem Frame | "Frame this problem" | `problem_frame.md` | Three-part problem/success/goal decomposition |

## Quality rituals

| Ritual | Trigger | File | Purpose |
|---|---|---|---|
| Deliverable Checks | Any customer-facing artifact | `deliverable_checks.md` | Cover-first protocol + dogshit checks |
| OOB Audit | "OOB audit" | `oob_audit.md` | Out-of-band validation for high-stakes |

## System rituals

| Ritual | Trigger | File | Purpose |
|---|---|---|---|
| Calibration | "Run calibration" | `calibration.md` | Monthly/quarterly system review |

## Planned rituals (Session 2+)

| Ritual | Trigger | Status |
|---|---|---|
| Hire Persona | "Hire [persona]" | Scheduled Session 2 |
| Retire Persona | "Retire [persona]" | Scheduled Session 2 |
| Signal Check | "Signal check" | Currently inline in Task Invocation; may be promoted |

## How to use this index

- Looking for what a ritual does? Check the Purpose column; for detail, open the file.
- Looking for a trigger you half-remember? Scan the Trigger column.
- Adding a new ritual? Add a row here in the same commit that adds the ritual file. Docs and rituals version together.

## Related docs

- `../01_architecture/ritual_model.md` — the formal ritual spec (stub)
- `../02_user_guide/trigger_phrases.md` — user-facing trigger reference (already written)
