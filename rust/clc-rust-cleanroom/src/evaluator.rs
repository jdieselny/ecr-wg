//! Authorize (§9) and Contains (§13.4), plus Resolve (§8.5).

use crate::constraints::{self, ConstraintOutcome};
use crate::error::Reason;
use crate::grammar::{self, CoverError};
use crate::lattice::LatticeState;
use crate::parameters::{self, BoundaryNotice};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grant {
    pub id: String,
    #[serde(default)]
    pub params: Option<Value>,
    #[serde(default)]
    pub param_bounds: Option<Value>,
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Allow,
    Deny,
    AllowUnresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub verdict: Verdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Residual obligations. Spec field name is `unresolved`.
    #[serde(default, alias = "unresolved_obligations")]
    pub unresolved: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_grant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_notices: Vec<BoundaryNotice>,
}

impl Decision {
    pub fn unresolved_obligations(&self) -> &[String] {
        &self.unresolved
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainmentResult {
    pub contains: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_notices: Vec<BoundaryNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub constraint: String,
    pub status: LatticeState,
}

/// Authorize, after the §12.1 major-revision gate.
///
/// A declared major greater than 1 is refused before empty-grant and
/// identifier checks. `None` is the same path as [`evaluate_authorization`].
pub fn evaluate_authorization_revision(
    grants: &[Grant],
    operation: Option<&Operation>,
    revision: Option<&str>,
) -> Decision {
    if let Some(declared) = revision {
        if let Err(reason) = grammar::check_clc_revision(declared) {
            return deny(reason, None);
        }
    }
    evaluate_authorization(grants, operation)
}

/// Entail's identifier step. Namespace equality wins over wildcard shape,
/// including a class-position `*` that a single-id parse rejects.
pub fn entail_identifiers(grant_id: &str, request_id: &str) -> Result<(), Reason> {
    grammar::relate_identifiers(grant_id, request_id)
}

pub fn evaluate_authorization(grants: &[Grant], operation: Option<&Operation>) -> Decision {
    if grants.is_empty() {
        return deny(Reason::CapabilityNotAuthorized, None);
    }
    let Some(operation) = operation else {
        return deny(Reason::MissingCapabilityId, None);
    };
    if operation.id.is_empty() {
        return deny(Reason::MissingCapabilityId, None);
    }
    let operation_id = match grammar::parse_capability_id(&operation.id) {
        Ok(id) => id,
        Err(reason) => return deny(reason, None),
    };

    let mut order: Vec<usize> = (0..grants.len()).collect();
    order.sort_by(|&a, &b| {
        grants[a]
            .id
            .as_bytes()
            .cmp(grants[b].id.as_bytes())
            .then(a.cmp(&b))
    });

    let mut allowing: Vec<Allowing<'_>> = Vec::new();
    let mut rejections: Vec<Reason> = Vec::new();

    for index in order {
        let grant = &grants[index];
        let Ok(grant_id) = grammar::parse_capability_id(&grant.id) else {
            continue;
        };
        if grammar::covers(&grant_id, &operation_id).is_err() {
            continue;
        }
        match parameters::entail_parameters(
            grant.params.as_ref(),
            grant.param_bounds.as_ref(),
            operation.params.as_ref(),
        ) {
            Err(reason) => rejections.push(reason),
            Ok(notices) => {
                let ConstraintOutcome {
                    unresolved,
                    rejection,
                } = constraints::evaluate_constraints(&grant.constraints, operation.params.as_ref());
                if let Some(reason) = rejection {
                    rejections.push(reason);
                } else {
                    allowing.push(Allowing {
                        id: &grant.id,
                        unresolved,
                        notices,
                    });
                }
            }
        }
    }

    if allowing.is_empty() {
        let reason = rejections
            .into_iter()
            .next()
            .unwrap_or(Reason::CapabilityNotAuthorized);
        return deny(reason, None);
    }

    let mut unresolved = Vec::new();
    let mut notices = Vec::new();
    for grant in &allowing {
        unresolved.extend(grant.unresolved.iter().cloned());
        notices.extend(grant.notices.iter().cloned());
    }
    unresolved.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    unresolved.dedup();
    let matched = Some(allowing[0].id.to_string());
    if unresolved.is_empty() {
        Decision {
            verdict: Verdict::Allow,
            reason: None,
            unresolved,
            matched_grant_id: matched,
            boundary_notices: notices,
        }
    } else {
        Decision {
            verdict: Verdict::AllowUnresolved,
            reason: None,
            unresolved,
            matched_grant_id: matched,
            boundary_notices: notices,
        }
    }
}

struct Allowing<'a> {
    id: &'a str,
    unresolved: Vec<String>,
    notices: Vec<BoundaryNotice>,
}

pub fn evaluate_containment(parent: &Grant, child: &Grant) -> ContainmentResult {
    let parent_id = match grammar::parse_capability_id(&parent.id) {
        Ok(id) => id,
        Err(reason) => return contain_fail(reason),
    };
    let child_id = match grammar::parse_capability_id(&child.id) {
        Ok(id) => id,
        Err(reason) => return contain_fail(reason),
    };
    if let Err(reason) =
        parameters::validate_grant_params(parent.params.as_ref(), parent.param_bounds.as_ref())
    {
        return contain_fail(reason);
    }
    if let Err(reason) =
        parameters::validate_grant_params(child.params.as_ref(), child.param_bounds.as_ref())
    {
        return contain_fail(reason);
    }
    match grammar::covers(&parent_id, &child_id) {
        Ok(()) => {}
        Err(CoverError::DifferentNamespace) => return contain_fail(Reason::DifferentNamespace),
        Err(_) => return contain_fail(Reason::ChildExceedsParent),
    }
    match parameters::narrow_parameters(
        parent.params.as_ref(),
        parent.param_bounds.as_ref(),
        child.params.as_ref(),
        child.param_bounds.as_ref(),
    ) {
        Ok(notices) => ContainmentResult {
            contains: true,
            reason: None,
            boundary_notices: notices,
        },
        Err(reason) => contain_fail(reason),
    }
}

/// §8.5 Resolve. Terminal allow/deny decisions are returned unchanged.
pub fn resolve(decision: &Decision, resolutions: &[Resolution], now: Option<&str>) -> Decision {
    if matches!(decision.verdict, Verdict::Allow | Verdict::Deny) {
        return decision.clone();
    }
    for resolution in resolutions {
        if resolution.constraint.is_empty() {
            return deny(Reason::InvalidResolution, None);
        }
        if !matches!(
            resolution.status,
            LatticeState::Satisfied | LatticeState::Violated | LatticeState::Unknown
        ) {
            return deny(Reason::InvalidResolution, None);
        }
    }
    if let Some(now) = now {
        if !constraints::timestamp_ok(now) {
            return deny(Reason::InvalidTimestamp, None);
        }
    }

    let mut statuses = Vec::with_capacity(decision.unresolved.len());
    for obligation in &decision.unresolved {
        let mut parts = Vec::new();
        if let Some(now) = now {
            match constraints::clock_status(obligation, now) {
                Ok(Some(status)) => parts.push(status),
                Ok(None) => {}
                Err(reason) => return deny(reason, None),
            }
        }
        for resolution in resolutions.iter().filter(|item| item.constraint == *obligation) {
            parts.push(resolution.status);
        }
        statuses.push((obligation.clone(), LatticeState::collapse(parts)));
    }

    if let Some((obligation, _)) = statuses
        .iter()
        .find(|(_, status)| *status == LatticeState::Violated)
    {
        let ty = constraints::constraint_type_name(obligation);
        return Decision {
            verdict: Verdict::Deny,
            reason: Some(Reason::Violated(ty).code()),
            unresolved: Vec::new(),
            matched_grant_id: decision.matched_grant_id.clone(),
            boundary_notices: decision.boundary_notices.clone(),
        };
    }
    if statuses.iter().all(|(_, status)| *status == LatticeState::Satisfied) {
        return Decision {
            verdict: Verdict::Allow,
            reason: None,
            unresolved: Vec::new(),
            matched_grant_id: decision.matched_grant_id.clone(),
            boundary_notices: decision.boundary_notices.clone(),
        };
    }
    let mut unresolved: Vec<String> = statuses
        .into_iter()
        .filter(|(_, status)| *status == LatticeState::Unknown)
        .map(|(obligation, _)| obligation)
        .collect();
    unresolved.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    unresolved.dedup();
    Decision {
        verdict: Verdict::AllowUnresolved,
        reason: None,
        unresolved,
        matched_grant_id: decision.matched_grant_id.clone(),
        boundary_notices: decision.boundary_notices.clone(),
    }
}

fn deny(reason: Reason, matched: Option<String>) -> Decision {
    Decision {
        verdict: Verdict::Deny,
        reason: Some(reason.code()),
        unresolved: Vec::new(),
        matched_grant_id: matched,
        boundary_notices: Vec::new(),
    }
}

fn contain_fail(reason: Reason) -> ContainmentResult {
    ContainmentResult {
        contains: false,
        reason: Some(reason.code()),
        boundary_notices: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn grant(id: &str, params: Value, constraints: &[&str]) -> Grant {
        Grant {
            id: id.to_string(),
            params: Some(params),
            param_bounds: None,
            constraints: constraints.iter().map(|c| (*c).to_string()).collect(),
        }
    }

    #[test]
    fn clc_2_is_an_unsupported_language_revision() {
        let grants = [Grant {
            id: "std/database-v1:query:SELECT".into(),
            params: None,
            param_bounds: None,
            constraints: vec![],
        }];
        let operation = Operation {
            id: "std/database-v1:query:SELECT".into(),
            params: None,
        };
        let decision =
            evaluate_authorization_revision(&grants, Some(&operation), Some("CLC-2.0"));
        assert_eq!(decision.verdict, Verdict::Deny);
        assert_eq!(
            decision.reason.as_deref(),
            Some("unsupported_language_revision")
        );
        let same_major =
            evaluate_authorization_revision(&grants, Some(&operation), Some("CLC-1.19"));
        assert_eq!(same_major.verdict, Verdict::Allow);
    }

    #[test]
    fn ccx_style_containment_holds_for_the_narrowed_shed() {
        let parent = grant(
            "example/power-v1:curtail:*",
            json!({"circuit_groups": [1, 2, 3], "duration_sec": 7200, "max_shed_kw": 10}),
            &[],
        );
        let child = grant(
            "example/power-v1:curtail:ups-unit-01",
            json!({"circuit_groups": [2], "duration_sec": 900, "max_shed_kw": 2.5}),
            &[],
        );
        let result = evaluate_containment(&parent, &child);
        assert!(result.contains, "{:?}", result.reason);
    }

    #[test]
    fn authorize_residualizes_time_and_network_then_clock_denies() {
        let window = "varwof/constraint-v1:time:window:[{\"end\":\"15:00\",\"start\":\"14:00\"}]";
        let cidr = "varwof/constraint-v1:network:cidr:[\"10.0.1.0/24\"]";
        let grants = [Grant {
            id: "example/power-v1:curtail:*".into(),
            params: None,
            param_bounds: None,
            constraints: vec![cidr.into(), window.into()],
        }];
        let operation = Operation {
            id: "example/power-v1:curtail:ups-unit-01".into(),
            params: None,
        };
        let decision = evaluate_authorization(&grants, Some(&operation));
        assert_eq!(decision.verdict, Verdict::AllowUnresolved);
        assert_eq!(decision.unresolved.len(), 2);

        let resolved = resolve(
            &decision,
            &[Resolution {
                constraint: cidr.into(),
                status: LatticeState::Satisfied,
            }],
            Some("2026-10-19T15:30:00Z"),
        );
        assert_eq!(resolved.verdict, Verdict::Deny);
        assert_eq!(resolved.reason.as_deref(), Some("time:violated"));
        assert!(resolved.unresolved.is_empty());
    }

    #[test]
    fn empty_grants_and_forbidden_wildcard_use_stable_reasons() {
        let operation = Operation {
            id: "std/database-v1:query:SELECT".into(),
            params: None,
        };
        assert_eq!(
            evaluate_authorization(&[], Some(&operation)).reason.as_deref(),
            Some("capability_not_authorized")
        );
        let bad = Operation {
            id: "*:query:SELECT".into(),
            params: None,
        };
        let grants = [Grant {
            id: "std/database-v1:query:*".into(),
            params: None,
            param_bounds: None,
            constraints: Vec::new(),
        }];
        assert_eq!(
            evaluate_authorization(&grants, Some(&bad)).reason.as_deref(),
            Some("unsupported_wildcard")
        );
        assert_eq!(
            evaluate_authorization(&[], None).reason.as_deref(),
            Some("capability_not_authorized")
        );
    }

    #[test]
    fn telemetry_sentinel_stays_unresolved_until_a_violated_source_joins() {
        let telemetry = "emilia/telemetry-v1:redundant_power_path";
        let grants = [Grant {
            id: "example/power-v1:curtail:ups-unit-01".into(),
            params: None,
            param_bounds: None,
            constraints: vec![telemetry.into()],
        }];
        let operation = Operation {
            id: "example/power-v1:curtail:ups-unit-01".into(),
            params: None,
        };
        let decision = evaluate_authorization(&grants, Some(&operation));
        assert_eq!(decision.verdict, Verdict::AllowUnresolved);
        assert_eq!(decision.unresolved, vec![telemetry.to_string()]);

        let denied = resolve(
            &decision,
            &[Resolution {
                constraint: telemetry.into(),
                status: LatticeState::Violated,
            }],
            None,
        );
        assert_eq!(denied.verdict, Verdict::Deny);
        assert_eq!(
            denied.reason.as_deref(),
            Some("redundant_power_path:violated")
        );
    }
}
