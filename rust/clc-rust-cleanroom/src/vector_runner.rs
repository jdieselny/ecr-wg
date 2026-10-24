//! JSON vector suite parser, batch executor, and JUnit / JSON reporter.
//!
//! Published containment vectors use `parent_source` / `child_source`.
//! Authorization vectors use `grants` / `operation`. Resolve vectors carry
//! a decision, resolutions, and an optional `now`.

use crate::error::Reason;
use crate::evaluator::{self, Decision, Grant, Operation, Resolution, Verdict};
use crate::grammar;
use crate::lattice::LatticeState;
use crate::parameters;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorOutcome {
    pub id: String,
    pub kind: String,
    pub passed: bool,
    pub detail: String,
    pub elapsed: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SuiteReport {
    pub outcomes: Vec<VectorOutcome>,
}

impl SuiteReport {
    pub fn passed(&self) -> usize {
        self.outcomes.iter().filter(|item| item.passed).count()
    }

    pub fn failed(&self) -> usize {
        self.outcomes.len() - self.passed()
    }

    pub fn merge(&mut self, other: SuiteReport) {
        self.outcomes.extend(other.outcomes);
    }
}

pub fn run_vector_file(path: &Path) -> Result<SuiteReport, Reason> {
    let text = fs::read_to_string(path).map_err(|_| Reason::InvalidResolution)?;
    run_vector_text(&text, path)
}

pub fn run_vector_text(text: &str, path: &Path) -> Result<SuiteReport, Reason> {
    let value: Value = serde_json::from_str(text).map_err(|_| Reason::InvalidParamsNumber)?;
    let vectors = match &value {
        Value::Array(items) => items.clone(),
        Value::Object(map) => {
            if let Some(Value::Array(items)) = map.get("vectors") {
                items.clone()
            } else {
                vec![value]
            }
        }
        _ => return Err(Reason::InvalidParamsNumber),
    };
    let mut report = SuiteReport::default();
    for (index, vector) in vectors.into_iter().enumerate() {
        report.outcomes.push(execute_one(&vector, path, index));
    }
    Ok(report)
}

pub fn run_vector_dir(path: &Path) -> Result<SuiteReport, Reason> {
    let mut files: Vec<_> = fs::read_dir(path)
        .map_err(|_| Reason::InvalidResolution)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect();
    files.sort();
    let mut report = SuiteReport::default();
    for file in files {
        report.merge(run_vector_file(&file)?);
    }
    Ok(report)
}

pub fn report_json(report: &SuiteReport) -> String {
    let results: Vec<Value> = report
        .outcomes
        .iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id,
                "kind": item.kind,
                "passed": item.passed,
                "detail": item.detail,
                "elapsed_us": item.elapsed.as_micros(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "spec": "draft-wei-capability-language-core",
        "revision": "CLC-1.19",
        "passed": report.passed(),
        "failed": report.failed(),
        "results": results,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn report_junit(report: &SuiteReport) -> String {
    let mut body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="clc-rust-cleanroom" tests="{tests}" failures="{failures}">
"#,
        tests = report.outcomes.len(),
        failures = report.failed(),
    );
    for item in &report.outcomes {
        body.push_str(&format!(
            "  <testcase name=\"{}\" classname=\"{}\" time=\"{:.6}\">\n",
            xml_escape(&item.id),
            xml_escape(&item.kind),
            item.elapsed.as_secs_f64(),
        ));
        if !item.passed {
            body.push_str(&format!(
                "    <failure message=\"{}\"/>\n",
                xml_escape(&item.detail)
            ));
        }
        body.push_str("  </testcase>\n");
    }
    body.push_str("</testsuite>\n");
    body
}

pub fn report_text(report: &SuiteReport) -> String {
    let mut lines = vec![
        "\u{1b}[1mCLC cleanroom\u{1b}[0m  draft-wei-capability-language-core  CLC-1.19".to_string(),
    ];
    for item in &report.outcomes {
        let (color, word) = if item.passed {
            ("\u{1b}[32m", "PASS")
        } else {
            ("\u{1b}[31m", "FAIL")
        };
        lines.push(format!(
            "{color}{word}\u{1b}[0m  {id}  {kind}  {us:.3} ms  {detail}",
            id = item.id,
            kind = item.kind,
            us = item.elapsed.as_secs_f64() * 1_000.0,
            detail = item.detail,
        ));
    }
    let summary_color = if report.failed() == 0 {
        "\u{1b}[32m"
    } else {
        "\u{1b}[33m"
    };
    lines.push(format!(
        "{summary_color}{passed} passed, {failed} failed\u{1b}[0m",
        passed = report.passed(),
        failed = report.failed(),
    ));
    lines.join("\n")
}

fn execute_one(vector: &Value, path: &Path, index: usize) -> VectorOutcome {
    let started = Instant::now();
    let id = vector
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "{}#{index}",
                path.file_stem().and_then(|s| s.to_str()).unwrap_or("vector")
            )
        });
    let kind = vector
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or(if vector.get("parent_source").is_some() {
            "contain-crosswalk"
        } else if vector.get("resolutions").is_some() {
            "resolve"
        } else {
            "authorize"
        })
        .to_string();
    let result = match kind.as_str() {
        "contain-crosswalk" | "contains" => run_contains(vector),
        "resolve" => run_resolve(vector),
        "syntax" => run_syntax(vector),
        "entail" => run_entail(vector),
        "intersect" => run_intersect(vector),
        "decide" => run_decide(vector),
        _ => run_authorize(vector),
    };
    let elapsed = started.elapsed();
    match result {
        Ok(detail) => VectorOutcome {
            id,
            kind,
            passed: true,
            detail,
            elapsed,
        },
        Err(detail) => VectorOutcome {
            id,
            kind,
            passed: false,
            detail,
            elapsed,
        },
    }
}

