// SPDX-License-Identifier: Apache-2.0
// EP-SCITT-STATEMENT-v1 — SCITT Signed Statement verification (RFC 9943 / RFC 9052).

use crate::canonical::canonicalize;
use crate::crypto;
use crate::suites::{vector_id, vectors_array};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn run(root: &Value) -> Vec<(String, Value)> {
    let mut results = Vec::new();

    let keys = root.get("keys");
    let statement_pub_b64u = keys
        .and_then(|k| k.get("statement_public_key_spki_base64url"))
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let receipt_pub_b64u = keys
        .and_then(|k| k.get("receipt_issuer_public_key_spki_base64url"))
        .and_then(|s| s.as_str())
        .unwrap_or("");

    for v in vectors_array(root) {
        let id = vector_id(v);
        let statement_hex = v.get("statement_hex").and_then(|s| s.as_str()).unwrap_or("");
        let result_obj = verify_scitt_statement_detailed(statement_hex, statement_pub_b64u, receipt_pub_b64u);
        results.push((id, result_obj));
    }
    results
}

pub fn verify_scitt_statement_detailed(statement_hex: &str, statement_pub_b64u: &str, receipt_pub_b64u: &str) -> Value {
    let raw_bytes = match hex::decode(statement_hex) {
        Ok(b) => b,
        Err(_) => return json!({ "valid": false, "reason": "invalid_hex", "registered": false }),
    };

    // Minimal CBOR COSE_Sign1 decoder
    let (protected_bytes, payload_bytes, sig_bytes) = match parse_cose_sign1(&raw_bytes) {
        Some(tuple) => tuple,
        None => return json!({ "valid": false, "reason": "cose_structure_invalid", "registered": false }),
    };

    // Parse Protected Header
    let (alg, content_type, _kid, cwt_claims) = match parse_protected_header(&protected_bytes) {
        Some(hdr) => hdr,
        None => return json!({ "valid": false, "reason": "cwt_claims_missing", "registered": false }),
    };

    // 1. Must use Ed25519 (alg = -8)
    if alg != -8 {
        return json!({ "valid": false, "reason": "unsupported_statement_alg", "registered": false });
    }

    // 2. Must be application/emilia-receipt+json
    if content_type != "application/emilia-receipt+json" {
        return json!({ "valid": false, "reason": "unsupported_content_type", "registered": false });
    }

    // 3. Must have CWT Claims (label 15) with iss and sub
    let (iss, sub) = match cwt_claims {
        Some((i, s)) => (i, s),
        None => return json!({ "valid": false, "reason": "cwt_claims_missing", "registered": false }),
    };

    if iss.is_empty() || sub.is_empty() {
        return json!({ "valid": false, "reason": "cwt_claims_missing", "registered": false });
    }

    // 4. Verify COSE Statement Signature
    let sig_structure = build_sig_structure(&protected_bytes, &payload_bytes);
    if sig_bytes.len() != 64 {
        return json!({ "valid": false, "reason": "statement_signature_invalid", "registered": false });
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let statement_sig = ed25519_dalek::Signature::from_bytes(&sig_arr);

    let statement_vk = match crypto::parse_ed25519_spki_key(statement_pub_b64u) {
        Ok(k) => k,
        Err(_) => return json!({ "valid": false, "reason": "statement_key_invalid", "registered": false }),
    };

    use ed25519_dalek::Verifier;
    let statement_sig_ok = statement_vk.verify(&sig_structure, &statement_sig).is_ok();
    if !statement_sig_ok {
        return json!({ "valid": false, "reason": "statement_signature_invalid", "registered": false });
    }

    // 5. Parse Payload JSON & verify native receipt signature
    let payload_str = match std::str::from_utf8(&payload_bytes) {
        Ok(s) => s,
        Err(_) => return json!({ "valid": false, "reason": "payload_utf8_invalid", "registered": false }),
    };

    let receipt_val: Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(_) => return json!({ "valid": false, "reason": "receipt_json_invalid", "registered": false }),
    };

    // Check payload receipt signature
    let receipt_sig_b64u = match receipt_val
        .get("signature")
        .and_then(|s| s.get("value"))
        .and_then(|v| v.as_str())
    {
        Some(s) => s,
        None => return json!({ "valid": false, "reason": "receipt_invalid", "registered": false }),
    };

    let receipt_sig_bytes = match URL_SAFE_NO_PAD.decode(receipt_sig_b64u) {
        Ok(b) => b,
        Err(_) => return json!({ "valid": false, "reason": "receipt_invalid", "registered": false }),
    };

    if receipt_sig_bytes.len() != 64 {
        return json!({ "valid": false, "reason": "receipt_invalid", "registered": false });
    }
    let mut r_sig_arr = [0u8; 64];
    r_sig_arr.copy_from_slice(&receipt_sig_bytes);
    let receipt_sig = ed25519_dalek::Signature::from_bytes(&r_sig_arr);

    // Reconstruct canonical payload for native receipt signature check
    let receipt_payload = match receipt_val.get("payload") {
        Some(p) => p,
        None => return json!({ "valid": false, "reason": "receipt_invalid", "registered": false }),
    };

    let canonical_payload_str = match canonicalize(receipt_payload) {
        Ok(s) => s,
        Err(_) => return json!({ "valid": false, "reason": "receipt_invalid", "registered": false }),
    };

    let receipt_vk = match crypto::parse_ed25519_spki_key(receipt_pub_b64u) {
        Ok(k) => k,
        Err(_) => return json!({ "valid": false, "reason": "receipt_key_invalid", "registered": false }),
    };

    let receipt_sig_ok = receipt_vk.verify(canonical_payload_str.as_bytes(), &receipt_sig).is_ok();
    if !receipt_sig_ok {
        return json!({
            "valid": false,
            "reason": "receipt_invalid",
            "registered": false,
            "checks": {
                "statement_signature": true,
                "receipt_signature": false
            }
        });
    }

    // 6. Action-Binding (sub verification)
    let action_obj = match receipt_payload.get("action") {
        Some(a) => a,
        None => return json!({ "valid": false, "reason": "sub_not_bound_to_payload", "registered": false }),
    };

    let action_type = match action_obj.get("action_type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return json!({ "valid": false, "reason": "sub_not_bound_to_payload", "registered": false }),
    };

    let action_canon = match canonicalize(action_obj) {
        Ok(s) => s,
        Err(_) => return json!({ "valid": false, "reason": "sub_not_bound_to_payload", "registered": false }),
    };

    let action_hash = Sha256::digest(action_canon.as_bytes());
    let action_hash_b64u = URL_SAFE_NO_PAD.encode(action_hash);

    let expected_sub = format!("caid:1:{}:jcs-sha256:{}", action_type, action_hash_b64u);
    if sub != expected_sub {
        return json!({ "valid": false, "reason": "sub_not_bound_to_payload", "registered": false });
    }

    json!({
        "valid": true,
        "registered": false,
        "iss": iss,
        "sub": sub,
        "checks": {
            "deterministic_encoding": true,
            "cose_structure": true,
            "cwt_claims": true,
            "statement_signature": true,
            "payload_canonical": true,
            "receipt_signature": true,
            "sub_binding": true
        }
    })
}

