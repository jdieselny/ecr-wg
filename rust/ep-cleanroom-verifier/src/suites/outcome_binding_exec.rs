
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Value};

use crate::crypto::{self, sha256_hex};
use crate::suites::trust_receipt::verify_trust_receipt;

const ATTESTATION_VERSION: &str = "EP-OUTCOME-ATTESTATION-v1";
const BINDING_VERSION: &str = "EP-OUTCOME-BINDING-v1";
const RESULT_VERSION: &str = "EP-OUTCOME-BINDING-RESULT-v1";
const ATTESTATION_DOMAIN: &[u8] = b"EP-OUTCOME-ATTESTATION-v1\0";

pub fn sha256_jcs(val: &Value) -> String {
    let jcs = crate::canonical::canonicalize(val).unwrap_or_default();
    format!("sha256:{}", sha256_hex(jcs.as_bytes()))
}

/// Exact decimal order, matching `compareDecimalStrings`.
///
/// Equal-length text order is not numeric order: "10.0" and "2.00" have the
/// same length, and "10.0" < "2.00" as text. Float comparison is not exact
/// either. None means one side is not a canonical decimal string.
pub fn compare_decimal(a: &str, b: &str) -> Option<i32> {
    let (a_neg, a_int, a_frac) = split_decimal(a)?;
    let (b_neg, b_int, b_frac) = split_decimal(b)?;
    if a_neg != b_neg {
        return Some(if a_neg { -1 } else { 1 });
    }
    let mag = if a_int.len() != b_int.len() {
        if a_int.len() < b_int.len() { -1 } else { 1 }
    } else if a_int != b_int {
        if a_int < b_int { -1 } else { 1 }
    } else {
        cmp_frac(a_frac, b_frac)
    };
    Some(if a_neg { -mag } else { mag })
}

/// Canonical decimal: optional sign, no leading zeros, optional fraction.
/// Trailing fraction zeros are ignored. "-0" and "0" are the same value.
fn split_decimal(s: &str) -> Option<(bool, &str, &str)> {
    if s.is_empty() {
        return None;
    }
    let (mut neg, rest) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    if rest.is_empty() {
        return None;
    }
    let (int_part, frac_raw) = match rest.split_once('.') {
        Some((int_part, frac)) => {
            if frac.is_empty() || !frac.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            (int_part, frac)
        }
        None => (rest, ""),
    };
    if int_part.is_empty() || !int_part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if int_part.len() > 1 && int_part.as_bytes()[0] == b'0' {
        return None;
    }
    let frac = trim_trailing_zeros(frac_raw);
    if int_part == "0" && frac.is_empty() {
        neg = false;
    }
    Some((neg, int_part, frac))
}

fn trim_trailing_zeros(s: &str) -> &str {
    let end = s.bytes().rposition(|b| b != b'0').map(|i| i + 1).unwrap_or(0);
    &s[..end]
}

fn cmp_frac(a: &str, b: &str) -> i32 {
    let len = a.len().max(b.len());
    for i in 0..len {
        let ac = a.as_bytes().get(i).copied().unwrap_or(b'0');
        let bc = b.as_bytes().get(i).copied().unwrap_or(b'0');
        if ac != bc {
            return if ac < bc { -1 } else { 1 };
        }
    }
    0
}

