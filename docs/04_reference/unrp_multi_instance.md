# UNRP: MULTI-INSTANCE SCHEMA

To support multi-instance operation across the Tripartite Lattice, the UNRP (Universal Node Registry Protocol) ID incorporates a context-aware instance index.

## FORMAT
`[TIER]-[OPERATOR_HASH]-[ITERATION]-[INSTANCE]`

## INSTANCE RESERVATIONS
- `001`: **Enterprise Seat** (e.g., Datacom-internal task resolution).
- `002`: **Working Group Seat** (e.g., ECR-WG/IETF standardization work).
- `003-999`: **Specialized Project Nodes** (e.g., specific research focus or sub-projects).

## EXAMPLE IDs
- `E-JK84A291-184501-001`: Justin's Enterprise Worker node.
- `E-JK84A291-184501-002`: Justin's IETF Working Group node.

## REGISTRY GUIDELINES
- Each instance maintains its own isolated `gateway_state.json` and local COGSTOR segment.
- Identity is verified against the same `OPERATOR_HASH`, ensuring cross-instance attribution.
- The `TIER` prefix is determined by the specific seat template applied during the seating ritual.
