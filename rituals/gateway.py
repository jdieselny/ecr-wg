"""COSA Gateway: central entry point for system initialization and operational check."""
import json

from rituals.identity_crypto import HMAC_ALGORITHM, generate_unrp_id, identity_thumbprint
from rituals.runtime_paths import (
    GATEWAY_STATE_PATH,
    LEGACY_GATEWAY_STATE_PATH,
    ensure_parent,
    load_json,
)

class Gateway:
    GATEWAY_STATE = GATEWAY_STATE_PATH

    def __init__(self):
        self.state = self._load_state()

    def _load_state(self):
        return load_json(
            self.GATEWAY_STATE,
            {"initialized": False, "unrp_id": None, "persona": None},
            legacy_path=LEGACY_GATEWAY_STATE_PATH,
        )

    def _save_state(self):
        ensure_parent(self.GATEWAY_STATE)
        with open(self.GATEWAY_STATE, 'w', encoding='utf-8') as f:
            json.dump(self.state, f, indent=2)

    def boot(self):
        if not self.state["initialized"]:
            self._init_ritual()
        else:
            self._operational_mode()

    def _init_ritual(self):
        print("\n" + "="*60)
        print("---[ COSA REFERENCE OVERLAY // SYSTEM INITIATION ]---")
        print("Initializing the standard execution gateway.")
        print("You are the local node operator.")
        print("="*60 + "\n")

        op_id = input("Enter your Operator Identity (Two Characters): ").strip().upper()
        if len(op_id) != 2:
            print("[ERROR: IDENTITY MUST BE TWO CHARACTERS]")
            return

        unrp_id = generate_unrp_id(op_id, "001")

        self.state["initialized"] = True
        self.state["unrp_id"] = unrp_id
        self.state["thumbprint_alg"] = HMAC_ALGORITHM
        self.state["thumbprint"] = identity_thumbprint(op_id, "LOCAL_NODE")
        self.state["persona"] = {"name": "Operator", "register": "immersive"}
        self._save_state()

        print(f"\n[IDENTITY_LOCKED // {unrp_id}]")
        print("[SESSION_CLOSE // RITUAL_COMPLETE]")
        print("\nCOSA initialization successful. The system is now anchored.")
        print("---[ SYSTEM MAINTENANCE: YOU MAY SAFELY CLOSE THIS WINDOW ]---")
        print("---[ RESUME: RERUN 'python -m rituals.gateway' AFTER REBOOT ]---")

    def _operational_mode(self):
        unrp_id = self.state.get("unrp_id", "UNKNOWN")
        persona_name = self.state.get("persona", {}).get("name", "Operator")

        print("\n" + "="*60)
        print(f"[SYSTEM_STATUS: STABLE // NODE: {unrp_id}]")
        print(f"[WELCOME BACK, {persona_name.upper()}]")
        print("="*60 + "\n")
        print("Gateway online. Ready to dispatch queries.")


if __name__ == "__main__":
    g = Gateway()
    g.boot()
