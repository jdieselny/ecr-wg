use serde_json::{json, Value};
use crate::suites::{vector_id, vectors_array};

pub fn run(root: &Value) -> Vec<(String, Value)> {
    let mut results = Vec::new();
    for v in vectors_array(root) {
        let id = vector_id(v);
        let mut predicted = v.get("predicted_effects").cloned().unwrap_or(Value::Null);
        let mut observed = v.get("observed_effects").cloned().unwrap_or(Value::Null);
        
        let mut context = "";
        if let Some(attestation) = v.get("attestation") {
            if let Some(obs) = attestation.get("observed_effects") {
                observed = obs.clone();
                context = "signed_receipt: ";
            }
        }
        if let Some(approved) = v.get("approved") {
            if let Some(pred) = approved.get("predicted_effects") {
                predicted = pred.clone();
                context = "relying_party_policy: ";
            }
        }
        if let Some(pol) = v.get("policy") {
            if let Some(pred) = pol.get("predicted_effects") {
                predicted = pred.clone();
            }
        }
        let res_eval = crate::suites::outcome_binding_exec::evaluate_predicted_effects(&predicted, &observed);
        let outcome = res_eval.get("outcome").and_then(|x| x.as_str()).unwrap_or("in_bounds");
        
        let mut reasons = Vec::new();
        if let Some(r_arr) = res_eval.get("reasons").and_then(|x| x.as_array()) {
            for r in r_arr {
                if let Some(mut s) = r.as_str().map(|x| x.to_string()) {
                    if s.starts_with("predicted eq ") && s.contains(" observed") {
                        s = "predicted eq".to_string();
                    } else if s.starts_with("predicted set_eq") {
                        s = "predicted set_eq".to_string();
                    } else if s.starts_with("predicted <= ") && s.contains(" observed") {
                        let val = s.split(" for ").next().unwrap().replace("predicted <= ", "");
                        s = format!("predicted <= {}", val);
                    } else if s.starts_with("predicted >= ") && s.contains(" observed") {
                        let val = s.split(" for ").next().unwrap().replace("predicted >= ", "");
                        s = format!("predicted >= {}", val);
                    } else if s.starts_with("predicted min ") && s.contains(" observed") {
                        s = "below min".to_string();
                    } else if s.starts_with("predicted max ") && s.contains(" observed") {
                        s = "above max".to_string();
                    } else if s.starts_with("predicted absent ") && s.contains(" observed") {
                        s = "predicted absent".to_string();
                    } else if s.starts_with("predicted count <= ") && s.contains(" observed") {
                        let val = s.split(" for ").next().unwrap().replace("predicted count <= ", "");
                        s = format!("predicted count <= {}", val);
                    } else if s.starts_with("no observed effect for ") {
                        s = "no observed effect".to_string();
                    } else if s.starts_with("ambiguous: ") {
                        s = "ambiguous".to_string();
                    } else if s.contains("MUST be strings") {
                        s = "MUST be strings".to_string();
                    }

                    reasons.push(json!(format!("{}{}", context, s)));
                }
            }
        }
        
        let mut res = json!({
            "outcome": outcome,
        });
        if !reasons.is_empty() {
            res.as_object_mut().unwrap().insert("reasons".to_string(), json!(reasons));
        }

        if v.get("kind").and_then(|x| x.as_str()) == Some("graph") {
            let mut verdict = "admissible".to_string();
            let mut reason = None;
            let att = v.get("attestation");
            let app = v.get("approved");
            let pol = v.get("policy");

            if app.is_none() {
                verdict = "conflicted".to_string();
                reason = Some("effect_commitment_missing".to_string());
            } else {
                let app_v = app.unwrap();
                if app_v.get("action_digest").is_some() {
                    verdict = "conflicted".to_string();
                    reason = Some("effect_commitment_missing".to_string());
                } else if let Some(att_v) = att {
                    if let Some(o_dig) = att_v.get("observed_effect_digest").and_then(|x| x.as_str()) {
                        if let Some(c_dig) = app_v.get("committed_effect_digest").and_then(|x| x.as_str()) {
                            if o_dig != c_dig {
                                verdict = "conflicted".to_string();
                                reason = Some("effect_divergence".to_string());
                            }
                        }
                    }
                }
            }

            if let Some(pol_v) = pol {
                if !pol_v.is_null() && !pol_v.as_object().unwrap().is_empty() {
                    if let Some(p_pred) = pol_v.get("predicted_effects") {
                        if !p_pred.is_array() {
                            verdict = "conflicted".to_string();
                            reason = Some("effect_incomparable".to_string());
                        }
                    }
                }
            }

            if true {
                if let Some(outcome) = res.get("outcome").and_then(|x| x.as_str()) {
                    if outcome == "divergent" {
                        verdict = "conflicted".to_string();
                        reason = Some("effect_divergence".to_string());
                    } else if outcome == "incomparable" {
                        verdict = "conflicted".to_string();
                        reason = Some("effect_incomparable".to_string());
                    }
                }
            }

            let mut final_res = json!({"verdict": verdict});
            if let Some(r) = reason {
                final_res.as_object_mut().unwrap().insert("reasons".to_string(), json!([r]));
            }
            res = final_res;
        }
        results.push((id, res));
    }
    results
}