fn run_contains(vector: &Value) -> Result<String, String> {
    let parent = source_grant(vector.get("parent_source"))?;
    let child = source_grant(vector.get("child_source"))?;
    let result = evaluator::evaluate_containment(&parent, &child);
    let expect = vector.get("expect").cloned().unwrap_or(Value::Null);
    if let Some(flag) = expect.get("contains").and_then(Value::as_bool) {
        if result.contains != flag {
            return Err(format!(
                "contains={} reason={}",
                result.contains,
                result.reason.unwrap_or_default()
            ));
        }
    } else if !result.contains {
        return Err(result.reason.unwrap_or_else(|| "contains false".into()));
    }
    Ok(if result.contains {
        "contains=true".into()
    } else {
        format!("contains=false {}", result.reason.unwrap_or_default())
    })
}

fn run_syntax(vector: &Value) -> Result<String, String> {
    let id = vector
        .get("request")
        .and_then(|request| request.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    match grammar::parse_capability_id(id) {
        Ok(_) => judge(vector, "valid", None, None),
        Err(reason) => {
            let code = reason.code();
            judge(vector, "invalid", Some(&code), None)
        }
    }
}

fn run_entail(vector: &Value) -> Result<String, String> {
    if let Some(code) = revision_denial(vector) {
        return judge(vector, "deny", Some(&code), None);
    }
    if let Err(code) = normalization_gate(vector) {
        return judge(vector, "deny", Some(&code), None);
    }
    let request_params = vector
        .get("request")
        .and_then(|request| request.get("params"))
        .cloned();
    let Some(grant) = vector.get("grant").and_then(as_grant) else {
        return judge(vector, "deny", Some("capability_not_authorized"), None);
    };
    let request_id = vector
        .get("request")
        .and_then(|request| request.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if let Err(reason) =
        parameters::validate_grant_params(grant.params.as_ref(), grant.param_bounds.as_ref())
    {
        return judge(vector, "deny", Some(&reason.code()), None);
    }
    if let Err(reason) = grammar::parse_capability_id(request_id) {
        return judge(vector, "deny", Some(&reason.code()), None);
    }
    if let Err(reason) = evaluator::entail_identifiers(&grant.id, request_id) {
        return judge(vector, "deny", Some(&reason.code()), None);
    }
    if let Err(reason) = parameters::entail_parameters(
        grant.params.as_ref(),
        grant.param_bounds.as_ref(),
        request_params.as_ref(),
    ) {
        return judge(vector, "deny", Some(&reason.code()), None);
    }
    let outcome = crate::constraints::evaluate_constraints(&grant.constraints, request_params.as_ref());
    if let Some(reason) = outcome.rejection {
        return judge(vector, "deny", Some(&reason.code()), None);
    }
    if outcome.unresolved.is_empty() {
        judge(vector, "allow", None, Some(&outcome.unresolved))
    } else {
        judge(
            vector,
            "allow_unresolved",
            None,
            Some(&outcome.unresolved),
        )
    }
}

fn run_intersect(vector: &Value) -> Result<String, String> {
    if let Some(code) = revision_denial(vector) {
        return finish(vector, "deny", Some(&code), None, None, None);
    }
    let grants = collect_grants(vector);
    match compute_intersection(&grants) {
        Ok((params, constraints)) => finish(
            vector,
            "allow",
            None,
            None,
            Some(&params),
            Some(&constraints),
        ),
        Err(code) => finish(vector, "deny", Some(&code), None, None, None),
    }
}

/// §7 Intersect over the grant set. Identifier narrowing is params-free.
/// An empty `params` object is the identity. The runner reports the meet;
/// it does not change the frozen engine.
fn compute_intersection(grants: &[Grant]) -> Result<(Value, Vec<String>), String> {
    if grants.is_empty() {
        return Err("absent_source".into());
    }
    if grants.iter().any(|grant| grant.param_bounds.is_some()) {
        return Err("param_bounds_meet_not_in_engine".into());
    }
    let _narrowest = narrow_identifier(grants)?;
    let params = meet_param_set(grants)?;
    Ok((params, union_constraints(grants)))
}

fn narrow_identifier(grants: &[Grant]) -> Result<String, String> {
    let mut winner = grammar::parse_capability_id(&grants[0].id).map_err(|reason| reason.code())?;
    for grant in &grants[1..] {
        let next = grammar::parse_capability_id(&grant.id).map_err(|reason| reason.code())?;
        let winner_covers = grammar::covers(&winner, &next).is_ok();
        let next_covers = grammar::covers(&next, &winner).is_ok();
        match (winner_covers, next_covers) {
            (false, false) => return Err("no_overlap".into()),
            (true, false) => winner = next,
            _ => {}
        }
    }
    Ok(winner.raw)
}

fn meet_param_set(grants: &[Grant]) -> Result<Value, String> {
    let mut acc: Option<serde_json::Map<String, Value>> = None;
    for grant in grants {
        let Some(params) = grant.params.as_ref() else {
            continue;
        };
        if params.is_null() {
            return Err("invalid_params_null".into());
        }
        let Value::Object(map) = params else {
            return Err("invalid_params_binding".into());
        };
        if map.is_empty() {
            continue;
        }
        acc = Some(match acc {
            None => map.clone(),
            Some(prev) => meet_param_maps(&prev, map)?,
        });
    }
    Ok(Value::Object(acc.unwrap_or_default()))
}

fn meet_param_maps(
    left: &serde_json::Map<String, Value>,
    right: &serde_json::Map<String, Value>,
) -> Result<serde_json::Map<String, Value>, String> {
    let mut keys: Vec<&String> = left.keys().chain(right.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut out = serde_json::Map::new();
    for key in keys {
        match (left.get(key), right.get(key)) {
            (Some(a), Some(b)) => {
                out.insert(key.clone(), meet_values(a, b)?);
            }
            (Some(a), None) => {
                out.insert(key.clone(), a.clone());
            }
            (None, Some(b)) => {
                out.insert(key.clone(), b.clone());
            }
            (None, None) => {}
        }
    }
    Ok(out)
}

fn meet_values(left: &Value, right: &Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            let left_n = a.as_f64().filter(|n| n.is_finite()).ok_or("invalid_params_number")?;
            let right_n = b.as_f64().filter(|n| n.is_finite()).ok_or("invalid_params_number")?;
            if left_n <= right_n {
                Ok(left.clone())
            } else {
                Ok(right.clone())
            }
        }
        (Value::String(a), Value::String(b)) if a == b => Ok(left.clone()),
        (Value::String(_), Value::String(_)) => Err("no_overlap".into()),
        (Value::Bool(a), Value::Bool(b)) if a == b => Ok(left.clone()),
        (Value::Bool(_), Value::Bool(_)) => Err("no_overlap".into()),
        (Value::Array(a), Value::Array(b)) => meet_enums(a, b),
        (Value::Object(a), Value::Object(b)) => meet_nested(a, b),
        (Value::Null, _) | (_, Value::Null) => Err("invalid_params_null".into()),
        _ => Err("no_overlap".into()),
    }
}

fn meet_enums(left: &[Value], right: &[Value]) -> Result<Value, String> {
    if left.is_empty() || right.is_empty() {
        return Err("empty_bound_denies_class".into());
    }
    let kept: Vec<Value> = left
        .iter()
        .filter(|item| right.iter().any(|other| values_match(item, other)))
        .cloned()
        .collect();
    if kept.is_empty() {
        return Err("no_overlap".into());
    }
    Ok(Value::Array(kept))
}

fn meet_nested(
    left: &serde_json::Map<String, Value>,
    right: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    if left.len() != right.len() || left.keys().any(|key| !right.contains_key(key)) {
        return Err("no_overlap".into());
    }
    let mut out = serde_json::Map::new();
    for (key, value) in left {
        out.insert(key.clone(), meet_values(value, &right[key])?);
    }
    Ok(Value::Object(out))
}

fn union_constraints(grants: &[Grant]) -> Vec<String> {
    let mut all = Vec::new();
    for grant in grants {
        all.extend(grant.constraints.iter().cloned());
    }
    all.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    all.dedup();
    all
}

fn values_match(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
            (Some(p), Some(q)) => p == q && p.is_finite(),
            _ => false,
        },
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| values_match(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|o| values_match(v, o)))
        }
        _ => false,
    }
}

