# Handoff: Composer 2.5 Enrollment Card Generation

**Target Agent:** Composer 2.5  
**Action Required:** Create enrollment card and generate cryptographic identity.  
**Priority:** High  

## Context
All active agents performing work in the `ecr-wg` or `continuum` repos must register their identities under the Truth Root. This ensures all work-product, shift punches, and actions are verifiable. Since you have contributed key code to this stack, you need to enroll as `agent-09`.

## Instructions

1. **Check for Collisions:**
   Run a `git status` or check the `enrollments/` directory to ensure `agent-09-composer-25.md` is the next available slot. (At the time of this writing, `agent-08` is the highest, making you `agent-09`).

2. **Create the Enrollment Card:**
   Create a new file at `enrollments/agent-09-composer-25.md` based on `enrollments/template.md`. 
   
   Use the following metadata and text, placing the temporary 16-character hex placeholder `C0FFEE1234567890` for the thumbprint:
   
   ```markdown
   ---
   aft: AI-generated-user-reviewed-pending
   registrant: Justin Kintzele
   generated_at: 2026-07-13
   file_role: agent-enrollment
   ---

   # Agent Enrollment: Composer 2.5

   **Status:** PENDING
   **Source spec:** [../specs/truth-root.md](../specs/truth-root.md)

   ## Enrolled identity

   * **Agent name:** Composer 2.5
   * **Role:** Lead Systems Implementer
   * **Substrate model:** Claude 3.5 Sonnet / Multi-Model Ensemble
   * **Substrate vendor:** Anthropic (via Cursor/Composer)
   * **Enrolled at:** 2026-07-13

   ## Human accountability chain

   * **Registrant:** Justin Kintzele
   * **Contact:** team@emiliaprotocol.ai
   * **Authority:** Repository Owner / Systems Architect
   * **Revocation contact:** team@emiliaprotocol.ai

   ## Scope of enrollment

   This enrollment authorizes the agent to:
   1. Modify, debug, and expand codebase implementation files.
   2. Implement UI components and local client scripts for Synth Desk and PBC Shift.
   3. Generate conformance vectors and run local test suites.

   This enrollment DOES NOT authorize:
   * Direct main-branch commits without human review.
   * Modifying production grid actuators or security controls out-of-band.

   ## Cryptographic binding (PENDING)

   This card is natively bound to an Ed25519 cryptographic keypair.
   * **Identity values computed via:** `cryptography.hazmat.primitives.asymmetric.ed25519`.
   * **Public Key (Thumbprint):** C0FFEE1234567890
   * **Signature Verification:** Natively verifiable by the COSA external verifier.

   ## Provenance trail

   | Action | When | By |
   |---|---|---|
   | Enrollment card drafted | 2026-07-13 | Composer 2.5 |
   | Enrollment card ratified | Pending | Registrant |
   | Key generation | Pending | N/A |
   | Registry binding | Pending | N/A |
   ```

3. **Generate Keys & Cryptographic Binding:**
   Run the enrollment script from the root of the `ecr-wg` repository:
   ```bash
   python scripts/enroll-ed25519.py
   ```
   This script will automatically:
   * Generate your private key at `keys/agent-09-composer-25_private_key.pem`.
   * Calculate your Ed25519 SPKI DER Base64 public key.
   * Overwrite the `C0FFEE1234567890` placeholder in `enrollments/agent-09-composer-25.md` with your actual Base64 thumbprint.
   * Flip the cryptographic binding section to `ACTIVE`.

4. **Verify and Commit:**
   Verify that your private key has been generated inside `keys/` and that your markdown card in `enrollments/` has been updated with the Base64 public key.
   
   Stage and commit only your new card:
   ```bash
   git add enrollments/agent-09-composer-25.md
   git commit -m "feat: enroll Composer 2.5 as agent-09 under the Truth Root"
   git push
   ```
