import json
import os

from rituals.runtime_paths import INTENT_LEDGER_PATH, LEGACY_INTENT_LEDGER_PATH, ensure_parent

class IntentLedger:
    def __init__(self, ledger_path=INTENT_LEDGER_PATH):
        self.ledger_path = ledger_path
        self.ledger = self._load_ledger()

    def _load_ledger(self):
        if not os.path.exists(self.ledger_path):
            if LEGACY_INTENT_LEDGER_PATH.exists():
                with open(LEGACY_INTENT_LEDGER_PATH, "r", encoding="utf-8") as f:
                    return json.load(f)
            return {}
        with open(self.ledger_path, "r", encoding="utf-8") as f:
            return json.load(f)

    def save(self):
        ensure_parent(self.ledger_path)
        with open(self.ledger_path, "w", encoding="utf-8") as f:
            json.dump(self.ledger, f, indent=2)

    def get(self, query_hash):
        return self.ledger.get(query_hash)

    def add(self, query_hash, classification):
        self.ledger[query_hash] = classification
        self.save()
