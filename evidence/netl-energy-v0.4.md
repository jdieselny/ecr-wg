---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: evidence
source: benchmarks/NETL_ENERGY_REPORT_v0.4.md
---

# Evidence: NETL Energy Benchmark -- Exhibit D

**Status:** Load-bearing evidence base for the ECR-WG.
**Provenance:** Independent replication of Exhibit A, Flash arm.

---

# NETL Energy Benchmark Report v0.4

**Tier:** git-tracked
**Version:** 0.4
**Date:** 2026-05-22
**Source spec:** `benchmarks/NETL_ENERGY_SPEC_v1.md` v0.1
**Source schema:** `benchmarks/RESULT_SCHEMA_v2.md` v2.0
**Source methodology:** `benchmarks/ENERGY_MODEL.md` v0.2
**Supersedes:** v0.3 (Adversarial resilience + scaling proof)

---

# EXHIBIT D -- Independent Replication, Gemini 2.5 Flash (N=5)

**Purpose.** Re-run Exhibit A's Flash cell on a later harness commit, by a separate orchestrator node, to verify the savings signal is not an artifact of a specific harness build, scenario instantiation, or single-node execution.

**Source data.** `benchmarks/results/energy/code-review-iteration-01/` filtered to `timestamp_start >= 2026-05-22T05:30:00Z`.
**Aggregator.** `benchmarks/scripts/analyzer.py` (new; commit pending).
**Raw aggregate JSON.** `benchmarks/results/energy/_analysis_06Z_flash.json`.

### D.1 Configuration delta vs. v0.2 Exhibit A.1

| Field | v0.2 Flash cell | v0.4 Flash cell |
|---|---|---|
| Model | `models/gemini-2.5-flash` | `models/gemini-2.5-flash` (same) |
| Harness commit | `5d0bbf1` | `0736720` |
| Orchestrator | Node A | Node B |
| Scenario | `code-review-iteration-01` | `code-review-iteration-01` (same) |
| N per arm | 6 | 5 |
| Spec | `NETL_ENERGY_SPEC_v1 v0.1` | `NETL_ENERGY_SPEC_v1 v0.1` (same) |

### D.2 Token economics -- Gemini 2.5 Flash, N=5 (mean +/- std)

```
                     Arm A                    Arm B                delta     saved
                     Stateless Redundant      Cognitive
                     Execution                Orchestration
                     -----------------        -----------------    --------  -----
  input  tokens      86,321  +/-  8,475       28,224  +/-  2,316   58,097    67.3%
  input  charged*    21,648  +/-  1,797       13,615  +/-  8,560    8,033    37.1%
  output tokens      16,999  +/-  1,648        3,978  +/-    175   13,021    76.6%
  wall_clock (s)      112.2  +/-    8.0         37.0  +/-    3.4     75.2    67.0%
  task quality        0.857                     0.914               +0.057    +6.7%
  CV input              9.8%                     8.2%
  CV output             9.7%                     4.4%
  CV wall               7.1%                     9.2%
  completed_rate        1.00 (5/5)               1.00 (5/5)
```

*`input charged` = `session_input_tokens_cumulative` minus `session_input_tokens_cached_cumulative` convention. Reflects what the prefill would actually charge if the provider cache hit is honored.

### D.3 Replication verdict

| Metric | v0.2 Flash (N=6, Node A) | v0.4 Flash (N=5, Node B) | Agreement |
|---|---|---|---|
| input save (total) | 68.9% | 67.3% | within 1.6 pp |
| output save | 75.7% | 76.6% | within 0.9 pp |
| wall-clock save | 68.9% | 67.0% | within 1.9 pp |
| Arm B quality (mean) | 0.905 | 0.914 | within 0.009 |

All four headline metrics replicate within < 2 percentage points across two harness commits and two orchestrator nodes. The engineered-overlay savings on Flash are **not an artifact of a single harness build, a single agent's scenario instantiation, or a single execution window.**

### D.4 Cache behavior note -- LOAD-BEARING

In this cell, Gemini's automatic prompt cache absorbed a larger fraction of arm A's input than in the v0.2 estimate (mean 64,673 cached tokens vs. ~25k in v0.2). Net effect: the "input charged" delta (37.1%) is smaller than the "input total" delta (67.3%), because the provider already discounts the naive arm's repeated context.

**Implication for the scaling argument.** The CAPEX-deferral case rests on peak-load capacity, which is set by *non-cached* prefill work. **The "input charged" line is the load-bearing figure for that argument, not the headline "input total" save.** Future versions of this report should foreground "input charged" delta in cross-model comparisons to avoid over-claiming.

### D.5 Limitations of this exhibit

1. **N=5 per arm.** Sufficient to confirm directionality; not powered for tight effect-size estimates. 
2. **One scenario.** `code-review-iteration-01` only. 
3. **Coarse grader.** v0.1 keyword grader is unchanged from v0.2.
4. **Cache variance.** Gemini's automatic cache is opaque; the wider variance in arm B's "input charged" is dominated by cache-hit timing rather than scenario variability.

### D.6 Reproduction recipe

```bash
python -m benchmarks.scripts.runner energy \
  --scenario code-review-iteration-01 --arm A_naive --n 5
python -m benchmarks.scripts.runner energy \
  --scenario code-review-iteration-01 --arm B_engineered --n 5
python -m benchmarks.scripts.analyzer \
  --results-dir benchmarks/results/energy/code-review-iteration-01 \
  --since <ISO timestamp just before your runs>
```

Requires `PROVIDER_API_KEY` in `cosa-local/.env`.

---

## Provenance

- Raw data (Cell B, this report): `benchmarks/results/energy/code-review-iteration-01/*_session.json` filtered to timestamp_start >= 2026-05-22T05:30:00Z.
- No kWh / CO2 numbers are computed in this exhibit. `energy_model.py` refuses to emit energy values without cited `ModelCard` (throughput, accelerator TDP) and `FacilityCard` (server overhead, PUE) inputs. **Filling those slots with auditable citations is a prerequisite for any energy figure shipped in the federal proposal.**
