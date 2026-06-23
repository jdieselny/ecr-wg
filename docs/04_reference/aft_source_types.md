# AFT Source Types

**Tier:** git-tracked.
**Status:** STUB — concept introduced in `system_overview.md`. Detailed classification guide pending.

## What this doc will contain

A guide to the AFT (Actual Fucking Truth) classification system. Every tracked-tier file carries a source classification in its header. Three values are currently defined:

- `user-authored` — Justin wrote or explicitly directed the content
- `public-documented` — from publicly-available facts
- `AI-generated, user-reviewed` — SEV drafted, Justin accepted

This doc will cover:

- **When to use each.** Specific criteria. Border cases: what about content Justin drafted and SEV rewrote? What about public facts that SEV assembled? What about a ritual authored collaboratively?
- **Why classification matters.** Audit trail for provenance. Prevents AI-generated content from becoming "truth" without review. Enables targeted review ("let me see all AI-generated content in the tracked tier") when needed.
- **Updating classification.** When a file's classification changes (e.g., user-reviewed becomes user-authored after heavy edits), how to update. Versioning implications.
- **What happens without classification.** Files without an AFT header are treated as suspect until classified. Calibration checks flag them.
- **Possible future classifications.** `AI-generated, pending-review` for content SEV produced that Justin hasn't yet approved. `external-attributed` for content sourced from a specific named external document.

## Why this doc matters

Classification is a contract. Without a clear guide, classifications drift to whatever feels convenient in the moment. With a guide, the contract stays enforceable.

## When this will be written

After the first handful of border cases force specific judgment calls. The doc captures those calls as precedent.

## Related docs

- `../01_architecture/system_overview.md` — where AFT is introduced
- `../06_contributing/review_checklist.md` — where classification is verified during commit (stub)