pub fn evaluate_entry(entry: &Value, matches: &[&Value]) -> (String, Option<String>) {
    let p = entry.get("predicate").unwrap();
    let op = p.get("op").and_then(|x| x.as_str()).unwrap_or("");
    let effect_type = entry.get("effect_type").and_then(|x| x.as_str()).unwrap_or("");
    let target = entry.get("target").and_then(|x| x.as_str()).unwrap_or("");
    let at = format!("{} on {}", effect_type, target);
    if !["eq", "set_eq", "count_lte", "lte", "gte", "range", "absent"].contains(&op) { return ("incomparable".to_string(), Some("not a known op".to_string())); }

    if op == "absent" {
        if matches.is_empty() {
            return ("in_bounds".to_string(), None);
        } else {
            return ("divergent".to_string(), Some(format!("predicted absent for {}, observed {} effect(s)", at, matches.len())));
        }
    }

    if op == "count_lte" {
        let p_val = p.get("value").and_then(|x| x.as_str()).unwrap_or("0");
        let count = matches.len().to_string();
        return match compare_decimal(&count, p_val) {
            Some(cmp) if cmp <= 0 => ("in_bounds".to_string(), None),
            Some(_) => ("divergent".to_string(), Some(format!("predicted count <= {} for {}, observed {}", p_val, at, count))),
            None => ("incomparable".to_string(), Some(format!("predicted count value \"{}\" for {} is not a decimal string", p_val, at))),
        };
    }

    if matches.is_empty() {
        return ("incomparable".to_string(), Some(format!("no observed effect for {}", at)));
    }
    if matches.len() > 1 {
        return ("incomparable".to_string(), Some(format!("ambiguous: {} observed effects match {}", matches.len(), at)));
    }

    let obs = matches[0];

    if op == "set_eq" {
        let p_vals = p.get("values").and_then(|x| x.as_array());
        let o_vals = obs.get("values").and_then(|x| x.as_array());
        if let (Some(pv), Some(ov)) = (p_vals, o_vals) {
            let mut ok = true;
            if pv.len() != ov.len() { ok = false; }
            for p_v in pv { if !ov.contains(p_v) { ok = false; } }
            for o_v in ov { if !pv.contains(o_v) { ok = false; } }
            if ok { return ("in_bounds".to_string(), None); }
        }
        return ("divergent".to_string(), Some(format!("predicted set_eq {}", at)));
    }

    let o_val = match obs.get("value") {
        Some(v) if v.is_number() => return ("incomparable".to_string(), Some(format!("observed value for {} is a number; values MUST be strings (canonicalization malleability)", at))),
        Some(v) if v.is_string() => v.as_str().unwrap(),
        _ => return ("incomparable".to_string(), Some(format!("observed effect for {} has no string value", at))),
    };

    if op == "eq" {
        let p_val_v = p.get("value"); if let Some(v) = p_val_v { if v.is_number() { return ("incomparable".to_string(), Some("predicted value MUST be strings".to_string())); } } let p_val = p_val_v.and_then(|x| x.as_str()).unwrap_or("");
        if o_val == p_val {
            return ("in_bounds".to_string(), None);
        } else {
            return ("divergent".to_string(), Some(format!("predicted eq \"{}\" for {}, observed \"{}\"", p_val, at, o_val)));
        }
    }

    if op == "lte" {
        let p_val_v = p.get("value"); if let Some(v) = p_val_v { if v.is_number() { return ("incomparable".to_string(), Some("predicted value MUST be strings".to_string())); } } let p_val = p_val_v.and_then(|x| x.as_str()).unwrap_or("");
        if split_decimal(o_val).is_none() {
            return ("incomparable".to_string(), Some(format!("observed value \"{}\" for {} is not a decimal string", o_val, at)));
        }
        return match compare_decimal(o_val, p_val) {
            Some(cmp) if cmp <= 0 => ("in_bounds".to_string(), None),
            Some(_) => ("divergent".to_string(), Some(format!("predicted <= {} for {}, observed {}", p_val, at, o_val))),
            None => ("incomparable".to_string(), Some(format!("predicted value \"{}\" for {} is not a decimal string", p_val, at))),
        };
    }

    if op == "gte" {
        let p_val_v = p.get("value"); if let Some(v) = p_val_v { if v.is_number() { return ("incomparable".to_string(), Some("predicted value MUST be strings".to_string())); } } let p_val = p_val_v.and_then(|x| x.as_str()).unwrap_or("");
        if split_decimal(o_val).is_none() {
            return ("incomparable".to_string(), Some(format!("observed value \"{}\" for {} is not a decimal string", o_val, at)));
        }
        return match compare_decimal(o_val, p_val) {
            Some(cmp) if cmp >= 0 => ("in_bounds".to_string(), None),
            Some(_) => ("divergent".to_string(), Some(format!("predicted >= {} for {}, observed {}", p_val, at, o_val))),
            None => ("incomparable".to_string(), Some(format!("predicted value \"{}\" for {} is not a decimal string", p_val, at))),
        };
    }

    if op == "range" {
        let p_min = p.get("min").and_then(|x| x.as_str()).unwrap_or("");
        let p_max = p.get("max").and_then(|x| x.as_str()).unwrap_or("");
        if split_decimal(o_val).is_none() {
            return ("incomparable".to_string(), Some(format!("observed value \"{}\" for {} is not a decimal string", o_val, at)));
        }
        match compare_decimal(o_val, p_min) {
            Some(cmp) if cmp < 0 => return ("divergent".to_string(), Some(format!("predicted min {} for {}, observed {}", p_min, at, o_val))),
            Some(_) => {}
            None => return ("incomparable".to_string(), Some(format!("predicted value \"{}\" for {} is not a decimal string", p_min, at))),
        }
        match compare_decimal(o_val, p_max) {
            Some(cmp) if cmp > 0 => return ("divergent".to_string(), Some(format!("predicted max {} for {}, observed {}", p_max, at, o_val))),
            Some(_) => return ("in_bounds".to_string(), None),
            None => return ("incomparable".to_string(), Some(format!("predicted value \"{}\" for {} is not a decimal string", p_max, at))),
        }
    }

    ("incomparable".to_string(), None)
}

