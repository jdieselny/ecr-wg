use crate::crypto;
use crate::suites::{vector_id, vectors_array};
use crate::{canonicalize, is_canonicalizable, verify_webauthn_signoff};
use serde_json::Value;

pub fn run(vectors: &Value) -> Vec<(String, bool)> {
    let mut results = Vec::new();

    for v in vectors_array(vectors) {
        let id = vector_id(v);
        results.push((id, verify_vector(v)));
    }
    results
}

fn verify_vector(v: &Value) -> bool {
    let (res_obj, is_auth) = if let Some(a) = v.get("resolution_authorization") {
        (a, true)
    } else if let Some(r) = v.get("resolution_receipt") {
        (r, false)
    } else {
        return false;
    };

    let signoff = match res_obj.get("signoff") {
        Some(s) => s,
        None => return false,
    };

    let context = match signoff.get("context") {
        Some(c) => c,
        None => return false,
    };

    if context.get("context_type").and_then(|x| x.as_str()) != Some("ep.resolution.v1") {
        return false;
    }

    let envelope_hash = match context.get("envelope_hash").and_then(|x| x.as_str()) {
        Some(h) => h,
        None => return false,
    };

    if let Some(bm) = v.get("binding_moment") {
        if !is_canonicalizable(bm) {
            return false;
        }
        if let Some(hatches) = bm.get("question").and_then(|q| q.get("hatches")) {
            if hatches.get("dialogue").is_none() || hatches.get("free_text").is_none() {
                return false;
            }
        } else {
            return false;
        }
        let canon = match canonicalize(bm) {
            Ok(c) => c,
            Err(_) => return false,
        };
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(canon.as_bytes());
        let hash_hex = format!("sha256:{:x}", hash);
        if hash_hex != envelope_hash {
            return false;
        }
    } else {
        return false; // MUST have binding moment?
    }

    let action_hash = context.get("action_hash").and_then(|x| x.as_str());
    let exp_action_hash = v.get("expected_action_hash").and_then(|x| x.as_str());
    if action_hash != exp_action_hash {
        return false;
    }

    let nonce = context.get("nonce").and_then(|x| x.as_str());
    let exp_nonce = v.get("expected_nonce").and_then(|x| x.as_str());
    if nonce != exp_nonce {
        return false;
    }

    let initiator = context.get("initiator").and_then(|x| x.as_str());
    let exp_initiator = v.get("expected_initiator").and_then(|x| x.as_str());
    if initiator != exp_initiator {
        return false;
    }

    let eval_str = v.get("evaluation_time").and_then(|x| x.as_str());
    let issued_str = context.get("issued_at").and_then(|x| x.as_str());
    let expires_str = context.get("expires_at").and_then(|x| x.as_str());

    if let (Some(eval), Some(iss), Some(exp)) = (eval_str, issued_str, expires_str) {
        let eval_ms = crate::suites::time_attestation::parse_instant_ms(eval).unwrap_or(0.0);
        let iss_ms = crate::suites::time_attestation::parse_instant_ms(iss).unwrap_or(f64::MAX);
        let exp_ms = crate::suites::time_attestation::parse_instant_ms(exp).unwrap_or(0.0);
        if eval_ms < iss_ms || eval_ms > exp_ms {
            return false;
        }
    } else {
        return false;
    }

    let resolution = match context.get("resolution") {
        Some(r) if r.is_object() => r,
        _ => return false,
    };

    let outcome = match resolution.get("outcome").and_then(|x| x.as_str()) {
        Some(o) => o,
        None => return false,
    };

    if is_auth && outcome != "approved" {
        return false;
    }

    match outcome {
        "approved" => {
            let sel = resolution.get("selected_option").and_then(|x| x.as_u64());
            if sel.is_none() {
                return false;
            }
            let sel_val = sel.unwrap();
            
            let bm = v.get("binding_moment").unwrap();
            let options = bm.get("question")
                .and_then(|q| q.get("options"))
                .and_then(|o| o.as_array());
                
            if let Some(opts) = options {
                if sel_val >= opts.len() as u64 {
                    return false;
                }
            } else {
                return false; // options must exist
            }

            if let Some(exp_sel) = v.get("expected_selected_option").and_then(|x| x.as_u64()) {
                if sel_val != exp_sel {
                    return false;
                }
            } else {
                if is_auth {
                    return false;
                }
            }
        }
        "amended" => {
            if resolution.get("response_hash").and_then(|x| x.as_str()).is_none() {
                return false;
            }
            if resolution.get("successor_envelope_hash").and_then(|x| x.as_str()) == Some(envelope_hash) {
                return false;
            }
        }
        "declined" => {
            if resolution.get("successor_envelope_hash").is_some() {
                return false;
            }
        }
        "rejected" => {
            if resolution.get("successor_envelope_hash").and_then(|x| x.as_str()) == Some(envelope_hash) {
                return false;
            }
        }
        _ => return false,
    }

    let principal = match context.get("principal").and_then(|x| x.as_str()) {
        Some(p) => p,
        None => return false,
    };

    let pk_id = match context.get("principal_key_id").and_then(|x| x.as_str()) {
        Some(p) => p,
        None => return false,
    };

    let pks = match v.get("principal_keys").and_then(|x| x.as_object()) {
        Some(p) => p,
        None => return false,
    };

    let pk_entry = match pks.get(pk_id) {
        Some(e) => e,
        None => return false,
    };

    if pk_entry.get("principal").and_then(|x| x.as_str()) != Some(principal) {
        return false;
    }

    let public_key = match pk_entry.get("public_key").and_then(|x| x.as_str()) {
        Some(p) => p,
        None => return false,
    };

    let rp_id = v.get("rp_id").and_then(|x| x.as_str());
    let allowed_origins = v.get("allowed_origins");

    let signoff_str = serde_json::to_string(signoff).unwrap_or_default();
    verify_webauthn_signoff(&signoff_str, public_key, rp_id, allowed_origins).unwrap_or(false)
}