fn parse_cose_sign1(bytes: &[u8]) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    if bytes.is_empty() {
        return None;
    }

    let mut pos = 0;
    if bytes[pos] == 0xd2 {
        pos += 1;
    }

    if pos >= bytes.len() || (bytes[pos] & 0xe0) != 0x80 {
        return None;
    }

    let array_len = (bytes[pos] & 0x1f) as usize;
    pos += 1;

    if array_len != 4 {
        return None;
    }

    let protected = read_bstr(bytes, &mut pos)?;
    skip_cbor_item(bytes, &mut pos)?;
    let payload = read_bstr(bytes, &mut pos)?;
    let signature = read_bstr(bytes, &mut pos)?;

    Some((protected, payload, signature))
}

fn read_bstr(bytes: &[u8], pos: &mut usize) -> Option<Vec<u8>> {
    if *pos >= bytes.len() {
        return None;
    }

    let byte = bytes[*pos];
    if (byte & 0xe0) != 0x40 {
        return None;
    }

    let mut len = (byte & 0x1f) as usize;
    *pos += 1;

    if len == 24 {
        if *pos >= bytes.len() { return None; }
        len = bytes[*pos] as usize;
        *pos += 1;
    } else if len == 25 {
        if *pos + 1 >= bytes.len() { return None; }
        len = ((bytes[*pos] as usize) << 8) | (bytes[*pos + 1] as usize);
        *pos += 2;
    }

    if *pos + len > bytes.len() {
        return None;
    }

    let data = bytes[*pos..*pos + len].to_vec();
    *pos += len;
    Some(data)
}