pub fn evaluate_predicted_effects(predicted: &Value, observed: &Value) -> Value {
    if !observed.is_null() && !observed.is_array() { return json!({"outcome": "incomparable", "reasons": ["not an array"]}); }
    if !predicted.is_null() && !predicted.is_array() { return json!({"outcome": "incomparable", "reasons": ["not an array"]}); }
    if observed.is_null() && !predicted.is_null() { return json!({"outcome": "incomparable", "reasons": ["not an array"]}); }
    let empty_vec_pred = vec![];
    let pred_arr = predicted.as_array().unwrap_or(&empty_vec_pred);
    let empty_vec = vec![];
    let obs_arr = observed.as_array().unwrap_or(&empty_vec);

    for obs in obs_arr { if let Some(obj) = obs.as_object() { for k in obj.keys() { if k != "effect_type" && k != "target" && k != "value" && k != "values" { return json!({"outcome": "incomparable", "reasons": ["unknown member"]}); } } } }
    for pred in pred_arr { if let Some(obj) = pred.as_object() { for k in obj.keys() { if k != "effect_type" && k != "target" && k != "predicate" { return json!({"outcome": "incomparable", "reasons": ["unknown member"]}); } } } }
    let mut results = vec![];
    let mut reasons = vec![];

    for entry in pred_arr {
        let p_type = entry.get("effect_type").and_then(|x| x.as_str()).unwrap_or("");
        let p_target = entry.get("target").and_then(|x| x.as_str()).unwrap_or("");
        
        let matching_obs: Vec<&Value> = obs_arr.iter().filter(|o| {
            o.get("effect_type").and_then(|x| x.as_str()).unwrap_or("") == p_type &&
            o.get("target").and_then(|x| x.as_str()).unwrap_or("") == p_target
        }).collect();

        let (outcome, reason) = evaluate_entry(entry, &matching_obs);
        let op = entry.get("predicate").and_then(|p| p.get("op")).and_then(|x| x.as_str()).unwrap_or("");
        
        results.push(json!({
            "effect_type": p_type,
            "target": p_target,
            "op": op,
            "outcome": outcome,
            "reason": if let Some(r) = &reason { json!(r) } else { Value::Null }
        }));

        if let Some(r) = reason {
            reasons.push(r);
        }
    }

    let outcome = if results.iter().any(|r| r["outcome"].as_str() == Some("divergent")) {
        "divergent"
    } else if results.iter().any(|r| r["outcome"].as_str() == Some("incomparable")) {
        "incomparable"
    } else {
        "in_bounds"
    };

    {
    let mut final_reasons = vec![];
    for r in &results {
        if r["outcome"].as_str() == Some(outcome) {
            if !r["reason"].is_null() {
                final_reasons.push(r["reason"].clone());
            }
        }
    }
    json!({"outcome": outcome, "results": results, "reasons": final_reasons})
    }
}