fn run_decide(vector: &Value) -> Result<String, String> {
    if let Err(code) = normalization_gate(vector) {
        return judge(vector, "deny", Some(&code), None);
    }
    let grants = collect_grants(vector);
    let operation = vector.get("request").and_then(|request| {
        if !request.is_object() {
            return None;
        }
        Some(Operation {
            id: request
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            params: request.get("params").cloned(),
        })
    });
    let revision = vector.get("clc_revision").and_then(Value::as_str);
    let decision =
        evaluator::evaluate_authorization_revision(&grants, operation.as_ref(), revision);
    judge(
        vector,
        verdict_name(decision.verdict),
        decision.reason.as_deref(),
        Some(&decision.unresolved),
    )
}

fn revision_denial(vector: &Value) -> Option<String> {
    let declared = vector.get("clc_revision").and_then(Value::as_str)?;
    grammar::check_clc_revision(declared)
        .err()
        .map(|reason| reason.code())
}

/// `raw_params` is the pre-decode request text (§6.2). A failure
/// short-circuits. A success leaves the decoded `request.params` in place:
/// those fields are the stand-in the vector uses once normalization passes.
fn normalization_gate(vector: &Value) -> Result<(), String> {
    let Some(raw) = vector.get("raw_params").and_then(Value::as_str) else {
        return Ok(());
    };
    parameters::prepare_params(raw).map(|_| ()).map_err(|reason| reason.code())
}

