# From Independent Verifier to Embodied Agent (COSA Body + God Terminal)

This guide shows how an independent clean-room verifier (158/158) combines with an embodied agent ("agent-in-body") that can self-enroll and act on grid curtailment receipts.

## 1. Run the Independent Verifier

See the external verification harness in the EMILIA repo (for now), but all coalition artifacts live here.

Produce your `statement.json` with a fresh key.

## 2. Self-Enrollment for the Body

An embodied agent (e.g. God Terminal running on a VM with Tailscale access to COSA scheduler) uses its local Ed25519 key to self-enroll:

```python
# In God Terminal backend
receipt = emilia_self_enroll(
    agent_id="god-terminal-vm-01",
    body="gcp-agent-smith-vm-tailscale-cosa",
    capabilities=["cosa_scheduler", "grid_curtailment", "power_control"],
    max_curtail_kw=5.0
)
# Persisted signed enrollment record
```

This produces a bilaterally-compatible enrollment record.

## 3. Receive and Act on a Curtailment Receipt

When a grid.curtailment receipt arrives (from EMILIA authority or bilateral):

```python
result = emilia_curtailment_gate(receipt_json)
if result["status"] == "curtailed":
    # Bind to COSA
    # - Evict non-protected lanes
    # - Route to cache
    # - Apply hardware power limits (NVML)
    telemetry = emilia_sign_telemetry(
        meter_id="gt-01",
        samples=[4.0, 4.1, 4.2],
        baseline_method_hash=receipt["action"]["baseline_method_hash"]
    )
    # Posture ack + telemetry form the Proof-of-Curtailment bundle
```

## 4. Proof-of-Curtailment Bundle

Compose:
- Original receipt (order)
- Posture acknowledgment (signed by facility/body)
- Attested telemetry (signed, binds baseline hash)

This can be verified offline by anyone.

## 5. Independence + Adoption

The independent verifier statement proves your runner correctly understands the vectors.

The body enrollment + gate actions prove you can act on real receipts without a central token issuer.

See:
- Related god-terminal/backend (in jdiesel-continuum for now)
- continuum-local/scratchpad/grid_telemetry_harness.py (simulator)
- draft-schrock-kintzele-grid-curtailment-00.txt

## Next Steps for Implementers

- Pin the independent verifier's public key out-of-band.
- Wire the gate into your actual workload scheduler.
- Generate your own conformance statement and share in the coalition.

This is how a COSA body becomes a first-class GRACE participant.
