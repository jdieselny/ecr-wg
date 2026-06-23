# Continuum Documentation

Welcome to the documentation branch of the **Continuum** system: Datacom Systems' persistent-context framework for AI-assisted Sales Engineering work.

This documentation is living. It grows as the system grows. It is intentionally structured so that future contributors (human or synthetic) can add to it without refactoring.

---

## What is Continuum, in one sentence?

A two-tier file-system-backed workspace that gives an LLM persistent context, enforced brand and product rules, and a sanitization contract that keeps customer data out of source control, while letting multiple collaborating AI personas operate against a shared source of truth.

If you want more than one sentence, start with [00_overview/what_is_continuum.md](00_overview/what_is_continuum.md).

---

## Start here, by audience

Different readers want different things. Pick your path.

### If you are a Sales Engineer who wants to USE this system
Read in this order:
1. [00_overview/what_is_continuum.md](00_overview/what_is_continuum.md) — what this is and why it exists
2. [02_user_guide/getting_started.md](02_user_guide/getting_started.md) — first-time setup
3. [02_user_guide/daily_workflow.md](02_user_guide/daily_workflow.md) — how to actually use it
4. [02_user_guide/trigger_phrases.md](02_user_guide/trigger_phrases.md) — reference card

Total reading time: about 20 minutes. After that you can run real sessions.

### If you are a Datacom stakeholder, manager, or reviewer
Read in this order:
1. [00_overview/what_is_continuum.md](00_overview/what_is_continuum.md) — what we built
2. [00_overview/why_it_exists.md](00_overview/why_it_exists.md) — the problems it solves
3. [01_architecture/system_overview.md](01_architecture/system_overview.md) — how it works technically
4. [01_architecture/diagrams/system_overview.svg](01_architecture/diagrams/system_overview.svg) — the visual

Total reading time: about 15 minutes. You will come away with a working mental model.

### If you are a security or compliance reviewer
Start at:
1. [01_architecture/two_tier_model.md](01_architecture/two_tier_model.md) — the sanitization contract and trust boundary
2. [03_operator_guide/sanitization_review.md](03_operator_guide/sanitization_review.md) — how to audit the tracked tier
3. [04_reference/tier_classification.md](04_reference/tier_classification.md) — what goes where, by category

### If you are contributing new rituals, rules, or documentation
Read:
1. [06_contributing/editing_rituals.md](06_contributing/editing_rituals.md) — how to propose ritual changes
2. [06_contributing/editing_rules.md](06_contributing/editing_rules.md) — how to propose hard-rule changes
3. [06_contributing/review_checklist.md](06_contributing/review_checklist.md) — what reviewers check

### If you are thinking about the long-term direction
Read:
1. [05_evolution/roadmap.md](05_evolution/roadmap.md) — what is planned
2. [05_evolution/parking_lot.md](05_evolution/parking_lot.md) — what we decided not to do, and why
3. [05_evolution/pattern_library_plan.md](05_evolution/pattern_library_plan.md) — the path to a shared framework

---

## Directory map

```
docs/
├── README.md                         ← you are here
├── 00_overview/                      ← What is this and why does it exist
├── 01_architecture/                  ← How it works, technically
│   └── diagrams/                     ← Visual references
├── 02_user_guide/                    ← How to use it as an SE
├── 03_operator_guide/                ← How to operate, audit, recover
├── 04_reference/                     ← Lookup tables, indices, specs
├── 05_evolution/                     ← Roadmap, lessons, history
│   └── lessons_learned/              ← Post-incident writeups over time
└── 06_contributing/                  ← How to change the system
```

The structure is numbered so files sort in reading order when listed alphabetically. Prefixes `00`–`06` are meaningful: lower numbers are more conceptual, higher numbers are more procedural or historical.

---

## Documentation status

Not every file in this tree is complete. Files fall into three states:

- **Complete:** fully written, reviewed, current
- **Stub:** placeholder exists with the intended scope, content will come in a future session
- **Planned:** listed in the directory map but not yet created

At the time of this writing (April 22, 2026, end of Session 1 of Continuum build), the complete files are:

- `README.md` (this file)
- `00_overview/what_is_continuum.md`
- `01_architecture/system_overview.md`
- `01_architecture/diagrams/system_overview.svg`
- `02_user_guide/getting_started.md`
- `02_user_guide/daily_workflow.md`
- `02_user_guide/trigger_phrases.md`

Everything else is a stub. Stubs open with a clearly marked `[STUB]` header and describe what the finished file will cover.

---

## Conventions used in this documentation

- **File references** use relative paths from the docs root so links work in any editor or git forge.
- **Workspace file references** (files outside docs) use the full path under `C:\Users\jkintzele\DatacomWorkspace\`.
- **Code fences** are used for command-line snippets, file path examples, and ritual text.
- **Callouts** are bolded and prefixed (**Note:**, **Warning:**, **Rationale:**) rather than relying on emoji or special blocks.
- **Writing style** matches the Datacom internal voice: direct, peer-to-peer, no em dashes (this is both a style preference and a hard brand rule).

---

## Questions, corrections, contributions

This documentation lives in the Continuum git repository. Changes follow the standard Continuum contribution flow: SEV proposes, Mrs. Code audits and commits. See [06_contributing/](06_contributing/) for specifics.

If you are reading this and something is unclear, wrong, or missing, that itself is a documentation bug. Open a note in `continuum-local/scratchpad/` with the specifics and we will fix it in the next session.

---

*Document owner: Justin Kintzele, Senior Sales Engineer, Datacom Systems Inc.*
*System architect (AI): SEV*
*Last substantive update: April 22, 2026*
