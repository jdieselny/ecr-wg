# Handoff to Mrs. Code

**Tier:** git-tracked.
**Status:** STUB — workflow described in outline form in `daily_workflow.md`. Full detail pending.

## What this doc will contain

The step-by-step commit workflow, end to end. Topics:

- **The "Commit this" output format.** What SEV produces when invoked. Section structure: tracked tier changes, local tier changes, files touched, summary of the change, commit message proposal.
- **Reviewing the commit packet.** What to look at first, what to look at second, what catches 90% of problems before handoff.
- **The copy-paste handoff.** Exactly what to copy from Claude Desktop into Mrs. Code's terminal. How to frame the request to Mrs. Code.
- **Mrs. Code's review.** What she checks for independently. Expected output: approval, corrections, or rejection.
- **Commit and push execution.** The specific git commands. Where to look for confirmation. What a clean commit looks like in the log.
- **When Mrs. Code catches something.** Step-by-step recovery. Amend vs new commit. When to start over from SEV.
- **The pre-push sanity check.** Final inspection before push (the remote is the last chance to stop a leak from leaving the machine).

## Why this doc matters

The handoff is the most common place for Continuum to break or leak. A detailed, step-by-step guide reduces the surface area for mistakes.

## When this will be written

After the first few real commits have run, enough to know which parts of the handoff are smooth and which create friction. The guide should reflect the actual workflow, not the hypothetical one.

## Related docs

- `daily_workflow.md` — overview with handoff in context
- `../01_architecture/tool_integration.md` — Mrs. Code's setup (stub)
- `../03_operator_guide/sanitization_review.md` — what a reviewer checks (stub)
