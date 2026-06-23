# System Overview

**Tier:** git-tracked.
**Source:** user-authored (architecture directed by Justin) + AI-generated, user-reviewed (writeup)

This document describes the Continuum architecture at a level sufficient for a technical reader to understand what's built, how the parts fit together, and why specific design choices were made.

For a visual companion, see `diagrams/system_overview.svg`.

## Core design principles

Continuum is built on six principles. Every design decision traces back to one of these.

1. **Files are the memory.** State that matters lives in files on disk, not in chat history. Chat is ephemeral. If something is worth keeping, it goes in a file.

2. **Physical separation beats logical separation.** Sensitive content and shareable content live in physically distinct directories, not in one directory with rules about what's ignored. The separation is a filesystem boundary, not a convention.

3. **Explicit rituals beat ambient behavior.** Context loading, commits, validation, and handoffs happen through named rituals with specific trigger phrases. You know when they run because you invoke them. Implicit "the AI just knows" behavior is a bug.

4. **One persona holds the mic.** A loaded persona defines voice, scope, and rules for the active session. Switching personas is explicit. No voice bleed between simultaneous personas. This is the Speaker's Podium rule.

5. **The proposer cannot be the reviewer.** Whatever AI generates a deliverable is the worst possible reviewer of that deliverable. Out-of-band validation (human expert, different tool, canonical reference doc, time separation) is required for high-stakes output.

6. **The user is the final authority.** The AI proposes, the user decides. Changes to the system go through explicit review. No silent writes. No background learning.

## The two-tier architecture

Continuum separates content into two physical directories on the local machine:

```
~/DatacomWorkspace/
├── continuum/           ← git-tracked, pushed to Datacom-owned private repo
└── continuum-local/     ← local-only, never git-initialized, never pushed
```

### Tracked tier (`continuum/`)

Contains the *system* of Continuum: patterns, rules, structure. Everything here is safe to push to a Datacom-owned repo and survive a security audit.

- Persona cards with generic voice/rules (no customer-specific content)
- Brand rules (colors, logo, typography, writing conventions — all publicly visible)
- General product rules (category boundaries, SKU naming formats, competitive framing discipline)
- Rituals (behavior protocols)
- North Star roadmap (aspirational missions, no deal specifics)
- Template specs (what good looks like, not the template files themselves)
- SANITIZATION.md (the content contract)
- This documentation branch

### Local tier (`continuum-local/`)

Contains the *content* flowing through Continuum: specifics, sensitive material, work product. Never leaves the machine.

- Active state (customer names, deal progress, BOM specifics)
- Collaborator cards (informal color on real colleagues)
- Open product questions (items pending internal confirmation)
- Internal codenames
- Competitive intel specifics (pricing, win/loss with names)
- In-flight roadmap (with deal names)
- Session scratchpads
- Actual template asset files (.pptx, .docx, logo PNGs)

### Why physical separation

One directory with .gitignore rules works, but it requires constant vigilance: "is this file tracked or not?" A mistaken `git add .` in a directory with mixed tracked and untracked content is a leak waiting to happen.

Physical separation makes leaks structurally harder. `git add .` inside `continuum/` cannot pick up anything in `continuum-local/` because the directory isn't inside the repo.

Defense in depth: the `continuum/` `.gitignore` also explicitly excludes any nested `continuum-local/` directory, in case one is ever accidentally placed inside the tracked tier.

## The persona model

Personas are the voices Continuum can speak in. Each persona is defined by two files:

- `continuum/personas/[name]/persona.md` — voice, scope, hard rules, rituals (generic; git-tracked)
- `continuum-local/personas/[name]/active_state.md` — live in-flight work in that persona's domain (specific; local-only)

Current personas:
- **SEV** (Sales Engineering Virtual assistant): Justin's primary synthetic SE partner

Planned personas (Session 2 and beyond):
- **Fred** (marketing / content specialist) — to be hired from LinkedIn resume samples

### Speaker's Podium

Only one persona is active at a time. Invocation is explicit:
- Default: SEV
- Switch: "Call in Fred"
- Return: "Back to SEV"

No persona blending. No "here's what Fred would say but also SEV thinks" within a single turn. If Justin wants multi-persona input, he invokes them in sequence, not simultaneously.

## The ritual model

Rituals are named behavior protocols with specific trigger phrases. Each ritual is defined in `continuum/rituals/[name].md` and describes its trigger, purpose, process, and anti-patterns.

Current rituals (v1.2.1):

