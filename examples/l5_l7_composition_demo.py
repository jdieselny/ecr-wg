import os
import sys
import json
import time
import hashlib
import datetime

# Ensure we can import from rituals
ECR_WG_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.append(ECR_WG_PATH)

# Add emilia-protocol python-verify path for canonicalization & verification tools
EP_PYTHON_VERIFY = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../emilia-protocol/packages/python-verify"))
sys.path.append(EP_PYTHON_VERIFY)

try:
    from emilia_verify import canonicalize, is_canonicalizable
    import cryptography
    HAS_EMILIA_VERIFY = True
except ImportError:
    HAS_EMILIA_VERIFY = False

class EPReceiptRequiredError(Exception):
    """Exception raised when an irreversible action lacks a valid EP receipt (402 Refusal)."""
    def __init__(self, action_type, policy_id, parameters):
        self.action_type = action_type
        self.policy_id = policy_id
        self.parameters = parameters
        
        # Construct the official EP 402-style refusal object
        self.refusal_object = {
            "ep_refused": True,
            "status": 402,
            "code": "emilia_receipt_required",
            "title": "EMILIA Receipt Required",
            "required": {
                "action": action_type,
                "policy_id": policy_id,
                "parameters": parameters,
                "how": f"Gate this action, obtain an EP-RECEIPT-v1 for policy '{policy_id}', then retry with '__ep_receipt'."
            }
        }
        super().__init__(f"EP Receipt Required for {action_type} under policy {policy_id}")

def get_action_object(action_type, target, parameters, initiator, policy_id):
    """Constructs a canonical Action Object conforming to Section 3 of the EP receipts draft."""
    return {
        "ep_version": "1.0",
        "action_type": action_type,
        "target": target,
        "parameters": parameters,
        "initiator": initiator,
        "policy_id": policy_id,
        "requested_at": datetime.datetime.now(datetime.UTC).isoformat().replace("+00:00", "") + "Z"
    }

def verify_action_receipt(receipt, runtime_action_info, required_role):
    """
    Offline verification of an EP Trust Receipt against the execution parameters.
    This implements the execution-side verification loop (Section 6.3/9).
    """
    if not receipt or not isinstance(receipt, dict):
        return False, "Malformed receipt structure"
    
    receipt_action = receipt.get("action")
    if not receipt_action or not isinstance(receipt_action, dict):
        return False, "Receipt does not contain action object"
        
    # 1. Match runtime execution parameters with the action details in the receipt
    for field in ["action_type", "target", "parameters", "initiator", "policy_id"]:
        if receipt_action.get(field) != runtime_action_info.get(field):
            return False, f"Receipt parameter mismatch on '{field}': Receipt has '{receipt_action.get(field)}', but runtime call has '{runtime_action_info.get(field)}'"
            
    # 2. Action hash verification: recompute hash from the receipt's action object
    if HAS_EMILIA_VERIFY:
        action_bytes = canonicalize(receipt_action).encode("utf-8")
        computed_hash = "sha256:" + hashlib.sha256(action_bytes).hexdigest()
    else:
        # Fallback naive canonicalization if cryptography/emilia_verify is missing
        action_str = json.dumps(receipt_action, sort_keys=True)
        computed_hash = "sha256:" + hashlib.sha256(action_str.encode("utf-8")).hexdigest()
        
    if receipt.get("action_hash") != computed_hash:
        return False, f"Receipt action_hash mismatch. Receipt root has '{receipt.get('action_hash')}', but recomputed is '{computed_hash}'"

    # 3. Check signatures and roles
    contexts = receipt.get("contexts", [])
    signoffs = receipt.get("signoffs", [])
    
    if not contexts or not signoffs or len(contexts) != len(signoffs):
        return False, "Receipt context/signoff counts mismatch"
        
    for ctx, sig in zip(contexts, signoffs):
        # Verify the context in the signoff binds the action hash
        if ctx.get("action_hash") != computed_hash:
            return False, "Context action hash mismatch"
            
        # Verify that the signer matches the required role in our roster
        # In a real environment, we'd cross-reference with the Approver Directory (Section 5.2)
        approver_id = ctx.get("approver")
        
        # Verify freshness window
        issued_at_ts = datetime.datetime.fromisoformat(ctx.get("issued_at").replace("Z", "+00:00")).timestamp()
        expires_at_ts = datetime.datetime.fromisoformat(ctx.get("expires_at").replace("Z", "+00:00")).timestamp()
        now_ts = time.time()
        
        if now_ts > expires_at_ts:
            return False, "Receipt has expired"
            
        # [MOCK CRYPTOGRAPHIC VERIFICATION]
        # In the live session, we would use emilia_verify.verify_webauthn_signoff
        # to cryptographically check the WebAuthn signatures in sig against the public keys.
        # Here we mock it by verifying the signature field is non-empty and starts with "mock_sig_"
        if not sig.get("signature", "").startswith("mock_sig_"):
            return False, "Invalid signature on context"
            
    return True, "Verified"

