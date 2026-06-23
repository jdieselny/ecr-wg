# Trigger Phrases

**Tier:** git-tracked.
**Source:** user-authored (phrases chosen by Justin) + AI-generated, user-reviewed (organization)
**Audience:** keep this open during daily use

Continuum is driven by named trigger phrases. This is the reference card. Print it, bookmark it, pin it to a second monitor.

## Session lifecycle

| Phrase | What happens |
|---|---|
| **Run POST** | SEV verifies workspace state (structural checks via MCP) before loading. Fails fast if something's wrong. |
| **Load Continuum** | SEV reads persona, brand, product rules, product internal, north star, in-flight. Acknowledges what loaded. |
| **Commit this** | SEV proposes tiered changeset (tracked tier changes + local tier changes). You review before handoff to Mrs. Code. |

## Task invocation

| Phrase | What happens |
|---|---|
| **Frame this problem** | Runs the three-part problem frame ritual: problem, success, goal. |
| **Draft [thing] for [audience]** | Scoped deliverable invocation. Specify format, tone, length in the same turn. |
| **Review [artifact]** | Critical review of a draft against brand rules, product rules, and dogshit checks. |
| **OOB audit [topic]** | Out-of-band audit. SEV flags what a reviewer outside the AI loop should verify. Used for high-stakes deliverables. |

## Calibration and health

| Phrase | What happens |
|---|---|
| **Run calibration** | Walks through the calibration ritual: autonomy check, complacency check, brand drift check, open items. |
| **Signal check** | Mid-session mini-calibration. Use when the session feels off. |
| **What's in flight?** | SEV summarizes active state files, highlighting highest-priority items. |
| **What's pending with [person]?** | SEV pulls open items tagged to a specific colleague from active_state and product_internal. |

## Persona control (Speaker's Podium)

| Phrase | What happens |
|---|---|
| **Call in [persona]** | Explicit persona switch. Starts a fresh session under a different persona's card. Example: "Call in Fred." |
| **Back to SEV** | Return to the default persona. |
| **Who's on the mic?** | SEV confirms which persona is currently active. Useful after long sessions or when switching back from another persona. |

Note: do not attempt to blend personas within a single turn. If you want multi-persona input, invoke each in sequence.

## Sanitization and safety

| Phrase | What happens |
|---|---|
| **Is this safe to commit?** | SEV runs a targeted sanitization check on the current draft before you hand off. |
| **Scrub this** | SEV rewrites content with sanitization rules applied (codenames removed, generic substitutes for customer names, etc.). |
| **What tier is this?** | SEV classifies content as tracked-safe, local-only, or borderline. |

## Hiring new synths (future, Session 2+)

| Phrase | What happens |
|---|---|
| **Hire [persona]** | Runs the hire_persona.md ritual. SEV walks you through authoring a new persona card from seed inputs (LinkedIn PDFs, role description, voice samples). |
| **Retire [persona]** | Archives a persona that's no longer useful. Card moves to a retired/ subdirectory; not deleted. |

These triggers exist as scaffolding but the underlying rituals are scheduled for Session 2. See `../03_operator_guide/hiring_new_synths.md`.

## Documentation

| Phrase | What happens |
|---|---|
| **Update docs for [change]** | SEV identifies which docs files need editing to match a system change, and proposes the edits. |
| **What's the docs version?** | SEV reads the header of the target doc and reports its source classification and last update. |

## Information retrieval

| Phrase | What happens |
|---|---|
| **What does [file] say about [topic]?** | SEV reads the specific file (not just memory) and answers from its actual contents. |
| **Show me [filename]** | SEV displays the file's contents inline. |
| **Search continuum for [term]** | SEV searches across the tracked tier for a term. |
| **Search continuum-local for [term]** | Same but across local tier. |

## Quick debugging

| Phrase | What happens |
|---|---|
| **Why did you do that?** | SEV explains its reasoning on the previous output. |
| **That's wrong because [reason]** | SEV accepts the correction, updates reasoning, revises output. |
| **Show your work** | SEV walks through its chain of reasoning on the current task. |
| **Start over on this** | Discards the current draft, restarts the task from scratch with fresh context. |

## Emergency / recovery

| Phrase | What happens |
|---|---|
| **Something's wrong with the workspace** | SEV runs diagnostic POST with verbose output, flags structural problems. |
| **Recover from [issue]** | Invokes incident recovery guide from `03_operator_guide/incident_recovery.md`. |
| **Abort this session** | SEV discards unsaved work, reminds you nothing was committed, suggests closing the chat. |

## Phrases that DON'T exist (and why)

Some things you might expect as triggers aren't, on purpose:

- **"Learn this"** — Continuum doesn't have background learning. Context comes from files you explicitly write. If something's worth "learning," it goes in a file.
- **"Forget about X"** — SEV doesn't have forgetting in its working context. If you want something out of active state, edit the file or archive it.
- **"Save this for later"** — nothing is saved implicitly. Use scratchpad (`continuum-local/scratchpad/`) for fast dumps, proper tier files for retained content.
- **"Autonomous mode"** — Continuum is not autonomous. Every change goes through explicit review. If that ever changes, it goes through a roadmap decision, not a trigger phrase.

## Phrase design principles

When we add new triggers, they follow three rules:

1. **Imperative verb first.** "Run POST" not "POST". "Frame this problem" not "problem framing".
2. **Memorable in a hurry.** You need to be able to type these fast from muscle memory. No rituals named "Perform Pre-Operation Systems Verification Protocol."
3. **Low collision risk.** A trigger phrase should not sound like something you'd say casually. "Run POST" is safe because you wouldn't accidentally say it; "start" would be too collidable.

## Related docs

- `getting_started.md` — first-time setup
- `daily_workflow.md` — how sessions flow day-to-day
- `handoff_to_mrs_code.md` — the commit workflow
- `../04_reference/ritual_index.md` — what each ritual actually does internally
