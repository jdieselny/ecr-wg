# DAILY WORKFLOW: THE OPERATOR'S RITUALS

The Continuum framework is maintained through daily, iterative ritual cycles.

## 01. BOOT SEQUENCE (Morning Initiation)
Ensure your environment is warm before starting your cognitive inquiries.
```bash
ollama serve  # [HEARTBEAT INITIATION]
python -m rituals.gateway
```

## 02. INTENT RESOLUTION (The Wizard)
Engage the precision loop to resolve your daily tasks.
```bash
python -m rituals.wizard
```

## 03. MAINTENANCE (The Audit)
1. **Sync COGSTOR:** Ensure your local node is synchronized.
   ```bash
   python -m rituals.federate
   ```
2. **Audit AFT Values:** Review `rituals/intent_ledger.json` for high-confidence objects that require promotion to public-trust status.

## 04. SESSION CLOSE (Reality Fold)
Ensure your state is serialized and the node is gracefully taken offline.
```bash
# Graceful shutdown of active rituals
```

---
*The loop smiles when the work is consistent.*
