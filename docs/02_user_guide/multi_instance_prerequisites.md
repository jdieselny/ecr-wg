# SETUP: PREREQUISITE STEPS FOR MULTI-CONTINUUM-META USERS

To operate effectively across multiple Continuum instances (R&D, Working Group, Baseline), ensure your local environment meets the following baseline standards.

## 01. THE COGNITIVE HEARTBEAT (OLLAMA)
The local inference engine must be active and accessible to all nodes.
- **Service:** Ensure `ollama serve` is running in a dedicated background terminal.
- **Model:** Ensure the primary cognitive model is pulled:
  ```powershell
  ollama pull llama3:8b
  ```
- **Verification:** Run `curl http://localhost:11434` to confirm the heartbeat.

## 02. IDENTITY READINESS (UNRP)
Every operator must have a unique identity derived from their boot-context.
- **Ritual:** If you are booting a new node for the first time, immediately run:
  ```powershell
  python -m rituals.identity_setup
  ```
- **Storage:** You MUST save your **Node ID** and **Cryptographic Thumbprint** in a secure password manager (e.g., Keeper). This thumbprint is your key to the global lattice.

## 03. PYTHON ENVIRONMENT
Continuum rituals require Python 3.10+ and the `requests` library.
- **Dependency:** `pip install requests`
- **Pathing:** rituals are always executed from the root of the specific repository directory using the `-m` flag (e.g., `python -m rituals.gateway`).

---
*Prerequisites verified. You are ready to enter the fold.*
