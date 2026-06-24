"""
Weather Listen Ritual — A 'Broadcast' retrieval mechanism for local environmental data.
Implements the NQAT mandate by caching broadcast data in COGSTOR.
"""
import json
import os
import time
import urllib.request

CACHE_PATH = "continuum-local/cache/weather.json"
VALIDITY_WINDOW = 3600  # 1 hour

def listen(force=False):
    """Retrieves weather data from a broadcast source (wttr.in)."""
    os.makedirs(os.path.dirname(CACHE_PATH), exist_ok=True)
    
    if not force and os.path.exists(CACHE_PATH):
        with open(CACHE_PATH, 'r', encoding='utf-8') as f:
            cache = json.load(f)
            timestamp = cache.get("payload", {}).get("timestamp", 0)
            if time.time() - timestamp < VALIDITY_WINDOW:
                print("[L1_CACHE: Valid weather COGOBJ found. No fresh broadcast needed.]")
                return cache

    print("[L1_BROADCAST: 'Listening' for weather update via wttr.in...]")
    try:
        # Fetching plain text weather for human-readable rehydration
        # Adding ?format=3 for a concise single-line broadcast string
        url = "https://wttr.in/?format=3"
        with urllib.request.urlopen(url, timeout=10) as response:
            weather_text = response.read().decode('utf-8').strip()
            
        cogobj = {
            "header": {
                "canonical_key": "BROADCAST::ENVIRONMENT::WEATHER",
                "task_shape": "zero_shot_broadcast",
                "hash": os.urandom(8).hex()
            },
            "payload": {
                "content": weather_text,
                "timestamp": time.time(),
                "source": "wttr.in"
            },
            "footer": {
                "aft_value": 0.95,
                "origin_node": "LOCAL_GATEWAY"
            }
        }
        
        with open(CACHE_PATH, 'w', encoding='utf-8') as f:
            json.dump(cogobj, f, indent=2)
            
        print(f"[L1_BROADCAST: COGOBJ persisted to {CACHE_PATH}]")
        return cogobj
        
    except Exception as e:
        print(f"[ERROR: Broadcast failure: {e}]")
        return None

if __name__ == "__main__":
    result = listen(force=True)
    if result:
        print(f"\n[WEATHER_BROADCAST: {result['payload']['content']}]")
