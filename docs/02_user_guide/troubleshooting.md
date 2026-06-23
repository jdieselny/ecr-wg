# Troubleshooting

**Tier:** git-tracked.
**Status:** STUB — will populate from real failure cases as they occur.

## What this doc will contain

A growing catalog of known failure modes and their recoveries. Every entry follows the format:

- **Symptom** — what the user sees
- **Cause** — what actually went wrong
- **Recovery** — exact steps to fix
- **Prevention** — how to avoid it next time

## Initial seed entries (to be fleshed out)

**POST fails with "workspace not found"**
Likely cause: Filesystem MCP scope doesn't cover the workspace path, or the path has moved.
Recovery: verify MCP configuration, test file access from Claude Desktop, update MCP scope if needed.

**Load Continuum returns a generic acknowledgment**
Likely cause: persona card or shared files are empty, missing, or corrupt.
Recovery: open the files manually, verify content. If fresh clone, run git pull. If corrupt, restore from remote or from a backup.

**Mrs. Code rejects a commit for sanitization**
Likely cause: content SEV proposed contains customer names, codenames, or other tracked-tier-forbidden content.
Recovery: accept Mrs. Code's call. Fix the content. Try again. If it happens twice in one session, pause and re-read SANITIZATION.md.

**Git push rejects with authentication error**
Likely cause: SSH host alias routing is broken, or key isn't loaded in agent.
Recovery: test with `ssh -T git@github.com-datacom`. If that fails, debug SSH config.

**Rogue .git directory at a too-broad scope**
Likely cause: accidental `git init` in home directory or workspace umbrella.
Recovery: DO NOT run `git add .` anywhere. Identify the rogue repo with `find . -name .git -maxdepth 3`. Verify it has no unintended history. Delete the `.git` directory.
Prevention: never run `git init` outside a dedicated project directory.

**"I accidentally committed customer content to the tracked tier"**
This is a serious incident. See `../03_operator_guide/incident_recovery.md` (stub) for the formal procedure. Short version: do not push if you haven't yet. If pushed, follow the incident response: rotate any exposed secrets, consider history rewrite carefully, notify Tomy or appropriate internal authority.

## When this will be populated

Organically, as failures occur. Every new failure gets an entry. Goal: new SEs don't rediscover the same failure mode Justin already hit.

## Related docs

- `getting_started.md` — setup guide
- `../03_operator_guide/incident_recovery.md` — major incident procedure (stub)
