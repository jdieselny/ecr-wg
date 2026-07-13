// SPDX-License-Identifier: Apache-2.0
// EP-AEC-ROLE-v1 — Role-substitution E2E verifier.

use crate::canonical::canonicalize;
use crate::suites::{vector_id, vectors_array};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub fn run(vectors: &Value) -> Vec<(String, bool)> {
    let mut results = Vec::new();
    for v in vectors_array(vectors) {
        let id = vector_id(v);
        let valid = verify_aec_vector(v, &id);
        results.push((id, valid));
    }
    results
}

fn norm_digest(s: &str) -> &str {
    s.strip_prefix("sha256:").unwrap_or(s)
}

fn verify_aec_vector(v: &Value, _id: &str) -> bool {
    let aec_chain = match v.get("aec_chain") {
        Some(c) => c,
        None => return false,
    };

    let version = aec_chain.get("@version").and_then(|x| x.as_str());
    if version != Some("EP-AEC-v1") {
        return false;
    }

    let action = match aec_chain.get("action") {
        Some(a) => a,
        None => return false,
    };

    let action_canon = match canonicalize(action) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let chain_digest = format!("sha256:{}", hex::encode(Sha256::digest(action_canon.as_bytes())));

    let expected_digest = match v.get("expected_action_digest").and_then(|x| x.as_str()) {
        Some(d) => d,
        None => return false,
    };
    if norm_digest(expected_digest) != norm_digest(&chain_digest) {
        return false;
    }

    let stub_types: HashSet<String> = v.get("stub_types")
        .and_then(|arr| arr.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let components = match aec_chain.get("components").and_then(|arr| arr.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return false,
    };

    let mut satisfied = HashSet::new();

    for c in components {
        let c_type = match c.get("type").and_then(|x| x.as_str()) {
            Some(t) => t,
            None => return false,
        };

        let evidence = match c.get("evidence") {
            Some(e) => e,
            None => return false,
        };

        // Reserved verifiers (ep-receipt, ep-quorum) cannot be overridden by stubs.
        let is_stub = stub_types.contains(c_type) && c_type != "ep-receipt" && c_type != "ep-quorum";
        if is_stub {
            let valid = evidence.get("valid").map(|val| val.as_bool() != Some(false)).unwrap_or(true);
            let attested_digest = evidence.get("action_digest").and_then(|x| x.as_str());
            if valid && attested_digest.map(norm_digest) == Some(norm_digest(&chain_digest)) {
                satisfied.insert(c_type.to_string());
            }
            continue;
        }

        if c_type == "ep-receipt" {
            let profile = match v.get("policies_by_type").and_then(|p| p.get("ep-receipt")) {
                Some(p) => p,
                None => return false,
            };

            if !verify_ep_receipt_component(evidence, profile, &chain_digest, v.get("verification_time").and_then(|x| x.as_str())) {
                return false;
            }
            satisfied.insert(c_type.to_string());
        } else if c_type == "ep-quorum" {
            let profile = match v.get("policies_by_type").and_then(|p| p.get("ep-quorum")) {
                Some(p) => p,
                None => return false,
            };

            if !verify_ep_quorum_component(evidence, profile, &chain_digest, v.get("verification_time").and_then(|x| x.as_str())) {
                return false;
            }
            satisfied.insert(c_type.to_string());
        } else {
            return false;
        }
    }

    let req_expr = match v.get("requirement").and_then(|x| x.as_str()) {
        Some(r) => r,
        None => return false,
    };

    eval_requirement(req_expr, &satisfied).unwrap_or(false)
}

fn verify_ep_receipt_component(evidence: &Value, profile: &Value, chain_digest: &str, verification_time: Option<&str>) -> bool {
    let rp_id = match profile.get("rp_id").and_then(|x| x.as_str()) {
        Some(s) => s,
        None => return false,
    };

    let expected_policy_hash = match profile.get("expected_policy_hash").and_then(|x| x.as_str()) {
        Some(s) => s,
        None => return false,
    };

    let max_age_sec = match profile.get("max_age_sec").and_then(|x| x.as_u64()) {
        Some(n) => n,
        None => return false,
    };

    if !fresh_registry_snapshot(profile, verification_time) {
        return false;
    }

    let allowed_origins: HashSet<String> = profile.get("allowed_origins")
        .and_then(|arr| arr.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let contexts = match evidence.get("contexts").and_then(|arr| arr.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return false,
    };

    let signoffs = match evidence.get("signoffs").and_then(|arr| arr.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return false,
    };

    let mut context_by_hash = std::collections::HashMap::new();

    for ctx in contexts {
        let policy_hash = match ctx.get("policy_hash").and_then(|x| x.as_str()) {
            Some(s) => s,
            None => return false,
        };
        if policy_hash != expected_policy_hash {
            return false;
        }

        let ctx_canon = match canonicalize(ctx) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let digest_hex = format!("sha256:{}", hex::encode(Sha256::digest(ctx_canon.as_bytes())));
        context_by_hash.insert(digest_hex, ctx);
    }

    let rp_hash = Sha256::digest(rp_id.as_bytes());

    for s in signoffs {
        let key_id = match s.get("approver_key_id").and_then(|x| x.as_str()) {
            Some(id) => id,
            None => return false,
        };

        let key_entry = match profile.get("approver_keys").and_then(|m| m.get(key_id)) {
            Some(e) => e,
            None => return false,
        };

        if !active_directory_entry(key_entry, verification_time) {
            return false;
        }

        if key_entry.get("key_class").and_then(|x| x.as_str()) != Some("A") {
            return false;
        }

        let context_hash = match s.get("context_hash").and_then(|x| x.as_str()) {
            Some(h) => h,
            None => return false,
        };

        let signed_context = match context_by_hash.get(context_hash) {
            Some(c) => c,
            None => return false,
        };

        let approver_id = key_entry.get("approver_id").and_then(|x| x.as_str());
        let signed_approver = signed_context.get("approver").and_then(|x| x.as_str());
        if approver_id != signed_approver {
            return false;
        }

        let webauthn = match s.get("webauthn") {
            Some(w) => w,
            None => return false,
        };

        let auth_data_b64 = match webauthn.get("authenticator_data").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return false,
        };
        let auth_data = match decode_base64url(auth_data_b64) {
            Ok(b) => b,
            Err(_) => return false,
        };
        if auth_data.len() < 37 || auth_data[..32] != rp_hash[..] {
            return false;
        }

        let origin = webauthn_origin(webauthn);
        if !allowed_origins.contains(&origin) {
            return false;
        }

        if !fresh_at(signed_context, verification_time, max_age_sec) {
            return false;
        }
    }

    let opts = crate::suites::trust_receipt::VerifyOpts {
        allow_legacy_merkle: false,
    };

    let verification_obj = json!({
        "approver_keys": profile.get("approver_keys"),
        "log_public_key": profile.get("log_public_key")
    });

    if !crate::suites::trust_receipt::verify_trust_receipt(evidence, &verification_obj, &opts) {
        return false;
    }

    let action_hash = match evidence.get("action_hash").and_then(|x| x.as_str()) {
        Some(h) => h,
        None => return false,
    };

    if norm_digest(action_hash) != norm_digest(chain_digest) {
        return false;
    }

    true
}

fn verify_ep_quorum_component(evidence: &Value, profile: &Value, chain_digest: &str, verification_time: Option<&str>) -> bool {
    let rp_id = match profile.get("rp_id").and_then(|x| x.as_str()) {
        Some(s) => s,
        None => return false,
    };

    let context_policy = match profile.get("context_policy").and_then(|x| x.as_str()) {
        Some(s) => s,
        None => return false,
    };

    let max_age_sec = match profile.get("max_age_sec").and_then(|x| x.as_u64()) {
        Some(n) => n,
        None => return false,
    };

    if !fresh_registry_snapshot(profile, verification_time) {
        return false;
    }

    let profile_policy = match profile.get("policy") {
        Some(p) => p,
        None => return false,
    };

    let mode = profile_policy.get("mode").and_then(|v| v.as_str());
    if mode != Some("threshold") && mode != Some("ordered") {
        return false;
    }

    let required = if mode == Some("ordered") {
        if profile_policy.get("ordered_chain").and_then(|v| v.as_bool()).unwrap_or(false) {
            profile_policy.get("approvers").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
        } else {
            1
        }
    } else {
        profile_policy.get("required").and_then(|v| v.as_u64()).unwrap_or(0) as usize
    };

    if required < 2 {
        return false;
    }

    if profile_policy.get("distinct_humans").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }

    // Presented policy must equal the relying-party-pinned policy (canonicalized)
    let evidence_policy = evidence.get("policy").unwrap_or(&Value::Null);
    let ev_policy_canon = match canonicalize(evidence_policy) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let prof_policy_canon = match canonicalize(profile_policy) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if ev_policy_canon != prof_policy_canon {
        return false;
    }

    let allowed_origins: HashSet<String> = profile.get("allowed_origins")
        .and_then(|arr| arr.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let members = match evidence.get("members").and_then(|arr| arr.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return false,
    };

    let rp_hash = Sha256::digest(rp_id.as_bytes());

    for m in members {
        let k = match m.get("approver_public_key").and_then(|x| x.as_str()) {
            Some(s) => s,
            None => return false,
        };

        let entry = match profile.get("approvers").and_then(|obj| obj.get(k)) {
            Some(e) => e,
            None => return false,
        };

        if !active_directory_entry(entry, verification_time) {
            return false;
        }

        let entry_pub = entry.get("public_key").and_then(|x| x.as_str());
        if entry_pub != Some(k) {
            return false;
        }

        let signoff = match m.get("signoff") {
            Some(s) => s,
            None => return false,
        };

        let context = match signoff.get("context") {
            Some(c) => c,
            None => return false,
        };

        let approver_id = entry.get("approver_id").and_then(|x| x.as_str());
        let signed_approver = context.get("approver").and_then(|x| x.as_str());
        if approver_id != signed_approver {
            return false;
        }

        let roles = match entry.get("roles").and_then(|arr| arr.as_array()) {
            Some(arr) => arr,
            None => return false,
        };
        let m_role = match m.get("role").and_then(|x| x.as_str()) {
            Some(r) => r,
            None => return false,
        };
        if !roles.iter().any(|x| x.as_str() == Some(m_role)) {
            return false;
        }

        if context.get("policy").and_then(|x| x.as_str()) != Some(context_policy) {
            return false;
        }

        let webauthn = match signoff.get("webauthn") {
            Some(w) => w,
            None => return false,
        };

        let auth_data_b64 = match webauthn.get("authenticator_data").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return false,
        };
        let auth_data = match decode_base64url(auth_data_b64) {
            Ok(b) => b,
            Err(_) => return false,
        };
        if auth_data.len() < 37 || auth_data[..32] != rp_hash[..] {
            return false;
        }

        let origin = webauthn_origin(webauthn);
        if !allowed_origins.contains(&origin) {
            return false;
        }

        if !fresh_at(context, verification_time, max_age_sec) {
            return false;
        }
    }

    if !crate::suites::quorum::verify_quorum(evidence) {
        return false;
    }

    let action_hash = match evidence.get("action_hash").and_then(|x| x.as_str()) {
        Some(h) => h,
        None => return false,
    };

    if norm_digest(action_hash) != norm_digest(chain_digest) {
        return false;
    }

    true
}

