# RITUAL: TRIPARTITE SYNC (SOP)

This ritual propagates operational updates from the R&D core to the downstream repositories (`meta` baseline and `public` artifact).

## 01. PRE-SYNC SANITIZATION
Before initiating a sync, verify all development artifacts are purged from the staging area.

```bash
# Clean binary and scratchpad artifacts
git rm --cached -r .
find . -name "*.pyc" -type f -delete
git add .
git commit -m "chore: pre-sync sanitization"
```

## 02. SYNC TO BASELINE (Continuum-meta)
1. Navigate to the `continuum-meta` directory.
2. Apply changes from `Continuum-RD` core.
3. Perform a final `SANITIZATION` sweep (move sensitive files to `continuum-local/`).
4. Commit and push:
   ```bash
   git add .
   git commit -m "chore: sync to baseline v1.x.x"
   git push origin main
   ```

## 03. SYNC TO PUBLIC (Continuum-Public)
1. Navigate to the `continuum-public` directory.
2. Copy the updated `manuscript/` and documentation files from Core.
3. Verify placeholder markers in `rituals/` and `thesis/` directories.
4. Update `VERSION.md` if the iteration count has incremented.
5. Commit and push:
   ```bash
   git add .
   git commit -m "chore: artifact sync v1844.x.x"
   git push origin main
   ```

---
*If a hook triggers a blockage, move the flagged content to your local-tier (`continuum-local/`) before proceeding.*