pub fn run_exec(root: &Value) -> Vec<(String, Value)> {
    let common = root.get("common").unwrap_or(&Value::Null);
    let receipt = common.get("receipt").unwrap_or(&Value::Null);
    let receipt_options = common.get("receipt_options").unwrap_or(&Value::Null);
    let now_str = common.get("now").and_then(|v| v.as_str());

    let mut mapped_opts = json!({});
    if let Some(ak) = receipt_options.get("approverKeys") {
        mapped_opts["approver_keys"] = ak.clone();
    }
    if let Some(lk) = receipt_options.get("logPublicKey") {
        mapped_opts["log_public_key"] = lk.clone();
    }
    let verify_opts = crate::suites::trust_receipt::VerifyOpts {
        now: now_str
            .and_then(crate::suites::time_attestation::parse_instant_ms)
            .map(|ms| ms as i64),
        allow_legacy_merkle: false,
    };
    let receipt_verified = verify_trust_receipt(receipt, &mapped_opts, &verify_opts);
    let receipt_digest = sha256_jcs(receipt);

    let mut results = Vec::new();
    let vectors = root.get("vectors").and_then(|v| v.as_array());
    let Some(vectors) = vectors else {
        return results;
    };
    for vector in vectors {
        let id = vector
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let attestation = vector.get("attestation").unwrap_or(&Value::Null);
        let executor_keys = vector
            .get("executor_keys")
            .or_else(|| common.get("executor_keys"))
            .unwrap_or(&Value::Null);
        let policy = vector.get("policy_predicted_effects");
        results.push((
            id,
            bind_one(
                receipt,
                attestation,
                executor_keys,
                policy,
                now_str,
                receipt_verified,
                &receipt_digest,
            ),
        ));
    }
    results
}

struct Checks {
    receipt_verified: bool,
    signed_predictions: bool,
    receipt_bound: bool,
    receipt_digest_bound: bool,
    action_bound: bool,
    consumption_bound: bool,
    attestation_verified: bool,
}

impl Checks {
    fn fresh() -> Self {
        Self {
            receipt_verified: false,
            signed_predictions: false,
            receipt_bound: false,
            receipt_digest_bound: false,
            action_bound: false,
            consumption_bound: false,
            attestation_verified: false,
        }
    }

    fn all(&self) -> bool {
        self.receipt_verified
            && self.signed_predictions
            && self.receipt_bound
            && self.receipt_digest_bound
            && self.action_bound
            && self.consumption_bound
            && self.attestation_verified
    }

    fn json(&self) -> Value {
        json!({
            "receipt_verified": self.receipt_verified,
            "signed_predictions": self.signed_predictions,
            "receipt_bound": self.receipt_bound,
            "receipt_digest_bound": self.receipt_digest_bound,
            "action_bound": self.action_bound,
            "consumption_bound": self.consumption_bound,
            "attestation_verified": self.attestation_verified
        })
    }
}