fn fresh_registry_snapshot(profile: &Value, verification_time: Option<&str>) -> bool {
    let checked = match profile.get("registry_checked_at").and_then(|v| v.as_str()).and_then(|s| crate::suites::time_attestation::parse_instant_ms(s)) {
        Some(ms) => ms,
        None => return false,
    };
    let max_age = match profile.get("max_registry_age_sec").and_then(|v| v.as_u64()) {
        Some(sec) => sec,
        None => return false,
    };
    let t = match verification_time {
        Some(s) => match crate::suites::time_attestation::parse_instant_ms(s) {
            Some(ms) => ms,
            None => return false,
        },
        None => 1776000000000.0,
    };
    let diff = t - checked;
    diff >= 0.0 && diff <= (max_age as f64) * 1000.0
}

fn active_directory_entry(entry: &Value, verification_time: Option<&str>) -> bool {
    if !entry.is_object() {
        return false;
    }
    if entry.get("status").and_then(|v| v.as_str()) != Some("active") {
        return false;
    }
    if !entry.get("revoked_at").map(|v| v.is_null()).unwrap_or(true) {
        return false;
    }
    let t = match verification_time {
        Some(s) => match crate::suites::time_attestation::parse_instant_ms(s) {
            Some(ms) => ms,
            None => return false,
        },
        None => 1776000000000.0,
    };
    let from = match entry.get("valid_from").and_then(|v| v.as_str()).and_then(|s| crate::suites::time_attestation::parse_instant_ms(s)) {
        Some(ms) => ms,
        None => return false,
    };
    let to = match entry.get("valid_to").and_then(|v| v.as_str()).and_then(|s| crate::suites::time_attestation::parse_instant_ms(s)) {
        Some(ms) => ms,
        None => return false,
    };
    t >= from && t <= to
}

