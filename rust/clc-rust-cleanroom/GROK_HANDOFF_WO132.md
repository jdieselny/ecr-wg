# 🐺 GROK HANDOFF: WO-20261024-132 (CLC-A Core Vector Harness)

See full workorder at `workorders/WO-20261024-132-CLC-A-CORE-VECTOR-HARNESS-AND-AMBIGUITY-AUDIT.md`.

## Assignment
Update `src/vector_runner.rs` to parse the 4 canonical vector kinds in `vectors/clc-a-core-vectors.json` (154 vectors):
1. `syntax` (9 vectors): `request.id` -> `grammar::parse_capability_id`. Check `expect.verdict` ("valid" vs "invalid") and `expect.reason`.
2. `entail` (65 vectors): single `grant` and `request`.
3. `intersect` (17 vectors): `grant` + `others`.
4. `decide` (63 vectors): `grant` / `others` -> `grants: Vec<Grant>`.

## Schema Rules
- Use `"grant"` (singular) or wrap in `Vec<Grant>`.
- Use `"reason"` (not `"reason_code"`) under `"expect"`.
- Do NOT touch `src/lattice.rs` or core math.
- Keep `#![forbid(unsafe_code)]` and zero warnings.
- Run `cargo test` and `cargo run -- --vector vectors/clc-a-core-vectors.json`.
- Report exact baseline: passed, failed, and ambiguous cases.
