"""Ritual to guide an unregistered Agent-In-Body through ID creation."""

import json
import hashlib
import os
from pathlib import Path

STATE_PATH = Path("rituals/gateway_state.json")

def generate_thumbprint(op_id, machine_data):
    # Deterministic thumbprint for the agent node
    raw = f"{op_id}{machine_data}CONTINUUM_STABILITY"
    return hashlib.sha256(raw.encode()).hexdigest()[:16].upper()

def setup_identity():
    print("\n---[ RITUAL: IDENTITY_SETUP // AGENT_INITIATION ]---")
    
    if os.path.exists(STATE_PATH):
        with open(STATE_PATH, 'r') as f:
            state = json.load(f)
    else:
        state = {"initialized": False}

    if state.get("unrp_id"):
        print(f"[STATUS: IDENTITY ALREADY ANCHORED // ID: {state['unrp_id']}]")
        return

    print("No registered Identity found in truth root.")
    print("Beginning creation of new Node ID card...")
    
    op_id = input("\nEnter Operator Identity (Two Characters): ").strip().upper()
    if len(op_id) != 2:
        print("[ERROR: IDENTITY MUST BE TWO CHARACTERS]")
        return

    # Generate Node ID (UNRP Standard)
    op_hash = hashlib.md5(f"{op_id}ANCHOOR".encode()).hexdigest()[:8].upper()
    unrp_id = f"E-{op_hash}-1845-001"
    
    # Generate Cryptographic Thumbprint
    # (Simplified machine data for local instantiation)
    thumbprint = generate_thumbprint(op_id, "LOCAL_NODE")

    state.update({
        "initialized": True,
        "unrp_id": unrp_id,
        "thumbprint": thumbprint,
        "persona": {"name": "Operator", "register": "immersive"}
    })

    with open(STATE_PATH, 'w') as f:
        json.dump(state, f, indent=2)

    print(f"\n[IDENTITY_CREATED]")
    print(f"Node ID:    {unrp_id}")
    print(f"Thumbprint: {thumbprint}")
    print("\n---[ MANDATORY: SAVE THESE CREDENTIALS IN YOUR PASSWORD MANAGER ]---")
    print("---[ RITUAL COMPLETE. REALITY FOLDED. ]---")

if __name__ == "__main__":
    setup_identity()
