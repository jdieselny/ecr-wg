"""Intent Normalizer — Maps raw human queries to canonical intent keys from the Lattice Registry."""
import json
import os
from rituals.cognitive_service import CognitiveService

def load_intent_table(path="rituals/intent_table.json"):
    if os.path.exists(path):
        with open(path, 'r', encoding='utf-8') as f:
            return json.load(f)
    return {}

def normalize(user_input):
    print(f"\n[Invoking Layer-0 Normalizer for input: '{user_input[:40]}...']")
    
    service = CognitiveService(model="llama3:8b")
    intent_table = load_intent_table()
    
    # Flatten the registry for the prompt
    registry = intent_table.get("LATTICE_REGISTRY", {})
    available_keys = []
    
    def extract_keys(data):
        if isinstance(data, dict):
            if "canonical_key" in data:
                available_keys.append(f"- {data['canonical_key']}: {data.get('description', '')}")
            else:
                for v in data.values():
                    extract_keys(v)
                    
    extract_keys(registry)
    keys_list = "\n".join(available_keys)
    
    prompt = f"""
    Analyze the following user input and map it to the MOST RELEVANT canonical intent key from the registry below.
    
    REGISTRY:
    {keys_list}
    
    USER INPUT: "{user_input}"
    
    Output ONLY the canonical_key (e.g., DOMAIN::FUNCTION::COMPONENT). 
    If no key is relevant, output "GENERAL_QUERY".
    """
    
    normalized_key = service.classify(prompt).strip().upper()
    
    # --- HARDENED VALIDATION ---
    valid_keys = set()
    def collect_valid_keys(data):
        if isinstance(data, dict):
            if "canonical_key" in data:
                valid_keys.add(data["canonical_key"].upper())
            else:
                for v in data.values():
                    collect_valid_keys(v)
    collect_valid_keys(registry)
    
    # Extract just the key if it's wrapped in text
    found_key = "GENERAL_QUERY"
    for word in normalized_key.replace('"', '').replace("'", "").split():
        clean_word = word.strip(".,!\"'")
        if "::" in clean_word:
            if clean_word in valid_keys:
                return clean_word
            else:
                print(f"[WARN: Normalizer returned non-canonical key: {clean_word}]")
    
    if normalized_key in valid_keys:
        return normalized_key
        
    return "GENERAL_QUERY"

if __name__ == "__main__":
    test_input = input("Enter test prompt: ")
    result = normalize(test_input)
    print(f"\n[NORMALIZED_KEY: {result}]")