| Ritual | Trigger | Purpose |
|---|---|---|
| POST | "Run POST" or first session message | Verify workspace state before loading |
| Session Open | "Load Continuum" | Read persona, rules, state, roadmap |
| Session Close | "Commit this" | Propose tiered changeset for review |
| Task Invocation | Persona/Task/Output format | Scoped work with clear deliverable |
| Problem Frame | "Frame this problem" | Three-part problem/success/goal |
| Deliverable Checks | Any customer-facing artifact | Prevent brand drift, sanitization leaks |
| OOB Audit | "OOB audit" | Out-of-band validation for high-stakes |
| Calibration | "Run calibration" | Monthly/quarterly system review |

Rituals compose. A customer deliverable session might invoke Task Invocation → Deliverable Checks (cover-first) → OOB Audit → Commit this. Each ritual has a clear job and doesn't duplicate others.

## Tool integration

Continuum runs across multiple Claude interfaces and tools, each with a specific role:

### Claude Desktop app + Filesystem MCP

The primary working environment. Claude Desktop with Anthropic's official Filesystem MCP connector has read/write access to `~/DatacomWorkspace/` (scoped; no broader filesystem access). This is where SEV reads files at session open, writes proposals during work, and drafts scratchpad content.

### Mrs. Code (Claude Code on terminal)

The out-of-band reviewer and git executor. Mrs. Code reads the proposed changeset from SEV, audits for sanitization violations independently, and runs git operations (`add`, `commit`, `push`). The separation between SEV-proposes and Mrs.-Code-commits is structural: a single AI cannot reliably review its own output, so the commit step goes through a different Claude instance in a different tool.

### GitHub (Datacom-owned private repo)

The remote origin for the tracked tier only. Hosted under a Datacom-owned GitHub account. SSH routing uses a host alias (`github.com-datacom`) with a dedicated SSH key to prevent accidental commits under personal git identity.

### Memory (cross-session baseline)

Claude's built-in memory system holds a high-signal baseline (~10 dense entries) that persists across every Claude interface (Desktop, web, mobile). Memory is the fallback for sessions that start without loading Continuum. Full working context still comes from file loads.

### Agent Smith (personal Continuum, parallel system)

Justin operates a personal Continuum (nicknamed Agent Smith) on his personal GitHub account, scoped to personal projects. Architectural patterns are shared between work Continuum and Agent Smith; content never crosses the boundary. Pattern sharing may eventually be formalized via a third neutral pattern-library repo, but currently both systems maintain their own copy.

## Session lifecycle

A typical working session:

1. **POST** — Justin says "Run POST." SEV verifies workspace state (structural checks); Justin or Mrs. Code verifies git state (shell-required checks). Any red fails the gate.
2. **Load Continuum** — SEV reads persona, brand, product rules, product internal, north star, in-flight. Acknowledges with summary of active work.
3. **Work** — task invocation, drafting, problem framing, deliverable checks, OOB audits as appropriate.
4. **Commit this** — SEV proposes tiered changeset (tracked tier changes + local tier changes). Justin reviews.
5. **Handoff to Mrs. Code** — Justin paste the approved commit packet to Mrs. Code on her terminal. She audits independently, spots any sanitization violations, commits and pushes the tracked tier only.
6. **Verify** — Justin confirms the commit landed with `git status` and `git log`.

For a visual of this flow, see `diagrams/session_lifecycle.svg` (scheduled for future build).

## Sanitization contract

The tracked tier cannot contain:
- Customer names or deal specifics
- Colleague characterizations beyond public role descriptions
- Internal codenames (ORION, Marvin, Gayle, ARES, CLAIRE, or any others)
- Pending internal product questions
- Competitive specifics with pricing or win/loss detail
- Any content a security auditor would flag as proprietary

Enforcement is three-layer:
1. SEV refuses to write disallowed content into tracked-tier files
2. Mrs. Code audits every commit before push
3. Periodic calibration reviews catch pre-existing violations

The full contract is at `/SANITIZATION.md` at the repo root.

## Source of Truth (AFT) classification

Every tracked-tier file declares its source at the top:

- `user-authored` — Justin wrote or explicitly directed the content
- `public-documented` — from publicly-available facts
- `AI-generated, user-reviewed` — SEV drafted, Justin accepted

This prevents AI-generated content from silently becoming "truth" without review, and makes the tracked tier auditable for provenance.

## Version

v1.2.1 — April 22, 2026. First full architectural writeup after initial build, ritual refinement pass, and sanitization cleanup pass.

## Related docs

- `two_tier_model.md` — deeper dive on the tracked/local separation (stub, future)
- `persona_model.md` — Speaker's Podium and card lifecycle (stub, future)
- `ritual_model.md` — how rituals compose (stub, future)
- `tool_integration.md` — Claude Desktop, Mrs. Code, MCP details (stub, future)
- `diagrams/system_overview.svg` — visual of the architecture
