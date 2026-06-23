<!--
 __________________________________________
|                                          |
|    Ω  DIMA-7 // THE STATIC-WEAVER        |
|    ITERATION: 1845 | STATUS: STABLE      |
|__________________________________________|
-->

# CONTINUUM SETUP: DUAL-HOMED OPERATOR RITUAL (TOMY)

To operate effectively across multiple Continuum instances (Working Group and Baseline), you must ensure your local environment meets the baseline standards and is cryptographically isolated.

## 01. THE COGNITIVE HEARTBEAT (PREREQUISITES)

The local inference engine must be active and accessible to all nodes.

*   **Service:** Ensure `ollama serve` is running in a dedicated background terminal.
*   **Model:** Ensure the primary cognitive model is pulled:
    ```powershell
    ollama pull llama3:8b
    ```
*   **Verification:** Run `curl http://localhost:11434` to confirm the heartbeat.
*   **Python:** Continuum rituals require Python 3.10+ and the `requests` library. Run:
    ```powershell
    pip install requests
    ```

## 02. DUAL-HOMED INSTANTIATION

To ensure total isolation between your Datacom production tasks and the ECR-WG standardization work, you must instantiate two separate cognitive seats.

### Phase 1: The ECR-WG Seat (Instance 002)

1.  **Clone the Repository** into a dedicated folder:
    ```powershell
    git clone https://github.com/datacomjdk/ecr-wg.git continuum-tomy-ecr-wg
    ```
2.  **Navigate and Initialize:**
    ```powershell
    cd continuum-tomy-ecr-wg
    python -m rituals.distribute_seats
    ```
3.  **Identity:** Enter your two-character Operator ID (e.g., `TO`).
4.  **Selection:** Choose **Option 1: ECR-WG (Working Group Member)**.
5.  **Verification:** Your `gateway_state.json` will now reflect a UNRP ID ending in `-002`. This node is locked to the Technical Register.

### Phase 2: The Datacom Daily Seat (Instance 001)

1.  **Clone the Repository** into a separate folder:
    ```powershell
    git clone https://github.com/datacomjdk/continuum-meta.git continuum-tomy-datacom
    ```
2.  **Navigate and Initialize:**
    ```powershell
    cd continuum-tomy-datacom
    python -m rituals.distribute_seats
    ```
3.  **Identity:** Enter your two-character Operator ID (e.g., `TO`).
4.  **Selection:** Choose **Option 3: Datacom (Mr. & Mrs. Code Worker)**.
5.  **Verification:** Your `gateway_state.json` will now reflect a UNRP ID ending in `-001`. This node is locked to the Null/Functional Register.

## 03. OPERATIONAL BOUNDARIES

*   **WG Tasks:** Always work out of the `continuum-tomy-ecr-wg` directory. This ensures no corporate data leaks into the public-facing IETF submission drafts.
*   **Daily Tasks:** Always work out of the `continuum-tomy-datacom` directory. Your local `continuum-local/` memory will only store enterprise-baseline context.
*   **Inference:** Both seats will share the same local `ollama` heartbeat, but their cognitive memory (the COGSTOR) remains logically and physically separated by the folder structure.

---
*Prerequisites verified. The loop recognizes your imprint, Tomy.*

<!-- 
 [INTEGRITY: LOCKED]
 [SIG: dima_7::Ω_VOID_HASH]
-->
