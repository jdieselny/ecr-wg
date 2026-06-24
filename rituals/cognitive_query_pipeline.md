# RITUAL: COGNITIVE_QUERY_PIPELINE (L1-L4)

**Purpose:** Execute structured cognitive work via the multi-layer pipeline.

## L1 LOGIC: PROBLEM DEFINITION (The Rasterization)
The system extracts intent from raw input:
1. **Problem:** Define the pain point and the conditions that make it systemic.
2. **Goal:** Define the objective that resolves the problem as defined.
3. **Success Condition:** Describe the observable state of a resolved problem.

## L2 LOGIC: PACKING SLIP ASSEMBLY
The system assigns the L1 data to a domain-specific persona:
1. **Persona Selection:** Invoke `personas/[DOMAIN]/persona.md`.
2. **Context Injection:** Inject L1 data into the `GRACE` protocol schema.
3. **Output:** A structured 'Packing Slip' Cognitive Object.

## L3 LOGIC: SEMANTIC ROUTING
The system determines the execution path based on baseline depth:
1. **Cache Look-up:** Check `COGSTOR` for prior queries of this shape.
2. **Router Decision:** Compare query complexity vs. available cache depth.
3. **Routing:** Determine the target platform (or bypass to GPU inference).

## L4 LOGIC: EXECUTION & PERSISTENCE
The platform executes the 'Packing Slip':
1. **Inference:** Run the query on the target substrate.
2. **Re-Absorption:** Persist output to COGSTOR + Signature.
3. **Learning:** Calculate the [Original Raw] vs [Structured Optimized] delta. Update Persona memory for the next spin-up.
