---
aft: AI-generated-user-reviewed-pending
file_role: security_rotation
generated_at: 2026-07-12
---

# Enrollment key rotation (2026-07-12)

Prior private PEMs were removed from the repo and **scrubbed from git history**.
New keypairs generated locally; only public thumbprints below.

| Agent | New thumbprint |
|-------|----------------|
| agent-02-cdawg-opus47 | `MCowBQYDK2VwAyEAeAFm+M8QN/M78iquE5otpIMQSVEAb49VFz5unLQvBes=` |
| agent-03-mrcode-claudecode-opus48 | `MCowBQYDK2VwAyEA0KF1pnVbBDsk40irbASuKtiS3LCnkCZRkVJZOtFAwRY=` |
| agent-04-grokbuild-grok43 | `MCowBQYDK2VwAyEAxf9pDw+okMCMBDh01Seo3MlqfvRyUVb187XBHCOuljI=` |
| agent-05-antigravity-claude-sonnet46 | `MCowBQYDK2VwAyEAgsnKPxtIKKBRNZRwCCBFwG9pvACk5T31kcClEbSrOmM=` |
| agent-06-openai-gpt55-xhigh | `MCowBQYDK2VwAyEAl2ChdgOBJB5zHYDQwUso0WVv3Ov9APSMjWgC05N904M=` |
| agent-07-antigravity-gemini35-flash | `MCowBQYDK2VwAyEAvI8wl0sXkmcJzNoYO1OPvfhrSkOdvsP+jjhfQyarAfY=` |
| agent-08-claude-desktop-opus48 | `MCowBQYDK2VwAyEAQfoNYkUSVEEVwF9p4Rbs2QRVloVqmEZGmADvLabJJ20=` |

Private keys: `keys/*_private_key.pem` (gitignored, operator machine only).
Re-sign any artifacts that bound to the old thumbprints.
