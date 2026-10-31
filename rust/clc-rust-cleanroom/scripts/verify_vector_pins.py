#!/usr/bin/env python3
"""Verify that the vendored CLC corpora are byte-exact upstream copies.

`vectors/clc-v1/`, `vectors/clc-d/` and the two CLC-A core files at the
`vectors/` root are third-party corpus files. They must stay unmodified: the
cleanroom independence claim says the vectors were "obtained by cryptographic
hash and pinned", and a silently edited expectation would turn a passing run
into a self-graded one. This check is the mechanism behind that claim, so the
directories can be excluded from the vendor-string leash without leaving a hole.

Each checked directory carries a `PINS.sha256`:

    <sha256>  <filename>

Two modes:

- ``strict`` — every ``*.json`` in the directory must appear in the pin file.
  Used where the directory holds only upstream corpora.
- ``listed`` — only the files named in the pin file are verified. Used for
  ``vectors/``, which mixes upstream corpora with project-owned vectors
  (``ccx-iicp-001.json``, ``clc-d-crosswalk-vectors.json``).

Exits non-zero on any missing or altered pinned file, or any unpinned JSON file
in a strict directory.
"""

from __future__ import annotations

import hashlib
import pathlib
import sys
from dataclasses import dataclass

PINS_NAME = "PINS.sha256"


@dataclass(frozen=True)
class VectorDir:
    path: pathlib.Path
    strict: bool


VECTOR_DIRS = [
    VectorDir(pathlib.Path("vectors/clc-v1"), strict=True),
    VectorDir(pathlib.Path("vectors/clc-d"), strict=True),
    VectorDir(pathlib.Path("vectors"), strict=False),
]


def read_pins(pins_path: pathlib.Path) -> dict[str, str]:
    pins: dict[str, str] = {}
    for line in pins_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        digest, _, name = line.partition("  ")
        if not name:
            raise SystemExit(f"{pins_path}: malformed line: {line!r}")
        pins[name] = digest
    return pins


def check_dir(vector_dir: VectorDir) -> tuple[int, list[str]]:
    pins_path = vector_dir.path / PINS_NAME
    if not pins_path.is_file():
        return 0, [f"missing   {pins_path}"]

    pins = read_pins(pins_path)
    problems: list[str] = []

    for name, expected in sorted(pins.items()):
        path = vector_dir.path / name
        if not path.is_file():
            problems.append(f"missing   {path}")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            problems.append(
                f"altered   {path}\n"
                f"            pinned {expected}\n"
                f"            actual {actual}"
            )

    if vector_dir.strict:
        for path in sorted(vector_dir.path.glob("*.json")):
            if path.name not in pins:
                problems.append(f"unpinned  {path}")

    return len(pins), problems


def main() -> int:
    total = 0
    problems: list[str] = []
    for vector_dir in VECTOR_DIRS:
        count, found = check_dir(vector_dir)
        total += count
        problems.extend(found)

    if problems:
        print("vector pins: FAIL")
        for problem in problems:
            print(f"  {problem}")
        return 1

    print(f"vector pins: {total} files match the pinned SHA-256 digests")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
