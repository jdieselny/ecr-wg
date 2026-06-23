# Sanitization Review

**Tier:** git-tracked.
**Status:** STUB — the formal contract lives at `/SANITIZATION.md` at repo root. This doc will translate it into auditor-facing language.

## What this doc will contain

A procedure for an auditor (internal security reviewer, Datacom IT, a future compliance check) to verify the tracked tier is clean. Topics:

- **Scope of review.** What the auditor is checking for and what's out of scope.
- **Review procedure.** Step-by-step: clone the repo, run specific checks, inspect specific file types, produce an audit report.
- **Red flags.** Specific patterns that should stop a review: customer name patterns, codename patterns, pricing-looking numbers, colleague name patterns.
- **Green flags.** Patterns that indicate the sanitization contract is being followed (source classification headers, generic language, absence of deal-level specifics).
- **Reporting format.** What the auditor produces at the end. Template for findings.
- **Remediation.** What to do when violations are found. How to distinguish "content that needs to move to local tier" from "content that needs to be rewritten generically."

## Why this doc matters

The sanitization contract is only as strong as the review that enforces it. Mrs. Code provides per-commit review; a human auditor provides periodic full-tree review. Both are needed.

## When this will be written

Before the first external audit (internal or formal). Must exist before anyone outside the current build team reviews the repo.

## Related docs

- `/SANITIZATION.md` (at repo root) — the formal contract
- `calibration_guide.md` — the recurring self-review (stub)
- `incident_recovery.md` — what to do if violations are found late (stub)
