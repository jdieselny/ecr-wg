//! Capability Language Core — independent cleanroom implementation.
//!
//! Normative source: `draft-wei-capability-language-core` (CLC-1.19).
//! This crate does not read or copy another implementation.

#![forbid(unsafe_code)]

mod constraints;
mod error;
mod evaluator;
mod grammar;
mod lattice;
mod parameters;
mod vector_runner;

pub use error::Reason;
pub use evaluator::{
    evaluate_authorization, evaluate_containment, evaluate_containment_any, resolve,
    ContainmentResult, Decision, Grant, Operation, Resolution, Verdict,
};
pub use grammar::{parse_capability_id, CapabilityId};
pub use lattice::LatticeState;
pub use parameters::{
    boundary_notices_of, prepare_params, BoundaryDomain, BoundaryNotice, PARAMETER_RISK_DRAFT,
};
pub use vector_runner::{
    report_json, report_junit, report_text, run_vector_dir, run_vector_file, run_vector_text,
    SuiteReport, VectorOutcome,
};

/// Top-level evaluation request.
#[derive(Debug, Clone)]
pub enum EvaluationRequest {
    Authorize {
        grants: Vec<Grant>,
        operation: Option<Operation>,
    },
    Contains {
        parent: Grant,
        child: Grant,
    },
    Resolve {
        decision: Decision,
        resolutions: Vec<Resolution>,
        now: Option<String>,
    },
}

/// Top-level evaluation result.
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationResult {
    Decision(Decision),
    Containment(ContainmentResult),
}

/// Evaluate one Authorize, Contains, or Resolve request.
pub fn evaluate(request: &EvaluationRequest) -> EvaluationResult {
    match request {
        EvaluationRequest::Authorize { grants, operation } => {
            EvaluationResult::Decision(evaluate_authorization(grants, operation.as_ref()))
        }
        EvaluationRequest::Contains { parent, child } => {
            EvaluationResult::Containment(evaluate_containment(parent, child))
        }
        EvaluationRequest::Resolve {
            decision,
            resolutions,
            now,
        } => EvaluationResult::Decision(resolve(decision, resolutions, now.as_deref())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn evaluate_dispatches_containment() {
        let parent = Grant {
            id: "example/power-v1:curtail:*".into(),
            params: Some(json!({"max_shed_kw": 10, "duration_sec": 7200, "circuit_groups": [1, 2, 3]})),
            param_bounds: None,
            constraints: Vec::new(),
        };
        let child = Grant {
            id: "example/power-v1:curtail:ups-unit-01".into(),
            params: Some(json!({"max_shed_kw": 2.5, "duration_sec": 900, "circuit_groups": [2]})),
            param_bounds: None,
            constraints: Vec::new(),
        };
        let result = evaluate(&EvaluationRequest::Contains { parent, child });
        match result {
            EvaluationResult::Containment(inner) => assert!(inner.contains),
            EvaluationResult::Decision(_) => panic!("expected containment"),
        }
    }
}
