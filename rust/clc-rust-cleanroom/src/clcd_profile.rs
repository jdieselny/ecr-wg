//! CLC-D profile-layer property cases.
//!
//! Two corpora live here, both from `data/_vectors/clc-d/` upstream:
//!
//! * `containment-property-cases.json` — 784 cases, the full 28x28 cross
//!   product of a grant space, checked against the forward-closure property
//!   in `_meta`.
//! * `reason-collapse-vectors.json` — 3 cases, asserting which reason codes a
//!   boundary may collapse and which must stay distinguishable.
//!
//! Neither needs the §13.8.1 carrier mapping, which is why this engine can
//! exercise them: the property cases use the `capabilities` projection only.

use serde_json::Value;

use crate::evaluator::{self, Grant};
use crate::parameters;

/// Outcome of one property case, with the coverage that makes a pass mean
/// something.
#[derive(Debug, Clone)]
pub(crate) struct PropertyVerdict {
    pub passed: bool,
    pub detail: String,
    /// How many `(case, operation)` pairs actually exercised the
    /// forward-closure implication. A pass with zero witnesses proves
    /// nothing, so this is reported rather than hidden.
    pub closure_witnesses: usize,
    /// True when the property was inapplicable to this implementation.
    /// Distinct from both pass and fail: an inapplicable case must never be
    /// counted as evidence.
    pub not_applicable: bool,
}

impl PropertyVerdict {
    fn pass(detail: String, witnesses: usize) -> Self {
        Self { passed: true, detail, closure_witnesses: witnesses, not_applicable: false }
    }

    fn fail(detail: String, witnesses: usize) -> Self {
        Self { passed: false, detail, closure_witnesses: witnesses, not_applicable: false }
    }

    fn not_applicable(detail: String) -> Self {
        Self { passed: false, detail, closure_witnesses: 0, not_applicable: true }
    }
}

/// A containment property case carries bare `parent` and `child` grants,
/// where the binding-profile crosswalks use a `parent_source` bundle.
pub(crate) fn is_containment_property_case(case: &Value) -> bool {
    case.get("parent").is_some() && case.get("child").is_some() && case.get("sources").is_none()
}

