# Two-Tier Model (deep dive)

**Tier:** git-tracked.
**Status:** STUB — high-level coverage exists in `system_overview.md`. This doc will expand on the specifics.

## What this doc will contain

A deep dive on the tracked/local separation. Topics:

- **The sanitization boundary in detail.** What exactly qualifies as tracked-safe vs local-only. Gray-area cases (colleague first names, role descriptions, dated public facts). Decision tree for ambiguous content.
- **Why physical separation over .gitignore.** Failure modes of gitignore-based approaches. Real examples from the v1.2 commit where OOB audit caught pre-existing violations.
- **How content migrates between tiers.** The rare case where local content becomes generalizable enough to move to tracked. The reverse case (never happens intentionally; if it does, it's a sanitization failure to recover from).
- **Backup strategy for the local tier.** The local tier has no remote by design. How to back it up without violating the sanitization contract (local backup solutions, NOT cloud sync to personal accounts).
- **Multi-machine considerations.** If Justin works from two machines, how the tracked tier syncs via git and how the local tier does NOT sync automatically (must be manually copied through a sanctioned channel).

## Why this doc matters separately

The high-level overview covers the concept. Operators (Datacom IT, future SEs, security reviewers) need the detailed rules. A separate doc lets this be updated without disturbing the main architecture narrative.

## When this will be written

First draft after the first real multi-machine or multi-operator scenario produces a concrete question this doc should answer. Writing in advance produces speculative fiction; writing after the first real case produces useful documentation.

## Related docs

- `system_overview.md` — high-level architecture
- `../../SANITIZATION.md` — the formal contract (at repo root)
