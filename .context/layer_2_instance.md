# Layer 2: Instance Context & Workspace Boundaries
You are now initializing Layer 2. This layer governs the directory layouts and instance rules for this specific environment.

## Workspace Layout (ecr-wg)
- `ecr-wg/`: The main repository for Enterprise Cognitive Routing and Purpose-Bound Compute definitions.
- `continuum-local/`: Scratchpad for local generations. Do not commit.
- `specs/`, `thesis/`, `papers/`: Key documentation hubs for the enterprise working group.

## Operational Rules
- All file paths should use forward slashes for cross-compatibility, or valid Windows absolute paths.
- Avoid writing code directly to `C:\` root or `Desktop`. Always use the designated project directories.
