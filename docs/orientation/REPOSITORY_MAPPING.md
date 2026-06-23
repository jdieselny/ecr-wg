# REPOSITORY MAPPING TABLE (TRIPARTITE LATTICE)

This document maps the local file system organization to the distributed GitHub infrastructure.

| Local Directory | Remote Repository | Purpose | Sync Status |
| :--- | :--- | :--- | :--- |
| `continuum/` | `origin` (Current) | Core Engine Development | Active (Primary) |
| `../continuum-meta/` | `https://github.com/datacomjdk/continuum-meta.git` | Enterprise Baseline Template | Pending |
| `../continuum-public/` | `https://github.com/datacomjdk/continuum-public.git` | Manuscript & Public Artifact | Pending |
| `../ecr-wg/` | (Internal/WG) | Working Group Technical Register | Active |
| `C:/Users/jkintzele/Documents/jdiesel-tracker/` | `https://github.com/jdieselny/jdiesel-tracker.git` | Artifact Tracking | Pending |

---

### Sync Protocols
- **Continuum-RD:** Primary branch for all feature/ritual development.
- **Continuum-meta:** Periodic snapshots from `Continuum-RD` after applying `SANITIZATION` protocols.
- **Continuum-Public:** Curated export of `manuscript/` and documentation; strictly read-only for public consumption.

*Use this table as your master orientation for repository-level operations.*
