# Continuum Chronicle

**Tier:** git-tracked.
**Source:** user-authored (concept by Justin, structure refined collaboratively)

## What the Chronicle is

A running log of the Continuum system's own development. One file per ISO week. Each entry captures what changed in the system itself: rituals added or modified, rules refined, personas hired, architecture decisions made, open questions carried forward.

The Chronicle is **not** a log of SE customer work, deal progress, or deliverables. Those live in `continuum-local/personas/sev/active_state.md` and `continuum-local/roadmap/in_flight.md` where they belong.

The Chronicle is the story of Continuum (SLA 3 work). It is tracked-tier safe because it never references customer names or deal specifics.

## Why weekly

- Daily is too granular. Most days have nothing worth recording at the system level.
- Monthly is too coarse. The narrative thread of "why did we do X" dissolves.
- Weekly matches the natural cadence of engineering work and keeps each entry small.

## File naming

ISO week convention: `YYYY-Www.md`

Examples:
- `2026-W17.md` for April 20-26, 2026
- `2026-W18.md` for April 27 - May 3, 2026

ISO weeks run Monday to Sunday. If work crosses a week boundary, the entry goes in the week where it was completed or committed.

## What goes in an entry

Each weekly entry covers, as applicable:

- **Rituals or rules added / modified / removed** — what and why
- **Persona changes** — hires, voice calibrations, scope updates
- **Architecture decisions** — tier changes, new mechanisms, directory reorganizations
- **What broke and how it was fixed** — postmortems belong in `lessons_learned/`, but a one-line reference from the Chronicle keeps the thread intact
- **Open questions carried forward** — things flagged this week that did not get resolved

Entries should be short. Prose. No forced structure. A slow week might be three sentences. A heavy week might be a page.

## What does NOT go in

- Customer names, deal specifics, deal progress
- Colleague characterizations or informal context
- Internal codenames or unreleased product specs
- Competitive intelligence with pricing or win/loss detail

All of those live in the local tier.

## When entries get written

At session close, as part of the `session_close.md` ritual: SEV proposes Chronicle additions for the current week as part of the changeset. If it has been a quiet week with no Continuum-system changes, the Chronicle is skipped. No filler entries.

If a week ends without a session close on the final day, the open week's entry stays partial and gets closed out at the next session that touches it.

## Why this file structure exists

Two reasons:

1. **File hygiene.** Scratchpads are daily-ish. Without a weekly rollup, the project becomes a swamp of timestamped fragments with no navigable narrative.

2. **Provability.** Continuum is a system. Systems that can show their development trail look (and are) more serious than systems that cannot. If this work ever goes from "Justin's tool" to "something other people use," the Chronicle is the evidence of how it got built.

## Relationship to other files

- `docs/05_evolution/lessons_learned/` — postmortems of specific failures. Referenced from Chronicle when relevant.
- `docs/05_evolution/roadmap.md` — forward-looking: where Continuum is headed next.
- `docs/05_evolution/parking_lot.md` — ideas not yet scheduled.
- **`docs/05_evolution/chronicle/`** (this directory) — backward-looking: what Continuum actually did, week by week.

Chronicle is the only one of these files that is append-only by design. Roadmap and parking lot get rewritten. Lessons learned accumulate case-by-case. Chronicle marches forward one week at a time and the earlier weeks are never edited after the fact.
