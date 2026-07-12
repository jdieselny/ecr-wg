# Local enrollment keys (private — never committed)

Private Ed25519 PEM files for Truth Root agent enrollment prototypes are **generated on your machine only**. They are not stored in this repository.

## Generate locally

```bash
python scripts/enroll-ed25519.py
```

Output lands in `keys/` (gitignored). Enrollment cards on `main` carry **public** thumbprints only.

## Rules

- Never `git add` any `*_private_key.pem` or `private-key.pem`
- Not production secrets; not human identity keys
- If a private key was ever pushed, **rotate** — generate a new pair and update the enrollment card thumbprint

## Public material in repo

- Enrollment thumbprints in `enrollments/agent-*.md`
- `rust/ep-cleanroom-verifier/keys/public.key` (cleanroom demo public key only)