fn bind_one(
    receipt: &Value,
    attestation: &Value,
    executor_keys: &Value,
    policy: Option<&Value>,
    now_str: Option<&str>,
    receipt_verified: bool,
    receipt_digest: &str,
) -> Value {
    let mut checks = Checks::fresh();
    checks.receipt_verified = receipt_verified;
    let mut errors: Vec<String> = Vec::new();

    if !checks.receipt_verified {
        return finish(
            &checks,
            push(errors, "receipt_verification_failed"),
            receipt,
            attestation,
            policy,
            receipt_digest,
        );
    }

    let action = receipt.get("action").unwrap_or(&Value::Null);
    let predicted = action.get("predicted_effects").unwrap_or(&Value::Null);
    let claimed_prediction = action
        .get("predicted_effects_digest")
        .and_then(|v| v.as_str());
    checks.signed_predictions = predicted.is_array()
        && normalize_digest(claimed_prediction) == Some(sha256_jcs(predicted));
    if !checks.signed_predictions {
        return finish(
            &checks,
            push(errors, "signed_predictions_missing_or_mismatched"),
            receipt,
            attestation,
            policy,
            receipt_digest,
        );
    }

    if let Some(policy_value) = policy {
        if !policy_value.is_array() {
            return finish(
                &checks,
                push(errors, "policy_predictions_present_but_not_array"),
                receipt,
                attestation,
                policy,
                receipt_digest,
            );
        }
    }

    let attestation_errors = verify_attestation(attestation, executor_keys, now_str);
    checks.attestation_verified = attestation_errors.is_empty();
    if !checks.attestation_verified {
        errors.extend(attestation_errors);
        return finish(
            &checks,
            push(errors, "outcome_attestation_verification_failed"),
            receipt,
            attestation,
            policy,
            receipt_digest,
        );
    }

    let receipt_id = receipt.get("receipt_id").and_then(|v| v.as_str());
    let attested_receipt_id = attestation.get("receipt_id").and_then(|v| v.as_str());
    checks.receipt_bound = receipt_id.is_some() && receipt_id == attested_receipt_id;

    checks.receipt_digest_bound = normalize_digest(
        attestation.get("receipt_digest").and_then(|v| v.as_str()),
    ) == Some(receipt_digest.to_string());

    checks.action_bound = normalize_digest(receipt.get("action_hash").and_then(|v| v.as_str()))
        == normalize_digest(attestation.get("action_hash").and_then(|v| v.as_str()))
        && normalize_digest(receipt.get("action_hash").and_then(|v| v.as_str())).is_some();

    let consumption_nonce = receipt
        .get("consumption")
        .and_then(|c| c.get("nonce"))
        .and_then(|v| v.as_str());
    let attested_nonce = attestation.get("consumption_nonce").and_then(|v| v.as_str());
    checks.consumption_bound =
        consumption_nonce.is_some() && consumption_nonce == attested_nonce;

    if !checks.receipt_bound {
        errors.push("receipt_id_mismatch".to_string());
    }
    if !checks.receipt_digest_bound {
        errors.push("receipt_digest_mismatch".to_string());
    }
    if !checks.action_bound {
        errors.push("action_hash_mismatch".to_string());
    }
    if !checks.consumption_bound {
        errors.push("consumption_nonce_mismatch".to_string());
    }
    if !checks.receipt_bound
        || !checks.receipt_digest_bound
        || !checks.action_bound
        || !checks.consumption_bound
    {
        return finish(
            &checks,
            push(errors, "attestation_not_bound_to_verified_receipt"),
            receipt,
            attestation,
            policy,
            receipt_digest,
        );
    }

    let observed = attestation.get("observed_effects").unwrap_or(&Value::Null);
    let signed_eval = with_source(
        evaluate_predicted_effects(predicted, observed),
        "signed_receipt",
    );
    let policy_eval = policy.filter(|value| value.is_array()).map(|value| {
        with_source(
            evaluate_predicted_effects(value, observed),
            "relying_party_policy",
        )
    });
    let outcome_binding = combine(&signed_eval, policy_eval.as_ref());
    let mut result_errors = errors;
    if let Some(reasons) = outcome_binding.get("reasons").and_then(|v| v.as_array()) {
        for reason in reasons {
            if let Some(text) = reason.as_str() {
                result_errors.push(text.to_string());
            }
        }
    }
    let valid = checks.all() && outcome_binding.get("outcome").and_then(|v| v.as_str()) == Some("in_bounds");
    emit(
        &checks,
        &result_errors,
        &outcome_binding,
        valid,
        receipt,
        attestation,
        policy,
        receipt_digest,
    )
}

