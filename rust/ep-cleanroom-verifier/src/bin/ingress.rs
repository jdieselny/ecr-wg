// SPDX-License-Identifier: Apache-2.0
// Ingress Challenge Server (RR-1 compliant) for ecr-wg.
// Runs the Receipt-Required protocol over grid.curtailment actions.
//
// Build: cargo build --bin ingress
// Run: cargo run --bin ingress

use axum::{
    http::{header, StatusCode, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Mutex;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

// In-memory registry to prevent replays (one-time consumption)
struct ServerState {
    spent_receipts: Mutex<HashSet<String>>,
    pinned_public_key: String,
}

static STATE: std::sync::OnceLock<ServerState> = std::sync::OnceLock::new();

fn get_state() -> &'static ServerState {
    STATE.get_or_init(|| {
        let default_key = "MCowBQYDK2VwAyEABIJEI__HmD5lnHsY1hPPHHUUaHHqdxxXJ6OKcjE9Imk".to_string();
        // Try to load public key from standard locations
        let candidates = [
            "keys/public.key",
            "rust/ep-cleanroom-verifier/keys/public.key",
            "../keys/public.key",
        ];
        let mut key = default_key;
        for c in &candidates {
            if Path::new(c).exists() {
                if let Ok(content) = fs::read_to_string(c) {
                    let cleaned: String = content.chars().filter(|ch| !ch.is_whitespace()).collect();
                    if !cleaned.is_empty() {
                        key = cleaned;
                        break;
                    }
                }
            }
        }
        ServerState {
            spent_receipts: Mutex::new(HashSet::new()),
            pinned_public_key: key,
        }
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/.well-known/agent-actions.json", get(get_manifest))
        .route("/actions/grid.curtailment", post(handle_curtailment));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("ECR-WG Ingress Challenge Server running on http://localhost:8080");
    println!("Manifest: http://localhost:8080/.well-known/agent-actions.json");
    println!("Ingress endpoint: POST http://localhost:8080/actions/grid.curtailment");
    axum::serve(listener, app).await.unwrap();
}

// 1. Manifest Delivery
async fn get_manifest() -> impl IntoResponse {
    let manifest = json!({
        "@version": "EP-ACTION-RISK-MANIFEST-v0.1",
        "service": {
            "name": "ecr-wg-ingress-challenge",
            "url": "http://localhost:8080"
        },
        "receipt_required": {
            "status": 428,
            "challenge_header": "Receipt-Required",
            "proof_header": "X-EMILIA-Receipt",
            "profile": "EP-RECEIPT-v1"
        },
        "defaults": {
            "allow_unprotected": false,
            "reject_malformed": true,
            "max_age_sec": 900
        },
        "actions": [
            {
                "id": "curtail",
                "description": "Grid curtailment execution action",
                "match": {
                    "protocol": "http",
                    "method": "POST",
                    "path": "/actions/grid.curtailment"
                },
                "action_type": "grid.curtailment",
                "risk": "critical",
                "receipt_required": true,
                "assurance_class": "class_a",
                "max_age_sec": 900
            }
        ]
    });
    Json(manifest)
}

// 2. HTTP 428 challenge constructor
fn build_challenge_response() -> Response {
    let challenge_val = "action=\"grid.curtailment\", proof=\"X-EMILIA-Receipt\", manifest=\"/.well-known/agent-actions.json\", profile=\"EP-RECEIPT-v1\", assurance=\"class_a\", max_age=\"900\"";
    
    let problem_body = json!({
        "type": "https://emiliaprotocol.ai/errors/emilia_receipt_required",
        "title": "EMILIA Receipt Required",
        "status": 428,
        "detail": "No EMILIA receipt presented.",
        "required": {
            "action": "grid.curtailment",
            "manifest": "/.well-known/agent-actions.json",
            "challenge_header": "Receipt-Required",
            "proof_header": "X-EMILIA-Receipt",
            "header": "X-EMILIA-Receipt: base64(<EP-RECEIPT-v1 JSON>)",
            "assurance_class": "class_a",
            "max_age_sec": 900
        }
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    headers.insert(
        header::HeaderName::from_static("receipt-required"),
        HeaderValue::from_str(challenge_val).unwrap(),
    );

    (StatusCode::PRECONDITION_REQUIRED, headers, Json(problem_body)).into_response()
}

// Custom error responses
fn build_error_response(status: StatusCode, detail: &str) -> Response {
    let body = json!({
        "error": "refused",
        "detail": detail
    });
    (status, Json(body)).into_response()
}

// Handler for the curtailment action
async fn handle_curtailment(headers: HeaderMap, _body_str: String) -> Response {
    // A. Check for receipt header
    let receipt_header = match headers.get("X-EMILIA-Receipt") {
        Some(h) => match h.to_str() {
            Ok(s) => s,
            Err(_) => return build_challenge_response(),
        },
        None => return build_challenge_response(),
    };

    // B. Base64-decode the receipt content
    let receipt_bytes = match URL_SAFE_NO_PAD.decode(receipt_header.trim()) {
        Ok(b) => b,
        Err(_) => {
            // Attempt standard base64 if url-safe fails
            match base64::engine::general_purpose::STANDARD.decode(receipt_header.trim()) {
                Ok(b) => b,
                Err(_) => return build_error_response(StatusCode::BAD_REQUEST, "invalid_base64_receipt"),
            }
        }
    };

    let receipt_str = match String::from_utf8(receipt_bytes) {
        Ok(s) => s,
        Err(_) => return build_error_response(StatusCode::BAD_REQUEST, "receipt_not_utf8"),
    };

    // C. Parse the receipt JSON
    let receipt_json: Value = match serde_json::from_str(&receipt_str) {
        Ok(v) => v,
        Err(_) => return build_error_response(StatusCode::BAD_REQUEST, "invalid_json_receipt"),
    };

    // D. Cryptographic verification using cleanroom lib
    let state = get_state();
    let verified = match emilia_rust_verifier::verify_receipt(&receipt_str, &state.pinned_public_key) {
        Ok(true) => true,
        _ => false,
    };

    if !verified {
        return build_error_response(StatusCode::UNAUTHORIZED, "forged_refused");
    }

    // E. Structural/Profile validation
    let payload = &receipt_json["payload"];
    let action_type = payload["action"]["action_type"].as_str().unwrap_or("");
    if action_type != "grid.curtailment" {
        return build_error_response(StatusCode::BAD_REQUEST, "wrong_action_type");
    }

    // F. Time window verification
    let now = Utc::now();
    if let Some(not_before_str) = payload["action"]["window"]["not_before"].as_str() {
        if let Ok(not_before) = DateTime::parse_from_rfc3339(not_before_str) {
            if now < not_before.with_timezone(&Utc) {
                return build_error_response(StatusCode::BAD_REQUEST, "window_not_started");
            }
        }
    }
    if let Some(not_after_str) = payload["action"]["window"]["not_after"].as_str() {
        if let Ok(not_after) = DateTime::parse_from_rfc3339(not_after_str) {
            if now > not_after.with_timezone(&Utc) {
                return build_error_response(StatusCode::BAD_REQUEST, "window_expired");
            }
        }
    }

    // G. Replay protection (One-time receipt consumption)
    let receipt_id = payload["receipt_id"].as_str().unwrap_or("").to_string();
    if receipt_id.is_empty() {
        return build_error_response(StatusCode::BAD_REQUEST, "missing_receipt_id");
    }

    {
        let mut spent = state.spent_receipts.lock().unwrap();
        if spent.contains(&receipt_id) {
            return build_error_response(StatusCode::CONFLICT, "replay_refused");
        }
        spent.insert(receipt_id);
    }

    // Success! Execute action and return acknowledgment
    let target_kw = payload["action"]["magnitude"].as_f64().unwrap_or(0.0);
    let ack = json!({
        "status": "curtailed",
        "action_executed": "grid.curtailment",
        "applied_magnitude_mw": target_kw,
        "detail": "Receipt verified via cleanroom. Posture modified: NVML caps and non-protected lane eviction engaged.",
        "timestamp": now.to_rfc3339()
    });

    Json(ack).into_response()
}
