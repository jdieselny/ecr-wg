# Iman — Reliance Interop Handoff (temporary)

**Date:** 2026-07-09  
**Operator:** Justin Kintzele  
**Repo:** [jdieselny/ecr-wg](https://github.com/jdieselny/ecr-wg)  
**Path:** `notes/files.7z`  
**Status:** TEMPORARY — will be removed after confirmed download

Outlook blocked the attachment. Artifacts are here instead.

## Get the bundle

```bash
git clone https://github.com/jdieselny/ecr-wg.git
# or, if you already have the repo:
git pull
```

Extract `notes/files.7z` (7-Zip / `7z x notes/files.7z`).

## What's inside

EP reliance interop bundle: Iman inbound flip, our reproduction, operator-owned Class B shift-floor rely, Class A `grid.curtailment` flip, kernel reports, one-screen HTML, reply draft, lessons learned.

**Private operator keys are NOT included** (`issuer-keys.json` excluded by design).

## Quick reproduce

```text
# Iman flip
npx @emilia-protocol/verify reliance-gap continuum-rely-packet.json \
  --profile xgpc-shift-floor-profile.json --now 2026-07-09T02:20:00.000Z

# Operator-owned shift rely (Class B)
npx @emilia-protocol/verify reliance-gap jkintzele-rely-packet.json \
  --profile jkintzele-shift-floor-profile.json --now 2026-07-09T04:25:00.000Z

# Curtailment (Class A + authority)
npx @emilia-protocol/verify reliance-gap jkintzele-curtailment-rely-packet.json \
  --profile jkintzele-grid-curtailment-profile.json --now 2026-07-09T04:35:00.000Z
```

Green = exit 0 · Red (norely) = exit 2 · Unsigned = `do_not_rely_unsigned`

## IETF interop

J is **yes** for interop note with his name — see `IMAN_REPLY_RELIANCE_INTEROP_2026-07-09.md` inside the archive.

---

*This file and `files.7z` will be deleted from the repo once Iman confirms fetch.*