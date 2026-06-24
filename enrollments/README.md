---
aft: AI-generated-user-reviewed-pending
registrant: Justin Kintzele
generated_at: 2026-06-24
file_role: governance
---

# Agent Enrollments

This directory contains the working prototype of the [Truth Root](../specs/truth-root.md) specification at file system scale.

Each enrollment card in this directory:

* Identifies a synthetic agent participating in ECR-WG work.
* Binds that agent to a human accountability chain (the registrant).
* States the explicit scope of authority and non-authorities.
* Marks cryptographic binding as PENDING until the Truth Root registry exists.

## Core Identity Principles

1. **Identity is a file, not a process**: An agent's identity is defined by a stable profile file. The hash of this file serves as the agent's unique fingerprint.
2. **The operator holds the registry**: The operator (registrant) acts as the authority holding the registry of known-good hashes.
3. **Nothing hashes itself**: The identity file does not contain its own hash to prevent self-reference loops.
4. **Signatures bind real hashes**: A signature binds the hash of a work product to the hash of the agent's identity file.
5. **Verification is comparison**: Verification requires comparing a presented fingerprint against the recorded public key or hash in the registry.

For a reusable template, see [template.md](template.md). For a reference example, see [agent-01.md](agent-01.md).
