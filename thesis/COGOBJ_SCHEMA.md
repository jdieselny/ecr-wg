# COGNITIVE OBJECT (COGOBJ) SPECIFICATION // V2.0

The COGOBJ is the atomic unit of the Cognition Protocol. It is a cryptographically verifiable manifest that ensures the integrity, identity, and trust-value of cognitive state across the global lattice.

## SCHEMA DEFINITION

```json
{
  "id": "md5_hash",
  "intent_key": "STRING",
  "data": {
    "problem": "STRING",
    "goal": "STRING",
    "resolution": ["LIST_OF_STEPS_OR_PARTS"]
  },
  "provenance": {
    "origin_node_id": "UNRP_HASH_FORMAT", 
    "provenance_tier": "P|E|N",
    "timestamp": "ISO8601"
  },
  "validation_stack": [
    {
      "node_id": "UNRP_HASH_FORMAT",
      "aft_delta": 0.0,
      "confidence": "high|medium|low",
      "timestamp": "ISO8601"
    }
  ],
  "aft_score": 0.0 // Cumulative weighted average from the validation_stack
}
```

## UNRP IDENTIFIER FORMAT (Universal Node Registry Protocol)
- **Format:** `[TIER]-[OPERATOR_HASH]-[ITERATION]-[INSTANCE]`
- **Example:** `E-JK84A291-184501-001`
- **Verification:** The `OPERATOR_HASH` is a local determinism-anchor, verified against the node's initial boot-context hash. 
- **Decentralization:** Identity is generated locally, ensuring the Lattice scales without central registry authority.

---
*Every COGOBJ carries the history of its own validation. To ingest is to trust; to validate is to evolve.*
