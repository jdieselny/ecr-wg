"""Shared runtime paths for COSA local state."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
LOCAL_ROOT = REPO_ROOT / "continuum-local"
STATE_DIR = LOCAL_ROOT / "state"
IDENTITY_DIR = LOCAL_ROOT / "identity"
SANITIZATION_DIR = LOCAL_ROOT / "sanitization"

GATEWAY_STATE_PATH = STATE_DIR / "gateway_state.json"
INTENT_LEDGER_PATH = STATE_DIR / "intent_ledger.json"
HMAC_SECRET_PATH = IDENTITY_DIR / "hmac_secret.key"
LOCAL_DENY_PATTERNS_PATH = SANITIZATION_DIR / "deny_patterns.txt"

LEGACY_GATEWAY_STATE_PATH = REPO_ROOT / "rituals" / "gateway_state.json"
LEGACY_INTENT_LEDGER_PATH = REPO_ROOT / "rituals" / "intent_ledger.json"


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def load_json(path: Path, default: Any, legacy_path: Path | None = None) -> Any:
    source = path if path.exists() else legacy_path
    if source and source.exists():
        with open(source, "r", encoding="utf-8") as f:
            return json.load(f)
    return default


def save_json(path: Path, payload: Any) -> None:
    ensure_parent(path)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