fn verify_attestation(
    attestation: &Value,
    executor_keys: &Value,
    now_str: Option<&str>,
) -> Vec<String> {
    let mut errors = Vec::new();
    if !attestation_well_formed(attestation) {
        errors.push("malformed_outcome_attestation".to_string());
        return errors;
    }
    let proof = attestation.get("proof").and_then(|v| v.as_object()).unwrap();
    let observed = attestation.get("observed_effects").unwrap();
    let claimed_obs = attestation
        .get("observed_effects_digest")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if sha256_jcs(observed) != claimed_obs {
        errors.push("observed_effects_digest_mismatch".to_string());
    }

    let presented = proof.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
    let presented_kid = proof.get("key_id").and_then(|v| v.as_str()).unwrap_or("");
    let derived = executor_key_id(presented);
    let executor_id = attestation.get("executor_id").and_then(|v| v.as_str()).unwrap_or("");
    let pin = executor_keys.get(executor_id);
    let pin_key = pin
        .and_then(|entry| entry.get("public_key"))
        .and_then(|v| v.as_str());
    let pin_kid_ok = match pin.and_then(|entry| entry.get("key_id")) {
        None => true,
        Some(value) => value.as_str() == derived.as_deref(),
    };
    let pinned = derived.as_deref() == Some(presented_kid)
        && pin_key == Some(presented)
        && pin_kid_ok
        && !presented.is_empty();
    if !pinned {
        errors.push("executor_key_not_pinned".to_string());
    }

    let signature_ok = if pinned {
        attestation_signing_bytes(attestation)
            .and_then(|bytes| {
                crypto::verify_ed25519(
                    presented,
                    &bytes,
                    proof
                        .get("signature_b64u")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                )
                .ok()
            })
            .unwrap_or(false)
    } else {
        false
    };
    if !signature_ok {
        errors.push("executor_signature_invalid".to_string());
    }

    let executed_at = attestation
        .get("executed_at")
        .and_then(|v| v.as_str())
        .and_then(crate::suites::time_attestation::parse_instant_ms);
    let now = now_str.and_then(crate::suites::time_attestation::parse_instant_ms);
    if executed_at.is_none() || now.is_none() || executed_at.unwrap() > now.unwrap() {
        errors.push("execution_time_invalid_or_future".to_string());
    }
    errors
}

fn attestation_well_formed(attestation: &Value) -> bool {
    let Some(proof) = attestation.get("proof").and_then(|v| v.as_object()) else {
        return false;
    };
    attestation.get("@version").and_then(|v| v.as_str()) == Some(ATTESTATION_VERSION)
        && nonempty(attestation, "receipt_id")
        && normalize_digest(attestation.get("receipt_digest").and_then(|v| v.as_str())).is_some()
        && normalize_digest(attestation.get("action_hash").and_then(|v| v.as_str())).is_some()
        && nonempty(attestation, "consumption_nonce")
        && nonempty(attestation, "execution_id")
        && nonempty(attestation, "executor_id")
        && exact_digest(attestation.get("observed_effects_digest").and_then(|v| v.as_str()))
        && attestation.get("observed_effects").is_some_and(|v| v.is_array())
        && proof.get("algorithm").and_then(|v| v.as_str()) == Some("Ed25519")
        && proof
            .get("key_id")
            .and_then(|v| v.as_str())
            .is_some_and(|kid| {
                kid.starts_with("ep:executor-key:sha256:") && exact_digest(Some(&kid["ep:executor-key:".len()..]))
            })
        && nonempty_in(proof, "public_key")
        && nonempty_in(proof, "signature_b64u")
}

fn attestation_signing_bytes(attestation: &Value) -> Option<Vec<u8>> {
    let object = attestation.as_object()?;
    let mut unsigned = serde_json::Map::new();
    for (key, value) in object {
        if key != "proof" {
            unsigned.insert(key.clone(), value.clone());
        }
    }
    let canonical = crate::canonical::canonicalize(&Value::Object(unsigned)).ok()?;
    let mut bytes = ATTESTATION_DOMAIN.to_vec();
    bytes.extend(canonical.as_bytes());
    Some(bytes)
}

fn executor_key_id(public_key_b64u: &str) -> Option<String> {
    let der = URL_SAFE_NO_PAD.decode(public_key_b64u).ok()?;
    Some(format!(
        "ep:executor-key:sha256:{}",
        crypto::sha256_hex(&der)
    ))
}

fn with_source(mut evaluation: Value, source: &str) -> Value {
    if let Some(object) = evaluation.as_object_mut() {
        object.insert("source".to_string(), json!(source));
    }
    evaluation
}

