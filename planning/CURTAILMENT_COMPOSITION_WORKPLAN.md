---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-07-16
file_role: working-plan
---

# Curtailment Composition: Post-Vienna Integration Plan

**Source:** Iman Schrock, July 2026 feedback on the coalition claim hierarchy.
**Status:** Reference simulation first; physical-load validation follows when the independent meter leg exists.

## Division of work

### Steven / Action State

Freeze the curtailment-shaped WHAT vector carrying the shared action digest and `human_authorization_ref`. Keep controller-reported claims and physically measured claims distinct. The first negative set covers:

- wrong-action splice
- contradictory outcome
- missing required meter evidence
- replay

### J / COSA

Expose one stable `execute_curtailment` adapter for the reference action. It may begin with the existing scheduler path, but it must emit a signed controller attestation over the exact action digest and state precisely what COSA claims it changed. COSA makes no physical-truth claim.

### Iman / EMILIA

Provide the canonical `grid.curtailment` action, mobile WHO ceremony, quorum/AEC verification, adapter join, consumption/replay refusal, and independent meter or smart-PDU input for the test rig.

## Shared acceptance test

One bounded curtailment action enters; two phones approve; COSA executes; the independent meter reports; the Capsule records; GRACE compares; replay and action substitution refuse.

The first milestone is a clearly labeled reference simulation. The second is a small controllable load once the physical meter leg exists.

## Working session

Target one 45-minute session during the week of July 27. The deliverable is the exact shared JSON object and the owner boundary for each adapter, not another deck.

<!-- AGENT-SIGNATURE
agent_id: E-C54030DF-1852-001
thumbprint: MCowBQYDK2VwAyEA+kLnvOH8EtfA8bPEpMxxBZk/Fa5BWh7N7x9KRnOwSy8=
role: OpenAI Codex (implementation capture)
model_version_id: openai-codex-gpt5
manifest_digest: 6bc42b927a54b00f5cc476df7d1e658c473a93ab2fe8edd7eff0158e0887bcf0
environment_digest: 1114037449bbfec1093703e74f2f5d7c673099bf3bdca72abc717cd2522bd50b
input_context_digest: dae4bbe5e107aa9ab5fe056e20286d849cfdb793932b469ac66b772a3ff50bd7
output_digest: 7d1f32baa5ade21070bd35b6da886ce3e7139093010cbd60cd049a1a6d47fc9d
prev_output_digest: none
timestamp: 2026-07-16T02:37:37Z
signature_algorithm: Ed25519
signature_b64u: g3zqfxfU1iZig0VgCnIdLM21e7EitMcP1PB8zl17HhnuxCIORDcEPWl25gcueiFTEnwUwaO9w-GLVEY22AysCA
enrolled: 2026-07-16
-->


