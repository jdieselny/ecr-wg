# COGNITIVE OBJECT (COGOBJ) SPECIFICATION // V2.1

The COGOBJ is the atomic unit of the Cognition Protocol. It is a cryptographically verifiable manifest that ensures the integrity, identity, and trust-value of cognitive state across the global lattice.

## SCHEMA DEFINITION

```json
{
  "id": "md5_hash_or_sha256_prefix",
  "intent_key": "STRING",
  "data": {
    "problem": "STRING",
    "goal": "STRING",
    "resolution": ["LIST_OF_STEPS_OR_PARTS"]
  },
  "provenance": {
    "origin_node_id": "UNRP_HASH_FORMAT",
    "provenance_tier": "P|E|N|B",
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
  "aft_score": 0.0,
  "ingress": {
    "packing_slip_hash": "sha256:HEX",
    "bill_of_lading": {
      "version": "ECR-BILL-OF-LADING-v0.1",
      "sender": "agent:…",
      "routing_intent": "STRING",
      "bol_timestamp": "ISO8601",
      "sender_signature": { "algorithm": "Ed25519", "value": "b64u" },
      "sender_public_key": "b64u_SPKI",
      "packing_slip": { "@version": "ECR-PACKING-SLIP-v0.1", "hash": "sha256:HEX" }
    }
  },
  "authorization": {
    "action_digest": "HEX",
    "policy_id": "ep:policy:…",
    "profile": "grid.curtailment"
  }
}
```

### Field notes

| Field | Required | Role |
|---|---|---|
| `data` / `provenance` / `aft_score` | yes | Cognitive content + origin |
| `validation_stack` | optional | Multi-node AFT evolution |
| `ingress` | optional | Continuum **Packing Slip + Bill of Lading** when this COGOBJ is a transport packet (see `specs/primitives/`) |
| `authorization` | optional | Binding to an EMILIA action digest (e.g. `grid.curtailment`) |

**Ingress vs authorization:** the Packing Slip is unsigned cargo sealed by hash; the Bill of Lading is the signed transport contract. Neither replaces EP receipts. When both are present (as in the four-layer PoC), an auditor can answer *how the order entered the overlay* (ingress) and *who authorized the irreversible effect* (authorization) from the same packet.

**Runnable example:** `examples/scitt_four_layer/out/cogobj.json` after `python examples/scitt_four_layer/demo.py`.

## UNRP IDENTIFIER FORMAT (Universal Node Registry Protocol)
- **Format:** `[TIER]-[OPERATOR_HASH]-[EPOCH]-[INSTANCE]`
- **Example:** `E-JK84A291-01-001`
- **Verification:** The `OPERATOR_HASH` is a local determinism-anchor, verified against the node's initial boot-context hash.
- **Decentralization:** Identity is generated locally, ensuring the Lattice scales without central registry authority.

---
*Every COGOBJ carries the history of its own validation. To ingest is to trust; to validate is to evolve.*