fn fresh_at(context: &Value, verification_time: Option<&str>, max_age_sec: u64) -> bool {
    let issued_at = match context.get("issued_at").and_then(|v| v.as_str()).and_then(|s| crate::suites::time_attestation::parse_instant_ms(s)) {
        Some(ms) => ms,
        None => return false,
    };
    let t = match verification_time {
        Some(s) => match crate::suites::time_attestation::parse_instant_ms(s) {
            Some(ms) => ms,
            None => return false,
        },
        None => 1776000000000.0,
    };
    let diff = t - issued_at;
    diff >= 0.0 && diff <= (max_age_sec as f64) * 1000.0
}

fn webauthn_origin(webauthn: &Value) -> String {
    let cdj_b64 = match webauthn.get("client_data_json").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return String::new(),
    };
    let cdj_bytes = match decode_base64url(cdj_b64) {
        Ok(b) => b,
        Err(_) => return String::new(),
    };
    let cdj: Value = match serde_json::from_slice(&cdj_bytes) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    cdj.get("origin").and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn decode_base64url(s: &str) -> Result<Vec<u8>, ()> {
    use base64::{Engine as _, engine::general_purpose::{URL_SAFE_NO_PAD, URL_SAFE}};
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if let Ok(bytes) = URL_SAFE_NO_PAD.decode(&cleaned) {
        Ok(bytes)
    } else if let Ok(bytes) = URL_SAFE.decode(&cleaned) {
        Ok(bytes)
    } else {
        Err(())
    }
}