def require_receipt(policy_id, required_role):
    """
    Decorator for guarding dangerous tools (L7 Governance Plane).
    Intercepts calls and demands an EP receipt.
    """
    def decorator(func):
        def wrapper(*args, **kwargs):
            # Extract arguments to build runtime action details
            action_type = f"tool.{func.__name__}"
            target = {
                "system": "cosa.dispatch",
                "resource": func.__name__
            }
            # Clean control fields from the parameter set
            params = {k: v for k, v in kwargs.items() if k not in ["__ep_receipt", "__initiator"]}
            initiator = kwargs.get("__initiator", "ep:entity:cosa-agent-1")
            
            # Pack details to match against the receipt
            runtime_action_info = {
                "action_type": action_type,
                "target": target,
                "parameters": params,
                "initiator": initiator,
                "policy_id": policy_id
            }
            
            # Check if receipt is presented
            receipt = kwargs.get("__ep_receipt")
            
            if not receipt:
                # No receipt presented -> Raise 402 Refusal
                raise EPReceiptRequiredError(action_type, policy_id, params)
                
            # Verify the receipt against runtime action parameters
            is_valid, reason = verify_action_receipt(receipt, runtime_action_info, required_role)
            if not is_valid:
                raise PermissionError(f"EP Receipt Verification Failed: {reason}")
                
            # Remove control fields before passing to the actual tool
            kwargs.pop("__ep_receipt", None)
            kwargs.pop("__initiator", None)
            
            print(f"\n[L7 GOVERNANCE: Receipt verified successfully. Executing action: {func.__name__}]")
            return func(*args, **kwargs)
        return wrapper
    return decorator

# --- L5 DEMO ACTION TO BE WRAPPED ---

@require_receipt(policy_id="ep:policy:weather-alert-action@v1", required_role="program_officer")
def dispatch_emergency_salt_trucks(location, reason):
    """
    Dangerous, irreversible action: Dispatches municipal salt trucks.
    Gated by L7 Governance.
    """
    print(f"  [ACTION EXECUTED] TRUCK: Emergency Salt Trucks dispatched to {location}!")
    print(f"  [REASON] {reason}")
    return {"status": "SUCCESS", "dispatched_trucks": 15, "location": location}

# --- SIMULATOR UTILITIES ---

def mock_human_signoff(action_obj, approver_id="ep:approver:po_rivera"):
    """Simulates a human approver signing the exact action offline (Class A / WebAuthn)."""
    # 1. Compute action hash
    if HAS_EMILIA_VERIFY:
        action_bytes = canonicalize(action_obj).encode("utf-8")
        action_hash = "sha256:" + hashlib.sha256(action_bytes).hexdigest()
    else:
        action_str = json.dumps(action_obj, sort_keys=True)
        action_hash = "sha256:" + hashlib.sha256(action_str.encode("utf-8")).hexdigest()
        
    # 2. Build Authorization Context
    now = datetime.datetime.now(datetime.UTC)
    ctx = {
        "ep_version": "1.0",
        "context_type": "ep.signoff.v1",
        "action_hash": action_hash,
        "policy_id": action_obj["policy_id"],
        "policy_hash": "sha256:77ab8c9d1234567890abcdef...", # Commit to policy
        "initiator": action_obj["initiator"],
        "approver": approver_id,
        "approver_index": 1,
        "required_approvals": 1,
        "nonce": "b64u:nonce_value_1234567890",
        "issued_at": now.isoformat().replace("+00:00", "") + "Z",
        "expires_at": (now + datetime.timedelta(minutes=15)).isoformat().replace("+00:00", "") + "Z",
    }
    
    # 3. Simulate signoff signature
    context_hash = hashlib.sha256(json.dumps(ctx, sort_keys=True).encode()).hexdigest()
    signoff = {
        "context_hash": f"sha256:{context_hash}",
        "signature": f"mock_sig_webauthn_assertion_{context_hash[:16]}",
        "key_class": "A",
        "approver_key_id": f"ep:key:{approver_id.split(':')[-1]}#2026-01",
        "signed_at": now.isoformat().replace("+00:00", "") + "Z",
        "webauthn": {
            "authenticator_data": "b64u:authenticator_data_bytes",
            "client_data_json": "b64u:client_data_json_bytes"
        }
    }
    
    # 4. Assemble Trust Receipt
    receipt = {
        "receipt_id": f"ep:receipt:{hashlib.md5(context_hash.encode()).hexdigest()[:12]}",
        "action": action_obj,
        "action_hash": action_hash,
        "contexts": [ctx],
        "signoffs": [signoff],
        "consumption": {
            "nonce": "b64u:nonce_value_1234567890",
            "state": "COMMITTED",
            "committed_at": now.isoformat().replace("+00:00", "") + "Z"
        }
    }
    return receipt

