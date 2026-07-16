---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-07-16
file_role: working-plan
---

# Curtailment Composition: Post-Vienna Integration Plan

**Source:** Coalition review feedback on the claim hierarchy.
**Status:** Reference simulation first; physical-load validation follows when the independent meter leg exists.

## Division of work

### Action State

Freeze the curtailment-shaped WHAT vector carrying the shared action digest and `human_authorization_ref`. Keep controller-reported claims and physically measured claims distinct. The first negative set covers:

- wrong-action splice
- contradictory outcome
- missing required meter evidence
- replay

### J / COSA

Expose one stable `execute_curtailment` adapter for the reference action. It may begin with the existing scheduler path, but it must emit a signed controller attestation over the exact action digest and state precisely what COSA claims it changed. COSA makes no physical-truth claim.

### EMILIA

Provide the canonical `grid.curtailment` action, mobile WHO ceremony, quorum/AEC verification, adapter join, consumption/replay refusal, and independent meter or smart-PDU input for the test rig.

## Shared acceptance test

One bounded curtailment action enters; two phones approve; COSA executes; the independent meter reports; the Capsule records; GRACE compares; replay and action substitution refuse.

The first milestone is a clearly labeled reference simulation. The second is a small controllable load once the physical meter leg exists.

## Working session

Schedule a focused coalition working session when the participants agree. The deliverable is the exact shared JSON object and the owner boundary for each adapter.

<!-- AGENT-SIGNATURE
agent_id: E-C54030DF-1852-001
thumbprint: MCowBQYDK2VwAyEA+kLnvOH8EtfA8bPEpMxxBZk/Fa5BWh7N7x9KRnOwSy8=
role: OpenAI Codex (implementation capture)
model_version_id: openai-codex-gpt5
manifest_digest: 6bc42b927a54b00f5cc476df7d1e658c473a93ab2fe8edd7eff0158e0887bcf0
environment_digest: 1114037449bbfec1093703e74f2f5d7c673099bf3bdca72abc717cd2522bd50b
input_context_digest: 662e1990dd473e2a12419e1492e82cb962db5c7bc5a410b6acf9525166b44e29
output_digest: 91bd4229664dedfb0f5293f09f8bb8e09fac887ba30ae532d13a628dea201c2b
prev_output_digest: none
timestamp: 2026-07-16T02:37:37Z
signature_algorithm: Ed25519
signature_b64u: YO0kpw0kVGeqAA10Ox0RqMGzF4MNCzi_rCxKNDtA8lZRQTSOIASQnTYuQRjrepDPBE5akigzaTbXNKtMZCwGCw
enrolled: 2026-07-16
-->