fn tokenize(expr: &str) -> Result<Vec<String>, ()> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch == '(' || ch == ')' {
            tokens.push(ch.to_string());
            i += 1;
        } else if (ch == '&' && chars.get(i + 1) == Some(&'&')) || (ch == '|' && chars.get(i + 1) == Some(&'|')) {
            tokens.push(format!("{}{}", ch, chars[i + 1]));
            i += 2;
        } else if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            let mut j = i;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_' || chars[j] == '-') {
                j += 1;
            }
            let ident: String = chars[i..j].iter().collect();
            tokens.push(ident);
            i = j;
        } else {
            return Err(());
        }
    }
    Ok(tokens)
}

fn eval_requirement(expr: &str, satisfied: &HashSet<String>) -> Result<bool, ()> {
    let tokens = tokenize(expr)?;
    let mut idx = 0;
    let res = parse_expr(&tokens, &mut idx, satisfied, 0)?;
    if idx == tokens.len() {
        Ok(res)
    } else {
        Err(())
    }
}

fn parse_expr(tokens: &[String], idx: &mut usize, satisfied: &HashSet<String>, depth: usize) -> Result<bool, ()> {
    if depth > 10 {
        return Err(());
    }
    let mut v = parse_term(tokens, idx, satisfied, depth)?;
    while *idx < tokens.len() {
        let op = &tokens[*idx];
        if op == "AND" || op == "&&" || op == "OR" || op == "||" {
            *idx += 1;
            let r = parse_term(tokens, idx, satisfied, depth)?;
            if op == "AND" || op == "&&" {
                v = v && r;
            } else {
                v = v || r;
            }
        } else {
            break;
        }
    }
    Ok(v)
}

fn parse_term(tokens: &[String], idx: &mut usize, satisfied: &HashSet<String>, depth: usize) -> Result<bool, ()> {
    if *idx >= tokens.len() {
        return Err(());
    }
    let t = &tokens[*idx];
    if t == "(" {
        *idx += 1;
        let v = parse_expr(tokens, idx, satisfied, depth + 1)?;
        if *idx >= tokens.len() || tokens[*idx] != ")" {
            return Err(());
        }
        *idx += 1;
        Ok(v)
    } else {
        if t == ")" || t == "AND" || t == "&&" || t == "OR" || t == "||" {
            return Err(());
        }
        let val = satisfied.contains(t);
        *idx += 1;
        Ok(val)
    }
}
