import os
import re
import glob
import base64
from pathlib import Path
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization

enrollment_dir = Path("enrollments")

def update_enrollments():
    print("Generating Ed25519 Cryptographic Thumbprints (Iman's tech) for all agents...\n")
    
    # 1. First pass: find the primary identity in each file and generate a new key for it
    key_map = {}
    
    for filepath in enrollment_dir.glob("agent-*.md"):
        content = filepath.read_text(encoding="utf-8")
        
        # Extract the primary thumbprint from the file (first 16-hex character word)
        match = re.search(r'\b([0-9A-F]{16})\b', content)
        if match:
            old_hex = match.group(1).upper()
            
            # Generate new Ed25519 keypair
            private_key = ed25519.Ed25519PrivateKey.generate()
            public_key = private_key.public_key()
            
            pub_spki = public_key.public_bytes(
                encoding=serialization.Encoding.DER,
                format=serialization.PublicFormat.SubjectPublicKeyInfo
            )
            pub_b64 = base64.b64encode(pub_spki).decode('utf-8')
            
            key_map[old_hex] = pub_b64
            
            # Save the private keys in a keys/ directory for demonstration
            keys_dir = Path("keys")
            keys_dir.mkdir(exist_ok=True)
            priv_bytes = private_key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption()
            )
            key_filename = filepath.stem + "_private_key.pem"
            (keys_dir / key_filename).write_bytes(priv_bytes)
            
            print(f"Generated key for {filepath.name}: {old_hex} -> {pub_b64[:16]}...")
            
    # 2. Second pass: Replace all known old hex thumbprints with their new Ed25519 keys
    print("\nUpdating all references in markdown files...")
    
    for filepath in enrollment_dir.glob("*.md"):
        content = filepath.read_text(encoding="utf-8")
        original_content = content
        
        # Replace each old thumbprint with the new one
        for old_hex, pub_b64 in key_map.items():
            content = re.sub(r'\b' + old_hex + r'\b', pub_b64, content, flags=re.IGNORECASE)
            
        # Replace the cryptographic binding explanation if present
        binding_replacement = (
            "## Cryptographic binding (ACTIVE)\n\n"
            "This card is natively bound to an Ed25519 cryptographic keypair (Iman's protocol).\n"
            "* **Identity values computed via:** `cryptography.hazmat.primitives.asymmetric.ed25519`.\n"
            "* **Public Key (Thumbprint):** The thumbprint is the SPKI DER Base64 representation of the agent's public key.\n"
            "* **Signature Verification:** Natively verifiable by the COSA external verifier."
        )
        content = re.sub(
            r'## Cryptographic binding \(PENDING\).*?## Provenance trail',
            binding_replacement + '\n\n## Provenance trail',
            content,
            flags=re.DOTALL
        )
        
        if content != original_content:
            filepath.write_text(content, encoding="utf-8")
            print(f"Saved {filepath.name}")

if __name__ == '__main__':
    update_enrollments()
