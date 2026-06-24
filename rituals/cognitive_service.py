import requests
import json
import hashlib
from rituals.intent_ledger import IntentLedger
from rituals.sanitization import Sanitizer
from rituals.identity_crypto import sha256_hex

class CognitiveService:
    def __init__(self, endpoint="http://localhost:11434/api/generate", model="llama3:8b"):
        self.endpoint = endpoint
        self.model = model
        self.ledger = IntentLedger()

    def _hash_query(self, text):
        return sha256_hex(text.lower().strip())

    def _legacy_hash_query(self, text):
        return hashlib.md5(text.lower().strip().encode("utf-8")).hexdigest()

    def classify(self, text, force_json=False):
        clean_text = Sanitizer.sanitize(text)
        query_hash = self._hash_query(clean_text)
        
        # 1. Primary: Local Ledger (Zero Compute)
        cached = self.ledger.get(query_hash) or self.ledger.get(self._legacy_hash_query(clean_text))
        if cached:
            return cached

        # 2. Secondary: Local Inference (Ollama)
        prompt = f"{'Output ONLY a valid JSON object. ' if force_json else ''}{clean_text}"
        payload = {"model": self.model, "prompt": prompt, "stream": False}
        
        try:
            response = requests.post(self.endpoint, json=payload, timeout=120)
            response.raise_for_status()
            classification = Sanitizer.sanitize(response.json().get("response"))
            self.ledger.add(query_hash, classification)
            return classification
        except requests.exceptions.RequestException as e:
            # 3. Tertiary: Local Fail-safe
            print(f"[DEBUG: Inference Error: {e}]")
            return json.dumps({"status": "OFFLINE_MODE", "error": f"Inference engine unreachable: {e}. Ledger empty for this intent."})
