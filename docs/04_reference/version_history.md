# Version History

**Tier:** git-tracked.
**Source:** user-authored (changes directed by Justin) + AI-generated, user-reviewed (writeup)

The change log for Continuum. Major versions only; see git log for every commit.

## v1.2.1 — April 22, 2026

**Sanitization cleanup pass.**

Five pre-existing sanitization violations caught by the OOB sweep ritual and corrected. Affected files: README.md, SANITIZATION.md, deliverable_checks.md, oob_audit.md, calibration.md. Specific fixes: replaced customer names with generic references ("yellow-arrow incident" for the origin story), replaced colleague names with role descriptions, replaced a specific branded hardening guide reference with a generic one. Added `.claude/` to `.gitignore`.

**Docs branch scaffolded.** This documentation tree created with 8 core files (README, overview, architecture, user guide, diagram) and stubs for all remaining paths. Reading order and audience map established.

Commit: `9b26e9b`

## v1.2 — April 22, 2026

**Ritual refinements.** POST honesty pass (ritual now clearly distinguishes structural checks SEV can do via MCP from git-state checks SEV can't). Session Open ritual's acknowledgment format made richer with specific active-work summaries. Codec deduplication pass. Live sanitization catch by Mrs. Code: customer names in a tracked-tier example replaced with generics before push.

Commit: `362d95f`

## v1.1 — April 22, 2026

**Initial two-tier build.** First working version. Created `continuum/` and `continuum-local/`. Authored initial persona card (SEV), brand rules, product rules, SANITIZATION contract, and 8 rituals. Set up Datacom-owned private GitHub repo, per-repo git identity, SSH host aliases. Caught and cleaned up a rogue `.git` directory at the home-directory level (pre-existing from April 14 jdiesel-docs project).

Commit: `77452e2`

## v1.0 — April 22, 2026 (design only)

**Architecture designed.** Two-tier model decided. Speaker's Podium rule adopted. SEV / Mrs. Code / Agent Smith role separation named. Gemini's three-root proposal reviewed point-by-point; accepted (SLA framing for North Star, semantic compression, OOB audit, three-part problem frame, AFT source classification), rejected (three-root-directory cathedral, "boot document into both agents" framing, work/personal Continuum blending).

No commit. Architecture lived in session notes until v1.1.

## Versioning convention

- **Major version** (vN.0) — structural changes to the two-tier model, introduction of new persona types, changes to the sanitization contract.
- **Minor version** (vN.M) — new rituals, new docs sections, significant revisions to existing rituals, new personas hired.
- **Patch version** (vN.M.P) — cleanup passes, stub fills, doc corrections, bug fixes.

Version bumps happen at commit time and are recorded in both this file and the commit message.

## Planned upcoming versions

**v1.3** — Session 2 planned scope: hire Fred (first non-SEV persona), build collaborator cards directory in local tier, author `hire_persona.md` ritual, first formal monthly calibration.

**v2.0** — scope TBD. Triggers for v2.0 would be: first multi-machine deployment, first non-Justin operator, or first formal external audit. Any of those forces architectural changes that justify a major version.