fn combine(signed: &Value, policy: Option<&Value>) -> Value {
    let mut evaluations = vec![signed.clone()];
    if let Some(policy) = policy {
        evaluations.push(policy.clone());
    }
    let outcome = if evaluations.iter().any(|item| item.get("outcome").and_then(|v| v.as_str()) == Some("divergent")) {
        "divergent"
    } else if evaluations.iter().any(|item| item.get("outcome").and_then(|v| v.as_str()) == Some("incomparable")) {
        "incomparable"
    } else {
        "in_bounds"
    };
    let mut reasons = Vec::new();
    for item in &evaluations {
        let source = item.get("source").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(item_reasons) = item.get("reasons").and_then(|v| v.as_array()) {
            for reason in item_reasons {
                if let Some(text) = reason.as_str() {
                    reasons.push(format!("{source}: {text}"));
                }
            }
        }
    }
    json!({
        "@version": BINDING_VERSION,
        "outcome": outcome,
        "evaluations": evaluations,
        "reasons": reasons
    })
}

fn finish(
    checks: &Checks,
    errors: Vec<String>,
    receipt: &Value,
    attestation: &Value,
    policy: Option<&Value>,
    receipt_digest: &str,
) -> Value {
    let outcome_binding = json!({
        "@version": BINDING_VERSION,
        "outcome": "incomparable",
        "evaluations": [],
        "reasons": errors.clone()
    });
    emit(
        checks,
        &errors,
        &outcome_binding,
        false,
        receipt,
        attestation,
        policy,
        receipt_digest,
    )
}

fn emit(
    checks: &Checks,
    errors: &[String],
    outcome_binding: &Value,
    valid: bool,
    receipt: &Value,
    attestation: &Value,
    policy: Option<&Value>,
    receipt_digest: &str,
) -> Value {
    let verdict = outcome_binding
        .get("outcome")
        .and_then(|v| v.as_str())
        .unwrap_or("incomparable");
    let core = json!({
        "@version": RESULT_VERSION,
        "input_commitments": input_commitments(receipt, attestation, policy),
        "exact_commitments": exact_commitments(receipt, attestation),
        "valid": valid,
        "verdict": verdict,
        "checks": checks.json(),
        "errors": errors,
        "outcome_binding": outcome_binding
    });
    let reasons = outcome_binding
        .get("reasons")
        .cloned()
        .unwrap_or_else(|| json!([]));
    json!({
        "outcome": verdict,
        "valid": valid,
        "checks": checks.json(),
        "reasons": reasons,
        "receipt_digest": receipt_digest,
        "attestation_digest": sha256_jcs(attestation),
        "result_digest": sha256_jcs(&core)
    })
}

fn input_commitments(receipt: &Value, attestation: &Value, policy: Option<&Value>) -> Value {
    let predicted = receipt
        .get("action")
        .and_then(|action| action.get("predicted_effects"));
    let commitment = receipt
        .get("action")
        .and_then(|action| action.get("predicted_effects_digest"))
        .and_then(|v| v.as_str());
    json!({
        "receipt_digest": sha256_jcs(receipt),
        "attestation_digest": sha256_jcs(attestation),
        "signed_predictions_digest": predicted.map(sha256_jcs).map(Value::String).unwrap_or(Value::Null),
        "signed_predictions_commitment": normalize_digest(commitment).map(Value::String).unwrap_or(Value::Null),
        "policy_predictions_present": policy.is_some(),
        "policy_predictions_digest": policy.map(sha256_jcs).map(Value::String).unwrap_or(Value::Null)
    })
}

