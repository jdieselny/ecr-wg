# Tool Integration (deep dive)

**Tier:** git-tracked.
**Status:** STUB — overview in `system_overview.md`. Detailed configuration guidance pending.

## What this doc will contain

The operational guide to the tools Continuum depends on. Topics:

- **Claude Desktop + Filesystem MCP.** Detailed install and configuration. Scope boundaries. What to do when MCP fails or misbehaves. Version compatibility notes.
- **Claude Code as Mrs. Code.** How to set up Claude Code for the OOB reviewer role. Authentication, working-directory conventions, the specific prompt pattern that makes Mrs. Code effective at sanitization review.
- **Git configuration.** SSH host aliases, per-repo identity, how to verify the config is correct, how to diagnose authentication failures.
- **The handoff packet format.** What SEV outputs for Mrs. Code to consume. The structure that makes OOB review reliable.
- **Cross-machine setup.** Considerations when Continuum runs from multiple machines (laptop + desktop). Conflict avoidance.
- **When tools change.** How to update docs when Anthropic ships a new Claude Desktop, new Claude Code version, new MCP connector, etc.

## Why this doc matters

The tools are under Anthropic's control and will evolve. This doc needs to capture current-state configuration clearly enough that tool updates produce known impacts rather than mysterious breakage.

## When this will be written

First full draft after the first tool-update-induced breakage that isn't trivially recoverable. We learn more from the real failure than from speculation.

## Related docs

- `system_overview.md` — high-level tool inventory
- `../02_user_guide/getting_started.md` — the setup guide for first-time users
- `../02_user_guide/handoff_to_mrs_code.md` — the commit workflow (stub)
