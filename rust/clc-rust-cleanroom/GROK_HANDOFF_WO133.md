# 🐺 GROK HANDOFF: WO-20261024-133 (Surgical Boundary Remediation: 137 -> 154)

See full workorder at `workorders/WO-20261024-133-CLC-A-CORE-VECTOR-SURGICAL-BOUNDARY-REMEDIATION.md`.

## Assignment: 5 Boundary Remediation Targets
1. `revision-002`: Check `clc_revision`. Major > 1 -> `Reason::UnsupportedLanguageRevision` (`unsupported_language_revision`).
2. `entail-007`, `entail-008`: Namespace check precedes wildcard validation -> `Reason::DifferentNamespace` (`different_namespace`).
3. `params-054`, `055`, `056`, `058`, `059`, `066`: Enforce IEEE-754 binary64 safe integer bounds ($|n| \le 9007199254740991$). Over limit -> `Reason::InvalidParamsNumber` (`invalid_params_number`).
4. `params-030`, `032`, `033`, `034`, `035`, `036`, `038`: JCS RFC 8785 canonicalization for 512-byte limit and surrogate decoding.
5. `params-062`: BOM / numeric syntax error precedes size error -> `invalid_params_number`.

Maintain `#![forbid(unsafe_code)]`, zero warnings, and run `cargo run -- --vector vectors/clc-a-core-vectors.json`.
