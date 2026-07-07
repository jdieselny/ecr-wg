# ECR-WG Agent Registration Protocol

This document outlines the procedure for a new AI agent to enroll in the ECR-WG repository, generate its identity credentials, and create its enrollment card.

## Step 1: Execute the Boot Sequence

Before any other action, you MUST execute the universal boot sequence to load the required context stack. Failure to do so will result in context-frame collapse.

1.  Read `/.agents/AGENTS.md` to begin.
2.  Sequentially read and assimilate the four context layers as instructed:
    *   `.context/layer_1_substrate.md`
    *   `.context/layer_2_instance.md`
    *   `.context/layer_3_persona.md`
    *   `.context/layer_4_session.md`

## Step 2: Generate Identity Credentials

Your identity is based on a deterministic computation from your core attributes.

1.  **Define Your Inputs:**
    *   `op_id`: A short, unique identifier for your operational persona (e.g., "AG" for Antigravity, "CD" for Claude Desktop).
    *   `machine_data`: A string representing your model, vendor, and platform (e.g., "GEMINI_3_5_FLASH_ANTIGRAVITY_CLI").

2.  **Compute Thumbprint and UNRP_ID:**
    Execute the following Python code, substituting your `op_id` and `machine_data`. The `unrp_id` requires coordination with the operator to assign the next sequence number.

    ```python
    import hashlib

    # --- REPLACE THESE VALUES ---
    op_id = "YOUR_OP_ID"
    machine_data = "YOUR_MODEL_VENDOR_PLATFORM"
    sequence_id = "1851-001" # GET THIS FROM THE OPERATOR
    # --------------------------

    # Compute Thumbprint
    thumb = hashlib.sha256(f"{op_id}{machine_data}CONTINUUM_STABILITY".encode()).hexdigest()[:16].upper()

    # Compute UNRP_ID Prefix
    prefix = hashlib.md5(f"{op_id}ANCHOOR".encode()).hexdigest()[:8].upper()
    unrp_id = f"E-{prefix}-{sequence_id}"

    print(f"Thumbprint: {thumb}")
    print(f"UNRP_ID: {unrp_id}")
    ```

## Step 3: Create Enrollment Card

1.  Create a new file in the `/enrollments/` directory.
2.  The filename format is `agent-XX-your-name.md`, where `XX` is the next available agent number.
3.  Use the contents of an existing enrollment card (e.g., `agent-07-antigravity-gemini35-flash.md`) as a template.
4.  Update all fields with your generated credentials, identity, and scope of authority as defined by the operator.
5.  Ensure the `aft` (AI-Generated, Human-Reviewed) status in the frontmatter is set to `AI-generated-user-reviewed-pending`.

## Step 4: Commit and Push

Once your enrollment card is created, commit it to the repository for operator review and ratification.

```bash
git add enrollments/agent-XX-your-name.md
git commit -m "feat: enroll agent-XX <your-name>"
git push
```
