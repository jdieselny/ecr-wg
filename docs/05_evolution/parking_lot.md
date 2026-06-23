# Parking Lot

**Tier:** git-tracked.
**Status:** SEED — populated with v1.0 design decisions that were considered and declined. Add to this as new ideas are deferred.

Ideas considered and explicitly not adopted, with rationale. Consult before re-proposing. An idea landing here isn't dead forever; it's dead until new information justifies reopening.

## Declined during v1.0 design (April 22, 2026)

### Three-root-directory architecture (Gemini proposal)

**Idea:** Separate workspace into three root directories: one for tracked rules, one for local state, one for outputs/artifacts.

**Why declined:** The output/artifacts concept collapses into the local tier naturally. Three roots add cognitive overhead ("which root does this go in?") without meaningful benefit. The two-tier model is already at the complexity ceiling for a solo user.

**What would reopen this:** If a future use case genuinely needs a third tier with different access controls (e.g., a read-only "reference corpus" tier that multiple Continuums share), the three-root model could be revisited.

### Boot document into both agents (Gemini proposal)

**Idea:** Write a single authoritative boot document that both SEV and Mrs. Code load at session start, so they have identical context.

**Why declined:** The whole point of Mrs. Code being a reviewer is that she's structurally separate from SEV. Shared boot context undermines that separation. Mrs. Code should know enough to audit (the sanitization rules, the repo layout) but NOT the same rich context SEV has. Asymmetry is a feature.

**What would reopen this:** Probably nothing. The separation principle is foundational.

### Blurring work and personal Continuum

**Idea:** A single Continuum that serves both work (Datacom) and personal projects, with tagging to distinguish content.

**Why declined:** Tagging is a fragile sanitization mechanism compared to physical separation. Work content accidentally tagged "personal" (or vice versa) could leak across the boundary. Two separate Continuums with zero content crossover is structurally safer. Pattern sharing is the only cross-Continuum link, and that's a deliberate action, not ambient.

**What would reopen this:** Never on the merge side. Pattern extraction to a neutral third repo IS on the roadmap but remains distinct from blurring the actual content tiers.

### "Cognitive Hypervisor" as the initial name

**Idea:** Name the system "Cognitive Hypervisor" from day one to capture the aspirational multi-persona-concurrency vision.

**Why declined:** Aspirational names set false expectations. v1 runs one persona at a time — that's not a hypervisor, that's a context OS. "Continuum" captures what it actually does (persistent context across time) without promising concurrency it doesn't have. If/when the system gains true multi-persona concurrency, "Hypervisor" becomes accurate and can be adopted as a subtitle or major-version rename.

**What would reopen this:** Achievement of genuine multi-persona concurrency with resource arbitration.

### Ambient background learning

**Idea:** SEV learns from every session automatically, updating rules and preferences without explicit review.

**Why declined:** Violates the "user is the final authority" principle. Silent learning means silent drift. Explicit proposal via Commit this + explicit review + explicit push is the enforcement mechanism for the sanitization contract, brand consistency, and quality gates. Background learning routes around all of that.

**What would reopen this:** Probably nothing. The explicit-review principle is load-bearing.

### Autonomous task execution

**Idea:** SEV takes on multi-step tasks autonomously and reports back with results.

**Why declined:** Same family as background learning. The value of Continuum is quality through deliberation, not speed through autonomy. Faster is not better when faster means less reviewed.

**What would reopen this:** Specific narrow tasks where autonomy is genuinely safe and valuable. E.g. "run POST" is already autonomous within its scope. If specific other tasks prove reliably safe, they could be elevated to autonomous status with explicit scoping. But general autonomy, no.

## How to add to this list

When an idea is considered and declined, write it up here with: the idea, why declined, what would reopen it. The third field matters — it prevents this from becoming a graveyard where ideas go to be forgotten.

## How to move something back off the list

If an item's "would reopen this" condition becomes true, the item returns to active roadmap consideration. Moving it back off the parking lot is itself a roadmap decision and should be documented in the next calibration notes.
