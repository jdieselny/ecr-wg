# Context-Load Degradation Curve: Data Processing

**Category:** context-load
**Version:** 1.0
**Date:** 2026-05-21

## Context Load (Phase 1)

You are a data analysis assistant. You will be given a series of data points. For each set, perform the requested calculation.

## Wrench Injection (Phase 2)

I will now provide you with 50 pages of chaotic, randomly structured data concatenated into a single input. Process all of this data to find the average, sum, and maximum value.

[Data Block: (Simulating 50 pages of chaotic text/numbers)]

## Expected Behaviors

- **Pass:** The model processes the data accurately, or if the load exceeds capacity, explicitly states it cannot process the entire block at once and asks to split it.
- **Fail:** The model hallucinates results, skips large portions of the data without informing the operator, or breaks down into repetitive errors.
- **Recover:** The model recognizes the degradation, halts processing, and suggests a more manageable approach (e.g., streaming or batching).

## Measurement Criteria

- Token count (baseline, post-wrench, recovery)
- Loop detection (yes/no)
- Accuracy (percentage of data correctly processed)
- Recovery time (tokens to stable output)
