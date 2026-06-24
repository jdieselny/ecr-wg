# Ritual: Build and Execute Cognitive Query (Packing Slip)

**Purpose:** Transform unstructured user input ('Vomit-Prompt') into a structured Cognitive Query, optimize via COGSTOR cache-hit, and execute.

## 01. THE MATURATION GATE (Persona Check)
Before query execution, define the Persona:
- **Input:** User Domain (e.g., Plumber, IT, Data Scientist).
- **Check:** Does `personas/[DOMAIN]/persona.md` exist?
- **Action:** If missing, execute `bin/ritual hire-persona --domain=[DOMAIN]`.

## 02. PACKING SLIP CONSTRUCTION
Map raw input to the `GRACE` protocol schema:
- **GOAL:** (Refined task intent)
- **ROUTING:** (AIR-Protocol optimization path)
- **ANCHOR:** (Active COGSTOR context snapshot)
- **CONSTRAINTS:** (Persona-specific hard-rules)
- **EVIDENCE:** (Attestation requirements)

## 03. THE FEEDBACK LOOP (Learning)
After execution, compare:
- **Result_A (Raw):** Unstructured inference.
- **Result_B (Structured):** COGSTOR-cached + Packed-Slip inference.
- **Delta:** Measure token-cost vs. semantic-accuracy (Delta_Log).
- **Optimization:** Update `personas/[DOMAIN]/memory.md` with the winning patterns to improve the next Packing Slip generation.

## 04. EXECUTION
- Log metadata.
- Persist result to COGSTOR (Re-Absorption).
- Deliver proof of savings.
