# Boot Paths — read before any Continuum work

**Rule:** Match your **workspace cwd** to the row below before loading Layer 4 or executing handoffs.

| cwd | Role | Layer 4 | SESSION_CLOSE |
|-----|------|---------|---------------|
| `~/Documents/ecr-wg` | **THIS REPO** — public standards, cleanroom, SCITT demo | `.context/layer_4_session.md` | `planning/session_close_grok_2026-07-12.md` |
| `~/Documents/continuum` | Private XGPC fleet, bridge, AFT diagrams | `.context/layer_4_session.md` | `session_close_grok_2026-07-12.md` |
| `~/Documents/jdiesel-continuum/continuum` | J Diesel commercial / ISR | `layer_4_session.md` | `rituals/session_close_grok_2026-07-12.md` |
| `~/Documents/jdiesel-continuum/god-terminal` | Mr Cloud VM runtime | jdiesel `layer_4_session.md` | jdiesel close |

## On SESSION_CLOSE
1. Assert cwd is `ecr-wg`
2. Rewrite `.context/layer_4_session.md` here only
3. Commit + push this repo (exclude `mcps/`, `terminals/` harness dirs)