# Calibration Guide

**Tier:** git-tracked.
**Status:** STUB — ritual file at `continuum/rituals/calibration.md`. This doc will be the operator-facing companion.

## What this doc will contain

How to run system calibration. Topics:

- **Cadence.** Monthly calibration (lightweight) vs quarterly calibration (deep). What each covers.
- **The monthly checks.** Autonomy check (am I letting SEV do things without review?), complacency check (am I accepting drafts without real scrutiny?), brand drift check (is output still in Justin's voice?), open-items check (what's fallen off the roadmap?).
- **The quarterly checks.** Everything in monthly, plus: full tracked-tier sanitization sweep, ritual effectiveness review (which rituals are actually used vs which are vestigial), persona performance review (are personas drifting?), tool performance review (is MCP flaky?), roadmap re-anchoring (are we still going where we said?).
- **What a "pass" looks like.** The output of a calibration: notes in `05_evolution/lessons_learned/`, commits to fix what's broken, parking-lot additions for what's deferred.
- **What a "fail" looks like.** Calibration identifying something material enough to reshape the roadmap, retire a ritual, or require an incident response.

## Why this doc matters

Without calibration, drift is inevitable. Without a guide, calibration becomes a vague intention that never happens. This doc operationalizes what "run calibration" actually means.

## When this will be written

Before the first scheduled monthly calibration (call it end of May 2026). Writing the guide IS the first calibration.

## Related docs

- `continuum/rituals/calibration.md` — the ritual file
- `../05_evolution/lessons_learned/` — where calibration findings land
- `sanitization_review.md` — the deeper audit process (stub)
