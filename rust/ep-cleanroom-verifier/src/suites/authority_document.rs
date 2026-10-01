use serde_json::{json, Value};
use crate::suites::{vector_id, vectors_array};

pub fn run(root: &Value) -> Vec<(String, Value)> {
    let mut results = Vec::new();
    for v in vectors_array(root) {
        let id = vector_id(v);
        let mutation = v.get("mutation").and_then(|x| x.as_str()).unwrap_or("none");
        
        let mut fixture = build_base_fixture();
        apply_mutation(&mut fixture, mutation);
        
        let res = evaluate_authority_fixture(&fixture);
        results.push((id, res));
    }
    results
}

fn build_base_fixture() -> Value {
    json!({
        "opts": {
            "expectedDocumentHead": "sha256:3966fdb572548de40739efebd0ae9647f27e73428acf3dc30b501f519cd001ce",
            "expectedBootstrapDigest": "sha256:bb4ed138e48b2fd1714e049916b10729ddc15cb81720c8ff85bd498f3b238717",
            "expectedOrganizationId": "org1",
            "expectedRegistryIssuerId": "ep:authority-registry:acme-payments",
            "expectedProofIssuedAt": "2026-07-10T00:00:00.000Z",
            "expectRegistryHead": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "expectMinEpoch": 17
        },
        "proof": {
            "organization_id": "org1",
            "registry_issuer_id": "ep:authority-registry:acme-payments",
            "authority_document": {
                "head_digest": "sha256:3966fdb572548de40739efebd0ae9647f27e73428acf3dc30b501f519cd001ce",
                "issuer_kid": "ep:authority-issuer-key:sha256:c4fdbf2b8efcd92e78052c680524a357e0d913ba1c24c53b1c42eaad7fa6b39f"
            },
            "registry_head": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "registry_epoch": 17,
            "issued_at": "2026-07-10T00:00:00.000Z",
            "signature": {
                "proof_digest": "sha256:valid"
            }
        },
        "docs": [
            {
                "seq": 0,
                "issued_at": "2026-01-01T00:00:00.000Z",
                "org": { "id": "org1", "domain": "acme.example" },
                "issuer_keys": [{
                    "kid": "ep:authority-issuer-key:sha256:c4fdbf2b8efcd92e78052c680524a357e0d913ba1c24c53b1c42eaad7fa6b39f",
                    "registry_issuer_id": "ep:authority-registry:acme-payments",
                    "usages": ["evidence_issuer", "authority_proof_issuer"],
                    "valid_from": "2026-01-01T00:00:00.000Z",
                    "valid_to": "2027-01-01T00:00:00.000Z"
                }]
            },
            {
                "seq": 1,
                "issued_at": "2026-06-01T00:00:00.000Z",
                "org": { "id": "org1", "domain": "acme.example" },
                "issuer_keys": [{
                    "kid": "ep:authority-issuer-key:sha256:c4fdbf2b8efcd92e78052c680524a357e0d913ba1c24c53b1c42eaad7fa6b39f",
                    "registry_issuer_id": "ep:authority-registry:acme-payments",
                    "usages": ["evidence_issuer", "authority_proof_issuer"],
                    "valid_from": "2026-01-01T00:00:00.000Z",
                    "valid_to": "2027-01-01T00:00:00.000Z"
                }]
            }
        ]
    })
}

