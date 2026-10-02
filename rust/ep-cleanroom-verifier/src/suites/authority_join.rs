// SPDX-License-Identifier: Apache-2.0
// EP-AUTHORITY-DOC-PROOF-JOIN-v1 over the execution companion.
//
// The catalogue file names a mutation. The v3 runner does not receive that
// name. It receives the proof, the document chain, and the relying-party
// pins, and it returns the computed typed result.

use crate::canonical::canonicalize;
use crate::crypto::{self, sha256_hex};
use crate::suites::{vector_id, vectors_array};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Map, Value};

const PROOF_DOMAIN: &[u8] = b"EP-AUTHORITY-PROOF-v1\0";
const DOC_DOMAIN: &[u8] = b"EP-AUTHORITY-DOC-v1\0";

pub fn run(root: &Value) -> Vec<(String, Value)> {
    let mut results = Vec::new();
    for vector in vectors_array(root) {
        let id = vector_id(vector);
        let proof = vector.get("proof").cloned().unwrap_or(Value::Null);
        let docs = vector.get("docs").cloned().unwrap_or(Value::Null);
        let opts = vector.get("opts").cloned().unwrap_or(Value::Null);
        results.push((id, verify_join(&proof, &docs, &opts)));
    }
    results
}

pub fn verify_join(proof: &Value, docs: &Value, opts: &Value) -> Value {
    let doc_list = docs.as_array().cloned().unwrap_or_default();
    let cores: Vec<Value> = doc_list.iter().map(document_core).collect();
    let core_digests: Vec<String> = cores.iter().map(tagged_jcs_digest).collect();
    let bootstrap = core_digests.first().cloned();
    let document_head = core_digests.last().cloned();

    let proof_body = unsigned_proof(proof);
    let proof_jcs = canonicalize(&proof_body).unwrap_or_default();
    let mut proof_bytes = PROOF_DOMAIN.to_vec();
    proof_bytes.extend_from_slice(proof_jcs.as_bytes());
    let proof_digest = format!("sha256:{}", sha256_hex(&proof_bytes));
    let proof_input_digest = tagged_jcs_digest(proof);
    let document_chain_digest = tagged_jcs_digest(docs);

    let chain_ok = document_chain_ok(&doc_list, &cores, &core_digests, opts);
    let continuity_ok = chain_ok && continuity_ok(&doc_list, &cores, &core_digests);
    let (anchor_present, anchor_match) = anchor_status(opts, bootstrap.as_deref(), document_head.as_deref());
    let anchor_ok = anchor_present && anchor_match;

    let proof_time = proof.get("issued_at").and_then(|v| v.as_str()).unwrap_or("");
    let effective = effective_index(&doc_list, proof_time);
    let org_ok = organization_ok(proof, &doc_list, opts);
    let (time_present, time_match) = time_anchor(proof_time, opts);
    let time_ok = time_present && time_match;
    let proof_head_ok = proof_document_ok(proof, &doc_list, &core_digests);
    let derived_kid = derived_issuer_kid(proof);
    let resolved = derived_kid
        .as_deref()
        .and_then(|kid| resolve_issuer(kid, &doc_list, effective, proof_time));
    let reg_issuer_ok = registry_issuer_ok(proof, opts, resolved, derived_kid.as_deref());
    let key_resolved = resolved.is_some();
    let key_usage = resolved
        .as_ref()
        .map(|key| has_usage(key, "authority_proof_issuer"))
        .unwrap_or(false);
    let sig_ok = proof_signature_ok(proof, &proof_bytes, &proof_digest);
    let (pins_present, head_ok, epoch_ok) = registry_pins(proof, opts);

    let mut reason: Option<&str> = None;
    if !chain_ok {
        reason = Some("authority_document_chain_invalid");
    } else if !continuity_ok {
        reason = Some("authority_document_continuity_break");
    } else if !anchor_present {
        reason = Some("authority_document_anchor_required");
    } else if !anchor_match {
        reason = Some("authority_document_anchor_mismatch");
    } else if !org_ok {
        reason = Some("authority_document_organization_mismatch");
    } else if !time_present {
        reason = Some("authority_proof_time_anchor_required");
    } else if !time_match {
        reason = Some("authority_proof_time_anchor_mismatch");
    } else if !proof_head_ok {
        reason = Some("authority_proof_document_head_mismatch");
    } else if !reg_issuer_ok {
        reason = Some("authority_registry_issuer_mismatch");
    } else if !key_resolved {
        reason = Some("authority_proof_key_unresolvable");
    } else if !key_usage {
        reason = Some("authority_proof_key_wrong_usage");
    } else if !sig_ok {
        reason = Some("proof_digest_mismatch");
    } else if !pins_present {
        reason = Some("registry_snapshot_pins_required");
    } else if !head_ok {
        reason = Some("registry_head_mismatch");
    } else if !epoch_ok {
        reason = Some("stale_registry");
    }

    let mut document_chain = chain_ok;
    let mut continuity = continuity_ok;
    let mut document_anchor = anchor_ok;
    let mut organization_binding = org_ok;
    let mut proof_document_binding = proof_head_ok;
    let mut registry_issuer_binding = reg_issuer_ok;
    let mut issuer_key_resolved = key_resolved;
    let mut issuer_key_usage = key_usage;
    let mut proof_time_anchor = time_ok;
    let mut registry_head = pins_present && head_ok;
    let mut epoch_fresh = pins_present && head_ok && epoch_ok;

    if !document_chain {
        continuity = false;
        document_anchor = false;
        organization_binding = false;
        proof_document_binding = false;
        registry_issuer_binding = false;
        issuer_key_resolved = false;
        issuer_key_usage = false;
        proof_time_anchor = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !continuity {
        document_anchor = false;
        organization_binding = false;
        proof_document_binding = false;
        registry_issuer_binding = false;
        issuer_key_resolved = false;
        issuer_key_usage = false;
        proof_time_anchor = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !document_anchor {
        organization_binding = false;
        proof_document_binding = false;
        registry_issuer_binding = false;
        issuer_key_resolved = false;
        issuer_key_usage = false;
        proof_time_anchor = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !organization_binding {
        proof_document_binding = false;
        registry_issuer_binding = false;
        issuer_key_resolved = false;
        issuer_key_usage = false;
        proof_time_anchor = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !proof_time_anchor {
        proof_document_binding = false;
        registry_issuer_binding = false;
        issuer_key_resolved = false;
        issuer_key_usage = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !proof_document_binding {
        registry_issuer_binding = false;
        issuer_key_resolved = false;
        issuer_key_usage = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !registry_issuer_binding {
        issuer_key_resolved = false;
        issuer_key_usage = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !issuer_key_resolved {
        issuer_key_usage = false;
        registry_head = false;
        epoch_fresh = false;
    } else if !issuer_key_usage {
        registry_head = false;
        epoch_fresh = false;
    } else if !sig_ok {
        registry_head = false;
        epoch_fresh = false;
    } else if !registry_head {
        epoch_fresh = false;
    }

    let verified = chain_ok && continuity_ok && sig_ok;
    let issuer_accepted = reason.is_none();
    let accepted = verified && issuer_accepted;

    let mut out = Map::new();
    out.insert("verified".into(), json!(verified));
    out.insert("issuer_accepted".into(), json!(issuer_accepted));
    out.insert("accepted".into(), json!(accepted));
    out.insert("authority_evaluated".into(), json!(false));
    out.insert("delegation_evaluated".into(), json!(false));
    out.insert(
        "checks".into(),
        json!({
            "document_chain": document_chain,
            "continuity": continuity,
            "document_anchor": document_anchor,
            "organization_binding": organization_binding,
            "proof_document_binding": proof_document_binding,
            "registry_issuer_binding": registry_issuer_binding,
            "issuer_key_resolved": issuer_key_resolved,
            "issuer_key_usage": issuer_key_usage,
            "proof_signature": sig_ok,
            "proof_time_anchor": proof_time_anchor,
            "registry_head": registry_head,
            "epoch_fresh": epoch_fresh
        }),
    );
    if let Some(reason) = reason {
        out.insert("reason".into(), json!(reason));
    }
    if chain_ok {
        if let Some(head) = document_head {
            out.insert("document_head".into(), json!(head));
        }
        if let Some(boot) = bootstrap {
            out.insert("bootstrap_digest".into(), json!(boot));
        }
    }
    let past_key = chain_ok
        && continuity_ok
        && anchor_ok
        && org_ok
        && time_ok
        && proof_head_ok
        && reg_issuer_ok
        && key_resolved
        && key_usage;
    if past_key {
        out.insert("proof_digest".into(), json!(proof_digest));
    }
    if accepted {
        if let Some(head) = proof
            .get("authority_document")
            .and_then(|d| d.get("head_digest"))
            .and_then(|v| v.as_str())
        {
            out.insert("proof_document_head".into(), json!(head));
        }
        if let Some(reg) = proof.get("registry_issuer_id").and_then(|v| v.as_str()) {
            out.insert("registry_issuer_id".into(), json!(reg));
        }
        if let Some(key) = resolved.as_ref() {
            if let Some(kid) = key.get("kid").and_then(|v| v.as_str()) {
                out.insert("key_id".into(), json!(kid));
            }
        }
    }
    out.insert("proof_input_digest".into(), json!(proof_input_digest));
    out.insert("document_chain_digest".into(), json!(document_chain_digest));
    let unsigned = Value::Object(out.clone());
    out.insert("result_digest".into(), json!(tagged_jcs_digest(&unsigned)));
    Value::Object(out)
}

fn unsigned_proof(proof: &Value) -> Value {
    let mut body = Map::new();
    if let Some(obj) = proof.as_object() {
        for (key, value) in obj {
            if key != "signature" {
                body.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(body)
}

fn document_core(doc: &Value) -> Value {
    let mut core = Map::new();
    if let Some(obj) = doc.as_object() {
        for (key, value) in obj {
            if key != "sig" && key != "continuity_sig" && key != "endorsements" {
                core.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(core)
}

fn tagged_jcs_digest(value: &Value) -> String {
    let jcs = canonicalize(value).unwrap_or_default();
    format!("sha256:{}", sha256_hex(jcs.as_bytes()))
}

fn document_chain_ok(docs: &[Value], cores: &[Value], digests: &[String], opts: &Value) -> bool {
    if docs.is_empty() || docs.len() != cores.len() {
        return false;
    }
    let expected_domain = opts
        .get("expectedOrganizationDomain")
        .and_then(|v| v.as_str());
    let mut previous_time: Option<&str> = None;
    for (index, doc) in docs.iter().enumerate() {
        if doc.get("@version").and_then(|v| v.as_str()) != Some("EP-AUTHORITY-DOC-v1") {
            return false;
        }
        let domain = doc
            .get("org")
            .and_then(|o| o.get("domain"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if let Some(expected) = expected_domain {
            if domain != expected {
                return false;
            }
        }
        if index > 0 {
            if let (Some(prev), Some(curr)) = (previous_time, doc.get("issued_at").and_then(|v| v.as_str())) {
                if curr <= prev {
                    return false;
                }
            } else {
                return false;
            }
        }
        previous_time = doc.get("issued_at").and_then(|v| v.as_str());
        let root = doc.get("root_key").and_then(|v| v.as_str()).unwrap_or("");
        let sig = doc.get("sig").and_then(|v| v.as_str()).unwrap_or("");
        if !signature_ok(root, sig, &cores[index], Some(digests[index].as_str())) {
            return false;
        }
    }
    true
}

fn continuity_ok(docs: &[Value], cores: &[Value], digests: &[String]) -> bool {
    if docs.is_empty() {
        return false;
    }
    if docs[0].get("seq").and_then(|v| v.as_i64()) != Some(0) {
        return false;
    }
    if !docs[0].get("prev_doc_digest").map(|v| v.is_null()).unwrap_or(false) {
        return false;
    }
    for index in 1..docs.len() {
        let prev_seq = docs[index - 1].get("seq").and_then(|v| v.as_i64());
        let seq = docs[index].get("seq").and_then(|v| v.as_i64());
        if prev_seq.and_then(|n| seq.map(|m| m == n + 1)) != Some(true) {
            return false;
        }
        if docs[index].get("prev_doc_digest").and_then(|v| v.as_str()) != Some(digests[index - 1].as_str()) {
            return false;
        }
        let at = docs[index].get("issued_at").and_then(|v| v.as_str()).unwrap_or("");
        let sig = docs[index].get("continuity_sig").and_then(|v| v.as_str()).unwrap_or("");
        let mut signed = false;
        if let Some(root) = docs[index - 1].get("root_key").and_then(|v| v.as_str()) {
            signed = signature_ok(root, sig, &cores[index], Some(digests[index].as_str()));
        }
        if !signed {
            if let Some(keys) = docs[index - 1].get("issuer_keys").and_then(|v| v.as_array()) {
                for key in keys {
                    if !rotation_key_current(key, at) {
                        continue;
                    }
                    if let Some(spki) = key.get("key").and_then(|v| v.as_str()) {
                        if signature_ok(spki, sig, &cores[index], Some(digests[index].as_str())) {
                            signed = true;
                            break;
                        }
                    }
                }
            }
        }
        if !signed {
            return false;
        }
    }
    true
}

fn rotation_key_current(key: &Value, at: &str) -> bool {
    if !has_usage(key, "authority_doc_rotation") {
        return false;
    }
    let from = key.get("valid_from").and_then(|v| v.as_str()).unwrap_or("");
    let to = key.get("valid_to").and_then(|v| v.as_str()).unwrap_or("");
    if at < from || ( !to.is_empty() && at > to) {
        return false;
    }
    if let Some(revoked) = key.get("revoked_at").and_then(|v| v.as_str()) {
        if !revoked.is_empty() && at >= revoked {
            return false;
        }
    }
    true
}

fn anchor_status(opts: &Value, bootstrap: Option<&str>, head: Option<&str>) -> (bool, bool) {
    let expected_head = opts.get("expectedDocumentHead").and_then(|v| v.as_str());
    let expected_boot = opts.get("expectedBootstrapDigest").and_then(|v| v.as_str());
    let present = expected_head.is_some() && expected_boot.is_some();
    let matched = present && expected_head == head && expected_boot == bootstrap;
    (present, matched)
}

fn organization_ok(proof: &Value, docs: &[Value], opts: &Value) -> bool {
    let expected = opts.get("expectedOrganizationId").and_then(|v| v.as_str());
    let Some(expected) = expected else {
        return false;
    };
    if proof.get("organization_id").and_then(|v| v.as_str()) != Some(expected) {
        return false;
    }
    docs.iter().all(|doc| {
        doc.get("org")
            .and_then(|o| o.get("id"))
            .and_then(|v| v.as_str())
            == Some(expected)
    })
}

fn time_anchor(proof_time: &str, opts: &Value) -> (bool, bool) {
    match opts.get("expectedProofIssuedAt").and_then(|v| v.as_str()) {
        Some(expected) => (true, proof_time == expected),
        None => (false, false),
    }
}

fn effective_index(docs: &[Value], at: &str) -> Option<usize> {
    if at.is_empty() {
        return None;
    }
    let mut found = None;
    for (index, doc) in docs.iter().enumerate() {
        if let Some(issued) = doc.get("issued_at").and_then(|v| v.as_str()) {
            if issued <= at {
                found = Some(index);
            }
        }
    }
    found
}

fn derived_issuer_kid(proof: &Value) -> Option<String> {
    let spki = proof
        .get("signature")
        .and_then(|s| s.get("public_key"))
        .and_then(|v| v.as_str())?;
    let der = URL_SAFE_NO_PAD.decode(spki).ok()?;
    Some(format!("ep:authority-issuer-key:sha256:{}", sha256_hex(&der)))
}

fn proof_document_ok(proof: &Value, docs: &[Value], digests: &[String]) -> bool {
    let anchor = proof.get("authority_document");
    let head = anchor.and_then(|d| d.get("head_digest")).and_then(|v| v.as_str());
    let seq = anchor.and_then(|d| d.get("head_seq")).and_then(|v| v.as_i64());
    let Some(head) = head else {
        return false;
    };
    docs.iter().enumerate().any(|(index, doc)| {
        digests.get(index).map(String::as_str) == Some(head)
            && doc.get("seq").and_then(|v| v.as_i64()) == seq
    })
}

fn resolve_issuer<'a>(kid: &str, docs: &'a [Value], effective: Option<usize>, at: &str) -> Option<&'a Value> {
    let index = effective?;
    let keys = docs.get(index)?.get("issuer_keys")?.as_array()?;
    let key = keys.iter().find(|key| key.get("kid").and_then(|v| v.as_str()) == Some(kid))?;
    let from = key.get("valid_from").and_then(|v| v.as_str()).unwrap_or("");
    let to = key.get("valid_to").and_then(|v| v.as_str()).unwrap_or("");
    if at < from || (!to.is_empty() && at > to) {
        return None;
    }
    if let Some(revoked) = key.get("revoked_at").and_then(|v| v.as_str()) {
        if !revoked.is_empty() && at >= revoked {
            return None;
        }
    }
    Some(key)
}

fn registry_issuer_ok(proof: &Value, opts: &Value, key: Option<&Value>, derived_kid: Option<&str>) -> bool {
    let expected = opts.get("expectedRegistryIssuerId").and_then(|v| v.as_str());
    let Some(expected) = expected else {
        return false;
    };
    if proof.get("registry_issuer_id").and_then(|v| v.as_str()) != Some(expected) {
        return false;
    }
    if let Some(key) = key {
        if key.get("registry_issuer_id").and_then(|v| v.as_str()) != Some(expected) {
            return false;
        }
        let claimed = proof
            .get("authority_document")
            .and_then(|d| d.get("issuer_kid"))
            .and_then(|v| v.as_str());
        if claimed != derived_kid {
            return false;
        }
    }
    true
}

fn registry_pins(proof: &Value, opts: &Value) -> (bool, bool, bool) {
    let expected_head = opts.get("expectRegistryHead").and_then(|v| v.as_str());
    let min_epoch = opts.get("expectMinEpoch").and_then(|v| v.as_i64());
    let present = expected_head.is_some() && min_epoch.is_some();
    let head_ok = expected_head.is_some()
        && proof.get("registry_head").and_then(|v| v.as_str()) == expected_head;
    let epoch = proof.get("registry_epoch").and_then(|v| v.as_i64()).unwrap_or(i64::MIN);
    let epoch_ok = min_epoch.map(|min| epoch >= min).unwrap_or(false);
    (present, head_ok, epoch_ok)
}

fn has_usage(key: &Value, usage: &str) -> bool {
    key.get("usages")
        .and_then(|v| v.as_array())
        .map(|usages| usages.iter().any(|item| item.as_str() == Some(usage)))
        .unwrap_or(false)
}

fn proof_signature_ok(proof: &Value, message: &[u8], computed_digest: &str) -> bool {
    let Some(sig) = proof.get("signature").and_then(|v| v.as_object()) else {
        return false;
    };
    if sig.get("algorithm").and_then(|v| v.as_str()) != Some("Ed25519") {
        return false;
    }
    if sig.get("proof_digest").and_then(|v| v.as_str()) != Some(computed_digest) {
        return false;
    }
    let public_key = sig.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
    let signature = sig.get("signature_b64u").and_then(|v| v.as_str()).unwrap_or("");
    crypto::verify_ed25519(public_key, message, signature).unwrap_or(false)
}

fn signature_ok(public_key: &str, signature: &str, core: &Value, digest: Option<&str>) -> bool {
    if public_key.is_empty() || signature.is_empty() {
        return false;
    }
    let jcs = canonicalize(core).unwrap_or_default();
    let mut domain = DOC_DOMAIN.to_vec();
    domain.extend_from_slice(jcs.as_bytes());
    let digest_bytes = digest.unwrap_or("").as_bytes();
    let raw_digest = hex::decode(digest.unwrap_or("").trim_start_matches("sha256:")).unwrap_or_default();
    crypto::verify_ed25519(public_key, jcs.as_bytes(), signature).unwrap_or(false)
        || crypto::verify_ed25519(public_key, &domain, signature).unwrap_or(false)
        || crypto::verify_ed25519(public_key, digest_bytes, signature).unwrap_or(false)
        || crypto::verify_ed25519(public_key, &raw_digest, signature).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::run;
    use serde_json::Value;
    use std::fs;

    #[test]
    fn authority_exec_companion_matches_typed_results() {
        let path = r"C:\Users\jkintzele\Documents\emilia-protocol\conformance\vectors\authority-document-proof-join.exec.v1.json";
        let raw = fs::read_to_string(path).expect("exec companion");
        let root: Value = serde_json::from_str(&raw).expect("json");
        let mut failed = Vec::new();
        for (id, got) in run(&root) {
            let expect = root
                .get("vectors")
                .and_then(|v| v.as_array())
                .and_then(|vecs| {
                    vecs.iter()
                        .find(|v| v.get("id").and_then(|x| x.as_str()) == Some(id.as_str()))
                })
                .and_then(|v| v.get("expect"))
                .cloned()
                .unwrap_or(Value::Null);
            if got != expect {
                failed.push(format!(
                    "{id}\n got={}\n exp={}",
                    serde_json::to_string(&got).unwrap_or_default(),
                    serde_json::to_string(&expect).unwrap_or_default()
                ));
            }
        }
        assert!(failed.is_empty(), "{}", failed.join("\n\n"));
    }
}