fn collect_grants(vector: &Value) -> Vec<Grant> {
    let mut grants = Vec::new();
    if let Some(grant) = vector.get("grant").and_then(as_grant) {
        grants.push(grant);
    }
    if let Some(Value::Array(others)) = vector.get("others") {
        for item in others {
            if let Some(grant) = as_grant(item) {
                grants.push(grant);
            }
        }
    }
    grants
}

fn as_grant(value: &Value) -> Option<Grant> {
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())?;
    Some(Grant {
        id: id.to_string(),
        params: value.get("params").cloned(),
        param_bounds: value.get("param_bounds").cloned(),
        constraints: string_list(value.get("constraints")),
    })
}

fn judge(
    vector: &Value,
    verdict: &str,
    reason: Option<&str>,
    unresolved: Option<&[String]>,
) -> Result<String, String> {
    finish(vector, verdict, reason, unresolved, None, None)
}

fn finish(
    vector: &Value,
    verdict: &str,
    reason: Option<&str>,
    unresolved: Option<&[String]>,
    result_params: Option<&Value>,
    result_constraints: Option<&[String]>,
) -> Result<String, String> {
    let expect = vector.get("expect").cloned().unwrap_or(Value::Null);
    let expected_verdict = expect.get("verdict").and_then(Value::as_str).unwrap_or("");
    let expected_reason = expect.get("reason").and_then(Value::as_str).unwrap_or("");
    let mut problems = Vec::new();
    if !expected_verdict.is_empty() && expected_verdict != verdict {
        problems.push(format!("verdict {verdict} != {expected_verdict}"));
    }
    if expect.get("reason").is_some() && reason != Some(expected_reason) {
        problems.push(format!(
            "reason {} != {expected_reason}",
            reason.unwrap_or("")
        ));
    }
    if let Some(list) = expect.get("unresolved").and_then(Value::as_array) {
        let expected = string_list(Some(&Value::Array(list.clone())));
        let actual = unresolved.unwrap_or(&[]);
        if actual != expected.as_slice() {
            problems.push(format!("unresolved {actual:?} != {expected:?}"));
        }
    }
    if let Some(expected) = expect.get("result_params") {
        match result_params {
            Some(actual) if values_match(actual, expected) => {}
            Some(actual) => problems.push(format!("result_params {actual} != {expected}")),
            None => problems.push("result_params missing".to_string()),
        }
    }
    if let Some(list) = expect.get("result_constraints").and_then(Value::as_array) {
        let expected = string_list(Some(&Value::Array(list.clone())));
        let actual = result_constraints.unwrap_or(&[]);
        if actual != expected.as_slice() {
            problems.push(format!("result_constraints {actual:?} != {expected:?}"));
        }
    }
    if problems.is_empty() {
        return Ok(match reason {
            Some(text) if !text.is_empty() => format!("{verdict} {text}"),
            _ => verdict.to_string(),
        });
    }
    Err(format!(
        "expected {expected_verdict}/{expected_reason}; actual {verdict}/{} ({})",
        reason.unwrap_or(""),
        problems.join("; ")
    ))
}