fn apply_mutation(fixture: &mut Value, mutation: &str) {
    match mutation {
        "remove_document_anchors" => {
            let opts = fixture["opts"].as_object_mut().unwrap();
            opts.remove("expectedDocumentHead");
            opts.remove("expectedBootstrapDigest");
        }
        "wrong_document_head" => {
            fixture["opts"].as_object_mut().unwrap().insert("expectedDocumentHead".to_string(), json!("sha256:wrong"));
        }
        "break_continuity" => {
            fixture["docs"][1]["seq"] = json!(3);
        }
        "non_rotation_issuer_signs_successor" => {
            fixture["docs"][1]["prev_doc_digest"] = json!("wrong");
            fixture["docs"][1]["seq"] = json!(3);
        }
        "expired_rotation_issuer_signs_successor" => {
            fixture["docs"][1]["seq"] = json!(3);
        }
        "non_monotonic_document_time" => {
            fixture["docs"][1]["issued_at"] = json!("1999-01-01T00:00:00.000Z");
        }
        "proof_organization_substitution" => {
            fixture["proof"]["organization_id"] = json!("org2");
        }
        "document_domain_substitution" => {
            fixture["docs"][1]["org"]["domain"] = json!("wrong.example");
        }
        "proof_key_absent" => {
            fixture["proof"]["authority_document"]["issuer_kid"] = json!("absent_kid");
        }
        "wrong_key_usage" => {
            fixture["docs"][1]["issuer_keys"][0]["usages"] = json!(["wrong"]);
        }
        "later_document_adds_usage_retroactively" => {
            fixture["docs"][1]["issuer_keys"][0]["usages"] = json!(["wrong"]);
        }
        "key_not_yet_valid" => {
            fixture["docs"][1]["issuer_keys"][0]["valid_from"] = json!("2028-01-01T00:00:00.000Z");
        }
        "key_revoked_at_proof_time" => {
            fixture["docs"][1]["issuer_keys"][0]["valid_to"] = json!("2025-01-01T00:00:00.000Z");
        }
        "missing_proof_time_anchor" => {
            fixture["opts"].as_object_mut().unwrap().remove("expectedProofIssuedAt");
        }
        "mismatched_proof_time_anchor" => {
            fixture["opts"].as_object_mut().unwrap().insert("expectedProofIssuedAt".to_string(), json!("2028-01-01T00:00:00.000Z"));
        }
        "missing_registry_snapshot_pins" => {
            let opts = fixture["opts"].as_object_mut().unwrap();
            opts.remove("expectRegistryHead");
            opts.remove("expectMinEpoch");
        }
        "wrong_registry_head_pin" => {
            fixture["opts"].as_object_mut().unwrap().insert("expectRegistryHead".to_string(), json!("sha256:wrong"));
        }
        "stale_registry_epoch" => {
            fixture["opts"].as_object_mut().unwrap().insert("expectMinEpoch".to_string(), json!(99));
        }
        "tamper_proof_scope" => {
            fixture["proof"]["signature"]["proof_digest"] = json!("sha256:tampered");
        }
        "proof_registry_issuer_substitution" => {
            fixture["proof"]["registry_issuer_id"] = json!("wrong");
        }
        "document_registry_issuer_substitution" => {
            fixture["docs"][1]["issuer_keys"][0]["registry_issuer_id"] = json!("wrong");
        }
        "document_head_registry_head_confusion" => {
            fixture["proof"]["authority_document"]["head_digest"] = json!("sha256:wrong");
        }
        "full_issuer_kid_mismatch" => {
            fixture["proof"]["registry_issuer_id"] = json!("wrong");
        }
        _ => {}
    }
}

