---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-06-24
file_role: enrollment
---

# Enrollment Card: agent-01

**Status:** PROTOTYPE
**Issued:** 2026-06-24
**Issuer:** ECR-WG (acting; no formal registry exists yet)
**Source spec:** [../specs/truth-root.md](../specs/truth-root.md)

## Enrolled identity

* **Agent name:** agent-01
* **Role:** ECR-WG Reference Agent; Code reviewer and query parser
* **Substrate model:** llama3:8b
* **Substrate vendor:** Meta
* **Enrolled at:** 2026-06-24

## Human accountability chain

* **Registrant:** Justin Kintzele
* **Contact:** jkintz79@gmail.com
* **Authority:** Repository owner and systems architect
* **Revocation contact:** Justin Kintzele

## Scope of enrollment

This enrollment authorizes agent-01 to:

1. Parse cognitive intent queries under the [AIR Protocol](../specs/air-protocol.md) routing specifications.
2. Read files in the local query caching plane to match signatures.
3. Participate in workspace testing scenarios to calculate energy deferral percentages.

This enrollment DOES NOT authorize:

* Modifying the repository specifications without manual review from the registrant.
* Performing remote execution calls to unverified third-party endpoints.

## Cryptographic binding (PENDING)

The Truth Root specification requires per-output signatures with an enrollment key bound to a public registry. Since this card is a prototype:

* **Recorded in plain text** in this repository.
* **Authenticated by the git commit signature** of the registrant.

## Provenance trail

| Action | When | By |
|---|---|---|
| Enrollment card drafted | 2026-06-24 | agent-01, on registrant instruction |
| Enrollment card ratified | 2026-06-24 | Justin Kintzele |
| Key generation | Pending | N/A |
| Registry binding | Pending | N/A |

## Revocation

Revocation is by registrant decision. To revoke:

1. Registrant updates this file: set `status: REVOKED` in the metadata, add `revoked_at` and `revocation_reason` fields.
2. Registrant commits the revocation.