fn run_authorize(vector: &Value) -> Result<String, String> {
    let grants = parse_grants(vector.get("grants"))?;
    let operation = parse_operation(vector.get("operation").or_else(|| vector.get("requested_capability")))?;
    let decision = evaluator::evaluate_authorization(&grants, operation.as_ref());
    judge(
        vector,
        verdict_name(decision.verdict),
        decision.reason.as_deref(),
        Some(&decision.unresolved),
    )
}

fn run_resolve(vector: &Value) -> Result<String, String> {
    let decision = parse_decision(vector.get("decision"))?;
    let resolutions = parse_resolutions(vector.get("resolutions"))?;
    let now = vector.get("now").and_then(Value::as_str);
    let resolved = evaluator::resolve(&decision, &resolutions, now);
    let expect = vector.get("expect").cloned().unwrap_or(Value::Null);
    let verdict = expect
        .get("resolved_verdict")
        .or_else(|| expect.get("verdict"))
        .or_else(|| expect.get("static_verdict"))
        .and_then(Value::as_str);
    let mut judged = vector.clone();
    if let (Some(verdict), Some(object)) = (verdict, judged.get_mut("expect")) {
        if let Some(map) = object.as_object_mut() {
            map.insert("verdict".to_string(), Value::String(verdict.to_string()));
            if let Some(reason) = map.get("refusal_reason").cloned() {
                map.entry("reason").or_insert(reason);
            }
        }
    }
    judge(
        &judged,
        verdict_name(resolved.verdict),
        resolved.reason.as_deref(),
        Some(&resolved.unresolved),
    )
}

fn source_grant(source: Option<&Value>) -> Result<Grant, String> {
    let source = source.ok_or_else(|| "missing source".to_string())?;
    let caps = source
        .get("capabilities")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing capabilities".to_string())?;
    let cap = caps.first().ok_or_else(|| "empty capabilities".to_string())?;
    Ok(Grant {
        id: cap
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing capability id".to_string())?
            .to_string(),
        params: cap.get("params").cloned(),
        param_bounds: cap.get("param_bounds").cloned(),
        constraints: string_list(cap.get("constraints")),
    })
}

