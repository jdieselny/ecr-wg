use crate::crypto;
use crate::suites::{vector_id, vectors_array};
use serde_json::{json, Value};

const REVOCATION_VERSION: &str = "EP-REVOCATION-v1";
const TARGET_TYPES: &[&str] = &["receipt", "commit", "delegation"];

fn sha256_jcs(val: &Value) -> String {
    if val.is_null() { return "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(); }
    let jcs = crate::canonical::canonicalize(val).unwrap_or_default();
    format!("sha256:{}", crate::crypto::sha256_hex(jcs.as_bytes()))
}

pub fn run(vectors: &Value) -> Vec<(String, Value)> {
    let mut results = Vec::new();
    let arr = vectors_array(vectors);
    for vec_val in arr.iter() {
        let id = vector_id(vec_val);
        let statement = vec_val.get("revocation").unwrap_or(&Value::Null);
        let target = vec_val.get("target").unwrap_or(&Value::Null);
        let revoker_keys = vec_val.get("revoker_keys").and_then(|v| v.as_object());
        let now = vec_val.get("now").and_then(|v| v.as_str()).or(Some("2026-06-20T12:00:00.000Z"));
        let max_age_seconds = vec_val.get("max_age_seconds").and_then(|v| v.as_i64());
        let res = verify_revocation(statement, target, revoker_keys, now, max_age_seconds);
        results.push((id, res));
    }
    results
}

fn finalize_result(
    valid: bool,
    checks: Value,
    reasons: Vec<&str>,
    target_digest: String,
    revocation_digest: String,
) -> Value {
    let mut res = json!({
        "valid": valid,
        "checks": checks,
        "reasons": reasons,
        "target_digest": target_digest,
        "revocation_digest": revocation_digest
    });
    let res_digest = sha256_jcs(&res);
    res.as_object_mut().unwrap().insert("result_digest".to_string(), json!(res_digest));
    res
}

fn revocation_signed_payload(stmt: &serde_json::Map<String, Value>) -> Result<String, ()> {
    let obj = json!({
        "@version": REVOCATION_VERSION,
        "action_hash": stmt.get("action_hash"),
        "reason": stmt.get("reason"),
        "revoked_at": stmt.get("revoked_at"),
        "revoker_id": stmt.get("revoker_id"),
        "target_id": stmt.get("target_id"),
        "target_type": stmt.get("target_type"),
    });
    canonicalize(&obj).map_err(|_| ())
}

fn hex_of(h: &str) -> String {
    h.strip_prefix("sha256:")
        .unwrap_or(h)
        .to_ascii_lowercase()
}

fn parse_instant_ms(s: &str) -> Option<f64> {
    if !is_rfc3339_offset(s) {
        return None;
    }
    parse_rfc3339_ms(s)
}

fn is_rfc3339_offset(value: &str) -> bool {
    let s = value.trim();
    if !s.contains('T') {
        return false;
    }
    if s.ends_with('Z') || s.ends_with('z') {
        return true;
    }
    if let Some(idx) = s.rfind('+') {
        let tz = &s[idx..];
        return tz.len() >= 6 && tz.as_bytes().get(3) == Some(&b':');
    }
    if let Some(after_t) = s.split('T').nth(1) {
        if let Some(idx) = after_t.rfind('-') {
            let tz = &after_t[idx..];
            return tz.len() >= 6 && tz.as_bytes().get(3) == Some(&b':');
        }
    }
    false
}

fn parse_rfc3339_ms(s: &str) -> Option<f64> {
    let s = s.trim();
    let (date, rest) = s.split_once('T')?;
    let (time_core, offset) = split_time_offset(rest)?;

    let mut ymd = date.split('-');
    let year: i32 = ymd.next()?.parse().ok()?;
    let month: u32 = ymd.next()?.parse().ok()?;
    let day: u32 = ymd.next()?.parse().ok()?;

    let mut hms_parts = time_core.split(':');
    let hour: u32 = hms_parts.next()?.parse().ok()?;
    let minute: u32 = hms_parts.next()?.parse().ok()?;
    let sec_str = hms_parts.next()?;
    let (second, ms) = if let Some((sec, frac)) = sec_str.split_once('.') {
        let sec: u32 = sec.parse().ok()?;
        let mut frac_str: String = frac.chars().take(3).collect();
        while frac_str.len() < 3 {
            frac_str.push('0');
        }
        let ms: u64 = frac_str.parse().ok()?;
        (sec, ms)
    } else {
        (sec_str.parse().ok()?, 0)
    };

    let sign: i64 = if offset.starts_with('-') { -1 } else { 1 };
    let off = offset.strip_prefix('+').or_else(|| offset.strip_prefix('-'))?;
    let mut offp = off.split(':');
    let oh: i64 = offp.next()?.parse().ok()?;
    let om: i64 = offp.next().unwrap_or("0").parse().ok()?;

    let days = days_from_civil(year, month, day) as i64;
    let local_ms =
        days as f64 * 86_400_000.0 + hour as f64 * 3_600_000.0 + minute as f64 * 60_000.0
            + second as f64 * 1000.0 + ms as f64;
    let offset_ms = sign as f64 * (oh as f64 * 3_600_000.0 + om as f64 * 60_000.0);
    Some(local_ms - offset_ms)
}

fn split_time_offset(rest: &str) -> Option<(&str, &str)> {
    if let Some(idx) = rest.rfind('Z') {
        return Some((&rest[..idx], "+00:00"));
    }
    if let Some(idx) = rest.rfind('+') {
        return Some((&rest[..idx], &rest[idx..]));
    }
    if rest.len() > 6 {
        if let Some(idx) = rest.rfind('-') {
            if rest[idx..].contains(':') {
                return Some((&rest[..idx], &rest[idx..]));
            }
        }
    }
    None
}

fn days_from_civil(y: i32, m: u32, d: u32) -> i32 {
    let m = m as i32;
    let y = if m <= 2 { y - 1 } else { y };
    let m = if m <= 2 { m + 12 } else { m };
    let era = if y >= 0 { y / 400 } else { -1 - (-1 - y) / 400 };
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + d as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn canonicalize(v: &Value) -> Result<String, ()> {
    crate::canonical::canonicalize(v).map_err(|_| ())
}

pub fn verify_revocation(
    statement: &Value,
    target: &Value,
    revoker_keys: Option<&serde_json::Map<String, Value>>,
    now: Option<&str>,
    max_age_seconds: Option<i64>,
) -> Value {
    let mut valid = true;
    let mut checks = json!({
        "version": false,
        "structure": false,
        "target_bound": true,
        "revoker_key_pinned": false,
        "revoker_key_bound": true,
        "revoked_at_present": false,
        "effective_at_or_before_T": true,
        "revoker_signature_valid": false,
        "signature_binds_statement": true
    });
    let mut reasons = Vec::new();

    let target_digest = sha256_jcs(target);
    let revocation_digest = sha256_jcs(statement);

    let stmt = match statement.as_object() {
        Some(o) => {
            let mut ok = true;
            let allowed_top = ["@version", "target_type", "target_id", "action_hash", "revoker_id", "revoked_at", "reason", "proof"];
            for k in o.keys() {
                if !allowed_top.contains(&k.as_str()) {
                    ok = false;
                }
            }
            if let Some(proof) = o.get("proof").and_then(|v| v.as_object()) {
                let allowed_proof = ["algorithm", "revoker_key_id", "public_key", "signature_b64u"];
                for k in proof.keys() {
                    if !allowed_proof.contains(&k.as_str()) {
                        ok = false;
                    }
                }
            } else {
                ok = false;
            }
            if ok {
                checks["structure"] = json!(true);
            } else {
                reasons.push("structure");
                valid = false;
            }
            o
        },
        None => {
            reasons.push("structure");
            valid = false;
            return finalize_result(valid, checks, reasons, target_digest, revocation_digest);
        }
    };

    let tgt = match target.as_object() {
        Some(o) => o,
        None => {
            checks["structure"] = json!(false);
            if !reasons.contains(&"structure") {
                reasons.push("structure");
            }
            valid = false;
            return finalize_result(valid, checks, reasons, target_digest, revocation_digest);
        }
    };

    let ver = stmt.get("@version").and_then(|v| v.as_str()).unwrap_or("");
    if ver == REVOCATION_VERSION {
        checks["version"] = json!(true);
    } else {
        reasons.push("version");
        valid = false;
    }

    let mut target_bound = true;
    if stmt.get("target_type") != tgt.get("target_type") || stmt.get("target_type").is_none() { target_bound = false; }
    if stmt.get("target_id") != tgt.get("target_id") || stmt.get("target_id").is_none() { target_bound = false; }

    let stmt_hash_raw = stmt.get("action_hash").and_then(|v| v.as_str()).unwrap_or("");
    if !stmt_hash_raw.starts_with("sha256:") { target_bound = false; }
    
    let tgt_id_raw = tgt.get("target_id").and_then(|v| v.as_str()).unwrap_or("");
    if tgt_id_raw == "" { target_bound = false; }

    let stmt_hash = hex_of(stmt_hash_raw);
    let tgt_hash = hex_of(tgt.get("action_hash").and_then(|v| v.as_str()).unwrap_or(""));
    if stmt_hash != tgt_hash { target_bound = false; }

    if !target_bound {
        checks["target_bound"] = json!(false);
        reasons.push("target_bound");
        valid = false;
    }

    let revoker_id = stmt.get("revoker_id").and_then(|v| v.as_str()).unwrap_or("");
    let mut is_pinned = false;
    let mut is_bound = true;
    let mut pinned_pk_str = "";
    
    let proof = stmt.get("proof").and_then(|v| v.as_object());
    let pres_pk = proof.and_then(|p| p.get("public_key")).and_then(|v| v.as_str()).unwrap_or("");
    let pres_kid = proof.and_then(|p| p.get("revoker_key_id")).and_then(|v| v.as_str()).unwrap_or("");

    if let Some(entry) = revoker_keys.and_then(|k| k.get(revoker_id)) {
        pinned_pk_str = entry.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
        let pinned_kid = entry.get("key_id").and_then(|v| v.as_str());

        if pres_pk != "" && pres_pk == pinned_pk_str {
            is_pinned = true;
        }

        if pres_pk == "" {
            is_bound = false;
        } else if let Some(kid) = pinned_kid {
            if pres_kid != kid {
                is_bound = false;
            }
        }
    }

    checks["revoker_key_pinned"] = json!(is_pinned);
    if !is_pinned {
        reasons.push("revoker_key_pinned");
        valid = false;
    }

    checks["revoker_key_bound"] = json!(is_bound);
    if !is_bound {
        reasons.push("revoker_key_bound");
        valid = false;
    }

    let revoked_str_opt = stmt.get("revoked_at").and_then(|v| v.as_str());
    let mut revoked_ms_opt = revoked_str_opt.and_then(parse_instant_ms);
    if let Some(s) = revoked_str_opt {
        if let Some(frac) = s.split('.').nth(1) {
            let frac_digits = frac.chars().take_while(|c| c.is_digit(10)).count();
            if frac_digits > 9 {
                revoked_ms_opt = None; // Reject more than 9 fractional digits
            }
        }
    }

    if revoked_ms_opt.is_some() {
        checks["revoked_at_present"] = json!(true);
    } else {
        checks["revoked_at_present"] = json!(false);
        reasons.push("revoked_at_present");
        checks["effective_at_or_before_T"] = json!(false);
        reasons.push("effective_at_or_before_T");
        valid = false;
    }

    if let Some(revoked_ms) = revoked_ms_opt {
        if let Some(now_ms) = now.and_then(parse_instant_ms) {
            if revoked_ms > now_ms {
                checks["effective_at_or_before_T"] = json!(false);
                if !reasons.contains(&"effective_at_or_before_T") {
                    reasons.push("effective_at_or_before_T");
                }
                valid = false;
            }

            if let Some(max_age) = max_age_seconds {
                if (now_ms - revoked_ms) / 1000.0 > max_age as f64 {
                    let is_terminal = tgt.get("target_type").and_then(|v| v.as_str()) == Some("receipt") || stmt.get("target_type").and_then(|v| v.as_str()) == Some("receipt");
                    if !is_terminal {
                        checks["effective_at_or_before_T"] = json!(false);
                        if !reasons.contains(&"effective_at_or_before_T") {
                            reasons.push("effective_at_or_before_T");
                        }
                        valid = false;
                    }
                }
            }
        }
    }

    let payload_res = revocation_signed_payload(stmt);
    if payload_res.is_err() {
        checks["signature_binds_statement"] = json!(false);
        reasons.push("signature_binds_statement");
        valid = false;
    }

    let key_to_check = if pinned_pk_str != "" { pinned_pk_str } else { pres_pk };
    if proof.and_then(|p| p.get("algorithm")).and_then(|v| v.as_str()) != Some("Ed25519") {
        checks["revoker_signature_valid"] = json!(false);
        reasons.push("revoker_signature_valid");
        valid = false;
    } else if key_to_check != "" {
        if let Ok(payload) = payload_res {
            let sig = proof.and_then(|p| p.get("signature_b64u")).and_then(|v| v.as_str()).unwrap_or("");
            if crypto::verify_ed25519(key_to_check, payload.as_bytes(), sig).unwrap_or(false) {
                checks["revoker_signature_valid"] = json!(true);
            } else {
                reasons.push("revoker_signature_valid");
                checks["signature_binds_statement"] = json!(false);
                reasons.push("signature_binds_statement");
                valid = false;
            }
        } else {
            reasons.push("revoker_signature_valid");
            valid = false;
        }
    } else {
        reasons.push("revoker_signature_valid");
        valid = false;
    }

    finalize_result(valid, checks, reasons, target_digest, revocation_digest)
}