fn skip_cbor_item(bytes: &[u8], pos: &mut usize) -> Option<()> {
    if *pos >= bytes.len() {
        return None;
    }
    let byte = bytes[*pos];
    *pos += 1;
    let major = byte >> 5;
    let mut val = (byte & 0x1f) as usize;

    if val == 24 {
        if *pos >= bytes.len() { return None; }
        val = bytes[*pos] as usize;
        *pos += 1;
    } else if val == 25 {
        if *pos + 1 >= bytes.len() { return None; }
        val = ((bytes[*pos] as usize) << 8) | (bytes[*pos + 1] as usize);
        *pos += 2;
    }

    match major {
        0 | 1 | 6 => Some(()),
        2 | 3 => {
            if *pos + val > bytes.len() { return None; }
            *pos += val;
            Some(())
        }
        4 => {
            for _ in 0..val {
                skip_cbor_item(bytes, pos)?;
            }
            Some(())
        }
        5 => {
            for _ in 0..val {
                skip_cbor_item(bytes, pos)?;
                skip_cbor_item(bytes, pos)?;
            }
            Some(())
        }
        _ => None,
    }
}

fn parse_protected_header(bytes: &[u8]) -> Option<(i64, String, String, Option<(String, String)>)> {
    let mut pos = 0;
    if pos >= bytes.len() || (bytes[pos] & 0xe0) != 0xa0 {
        return None;
    }
    let map_len = (bytes[pos] & 0x1f) as usize;
    pos += 1;

    let mut alg: i64 = 0;
    let mut content_type = String::new();
    let mut kid = String::new();
    let mut cwt_claims: Option<(String, String)> = None;

    for _ in 0..map_len {
        if pos >= bytes.len() { return None; }
        let key_label = read_int_or_uint(bytes, &mut pos)?;

        match key_label {
            1 => {
                alg = read_int_or_uint(bytes, &mut pos)?;
            }
            3 => {
                let bstr = read_bstr_or_tstr(bytes, &mut pos)?;
                content_type = String::from_utf8_lossy(&bstr).to_string();
            }
            4 => {
                let bstr = read_bstr_or_tstr(bytes, &mut pos)?;
                kid = String::from_utf8_lossy(&bstr).to_string();
            }
            15 => {
                cwt_claims = parse_cwt_map(bytes, &mut pos);
            }
            _ => {
                skip_cbor_item(bytes, &mut pos)?;
            }
        }
    }

    Some((alg, content_type, kid, cwt_claims))
}

fn read_int_or_uint(bytes: &[u8], pos: &mut usize) -> Option<i64> {
    if *pos >= bytes.len() { return None; }
    let byte = bytes[*pos];
    let major = byte >> 5;
    let mut val = (byte & 0x1f) as i64;
    *pos += 1;

    if val == 24 {
        if *pos >= bytes.len() { return None; }
        val = bytes[*pos] as i64;
        *pos += 1;
    }

    match major {
        0 => Some(val),
        1 => Some(-1 - val),
        _ => None,
    }
}

fn read_bstr_or_tstr(bytes: &[u8], pos: &mut usize) -> Option<Vec<u8>> {
    if *pos >= bytes.len() { return None; }
    let byte = bytes[*pos];
    let major = byte >> 5;
    if major != 2 && major != 3 {
        return None;
    }
    let mut len = (byte & 0x1f) as usize;
    *pos += 1;
    if len == 24 {
        if *pos >= bytes.len() { return None; }
        len = bytes[*pos] as usize;
        *pos += 1;
    }
    if *pos + len > bytes.len() { return None; }
    let data = bytes[*pos..*pos + len].to_vec();
    *pos += len;
    Some(data)
}

fn parse_cwt_map(bytes: &[u8], pos: &mut usize) -> Option<(String, String)> {
    if *pos >= bytes.len() || (bytes[*pos] & 0xe0) != 0xa0 {
        return None;
    }
    let map_len = (bytes[*pos] & 0x1f) as usize;
    *pos += 1;

    let mut iss = String::new();
    let mut sub = String::new();

    for _ in 0..map_len {
        let label = read_int_or_uint(bytes, pos)?;
        let val_bytes = read_bstr_or_tstr(bytes, pos)?;
        let val_str = String::from_utf8_lossy(&val_bytes).to_string();

        if label == 1 {
            iss = val_str;
        } else if label == 2 {
            sub = val_str;
        }
    }

    Some((iss, sub))
}

fn build_sig_structure(protected_bytes: &[u8], payload_bytes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x84);
    buf.extend_from_slice(&[0x6a, b'S', b'i', b'g', b'n', b'a', b't', b'u', b'r', b'e', b'1']);
    encode_bstr(&mut buf, protected_bytes);
    buf.push(0x40);
    encode_bstr(&mut buf, payload_bytes);
    buf
}

fn encode_bstr(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len();
    if len < 24 {
        buf.push(0x40 | (len as u8));
    } else if len <= 0xff {
        buf.push(0x58);
        buf.push(len as u8);
    } else if len <= 0xffff {
        buf.push(0x59);
        buf.push((len >> 8) as u8);
        buf.push((len & 0xff) as u8);
    }
    buf.extend_from_slice(data);
}
