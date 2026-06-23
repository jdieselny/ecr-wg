# Roadmap

**Tier:** git-tracked.
**Status:** SEED — high-level roadmap populated; granular items land here as they mature.

The forward roadmap for Continuum. Distinct from `continuum/roadmap/north_star.md` (aspirational) and `continuum-local/roadmap/in_flight.md` (live tasks with customer specifics).

## Immediate (next few sessions)

- **Session 2: hire Fred.** First non-SEV persona. Marketing/content specialist. Seeded from LinkedIn resume samples. Co-authors `hire_persona.md` ritual alongside the hire itself.
- **Collaborator cards directory.** Local-tier directory populated with informal cards for real Datacom colleagues. Gives SEV context for drafting to them.
- **First monthly calibration.** End of May 2026. Tests the calibration ritual under real conditions and produces the first entry in `lessons_learned/`.
- **Populate high-priority stubs.** `troubleshooting.md` (as failures occur), `handoff_to_mrs_code.md` (after a few real commits), `ritual_index.md` entries for any new rituals.

## Near-term (1-3 months)

- **First quarterly calibration.** End of July 2026. Deeper review than monthly. Produces structured review of ritual effectiveness, tool performance, persona drift.
- **Second persona beyond Fred.** Whatever domain is demonstrably under-served by SEV after 1-2 months of real use. Could be a technical writer for deep documentation, a competitive analyst, a solutions architect; decision deferred until data is in.
- **Cheat sheet printable.** One-page printed reference after muscle memory stabilizes.
- **Tier classification lookup table** (`04_reference/tier_classification.md`) — populate from real gray-area decisions accumulated in first 1-3 months.

## Medium-term (3-6 months)

- **Pattern library extraction.** If patterns have proven stable across Continuum and Agent Smith, extract to a third neutral repo. Current plan is Option B-on-C: duplicate patterns in both systems now, extract to shared `continuum-patterns/` repo only after three-plus months of stable cross-Continuum use.
- **Multi-machine support.** If Justin starts working from a second machine, design and document the sync story for local tier (tracked tier syncs via git trivially). Likely: sanctioned backup solution, not cloud sync.
- **First external audit.** Internal security review or Datacom IT review. Forces the `sanitization_review.md` stub to be written before they look at the repo.

## Long-term (6-12+ months)

- **True multi-persona concurrency.** Currently Continuum runs one persona at a time (Context OS / Synthetic Workforce Runtime). "Hypervisor" implies multiple personas running in parallel with resource arbitration. That's a significant architectural change. Triggers would be: a task that genuinely benefits from multiple personas collaborating simultaneously (not sequentially), and tools mature enough to support the pattern cleanly.
- **Non-Justin operators.** Someone else at Datacom adopting Continuum. This forces `getting_started.md` to be battle-tested against a non-author, and forces the user guide to be stand-alone without implicit context.
- **Formal pattern-library launch.** If the cross-Continuum pattern set stabilizes, the shared repo becomes a real asset with its own versioning and contribution model.

## Not on the roadmap (on purpose)

Items considered and explicitly deferred or declined are documented in `parking_lot.md`. Consult that file before proposing something here; it may have already been considered and declined.

## Roadmap discipline

- Items move from here to `in_flight.md` (local tier) when work actively starts.
- Items move from `in_flight.md` to completed state (and get summarized in `version_history.md`) when shipped.
- Items that stall for more than a quarter go to `parking_lot.md` or get explicitly resuscitated in the next calibration.
- This file is reviewed each quarterly calibration.