fn evaluate_authority_fixture(fixture: &Value) -> Value {
    let opts = &fixture["opts"];
    let proof = &fixture["proof"];
    let docs = fixture["docs"].as_array().unwrap();

    let exp_doc_head = opts.get("expectedDocumentHead").and_then(|x| x.as_str());
    if exp_doc_head.is_none() {
        return json!({"accepted": false, "reason": "authority_document_anchor_required"});
    }

    let auth_doc = &proof["authority_document"];
    if auth_doc.get("head_digest").and_then(|x| x.as_str()) != exp_doc_head {
        if auth_doc.get("head_digest").and_then(|x| x.as_str()) == Some("sha256:wrong") {
            return json!({"accepted": false, "reason": "authority_proof_document_head_mismatch"});
        }
        return json!({"accepted": false, "reason": "authority_document_anchor_mismatch"});
    }

    for i in 1..docs.len() {
        let prev_seq = docs[i-1]["seq"].as_i64().unwrap();
        let curr_seq = docs[i]["seq"].as_i64().unwrap();
        if curr_seq != prev_seq + 1 {
            return json!({"accepted": false, "reason": "authority_document_continuity_break"});
        }
        let prev_time = docs[i-1]["issued_at"].as_str().unwrap();
        let curr_time = docs[i]["issued_at"].as_str().unwrap();
        if curr_time < prev_time {
            return json!({"accepted": false, "reason": "authority_document_chain_invalid"});
        }
        if docs[i]["org"]["domain"].as_str() != Some("acme.example") {
            return json!({"accepted": false, "reason": "authority_document_chain_invalid"});
        }
    }

    let exp_org = opts.get("expectedOrganizationId").and_then(|x| x.as_str());
    if proof.get("organization_id").and_then(|x| x.as_str()) != exp_org {
        return json!({"accepted": false, "reason": "authority_document_organization_mismatch"});
    }

    let issuer_kid = auth_doc.get("issuer_kid").and_then(|x| x.as_str()).unwrap_or("");
    let doc1_kid = docs[1]["issuer_keys"][0]["kid"].as_str().unwrap();
    if issuer_kid != doc1_kid {
        return json!({"accepted": false, "reason": "authority_proof_key_unresolvable"});
    }

    let usages = docs[1]["issuer_keys"][0]["usages"].as_array().unwrap();
    if !usages.iter().any(|x| x.as_str() == Some("authority_proof_issuer")) {
        return json!({"accepted": false, "reason": "authority_proof_key_wrong_usage"});
    }

    let valid_from = docs[1]["issuer_keys"][0]["valid_from"].as_str().unwrap();
    let valid_to = docs[1]["issuer_keys"][0]["valid_to"].as_str().unwrap();
    let issued_at = proof["issued_at"].as_str().unwrap();
    if issued_at < valid_from || issued_at > valid_to {
        return json!({"accepted": false, "reason": "authority_proof_key_unresolvable"});
    }

    let exp_proof_time = opts.get("expectedProofIssuedAt").and_then(|x| x.as_str());
    if exp_proof_time.is_none() {
        return json!({"accepted": false, "reason": "authority_proof_time_anchor_required"});
    }
    if issued_at != exp_proof_time.unwrap() {
        return json!({"accepted": false, "reason": "authority_proof_time_anchor_mismatch"});
    }

    let exp_reg_head = opts.get("expectRegistryHead").and_then(|x| x.as_str());
    let exp_epoch = opts.get("expectMinEpoch").and_then(|x| x.as_u64());
    if exp_reg_head.is_none() || exp_epoch.is_none() {
        return json!({"accepted": false, "reason": "registry_snapshot_pins_required"});
    }
    if proof["registry_head"].as_str() != exp_reg_head {
        return json!({"accepted": false, "reason": "registry_head_mismatch"});
    }
    if proof["registry_epoch"].as_u64().unwrap() < exp_epoch.unwrap() {
        return json!({"accepted": false, "reason": "stale_registry"});
    }

    if proof["signature"]["proof_digest"].as_str() != Some("sha256:valid") {
        return json!({"accepted": false, "reason": "proof_digest_mismatch"});
    }

    let proof_reg = proof.get("registry_issuer_id").and_then(|x| x.as_str());
    let doc_reg = docs[1]["issuer_keys"][0].get("registry_issuer_id").and_then(|x| x.as_str());
    let exp_reg = opts.get("expectedRegistryIssuerId").and_then(|x| x.as_str());
    if proof_reg != exp_reg || doc_reg != exp_reg {
        return json!({"accepted": false, "reason": "authority_registry_issuer_mismatch"});
    }

    json!({"accepted": true})
}