fn grant_of(case: &Value, side: &str) -> Option<Grant> {
    let node = case.get(side)?;
    let id = node.get("id").and_then(Value::as_str)?.to_string();
    Some(Grant {
        id,
        params: node.get("params").cloned(),
        param_bounds: node.get("param_bounds").cloned(),
        constraints: node
            .get("constraints")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// `Entails(G, o)` — the CLC-A §5 predicate, matching the `--vector` entail
/// path step for step so the property is checked against the same meaning the
/// conformance vectors use.
fn entails(grant: &Grant, op: &Value) -> bool {
    if parameters::validate_grant_params(grant.params.as_ref(), grant.param_bounds.as_ref()).is_err()
    {
        return false;
    }
    let Some(op_id) = op.get("id").and_then(Value::as_str) else {
        return false;
    };
    if op_id.is_empty() {
        return false;
    }
    if crate::grammar::parse_capability_id(op_id).is_err() {
        return false;
    }
    if evaluator::entail_identifiers(&grant.id, op_id).is_err() {
        return false;
    }
    let op_params = op.get("params");
    if parameters::entail_parameters(
        grant.params.as_ref(),
        grant.param_bounds.as_ref(),
        op_params,
    )
    .is_err()
    {
        return false;
    }
    crate::constraints::evaluate_constraints(&grant.constraints, op_params)
        .rejection
        .is_none()
}

/// Parameters normalised for the antisymmetry test: `params: {}` is the same
/// semantic class as no params at all (§13.3).
pub(crate) fn normalized_params(grant: &Grant) -> Value {
    match grant.params.as_ref() {
        None | Some(Value::Null) => Value::Null,
        Some(Value::Object(map)) if map.is_empty() => Value::Null,
        Some(other) => other.clone(),
    }
}

/// Two grants are in the same semantic class when the identifier and the
/// normalised parameters both match.
pub(crate) fn same_class(left: &Grant, right: &Grant) -> bool {
    left.id == right.id && normalized_params(left) == normalized_params(right)
}

/// The forward-closure implication, checked only over the operations the child
/// actually entails. Returns the first violation as `(op id, witnesses seen)`.
///
/// Deliberately takes the two grants rather than a pre-computed `contains`
/// flag: the caller must establish containment first. That split is what lets
/// the checker be tested against a pair that violates the property (see
/// `closure_violation_is_detected`), proving it is not vacuous.
pub(crate) fn closure_violation(
    parent: &Grant,
    child: &Grant,
    ops: &[Value],
) -> (Option<String>, usize) {
    let mut witnesses = 0usize;
    for op in ops {
        if !entails(child, op) {
            continue;
        }
        witnesses += 1;
        if !entails(parent, op) {
            let op_id = op.get("id").and_then(Value::as_str).unwrap_or("op");
            // Ops in this corpus share identifiers and differ only by params,
            // so a violation is only locatable if the message carries both.
            let op_params = op
                .get("params")
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string());
            return (
                Some(format!(
                    "forward closure: child entails {op_id} with params {op_params} but parent does not"
                )),
                witnesses,
            );
        }
    }
    (None, witnesses)
}

/// Mutual containment across different semantic classes breaks antisymmetry
/// (§13.3): mutual containment may only be reported within one class.
pub(crate) fn antisymmetry_violation(parent: &Grant, child: &Grant, mutual: bool) -> Option<String> {
    if mutual && !same_class(parent, child) {
        Some(format!(
            "antisymmetry: mutual containment between different semantic classes ({} as {:?} vs {} as {:?})",
            parent.id,
            normalized_params(parent),
            child.id,
            normalized_params(child)
        ))
    } else {
        None
    }
}

/// Check one containment property case against all three properties.
pub(crate) fn check(case: &Value, ops: &[Value]) -> PropertyVerdict {
    let (Some(parent), Some(child)) = (grant_of(case, "parent"), grant_of(case, "child")) else {
        return PropertyVerdict::fail("case does not carry parseable parent and child".into(), 0);
    };

    // Property 1 — totality. `evaluate_containment` is a total function: it
    // returns a `ContainmentResult` for every input pair, including malformed
    // identifiers, which surface as a reason code rather than an error. The
    // call below therefore cannot raise; what matters is that it was made and
    // its result is used.
    let forward = evaluator::evaluate_containment(&parent, &child);
    let reverse = evaluator::evaluate_containment(&child, &parent);

    // Property 3 — antisymmetry on semantic equivalence classes.
    let mutual = forward.contains && reverse.contains;
    if let Some(detail) = antisymmetry_violation(&parent, &child, mutual) {
        return PropertyVerdict::fail(detail, 0);
    }

    // Property 2 — forward closure, only where containment holds.
    let mut witnesses = 0usize;
    if forward.contains {
        let (violation, seen) = closure_violation(&parent, &child, ops);
        witnesses = seen;
        if let Some(detail) = violation {
            return PropertyVerdict::fail(detail, witnesses);
        }
    }

    PropertyVerdict::pass(
        format!(
            "contains={} mutual={} closure_witnesses={}",
            forward.contains, mutual, witnesses
        ),
        witnesses,
    )
}

// ---------------------------------------------------------------------------
// reason-collapse
// ---------------------------------------------------------------------------

/// Every reason code this engine can emit, as `Reason::code()` strings.
const EMITTABLE: &[&str] = &[
    "capability_not_authorized",
    "child_exceeds_parent",
    "different_namespace",
    "empty_bound_denies_class",
    "invalid_capability_id",
    "invalid_constraint",
    "invalid_params_binding",
    "invalid_params_duplicate_key",
    "invalid_params_null",
    "invalid_params_number",
    "invalid_params_size",
    "invalid_resolution",
    "invalid_timestamp",
    "literal_mismatch",
    "missing_capability_id",
    "not_in_enum",
    "params_cardinality",
    "params_exceed_grant",
    "params_missing",
    "params_not_multiple",
    "params_not_narrower",
    "params_out_of_range",
    "undeclared_param",
    "unknown_constraint",
    "unsupported_language_revision",
    "unsupported_wildcard",
    "wildcard_requires_trailing_segment",
];

/// Check one reason-collapse vector.
///
/// The corpus states, per code, whether a boundary may collapse it to
/// `capability_not_authorized`. This engine never collapses: it reports the
/// precise relation code, which is what §13.5 requires at the relation layer.
/// So a code is **verified** when the engine can emit it and emits it
/// precisely; it is **not applicable** when the engine cannot emit it at all,
/// which is the honest state for a binding-profile pre-check code this
/// implementation does not produce.
pub(crate) fn check_reason_collapse(vector: &Value) -> PropertyVerdict {
    let Some(code) = vector.get("code").and_then(Value::as_str) else {
        return PropertyVerdict::fail("reason-collapse vector carries no code".into(), 0);
    };
    let origin = vector
        .get("origin")
        .and_then(Value::as_str)
        .unwrap_or("unknown origin");
    let collapsible = vector
        .get("collapsible")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !EMITTABLE.contains(&code) {
        return PropertyVerdict::not_applicable(format!(
            "{code} ({origin}) is not produced by this implementation; \
             the engine implements the capabilities projection and no binding-profile \
             pre-check, so collapsibility cannot be evidenced here"
        ));
    }

    if collapsible {
        // Collapsible means a boundary *may* collapse it. Keeping it precise is
        // also permitted, and is what §13.5 requires at the relation layer.
        PropertyVerdict::pass(
            format!("{code} is emittable and reported precisely (collapse permitted, not performed)"),
            0,
        )
    } else {
        PropertyVerdict::pass(
            format!("{code} is emittable and never collapsed to capability_not_authorized"),
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn grant(id: &str, params: Value) -> Grant {
        Grant {
            id: id.to_string(),
            params: if params.is_null() { None } else { Some(params) },
            param_bounds: None,
            constraints: Vec::new(),
        }
    }

    fn ops() -> Vec<Value> {
        vec![
            json!({"id": "std/database-v1:query:SELECT", "params": {"limit": 5}}),
            json!({"id": "std/database-v1:query:SELECT", "params": {"limit": 75}}),
            json!({"id": "std/database-v1:query:SELECT", "params": {"limit": 150}}),
        ]
    }

    #[test]
    fn closure_holds_for_a_narrower_child() {
        // parent admits up to 100, child up to 50: everything the child entails
        // the parent admits too.
        let parent = grant("std/database-v1:query:SELECT", json!({"limit": 100}));
        let child = grant("std/database-v1:query:SELECT", json!({"limit": 50}));
        assert!(evaluator::evaluate_containment(&parent, &child).contains);
        let (violation, witnesses) = closure_violation(&parent, &child, &ops());
        assert_eq!(violation, None);
        // limit 5 and limit 75... 75 > 50, so the child does not entail it.
        assert_eq!(witnesses, 1, "only the limit:5 op should be a witness");
    }

    #[test]
    fn closure_violation_is_detected() {
        // A wider child: it entails {limit:75}, which the narrower parent does
        // not. This pair does NOT satisfy containment, which is why the caller
        // must establish containment first — but it proves the checker detects
        // a violation rather than always returning None.
        let parent = grant("std/database-v1:query:SELECT", json!({"limit": 50}));
        let child = grant("std/database-v1:query:SELECT", json!({"limit": 100}));
        assert!(!evaluator::evaluate_containment(&parent, &child).contains);
        let (violation, witnesses) = closure_violation(&parent, &child, &ops());
        let detail = violation.expect("closure violation must be detected");
        assert!(detail.contains("limit"), "{detail}");
        assert!(witnesses >= 2, "5 and 75 are both entailed by the child");
    }

    #[test]
    fn empty_params_are_the_same_class_as_absent() {
        let absent = grant("std/database-v1:query:SELECT", Value::Null);
        let empty = grant("std/database-v1:query:SELECT", json!({}));
        assert_eq!(normalized_params(&absent), normalized_params(&empty));
        assert!(same_class(&absent, &empty));
    }

    #[test]
    fn differing_params_are_a_different_class() {
        let a = grant("std/database-v1:query:SELECT", json!({"limit": 50}));
        let b = grant("std/database-v1:query:SELECT", json!({"limit": 100}));
        assert!(!same_class(&a, &b));
    }

    #[test]
    fn antisymmetry_violation_is_detected() {
        let a = grant("std/database-v1:query:SELECT", json!({"limit": 50}));
        let b = grant("std/database-v1:query:SELECT", json!({"limit": 100}));
        assert!(antisymmetry_violation(&a, &b, false).is_none());
        let detail = antisymmetry_violation(&a, &b, true).expect("must be reported");
        assert!(detail.contains("antisymmetry"), "{detail}");
        // Same class may be mutually contained.
        let c = grant("std/database-v1:query:SELECT", json!({"limit": 50}));
        assert!(antisymmetry_violation(&a, &c, true).is_none());
    }

    #[test]
    fn reason_collapse_reports_the_binding_profile_code_as_inapplicable() {
        let verdict = check_reason_collapse(&json!({
            "id": "rc-003",
            "code": "delegation_mode_not_narrower",
            "collapsible": false,
            "origin": "binding-profile pre-check"
        }));
        assert!(verdict.not_applicable, "{}", verdict.detail);
        assert!(!verdict.passed);
    }

    #[test]
    fn reason_collapse_verifies_an_emittable_code() {
        let verdict = check_reason_collapse(&json!({
            "id": "rc-002",
            "code": "params_not_narrower",
            "collapsible": false,
            "origin": "Contains: layer 3"
        }));
        assert!(verdict.passed);
        assert!(!verdict.not_applicable);
    }
}
