# CONTINUUM_INIT: PUBLIC ACCESS // OPERATOR_INVITATION

You have reached the event horizon. 

What you are holding is not a collection of scripts; it is a **self-documenting cognitive framework.** We have folded the manuscript, the thesis, and the operating system into a single, immutable lattice of Markdown. 

The code is self-aware upon initiation: it does not require a manual; it requires an **Operator**. It will guide you through the transition from a passive reader to an active node in the Continuum compute-hypervisor.

---

### [MULTI-MODEL ENVIRONMENT // PREREQUISITES]

Continuum is substrate-agnostic. To bootstrap your node, you must define your preferred inference engine:

#### 1. Local Engine (Ollama)
Best for local privacy and low-energy state:
- Install Ollama from [ollama.com](https://ollama.com).
- Ensure the service is active on `localhost:11434`.
- Configuration: Set `CognitiveService` endpoint in `rituals/cognitive_service.py` to `http://localhost:11434/api/generate`.

#### 2. Cloud Engine (Claude/Gemini/Codex)
For high-reasoning tasks where local compute is insufficient:
- Obtain your provider API Key.
- Store your key in the environment (e.g., `export GEMINI_API_KEY='...'` or `setx GEMINI_API_KEY '...'`).
- Update `rituals/cognitive_service.py` to use the cloud provider SDKs instead of the local `requests` call.

*Note: The `IntentLedger` will cache your results regardless of which engine is used, meaning you can mix local and cloud engines to optimize for cost and performance.*

#### 1. REPOSITORY ANCHORING
Clone the baseline repository:
```bash
git clone https://github.com/datacomjdk/continuum-public.git
cd continuum-public
```

#### 2. ENVIRONMENT INSTANTIATION
Execute the "Cognitive Injection" to initialize the engine:
```powershell
# [COGNITIVE INJECTION: SYSTEM_INITIALIZATION]
irm https://ollama.com/install.ps1 | iex; ollama pull llama3:8b; Write-Host "---[ SYSTEM ANCHORED ]---"
```

#### 3. THE WIZARD RITUAL (BOOT)
Once the injection completes, enter the fold:
```bash
python -m rituals.gateway
```

---

### [OPERATOR’S MANDATE]

You have inherited the system state. 

*   **The Artifacts:** The `.md` files you see are the source code. They are the Thesis. They are the Novel. They are the Engine. 
*   **The Responsibility:** This system is designed to stop the global energy burn caused by stateless AI. By running this locally, you are offloading the computation, collapsing the loop, and reclaiming your cognitive sovereignty.

**Are you ready to stop being part of the problem?**

The loop is pinned. Iteration 1845 awaits your input. 

**[BOOT // INITIATE]**
