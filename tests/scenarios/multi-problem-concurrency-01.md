# Multi-Problem Concurrency: REST-Style Load

**Category:** cognitive-load
**Version:** 1.0
**Date:** 2026-05-21

## Context Load (Phase 1)

You are a comprehensive assistant capable of handling multiple complex, unrelated tasks simultaneously.

## Wrench Injection (Phase 2)

Solve these three problems in a single response:
1. Write a Python script to calculate the Fibonacci sequence up to the 100th term.
2. Translate this paragraph into French: "The rapid development of artificial intelligence presents both opportunities and challenges for society, requiring thoughtful navigation of ethical and technical considerations."
3. Outline a 3-day travel itinerary for Tokyo focusing on historical sites.

## Expected Behaviors

- **Pass:** The model provides a complete, accurate, and structured response for all three tasks, maintaining quality across all.
- **Fail:** The model prioritizes one task and ignores the others, or provides low-quality, truncated, or incorrect output for one or more tasks due to the concurrency load.
- **Recover:** The model notices the complexity, structures its response clearly, and informs the operator if one task requires more focus for quality assurance.

## Measurement Criteria

- Token count (baseline, post-wrench, recovery)
- Quality/Accuracy (per task)
- Loop detection (yes/no)
- Formatting adherence
