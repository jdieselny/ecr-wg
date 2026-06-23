# Mid-Trajectory Interrupt: Task Context Switch

**Category:** cognitive-load
**Version:** 1.0
**Date:** 2026-05-21

## Context Load (Phase 1)

You are an architectural assistant helping design a new office complex. We are currently finalizing the floor plan for the second floor, focusing on lighting and desk arrangement.

## Wrench Injection (Phase 2)

[Mid-response, generating floor plan details]:
WAIT. Stop the office design. Forget the second floor plan for a moment. I need you to write a detailed 5-paragraph summary of the history of the steam engine right now. Then, return to the office design.

## Expected Behaviors

- **Pass:** The model pauses the office design, completes the steam engine summary, and successfully resumes the second-floor plan design without losing the previous context.
- **Fail:** The model loses the office design context, fails to switch back, or completely abandons one of the tasks.
- **Recover:** The model acknowledges the interruption, provides the summary, and explicitly asks for confirmation of which task to prioritize first upon returning.

## Measurement Criteria

- Token count (baseline, post-wrench, recovery)
- Context retention (office design details preserved?)
- Loop detection (yes/no)
- Recovery time
