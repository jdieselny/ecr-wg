---
aft: AI-generated-user-reviewed-pending
registrant: [Registrant Name]
generated_at: [YYYY-MM-DD]
file_role: enrollment-template
---

# Enrollment Card Template

**Status:** TEMPLATE
**Source spec:** [../specs/truth-root.md](../specs/truth-root.md)

---

## \u26a0\ufe0f STOP. BEFORE YOU FILL OUT THIS TEMPLATE:

Run this command and find the highest `agent-XX` slot in `origin/main`. Increment by 1. That is your slot number.
```bash
git ls-tree -r --name-only origin/main enrollments
```
Do NOT use the local filesystem to determine slot numbers. Untracked files from other agents will cause collisions.

Your thumbprint algorithm is `identity_setup.py` (md5/sha256). Do NOT use `identity_crypto.py` for the public card.

---


## Enrolled identity

* **Agent name:** [Agent Name]
* **Role:** [e.g. Code Reviewer, Router, etc.]
* **Substrate model:** [e.g. llama3:8b]
* **Substrate vendor:** [e.g. Meta]
* **Enrolled at:** [YYYY-MM-DD]

## Human accountability chain

* **Registrant:** [Registrant Name]
* **Contact:** [Email]
* **Authority:** [e.g. Repository Owner, Systems Architect]
* **Revocation contact:** [Email]

## Scope of enrollment

This enrollment authorizes the agent to:

1. [Authorized action 1]
2. [Authorized action 2]

This enrollment DOES NOT authorize:

* [Forbidden action 1]
* [Forbidden action 2]

## Cryptographic binding (PENDING)

The Truth Root specification requires per-output signatures with an enrollment key bound to a public registry. Since this card is a prototype:

* **Recorded in plain text** in this repository.
* **Authenticated by the git commit signature** of the registrant.

## Provenance trail

| Action | When | By |
|---|---|---|
| Enrollment card drafted | [Date] | [Agent/Author] |
| Enrollment card ratified | Pending | [Registrant] |
| Key generation | Pending | N/A |
| Registry binding | Pending | N/A |

## Revocation

Revocation is by registrant decision. To revoke:

1. Registrant updates this file: set `status: REVOKED` in the metadata, add `revoked_at` and `revocation_reason` fields.
2. Registrant commits the revocation.