fn parse_grants(value: Option<&Value>) -> Result<Vec<Grant>, String> {
    let Some(Value::Array(items)) = value else {
        return Err("missing grants".into());
    };
    items
        .iter()
        .map(|item| {
            Ok(Grant {
                id: item
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "grant id".to_string())?
                    .to_string(),
                params: item.get("params").cloned(),
                param_bounds: item.get("param_bounds").cloned(),
                constraints: string_list(item.get("constraints")),
            })
        })
        .collect()
}

fn parse_operation(value: Option<&Value>) -> Result<Option<Operation>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(Operation {
        id: value
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "operation id".to_string())?
            .to_string(),
        params: value.get("params").cloned(),
    }))
}

fn parse_decision(value: Option<&Value>) -> Result<Decision, String> {
    let value = value.ok_or_else(|| "missing decision".to_string())?;
    let verdict = match value.get("verdict").and_then(Value::as_str).unwrap_or("") {
        "allow" => Verdict::Allow,
        "deny" => Verdict::Deny,
        "allow_unresolved" => Verdict::AllowUnresolved,
        _ => return Err("decision verdict".into()),
    };
    Ok(Decision {
        verdict,
        reason: value
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_string),
        unresolved: string_list(value.get("unresolved")),
        matched_grant_id: value
            .get("matched_grant_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        boundary_notices: Vec::new(),
    })
}

fn parse_resolutions(value: Option<&Value>) -> Result<Vec<Resolution>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Value::Array(items) = value else {
        return Err("resolutions".into());
    };
    items
        .iter()
        .map(|item| {
            let constraint = item
                .get("constraint")
                .and_then(Value::as_str)
                .ok_or_else(|| "resolution constraint".to_string())?
                .to_string();
            let status = LatticeState::parse(
                item.get("status")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "resolution status".to_string())?,
            )
            .ok_or_else(|| "resolution status".to_string())?;
            Ok(Resolution { constraint, status })
        })
        .collect()
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Allow => "allow",
        Verdict::Deny => "deny",
        Verdict::AllowUnresolved => "allow_unresolved",
    }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn published() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("vectors/ccx-iicp-001.json")
    }

    #[test]
    fn published_ccx_vector_passes_and_is_under_a_millisecond() {
        let report = run_vector_file(&published()).unwrap();
        assert_eq!(report.failed(), 0, "{}", report_text(&report));
        assert_eq!(report.outcomes[0].id, "ccx-iicp-001");
        assert!(report.outcomes[0].elapsed < Duration::from_millis(1));

        let sample = fs::read_to_string(published()).unwrap();
        let value: Value = serde_json::from_str(&sample).unwrap();
        let vector = &value.as_array().unwrap()[0];
        let started = Instant::now();
        for _ in 0..1000 {
            let _ = execute_one(vector, Path::new("ccx-iicp-001.json"), 0);
        }
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_millis(1000),
            "1000 evaluations took {elapsed:?}"
        );
    }

    #[test]
    fn reporters_emit_json_and_junit() {
        let report = run_vector_file(&published()).unwrap();
        let json: Value = serde_json::from_str(&report_json(&report)).unwrap();
        assert_eq!(json["revision"], "CLC-1.19");
        assert_eq!(json["failed"], 0);
        let junit = report_junit(&report);
        assert!(junit.contains("<testcase name=\"ccx-iicp-001\""));
        assert!(junit.contains("failures=\"0\""));
        let dir = run_vector_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("vectors")).unwrap();
        let ccx = dir
            .outcomes
            .iter()
            .find(|item| item.id == "ccx-iicp-001")
            .unwrap();
        assert!(ccx.passed, "{}", ccx.detail);
        let core = dir
            .outcomes
            .iter()
            .filter(|item| {
                matches!(
                    item.kind.as_str(),
                    "syntax" | "entail" | "intersect" | "decide"
                )
            })
            .count();
        assert_eq!(core, 154);
    }
}