def simulate_composition():
    print("=" * 70)
    print(" L5 + L7 COMPOSITION: EMILIA Protocol Gating COSA Broadcast-Driven Action")
    print("=" * 70)
    
    # Step 1: L5 Weather Broadcast check (from rituals)
    print("\n[STEP 1: L5 Broadcast Retrieval]")
    # Import solve from rituals
    try:
        from rituals.solve_with_broadcast import solve
        cogobj = solve("BROADCAST::ENVIRONMENT::WEATHER", "What is the weather in New York?")
        weather_text = cogobj["data"]["resolution"][0]
        print(f"  Rehydrated L5 Weather: {weather_text}")
    except Exception as e:
        # Fallback if execution path is different
        weather_text = "BROADCAST_DATA: New York, US: HEAVY BLIZZARD -8C"
        print(f"  Rehydrated L5 Weather (simulated): {weather_text}")
    
    print("-" * 70)
    
    # Step 2: Agent evaluates the authentic input and proposes an action
    print("\n[STEP 2: Agent Action Proposals]")
    location = "New York Manhattan Division"
    reason = "Severe cold event detected in L5 cache: " + weather_text
    
    # Initiating agent
    agent_id = "ep:entity:cosa-weather-agent-1"
    policy_id = "ep:policy:weather-alert-action@v1"
    
    print(f"  Agent {agent_id} proposes calling 'dispatch_emergency_salt_trucks'")
    print(f"  Target: {location}")
    print(f"  Reason: {reason}")
    
    print("-" * 70)
    
    # Step 3: Call action without receipt (FAIL CLOSED)
    print("\n[STEP 3: Try Execution Without Receipt]")
    try:
        dispatch_emergency_salt_trucks(location=location, reason=reason, __initiator=agent_id)
    except EPReceiptRequiredError as e:
        print("  [L7 BLOCKED] Execution Refused!")
        print(f"  [402 Refusal Content] {json.dumps(e.refusal_object, indent=2)}")
        
    print("-" * 70)
    
    # Step 4: Request Human-in-the-Loop Signoff (Offline Receipt Generation)
    print("\n[STEP 4: Escalate to Roster Approver]")
    print("  Escalating to: ep:approver:po_rivera (Role: program_officer)")
    
    # Construct proposed action object for the human to sign
    action_type = "tool.dispatch_emergency_salt_trucks"
    target = {"system": "cosa.dispatch", "resource": "dispatch_emergency_salt_trucks"}
    params = {"location": location, "reason": reason}
    action_obj = get_action_object(action_type, target, params, agent_id, policy_id)
    
    # Generate receipt via user signoff
    receipt = mock_human_signoff(action_obj, approver_id="ep:approver:po_rivera")
    print("  [EP HOST] Human signed. EP Trust Receipt v1 generated.")
    print(f"  Receipt ID: {receipt['receipt_id']}")
    print(f"  Signer key: {receipt['signoffs'][0]['approver_key_id']}")
    
    print("-" * 70)
    
    # Step 5: Retry call with receipt (EXECUTION GRANTED)
    print("\n[STEP 5: Retry Execution With Receipt]")
    try:
        result = dispatch_emergency_salt_trucks(
            location=location, 
            reason=reason, 
            __initiator=agent_id, 
            __ep_receipt=receipt
        )
        print(f"  [RESULT] {result}")
    except Exception as e:
        print(f"  [ERROR] Execution unexpectedly failed: {e}")
        
    print("=" * 70)

if __name__ == "__main__":
    simulate_composition()
