// SPDX-License-Identifier: Apache-2.0
// EP-SIGNOFF-v1 — Class A WebAuthn device signoff verification (9 vectors).

use crate::verify_webauthn_signoff;
use crate::suites::{vector_id, vectors_array};
use serde_json::Value;

pub fn run(vectors: &Value) -> Vec<(String, bool)> {
    let mut results = Vec::new();

    for v in vectors_array(vectors) {
        let id = vector_id(v);
        let rp_id = v.get("rp_id").and_then(|x| x.as_str());
        let approver_pk = v.get("approver_public_key").and_then(|x| x.as_str());
        let signoff = v.get("signoff").unwrap_or(&Value::Null);
        let valid = match (rp_id, approver_pk) {
            (Some(rp_id), Some(approver_pk)) => {
                if let Ok(so_str) = serde_json::to_string(signoff) {
                    verify_webauthn_signoff(&so_str, approver_pk, Some(rp_id)).unwrap_or(false)
                } else {
                    false
                }
            }
            _ => false,
        };
        results.push((id, valid));
    }
    results
}