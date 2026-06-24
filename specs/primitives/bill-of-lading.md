---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-05-22
file_role: primitive-draft
---

# Bill of Lading

**Status:** DRAFT
**Version:** 0.1
**Source:** Continuum-meta design whiteboard (2026-05, as transcribed by synthesis seat)

## Abstract

The Bill of Lading (BoL) is the **execution contract** that wraps a [Packing Slip](packing-slip.md) for transport into the Continuum-meta overlay. It binds the sender (the synthetic agent), the transport layer, and the request -- and attests that the call has passed [GRACE Contract](../grace-contract.md) constraint checks.

If the Packing Slip is the cargo, the Bill of Lading is the signed manifest that travels with it.

## Position in the data flow

```
[PACKING SLIP + HASH] -> [BILL OF LADING] -> [CONTINUUM-META]
```

## Required fields

A conformant Bill of Lading MUST contain:

- **`packing_slip_hash`** -- the hash from the wrapped Packing Slip. Forms the cache key for overlay lookup.
- **`sender_signature`** -- signature by the synthetic agent's enrollment key (see [Truth Root](../truth-root.md)).
- **`grace_attestation`** -- assertion that the call's GRACE fields are populated and CONSTRAINTS have been checked. Signed.
- **`routing_intent`** -- populated GRACE ROUTING field. May be a specific node, a capability requirement, or `any-overlay`.
- **`return_path`** -- how the response should be returned to the originating synth. May be a callback endpoint, a queue reference, or a streaming session token.
- **`bol_timestamp`** -- when the Bill of Lading was sealed.

## Required properties

- **Wraps the Packing Slip, does not modify it.** The Packing Slip's hash is referenced; the slip itself is carried unchanged.
- **Signed before transmission.** A BoL without a valid `sender_signature` MUST be rejected at the overlay boundary.
- **Verifiable by any overlay node.** The signature is checkable against the Truth Root registry without contacting the originating synth.
- **GRACE-binding.** The `grace_attestation` is the contract the sender accepts for this call. Violation by the responder is a protocol error.

## Non-goals

- **The BoL does not specify the AIR routing decision.** It carries `routing_intent`; the AIR protocol resolves it.
- **The BoL does not encrypt the Packing Slip.** Confidentiality is a separate concern; the BoL provides authenticity and attestation.

## Open problems

1. **Signature scheme.** Tied to the [Truth Root](../truth-root.md) decision. Open.
2. **Wire format.** JSON-LD, CBOR, protobuf -- open.
3. **Return-path schema.** What return mechanisms are first-class (HTTP callback, message queue, streaming, polling)? Open.
4. **BoL revocation mid-flight.** Can a sender revoke a BoL that is already in the overlay's processing queue? If yes, how? Open.
5. **Multi-hop attestation.** If a BoL is forwarded between overlay nodes, does each hop add an attestation, or only the originator and the responder? Open.
6. **Replay protection.** A captured BoL with a valid signature could be replayed. Nonce, sequence, or timestamp-window mechanism unspecified.

## Status to advance

DRAFT advances to STABLE when:

- Wire format is specified.
- Signature scheme is specified (depends on Truth Root advancement).
- Replay protection mechanism is specified.
- At least two independent implementations interoperate end-to-end.

## Relation to other specs

- Wraps: [Packing Slip](packing-slip.md)
- Signed against: [Truth Root](../truth-root.md)
- Routed by: [AIR Protocol](../air-protocol.md)
- Binds: [GRACE Contract](../grace-contract.md)
