# Incident Recovery

**Tier:** git-tracked.
**Status:** STUB — will be populated after first real incident (or pre-written if a plausible scenario forces the issue).

## What this doc will contain

Formal procedures for recovering from serious incidents. Topics:

- **Sanitization leak pushed to remote.** The nightmare case. What to do first (don't panic, don't rewrite history reflexively), how to assess exposure, how to consult with Tomy / Datacom IT, how to remediate.
- **Accidental commit to wrong remote.** E.g. Datacom content accidentally pushed to a personal GitHub. Same family of problem, different specific recovery.
- **Rogue .git capture.** The April 14 jdiesel-docs near-miss escalated: what if you'd run `git add .` before catching it? Recovery steps.
- **Persona card corruption.** If a persona.md is deleted, corrupted, or accidentally rewritten. How to restore from git history.
- **Tool outage.** Filesystem MCP stops working, Claude Code stops authenticating, GitHub is down. How to keep working in a degraded state.
- **Catastrophic machine failure.** Laptop dies. Local tier is gone. How to recover what's in the tracked tier from remote, and what's lost from the local tier.

## Why this doc matters

Incidents happen. Without a procedure, the response is improvised, and improvised responses under stress make incidents worse. The goal is that a bad day produces a recoverable problem, not a career-defining one.

## When this will be written

Written pre-emptively for the most serious scenarios (sanitization leak, wrong-remote commit). Populated from real cases for others. The pre-emptive writing is important because the incident is the worst time to write the playbook.

## Related docs

- `sanitization_review.md` — the audit process (stub)
- `../02_user_guide/troubleshooting.md` — everyday issues (stub)
- `/SANITIZATION.md` (at repo root) — the contract that incident response protects
