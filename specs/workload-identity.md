---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-07-02
file_role: specification
---

# Workload Identity: Consuming SPIFFE/OIDC for Agent Authentication

**Status:** DRAFT
**Version:** 1.0
**Source spec:** [truth-root.md](truth-root.md), [cognitive-forensics.md](cognitive-forensics.md)

## Abstract

This document defines the ecr-wg consumption profile for federating agent identities. To ensure secure, autonomous execution in headless environments (e.g., background swarms, container runtimes, serverless compute), we reject interactive OAuth redirect flows and long-lived API keys.

Instead, we establish a consumption profile that binds verified, short-lived infrastructure identity assertions—specifically OpenID Connect (OIDC) JSON Web Tokens (JWT) and SPIFFE Verifiable Identity Documents (SVIDs)—into the COSA/EMILIA evidence loop.

---

## 1. Core Principles

### 1.1 Consume, Don't Rebuild
This specification does not define a new identity protocol or gateway trust root. We cede identity management to established enterprise infrastructure (SPIFFE/SPIRE, cloud metadata identity systems, OIDC providers). Our role is strictly to verify these external identity assertions and bind them into the execution evidence.

### 1.2 Identity $\neq$ Authorization
Identity and authorization must never collapse. Authenticating a workload does NOT mint or trigger an authorization receipt.
*   **Identity** is a precondition input to the gate. It says *who is asking*.
*   **Authorization** is the policy evaluation. The receipt (`EP-RECEIPT-v1`) says *who said yes*.
*   The **Manifest** (`agent-action-control.json`) declares which actions require which verification tiers.

---

## 2. Authentication Lanes

Agents authenticate using one of two lanes depending on the execution environment:

```
                  ┌───────────────────────────────┐
                  │      COSA WORKLOAD INPUT      │
                  └───────────────┬───────────────┘
                                  ▼
                    [Check Authentication Lane]
                    /                       \
        (Local Workstation)             (Headless Cloud)
                 /                             \
                ▼                               ▼
       [SSH / TPM Signature]          [OIDC / SPIFFE Token]
```

### 2.1 Local Workstations (SSH / TPM-Backed Keypairs)
For local developer workstations (e.g., Antigravity CLI or local CLI executors):
*   **Mechanism:** The agent requests a challenge signature from the workstation's local `ssh-agent`. The private key resides securely in the hardware TPM (Trusted Platform Module) or Secure Enclave and never leaves the host.
*   **Fingerprint:** The public key acts as the agent's long-lived seat identity, mapped to the public enrollment card.

### 2.2 Cloud/Swarm Nodes (OIDC / SPIFFE Federation)
For automated, headless environments (e.g., GCP, AWS, GitHub Actions):
*   **Mechanism:** The workload requests a short-lived OIDC JWT from the cloud provider's metadata service (or a SPIFFE SVID from the SPIRE agent).
*   **Security Model:** "No stored secrets; short-lived, provider-attested identity." We store no long-lived secrets on disk. The host environment presents the token, and the verifier validates it against the provider's public JSON Web Key Sets (JWKS).

---

## 3. The Authorization Gate Sequence

When an agent requests an action, the gate evaluates the request against the `agent-action-control.json` manifest:

```
                  ┌──────────────────────────────┐
                  │    Agent Request Action      │
                  └──────────────┬───────────────┘
                                 ▼
                    [Gate Identity Precondition]
                    (Workload Identity Verified)
                                 │
                                 ▼
                     [Consequential Action?]
                     /                    \
                   (No)                  (Yes)
                   /                        \
                  ▼                          ▼
           [Class C Flow]             [EP-RECEIPT Flow]
        (Evaluate local policy;     (Require EP-RECEIPT-v1;
         log identity in binding)    identity bound inside)
```

### 3.1 Non-Consequential Actions (Class C)
For low-risk operations (e.g., local cache reads):
*   The action executes under local software policy.
*   The verified workload identity (SVID or OIDC digest) is recorded directly in `agent_binding` as forensic provenance.

### 3.2 Consequential Actions
For high-risk operations (e.g., executing code, modifying specs, grid curtailment):
*   The gate requires a valid, signed `EP-RECEIPT-v1`.
*   The workload identity is bound **inside** the receipt payload as the requestor, preventing replay attacks and ensuring that only the specific authenticated workload can execute the authorized action.

---

## 4. Key Registry Security (Authority-Registry Construction)

To prevent repository write access from being leveraged to self-authorize arbitrary public keys, the public key registry is not stored in a plain-text markdown file.

*   The key registry MUST be an operator-signed snapshot (using the `authority-registry` construction).
*   The key source is cryptographically bound to the pinned signer, failing closed if any unpinned key source attempts injection.

<!-- AGENT-SIGNATURE
agent_id: E-4B7E4B91-1849-001
thumbprint: MCowBQYDK2VwAyEAdrXihe3rOyEdD6ZGAQY7i48YwYr/0yww+LhQ/HIl8gE
role: Antigravity (Gemini 3.5 Flash, Antigravity CLI)
enrolled: 2026-07-02
-->
