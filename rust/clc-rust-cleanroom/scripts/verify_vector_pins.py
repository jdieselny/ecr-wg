#!/usr/bin/env python3
"""Verify that the vendored CLC-1.19 corpora are byte-exact upstream copies.

`vectors/clc-v1/` holds third-party corpus files. They must stay unmodified:
the cleanroom independence claim says the vectors were "obtained by
cryptographic hash and pinned", and a silently edited expectation would turn a
passing run into a self-graded one. This check is the mechanism behind that
claim, so the files can be excluded from the vendor-string leash without
leaving a hole.

Pins live in `vectors/clc-v1/PINS.sha256`:
    <sha256>  <filename>

Exits non-zero on any missing, altered, or unpinned JSON file in the directory.
"""

from __future__ import annotations

import hashlib
import pathlib
import sys

VECTOR_DIR = pathlib.Path("vectors/clc-v1")
PINS = VECTOR_DIR / "PINS.sha256"


def read_pins() -> dict[str, str]:
    pins: dict[str, str] = {}
    for line in PINS.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        digest, _, name = line.partition("  ")
        if not name:
            raise SystemExit(f"PINS.sha256: malformed line: {line!r}")
        pins[name] = digest
    return pins


def main() -> int:
    if not PINS.is_file():
        print(f"vector pins: missing {PINS}", file=sys.stderr)
        return 1

    pins = read_pins()
    problems: list[str] = []

    for name, expected in sorted(pins.items()):
        path = VECTOR_DIR / name
        if not path.is_file():
            problems.append(f"missing   {name}")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            problems.append(f"altered   {name}\n            pinned {expected}\n            actual {actual}")

    for path in sorted(VECTOR_DIR.glob("*.json")):
        if path.name not in pins:
            problems.append(f"unpinned  {path.name}")

    if problems:
        print("vector pins: FAIL")
        for problem in problems:
            print(f"  {problem}")
        return 1

    print(f"vector pins: {len(pins)} files match the pinned SHA-256 digests")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
