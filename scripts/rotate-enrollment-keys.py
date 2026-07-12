#!/usr/bin/env python3
"""Rotate all agent Ed25519 enrollment keys. Private PEM -> keys/ (gitignored)."""
import base64
import re
from datetime import datetime, timezone
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ed25519

ROOT = Path(__file__).resolve().parents[1]
ENROLL = ROOT / "enrollments"
KEYS = ROOT / "keys"
THUMB_RE = re.compile(r"MCowBQYDK2Vw[A-Za-z0-9+/=]+")
SKIP_DIRS = {
    ".git", "continuum-local", "mcps", "terminals", "__pycache__",
    "node_modules", "target", "out",
}
SKIP_SUFFIX = {".pem", ".pyc", ".cose", ".sig"}


def should_scan(path: Path) -> bool:
    if any(part in SKIP_DIRS for part in path.parts):
        return False
    if path.suffix in SKIP_SUFFIX:
        return False
    return True


def main() -> None:
    KEYS.mkdir(exist_ok=True)
    rotations: list[dict] = []

    for card in sorted(ENROLL.glob("agent-*.md")):
        text = card.read_text(encoding="utf-8")
        m = re.search(r"\*\*thumbprint:\*\*\s*(MCowBQYDK2Vw[A-Za-z0-9+/=]+)", text)
        if not m:
            print(f"SKIP {card.name}: no thumbprint")
            continue
        old = m.group(1)

        priv = ed25519.Ed25519PrivateKey.generate()
        pub = priv.public_key()
        spki = pub.public_bytes(
            encoding=serialization.Encoding.DER,
            format=serialization.PublicFormat.SubjectPublicKeyInfo,
        )
        new = base64.b64encode(spki).decode("ascii")

        pem = priv.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption(),
        )
        (KEYS / f"{card.stem}_private_key.pem").write_bytes(pem)

        rotations.append({"agent": card.stem, "old": old, "new": new})
        print(f"{card.stem}: rotated")

    # Global replace old -> new across repo text files
    for rot in rotations:
        old, new = rot["old"], rot["new"]
        for path in ROOT.rglob("*"):
            if not path.is_file() or not should_scan(path):
                continue
            try:
                content = path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, OSError):
                continue
            if old not in content:
                continue
            path.write_text(content.replace(old, new), encoding="utf-8")
            print(f"  updated {path.relative_to(ROOT)}")

    manifest = ROOT / "planning" / "KEY_ROTATION_2026-07-12.md"
    lines = [
        "---",
        "aft: AI-generated-user-reviewed-pending",
        "file_role: security_rotation",
        f"generated_at: {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
        "---",
        "",
        "# Enrollment key rotation (2026-07-12)",
        "",
        "Prior private PEMs were removed from the repo and **scrubbed from git history**.",
        "New keypairs generated locally; only public thumbprints below.",
        "",
        "| Agent | New thumbprint |",
        "|-------|----------------|",
    ]
    for rot in rotations:
        lines.append(f"| {rot['agent']} | `{rot['new']}` |")
    lines += [
        "",
        "Private keys: `keys/*_private_key.pem` (gitignored, operator machine only).",
        "Re-sign any artifacts that bound to the old thumbprints.",
    ]
    manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"\nWrote {manifest.relative_to(ROOT)} ({len(rotations)} agents)")


if __name__ == "__main__":
    main()