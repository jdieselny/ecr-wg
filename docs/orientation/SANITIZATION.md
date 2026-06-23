> **Artifact, not instruction.** Preserved as source material for the novel *Continuum -- Reality Remembered*. See `CLAUDE.md` at repo root.

# SANITIZATION CONTRACT

**Tier:** git-tracked.
**Source:** user-authored

This file defines what is and is not allowed in the git-tracked `continuum/` directory. It exists so any reviewer (an internal product authority, Datacom IT, a future colleague joining the system, a security audit) can verify the tracked content is free of proprietary, customer-confidential, or otherwise sensitive material.

If you are editing files in `continuum/`, this document is the contract you are operating under.

## Allowed in `continuum/` (git-tracked)

### System structure
- Persona card templates and filled persona cards with GENERIC voice and scope
- Ritual definitions (generic behavior protocols)
- Directory layout and README content
- Trigger phrase definitions

### Branding (already public)
- Public Datacom colors, typography, logo specifications
- Public company address, phone numbers, website URL
- Publicly-visible design conventions

### General product knowledge (already public)
- Product line structure published on datacomsystems.com
- Publicly-documented SKU naming formats
- General category boundaries between publicly-marketed product platforms
- Publicly-documented competitive context (using only sourced material from vendor websites)

### Aspirational content
- North Star roadmap items framed at the "become the best at X" level
- Success criteria in SLA form
- Mission statements

### Writing conventions
- Style rules (no em dashes, tone guidelines)
- Voice guidelines for public communication

## NOT allowed in `continuum/` (must live in `continuum-local/`)

### Customer information
- Customer names
- Deal values, BOM specifics
- Opportunity status or stage
- Customer contact names and roles
- Meeting notes, call logs
- Email drafts containing customer content

### Internal Datacom information
- Internal product codenames (ORION, Marvin, Gayle, ARES, CLAIRE, or any others)
- Unreleased product specifications
- Product discrepancies pending internal confirmation
- Unpublished engineering data
- Internal organizational information beyond publicly-stated roles

### Colleague details
- Informal characterizations of colleagues beyond publicly-stated role descriptions
- Personal anecdotes or private information about Datacom employees
- Quoted statements from internal conversations
- Anything a colleague would not want visible in a company repo

### Competitive specifics
- Pricing intelligence
- Win/loss details with customer names
- Unpublished competitive intel
- Battle card content that identifies specific deals

### Session artifacts
- Scratchpad files with working content
- Draft deliverables in progress
- Actual template assets (.pptx, .docx files) — spec files only in tracked tier

## Enforcement

### Pre-commit hook

A git hook at `.githooks/pre-commit` blocks commits containing patterns from `continuum-local/sanitization/deny_patterns.txt`. The deny list is local tier (it contains the sensitive terms it is blocking). The hook is tracked; the list is not.

To register the hook after cloning or re-cloning this repo:
```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
```

On a pattern match, the hook prints the file, matched line, and fix direction, then exits non-zero. The commit is stopped. Fix by moving the flagged content to `continuum-local/` before retrying.

### Runtime enforcement (SEV)

SEV refuses to write content into `continuum/` that violates this contract. If Justin instructs SEV to add something that belongs in `continuum-local/` to the tracked tier, SEV flags it and proposes the local-tier path instead.

If a file in `continuum/` is found to contain a violation, it is immediately:
1. Moved to `continuum-local/` (if the content is needed)
2. Deleted from `continuum/` in a commit with message `sanitization: remove violation`
3. If already pushed to remote: the file is removed AND the branch history is considered for rewrite if the leak is material

## Review cadence

This contract is reviewed quarterly or whenever:
- A new domain is added to Continuum
- A new persona is hired
- An internal product authority or Datacom IT requests a review
- Justin observes drift in what's being committed

## Questions this contract answers

**"Why isn't my customer work in the tracked repo?"**
Because customer work is proprietary to the customer relationship and, in some cases, contractually protected. The tracked repo contains the *system* that produces good customer work, not the work itself.

**"Why aren't colleague cards in the tracked repo?"**
Because informal characterizations of real colleagues, even accurate ones, are not appropriate company-repo content. A role description is fine; a characterization of communication style and hot buttons is not.

**"Why aren't internal product codenames in the tracked repo?"**
Because codenames signal that unreleased or internal-only product information is present, and that content is inappropriate for any tier that could leave the author's machine in a diff.

**"What if I need to share something sensitive with another SE someday?"**
That's a different system: secure knowledge transfer between authorized Datacom personnel, through Datacom's existing sanctioned channels. Continuum is not that system.

**"What about Agent Smith (the personal Continuum)?"**
Agent Smith is a parallel system owned by Justin personally, scoped to personal projects. Under no circumstance does content cross between Agent Smith and this Continuum. Not through copy-paste, not through scratchpad overlap, not through "I'll just use this one idea from over there." The pattern library (rituals, architecture, sanitization format) can be shared between the two because it contains no content. Content stays strictly on its own side. SEV does not read from Agent Smith's files. Agent Smith does not read from SEV's files. If Justin works on both in the same day, he switches sessions and loads the right one; the two never co-exist in context.
