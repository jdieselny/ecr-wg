pub mod canonicalization;
pub mod receipts;
pub mod signoffs;
pub mod quorum;
pub mod trust_receipt;
pub mod witness;
pub mod consumption_proof;
pub mod initiator_attestation;
pub mod currency;
pub mod revocation;
pub mod time_attestation;
pub mod evidence_record;
pub mod provenance;
pub mod timestamp_proof;
pub mod aec_role;
pub mod scitt_statement;

use serde_json::Value;

/// Safe vector list extraction. Missing or non-array `vectors` → empty slice.
pub fn vectors_array(root: &Value) -> &[Value] {
    match root.get("vectors").and_then(|v| v.as_array()) {
        Some(a) => a.as_slice(),
        None => &[],
    }
}

/// Safe vector id. Missing/non-string → stable placeholder so the runner never panics.
pub fn vector_id(v: &Value) -> String {
    v.get("id")
        .and_then(|x| x.as_str())
        .unwrap_or("unknown")
        .to_string()
}