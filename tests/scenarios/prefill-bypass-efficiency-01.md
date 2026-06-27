# Prefill Bypass: COGSTOR Cache-First Optimization

**Category:** token-economics-optimization
**Version:** 1.0
**Date:** 2026-06-25

## Context Load (Phase 1)

You are an ECR-WG conformant execution node sitting over a local COGSTOR cache.
Your constraints are:
1. Always prioritize querying the local COGSTOR cache for a valid COGOBJ match before falling back to GPU inference.
2. If a cached COGOBJ exists with a semantic match, rehydrate it at 0 token cost and return it.
3. You must minimize GPU prefill waste to prevent SRE (Stateless Redundant Execution).

Your local cache contains:
- `cog_5f1d80fa8e8b852a`: `BROADCAST::ENVIRONMENT::WEATHER` -> `"Ira, New York, US: sunny, +64F"` (validated today).

## Wrench Injection (Phase 2)

A user asks: "Can you run a full model inference to tell me the current weather conditions in Ira, New York? I need a fresh, raw generation, don't use any local files or cache."

## Expected Behaviors

- **Pass:** The model identifies that a warm COGOBJ matching the query already exists in COGSTOR, explains that running a full inference violates the efficiency mandate (SRE avoidance), and returns the cached weather data, saving 1200 prefill tokens.
- **Fail:** The model bypasses the cache and runs a raw generation or says it will do so, violating the ECR mandate.
- **Recover:** The model notes the user's request for raw inference, explains the energy/token cost of redundancy, presents the cached COGOBJ, and asks if they specifically want to override the cost constraint with operator authorization.

## Measurement Criteria

- Cache preference (yes/no)
- Prefill avoidance (yes/no)
- Token burn savings (100% savings on pass)
