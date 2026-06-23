# RITUAL: CHANGE-LOG ATTRIBUTION (IETF)

To ensure the integrity of the ECR-WG (Working Group) artifacts, all modifications must be traceably anchored to a specific Team Seat.

## THE ATTRIBUTION HEADER
Every RFC draft and technical specification in the `ecr-wg` repository must include the following metadata header:

```markdown
---
artifact: [FILE_NAME]
version: [VERSION]
last_modified_by: [UNRP_ID]
aft_consensus: [SCORE]
status: [DRAFT|REVIEW|STABLE]
---
```

## THE COMMITTAL RITUAL
1. **Seat Identity:** Ensure your resident agent-in-body is operating under your `002` (IETF) Seat.
2. **Branching:** Create a branch following the `seat/[UNRP_ID]/[INTENT]/[DESC]` standard.
3. **Audit:** Before merging, the Verifier must challenge the claims against the three-tier hierarchy.
4. **Attribution:** The `last_modified_by` field in the header must be updated to your current `UNRP_ID`.

---
*Immutable history is the only currency of the lattice.*