fn exact_commitments(receipt: &Value, attestation: &Value) -> Value {
    let consumption = receipt.get("consumption");
    let proof = attestation.get("proof");
    json!({
        "receipt_id": string_or_null(receipt.get("receipt_id")),
        "attested_receipt_id": string_or_null(attestation.get("receipt_id")),
        "receipt_digest": sha256_jcs(receipt),
        "attested_receipt_digest": normalize_digest(attestation.get("receipt_digest").and_then(|v| v.as_str())).map(Value::String).unwrap_or(Value::Null),
        "action_hash": normalize_digest(receipt.get("action_hash").and_then(|v| v.as_str())).map(Value::String).unwrap_or(Value::Null),
        "attested_action_hash": normalize_digest(attestation.get("action_hash").and_then(|v| v.as_str())).map(Value::String).unwrap_or(Value::Null),
        "consumption_nonce": string_or_null(consumption.and_then(|c| c.get("nonce"))),
        "attested_consumption_nonce": string_or_null(attestation.get("consumption_nonce")),
        "execution_id": string_or_null(attestation.get("execution_id")),
        "executor_id": string_or_null(attestation.get("executor_id")),
        "executor_key_id": string_or_null(proof.and_then(|p| p.get("key_id"))),
        "observed_effects_digest": normalize_digest(attestation.get("observed_effects_digest").and_then(|v| v.as_str())).map(Value::String).unwrap_or(Value::Null)
    })
}

fn string_or_null(value: Option<&Value>) -> Value {
    match value.and_then(|v| v.as_str()) {
        Some(text) => Value::String(text.to_string()),
        None => Value::Null,
    }
}

fn normalize_digest(value: Option<&str>) -> Option<String> {
    let text = value?;
    if !exact_digest(Some(text)) && !exact_digest_ignore_case(text) {
        return None;
    }
    Some(format!("sha256:{}", text[7..].to_ascii_lowercase()))
}

fn exact_digest(value: Option<&str>) -> bool {
    let Some(text) = value else {
        return false;
    };
    exact_digest_ignore_case(text) && text[7..].chars().all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn exact_digest_ignore_case(text: &str) -> bool {
    let Some(hex) = text.strip_prefix("sha256:").or_else(|| text.strip_prefix("SHA256:")) else {
        return false;
    };
    hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn nonempty(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .is_some_and(|text| !text.is_empty())
}

fn nonempty_in(object: &serde_json::Map<String, Value>, key: &str) -> bool {
    object
        .get(key)
        .and_then(|v| v.as_str())
        .is_some_and(|text| !text.is_empty())
}

fn push(mut errors: Vec<String>, reason: &str) -> Vec<String> {
    errors.push(reason.to_string());
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_order_is_numeric_for_equal_length_strings() {
        assert_eq!(compare_decimal("10.0", "2.00"), Some(1));
        assert_eq!(compare_decimal("9.00", "10.0"), Some(-1));
        assert_eq!(compare_decimal("9.00", "10.00"), Some(-1));
        assert_eq!(compare_decimal("10.0", "2.000"), Some(1));
        assert_eq!(compare_decimal("1.50", "1.5"), Some(0));
        assert_eq!(compare_decimal("1.10", "1.1"), Some(0));
        assert_eq!(compare_decimal("-10.0", "-2.00"), Some(-1));
        assert_eq!(compare_decimal("-2.00", "-10.0"), Some(1));
        assert_eq!(compare_decimal("-0", "0.00"), Some(0));
        assert_eq!(compare_decimal("01", "1"), None);
        assert_eq!(compare_decimal("10.", "10"), None);
    }

    #[test]
    fn policy_limits_use_numeric_order() {
        let over = json!({
            "effect_type": "payment",
            "target": "acct:vendor-9",
            "predicate": {"op": "lte", "value": "2.00"}
        });
        let observed_over = json!({
            "effect_type": "payment",
            "target": "acct:vendor-9",
            "value": "10.0"
        });
        let (outcome, reason) = evaluate_entry(&over, &[&observed_over]);
        assert_eq!(outcome, "divergent");
        assert_eq!(
            reason.as_deref(),
            Some("predicted <= 2.00 for payment on acct:vendor-9, observed 10.0")
        );

        let under = json!({
            "effect_type": "payment",
            "target": "acct:vendor-9",
            "predicate": {"op": "lte", "value": "10.0"}
        });
        let observed_under = json!({
            "effect_type": "payment",
            "target": "acct:vendor-9",
            "value": "9.00"
        });
        let (outcome, reason) = evaluate_entry(&under, &[&observed_under]);
        assert_eq!(outcome, "in_bounds");
        assert_eq!(reason, None);
    }
}
