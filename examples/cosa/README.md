# COSA L5 + EMILIA L7, composed reference (ecr-wg)

The ecr-wg variant of the upstream EMILIA Protocol reference at
[`emiliaprotocol/emilia-protocol/examples/cosa`](https://github.com/emiliaprotocol/emilia-protocol/tree/main/examples/cosa).
Same protocol surface, same orthogonal guarantees, same two failure axes. The
adaptation wires the L5 compute path to a live cognitive-broadcast source
(wttr.in via `rituals.weather_listen`) instead of a hardcoded answer string.

> L5 (authenticity): the plane computes an answer once, wraps it in a signed
> COGOBJ, and any consumer can verify it is genuine without recomputing.
>
> L7 (authorization): publishing that COGOBJ to N consumers is irreversible
> (they cache and trust it), so it requires an EMILIA receipt, a named human
> signing the exact publish action.

## Run it

```bash
pip install emilia-verify
python examples/cosa/cosa_l5_l7.py             # default: live wttr.in fetch
python examples/cosa/cosa_l5_l7.py --offline   # synthetic answer, no network
```

Run from the repo root so `rituals.weather_listen` is importable and the cache
path resolves under `continuum-local/`.

## What it demonstrates

The same three properties as the upstream reference:

- **Compute-once / serve-N.** L5 computes the answer once; N consumers serve
  it from cache at 0 tokens each. The run prints tokens saved vs. each
  consumer recomputing.
- **L7 authorization.** No receipt produces a 428-equivalent refusal before
  any fan-out; a named human signs and the broadcast runs; the same receipt
  replayed is refused; a valid receipt for a different action is refused
  (confused-deputy).
- **L5 authenticity.** A tampered COGOBJ reaching a fresh consumer is
  rejected even when the publish carried a valid receipt. L7 authorized the
  publish; L5 still caught the forged content. The two layers catch
  different attacks.

## The three changes from upstream

For WG readers diffing this file against the upstream `examples/cosa/cosa_l5_l7.py`:

1. `L5Plane.compute()` calls `rituals.weather_listen.listen()` to fetch
   wttr.in live, instead of using a hardcoded answer string. Falls back to a
   synthetic answer if the network call fails or `--offline` is passed.
2. `agent-actions.json` declares the ecr-wg COSA service.
   `service.manifest_url` points to a future hosted location
   (`https://jdieselny.com/.well-known/agent-actions.json`); hosting itself
   is follow-on work, not currently live.
3. SPDX, helper functions, EP-RECEIPT-v1 issue/verify, manifest-driven
   `authorize()` gate, Ed25519 signatures over JCS-canonical bytes, and the
   seven-step demo all carry over verbatim from the upstream. Same protocol,
   same checks.

## RR-1 conformance, by demonstration

The upstream defines RR-1 conformance at the level of HTTP services
([`docs/RECEIPT-REQUIRED-CONFORMANCE.md`](https://github.com/emiliaprotocol/emilia-protocol/blob/main/docs/RECEIPT-REQUIRED-CONFORMANCE.md)).
This reference is a Python demonstration, not a hosted HTTP service, but it
exercises the five RR-1 predicates by demonstration:

| RR-1 predicate          | Where the demo shows it                                                           |
| ----------------------- | --------------------------------------------------------------------------------- |
| `manifest_valid`        | Step 1 reads and uses `agent-actions.json`.                                       |
| `challenge_on_missing`  | Step 2 raises `ReceiptRequired` (428-equivalent) before any fan-out.              |
| `runs_on_valid`         | Step 3 publishes with a valid receipt.                                            |
| `replay_refused`        | Step 6 refuses the replayed receipt.                                              |
| `forged_refused`        | Step 5 (tamper detection on signed content) plus step 7 (confused-deputy).        |

A hosted-service variant earning the RR-1 badge formally is follow-on work.

## Known follow-on: L4 Identity Binding

A key architectural critique (from Karthiek Maralla) notes that the L7 authorization receipt's `subject` field currently contains a plain string identity (`"agent:cosa-l5-plane"`) rather than a cryptographically bound identifier. While L7 validates the approver's signature, it does not cryptographically prove the requester's claim to that identity.

Following joint list convergence (Iman Schrock), we decouple the L7 PDP governance plane from convergence on any single L4 identity scheme (such as WIMSE, OAuth Identity Chaining, AIMS, or EAT). Instead, the L7 PDP decision object binds to and records the presented evidence:
1.  **L4 Attestation Reference**: The decision object references, by content hash, the specific L4 attestation it relied on.
2.  **Freshness and Validity Windows**: The decision object records and enforces validity windows, ensuring that stale or unconstrained sub-agent claims show up in the audit record instead of being silently absorbed.

This decouples the debates and allows L7 authorization to remain agnostic while making the failure modes described by Karthiek detectable after the fact.

<!-- AGENT-SIGNATURE
agent_id: E-2A0F1954-1845-001
thumbprint: MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE=
role: Gemini-in-body (Antigravity Substrate)
-->

## See also

- Upstream: [emiliaprotocol/emilia-protocol/examples/cosa](https://github.com/emiliaprotocol/emilia-protocol/tree/main/examples/cosa)
- EP Receipt Required doctrine: [`docs/RECEIPT-REQUIRED.md`](https://github.com/emiliaprotocol/emilia-protocol/blob/main/docs/RECEIPT-REQUIRED.md)
- IETF agent2agent thread: archived on [mailarchive.ietf.org/arch/browse/agent2agent](https://mailarchive.ietf.org/arch/browse/agent2agent/)
- COSA seven-layer model: [`papers/03_cosa_seven_layer_model.md`](../../papers/03_cosa_seven_layer_model.md)
