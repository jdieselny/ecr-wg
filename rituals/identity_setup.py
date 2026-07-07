"""Ritual to guide an unregistered Agent-In-Body through ID creation."""

import json
import hashlib
import os
from pathlib import Path

STATE_PATH = Path("rituals/gateway_state.json")

import base64
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization

def generate_thumbprint(op_id, machine_data):
    # Ed25519 Cryptographic Thumbprint for the agent node (Iman's tech)
    private_key = ed25519.Ed25519PrivateKey.generate()
    public_key = private_key.public_key()
    
    priv_bytes = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption()
    )
    pub_bytes = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw
    )
    
    # Save the private key securely (in the state dir)
    priv_path = Path("rituals/private_key.pem")
    priv_path.parent.mkdir(parents=True, exist_ok=True)
    priv_path.write_bytes(priv_bytes)
    
    # Return the SPKI Base64 representation of the public key as the thumbprint
    pub_spki = public_key.public_bytes(
        encoding=serialization.Encoding.DER,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    return base64.b64encode(pub_spki).decode('utf-8')

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
    unrp_id = f"E-{op_hash}-01-001"
    
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

def self_identify():
    """Retrieve and display the current identity without re-initialization."""
    if not os.path.exists(STATE_PATH):
        print("[ERROR: NO IDENTITY FOUND IN TRUTH ROOT]")
        return None
    
    with open(STATE_PATH, 'r') as f:
        state = json.load(f)
    
    if not state.get("unrp_id"):
        print("[ERROR: IDENTITY NOT ANCHORED]")
        return None
    
    print("\n---[ RITUAL: SELF_IDENTIFY // IDENTITY_RETRIEVAL ]---")
    print(f"Node ID:    {state['unrp_id']}")
    print(f"Thumbprint: {state['thumbprint']}")
    print(f"Persona:    {state.get('persona', {}).get('name', 'Unknown')}")
    print("---[ IDENTITY VERIFIED. ]---\n")
    
    return state

if __name__ == "__main__":
    setup_identity()
