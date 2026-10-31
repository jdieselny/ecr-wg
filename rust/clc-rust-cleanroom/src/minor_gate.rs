//! CLC-1.19 §12.1 minor-gate profile.
//!
//! Parameter-bound entailment, §6.6 bound meet, constraint-union collation,
//! scheme declarations, quantity and string domains, and the optional
//! material-field symmetry check. Decisions come from the pinned vectors'
//! own `spec_clause` and `derivation` fields.

use crate::evaluator;
use crate::grammar;
use crate::parameters;
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct PropertyVerdict {
    pub passed: bool,
    pub detail: String,
    pub commutativity_verified: bool,
    pub full_source_coverage: bool,
}

#[derive(Clone)]
struct Bound {
    min: Option<Value>,
    max: Option<Value>,
    step: Option<Value>,
    enumeration: Option<Vec<Value>>,
    min_items: Option<Value>,
    max_items: Option<Value>,
    nested: Option<BTreeMap<String, Bound>>,
    optional: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Empty,
    Numeric,
    Enum,
    Nested,
}

struct Meet {
    params: Option<Value>,
    param_bounds: Option<Value>,
}

pub(crate) fn is_string_domain(vector: &Value) -> bool {
    vector.get("domain").and_then(Value::as_str).is_some()
        && vector.get("grant").is_some_and(Value::is_string)
        && vector.get("op").is_some_and(Value::is_string)
}

pub(crate) fn is_scheme_declaration(vector: &Value) -> bool {
    vector.get("declaration").is_some_and(Value::is_object)
}

pub(crate) fn is_quantity(vector: &Value) -> bool {
    fn amount(value: &Value) -> bool {
        value
            .as_object()
            .is_some_and(|map| map.contains_key("value") && map.contains_key("unit"))
    }
    fn params_of(slot: Option<&Value>) -> bool {
        slot.and_then(|value| value.get("params"))
            .and_then(Value::as_object)
            .is_some_and(|map| map.values().any(amount))
    }
    if params_of(vector.get("operation"))
        || params_of(vector.get("request"))
        || params_of(vector.get("parent"))
        || params_of(vector.get("child"))
        || params_of(vector.get("grant"))
    {
        return true;
    }
    vector
        .get("grants")
        .and_then(Value::as_array)
        .is_some_and(|grants| grants.iter().any(|grant| params_of(Some(grant))))
}

pub(crate) fn run_param_bounds(vector: &Value) -> Result<String, String> {
    if let Some(code) = revision_code(vector) {
        return judge_decision(vector, "deny", Some(&code), None);
    }
    if let Some(raw) = vector.get("raw_params").and_then(Value::as_str) {
        if let Err(reason) = parameters::prepare_params(raw) {
            return judge_decision(vector, "deny", Some(&reason.code()), None);
        }
    }
    let grant = vector.get("grant").ok_or_else(|| "missing grant".to_string())?;
    let request = vector
        .get("request")
        .ok_or_else(|| "missing request".to_string())?;
    let grant_params = grant.get("params");
    let grant_bounds = grant.get("param_bounds");
    if let Err(reason) = parameters::validate_grant_params(grant_params, grant_bounds) {
        return judge_decision(vector, "deny", Some(&reason.code()), None);
    }
    let grant_id = grant.get("id").and_then(Value::as_str).unwrap_or("");
    let request_id = request.get("id").and_then(Value::as_str).unwrap_or("");
    if let Err(reason) = evaluator::entail_identifiers(grant_id, request_id) {
        return judge_decision(vector, "deny", Some(&reason.code()), None);
    }
    let op_params = materialize_defaults(grant, request, vector.get("scheme_defaults"))?;
    if let Err(reason) =
        parameters::entail_parameters(grant_params, grant_bounds, Some(&op_params))
    {
        return judge_decision(vector, "deny", Some(&reason.code()), None);
    }
    let constraints = string_list(grant.get("constraints"));
    let outcome = crate::constraints::evaluate_constraints(&constraints, Some(&op_params));
    if let Some(reason) = outcome.rejection {
        return judge_decision(vector, "deny", Some(&reason.code()), None);
    }
    if outcome.unresolved.is_empty() {
        judge_decision(vector, "allow", None, Some(&outcome.unresolved))
    } else {
        judge_decision(
            vector,
            "allow_unresolved",
            None,
            Some(&outcome.unresolved),
        )
    }
}

pub(crate) fn run_param_bounds_meet(vector: &Value) -> Result<String, String> {
    if let Some(code) = revision_code(vector) {
        return judge_decision(vector, "deny", Some(&code), None);
    }
    let sources = vector
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing sources".to_string())?;
    match meet_sources(sources) {
        Ok(met) => judge_meet(vector, &met),
        Err(code) => {
            if vector.get("expect").and_then(|e| e.get("reason")).is_some() {
                judge_decision(vector, "deny", Some(&code), None)
            } else {
                Err(format!("meet denied {code}"))
            }
        }
    }
}

pub(crate) fn run_constraint_union(vector: &Value) -> Result<String, String> {
    if let Some(code) = revision_code(vector) {
        return judge_decision(vector, "deny", Some(&code), None);
    }
    let chain = vector
        .get("chain")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing chain".to_string())?;
    if chain.is_empty() {
        return judge_decision(vector, "deny", Some("absent_source"), None);
    }
    let mut all = Vec::new();
    for hop in chain {
        if let Some(items) = hop.get("constraints").and_then(Value::as_array) {
            for item in items {
                if let Some(text) = item.as_str() {
                    all.push(text.to_string());
                }
            }
        }
    }
    all.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    all.dedup();
    let expect = vector.get("expect").cloned().unwrap_or(Value::Null);
    if let Some(reason) = expect.get("reason").and_then(Value::as_str) {
        return Err(format!("union produced {all:?}; expected {reason}"));
    }
    let Some(expected) = expect.get("union").and_then(Value::as_array) else {
        return Err("missing expect.union".into());
    };
    let expected: Vec<String> = expected
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    if all == expected {
        Ok(format!("union {}", all.len()))
    } else {
        Err(format!("union {all:?} != {expected:?}"))
    }
}

pub(crate) fn run_scheme(vector: &Value) -> Result<String, String> {
    let declaration = vector
        .get("declaration")
        .ok_or_else(|| "missing declaration".to_string())?;
    let expect = vector
        .get("expect")
        .and_then(Value::as_str)
        .unwrap_or("");
    match check_declaration(declaration) {
        Ok(()) => {
            if expect == "accept" {
                Ok("accept".into())
            } else {
                Err(format!("declaration accepted; expected {expect}"))
            }
        }
        Err(code) => {
            let wanted = vector.get("code").and_then(Value::as_str).unwrap_or("");
            if expect == "reject" && code == wanted {
                Ok(format!("reject {code}"))
            } else {
                Err(format!("code {code} != {wanted} (expect {expect})"))
            }
        }
    }
}

pub(crate) fn run_quantity(vector: &Value) -> Result<String, String> {
    let kind = vector.get("kind").and_then(Value::as_str).unwrap_or("authorize");
    if kind == "contains" {
        let parent = vector.get("parent").ok_or_else(|| "missing parent".to_string())?;
        let child = vector.get("child").ok_or_else(|| "missing child".to_string())?;
        let parent_id = parent.get("id").and_then(Value::as_str).unwrap_or("");
        let child_id = child.get("id").and_then(Value::as_str).unwrap_or("");
        if let Err(reason) = evaluator::entail_identifiers(parent_id, child_id) {
            return judge_contains(vector, false, Some(&reason.code()));
        }
        return match quantity_params(parent.get("params"), child.get("params"), true) {
            Ok(()) => judge_contains(vector, true, None),
            Err(code) => judge_contains(vector, false, Some(&code)),
        };
    }
    let operation = vector
        .get("operation")
        .ok_or_else(|| "missing operation".to_string())?;
    let op_id = operation.get("id").and_then(Value::as_str).unwrap_or("");
    if op_id.is_empty() {
        return judge_decision(vector, "deny", Some("missing_capability_id"), None);
    }
    let grants = vector
        .get("grants")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing grants".to_string())?;
    if grants.is_empty() {
        return judge_decision(vector, "deny", Some("capability_not_authorized"), None);
    }
    let mut denial = None;
    for grant in grants {
        let grant_id = grant.get("id").and_then(Value::as_str).unwrap_or("");
        if let Err(reason) = evaluator::entail_identifiers(grant_id, op_id) {
            if denial.is_none() {
                denial = Some(reason.code());
            }
            continue;
        }
        match quantity_params(grant.get("params"), operation.get("params"), false) {
            Ok(()) => return judge_decision(vector, "allow", None, None),
            Err(code) => {
                if denial.is_none() {
                    denial = Some(code);
                }
            }
        }
    }
    judge_decision(
        vector,
        "deny",
        Some(&denial.unwrap_or_else(|| "capability_not_authorized".into())),
        None,
    )
}

pub(crate) fn run_string_domain(vector: &Value) -> Result<String, String> {
    let domain = vector.get("domain").and_then(Value::as_str).unwrap_or("");
    let grant = vector
        .get("grant")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing grant".to_string())?;
    let op = vector
        .get("op")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing op".to_string())?;
    let decision = match domain {
        "decimal-string" => decimal_within(grant, op),
        "subdomain" => subdomain_within(grant, op),
        "path-prefix" => path_within(grant, op),
        _ => Err("invalid_params_binding".into()),
    };
    match decision {
        Ok(()) => judge_decision(vector, "allow", None, None),
        Err(code) => judge_decision(vector, "deny", Some(&code), None),
    }
}

pub(crate) fn run_symmetry(vector: &Value) -> Result<String, String> {
    let actual = symmetry_match(vector);
    let expect = vector
        .get("expect")
        .and_then(|value| value.get("match"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if actual == expect {
        Ok(actual)
    } else {
        Err(format!("match {actual} != {expect}"))
    }
}

pub(crate) fn check_bound_meet_property(case: &Value, ops: &[Value]) -> PropertyVerdict {
    let Some(sources) = case.get("sources").and_then(Value::as_array) else {
        return property_fail("missing sources array");
    };
    if sources.is_empty() {
        return property_fail("empty sources");
    }
    let forward = meet_sources(sources);
    let mut reversed = sources.clone();
    reversed.reverse();
    let backward = meet_sources(&reversed);
    match (forward, backward) {
        (Ok(left), Ok(right)) if meet_eq(&left, &right) => {
            let mut widened = None;
            for op in ops {
                if !entails_meet(&left, op) {
                    continue;
                }
                for source in sources {
                    if !entails_source(source, op) {
                        widened = Some(format!(
                            "meet admits {} which a source denies",
                            op.get("id").and_then(Value::as_str).unwrap_or("op")
                        ));
                        break;
                    }
                }
                if widened.is_some() {
                    break;
                }
            }
            if let Some(detail) = widened {
                PropertyVerdict {
                    passed: false,
                    detail,
                    commutativity_verified: true,
                    full_source_coverage: false,
                }
            } else {
                PropertyVerdict {
                    passed: true,
                    detail: "allow".into(),
                    commutativity_verified: true,
                    full_source_coverage: true,
                }
            }
        }
        (Err(left), Err(right)) if left == right && normative(&left) => PropertyVerdict {
            passed: true,
            detail: format!("deny {left}"),
            commutativity_verified: true,
            full_source_coverage: true,
        },
        (Err(left), Err(right)) if left == right => PropertyVerdict {
            passed: false,
            detail: format!("non-normative reason {left}"),
            commutativity_verified: true,
            full_source_coverage: false,
        },
        (Ok(_), Ok(_)) => property_fail("commutativity mismatch on meet value"),
        (Ok(_), Err(code)) => property_fail(&format!("forward allowed, reverse denied {code}")),
        (Err(code), Ok(_)) => property_fail(&format!("forward denied {code}, reverse allowed")),
        (Err(left), Err(right)) => {
            property_fail(&format!("commutativity mismatch: {left} != {right}"))
        }
    }
}

fn property_fail(detail: &str) -> PropertyVerdict {
    PropertyVerdict {
        passed: false,
        detail: detail.to_string(),
        commutativity_verified: false,
        full_source_coverage: false,
    }
}

fn normative(code: &str) -> bool {
    matches!(
        code,
        "absent_source"
            | "no_overlap"
            | "invalid_params_binding"
            | "empty_bound_denies_class"
            | "invalid_params_null"
            | "invalid_params_number"
            | "invalid_params_size"
            | "invalid_params_duplicate_key"
            | "params_missing"
            | "undeclared_param"
            | "params_exceed_grant"
            | "params_out_of_range"
            | "params_not_multiple"
            | "not_in_enum"
            | "params_cardinality"
            | "capability_not_authorized"
            | "different_namespace"
            | "literal_mismatch"
            | "unsupported_wildcard"
            | "invalid_capability_id"
            | "missing_capability_id"
            | "wildcard_requires_trailing_segment"
            | "unsupported_language_revision"
            | "child_exceeds_parent"
            | "params_not_narrower"
    )
}

fn entails_meet(met: &Meet, op: &Value) -> bool {
    parameters::entail_parameters(met.params.as_ref(), met.param_bounds.as_ref(), op.get("params"))
        .is_ok()
}

fn entails_source(source: &Value, op: &Value) -> bool {
    parameters::entail_parameters(source.get("params"), source.get("param_bounds"), op.get("params"))
        .is_ok()
}

fn meet_eq(left: &Meet, right: &Meet) -> bool {
    match (&left.params, &right.params) {
        (None, None) => {}
        (Some(a), Some(b)) if values_eq(a, b) => {}
        _ => return false,
    }
    match (&left.param_bounds, &right.param_bounds) {
        (None, None) => true,
        (Some(a), Some(b)) => values_eq(a, b),
        _ => false,
    }
}

/// Chain intersection for the CLC-D `authorize-chain` corpus (§13.11).
///
/// AuthorizeWithChain authorizes the request against the *effective* grant,
/// not the leaf: the §6.6 meet of every grant in the chain. `ac-009` is the
/// case that pins this down — the child carries no constraint, so authorizing
/// against the leaf alone would allow a request the ancestor forbids.
///
/// Returns `(params, param_bounds)` already met across the whole chain.
pub(crate) fn chain_intersection(
    sources: &[Value],
) -> Result<(Option<Value>, Option<Value>), String> {
    let met = meet_sources(sources)?;
    Ok((met.params, met.param_bounds))
}

fn meet_sources(sources: &[Value]) -> Result<Meet, String> {
    if sources.is_empty() {
        return Err("absent_source".into());
    }
    let mut ids = Vec::new();
    for source in sources {
        let id = source
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "invalid_capability_id".to_string())?;
        ids.push(id.to_string());
    }
    identifiers_overlap(&ids)?;
    params_bounds_clash(sources)?;
    let params = meet_params(sources)?;
    let mut bound_maps = Vec::new();
    for source in sources {
        match source.get("param_bounds") {
            None => bound_maps.push(BTreeMap::new()),
            Some(value) => bound_maps.push(parse_bound_map(value)?),
        }
    }
    if bound_maps.iter().any(|map| map.values().any(contains_empty_enum)) {
        return Err("empty_bound_denies_class".into());
    }
    let met = meet_bound_maps(&bound_maps)?;
    let param_bounds = if met.is_empty() {
        None
    } else {
        Some(emit_bound_map(&met))
    };
    Ok(Meet {
        params,
        param_bounds,
    })
}

fn identifiers_overlap(ids: &[String]) -> Result<(), String> {
    let mut winner = grammar::parse_capability_id(&ids[0]).map_err(|reason| reason.code())?;
    for id in &ids[1..] {
        let next = grammar::parse_capability_id(id).map_err(|reason| reason.code())?;
        let winner_covers = grammar::covers(&winner, &next).is_ok();
        let next_covers = grammar::covers(&next, &winner).is_ok();
        match (winner_covers, next_covers) {
            (false, false) => return Err("no_overlap".into()),
            (true, false) => winner = next,
            _ => {}
        }
    }
    Ok(())
}

fn params_bounds_clash(sources: &[Value]) -> Result<(), String> {
    let mut params = BTreeSet::new();
    let mut bounds = BTreeSet::new();
    for source in sources {
        if let Some(map) = source.get("params").and_then(Value::as_object) {
            if !map.is_empty() {
                params.extend(map.keys().cloned());
            }
        }
        if let Some(map) = source.get("param_bounds").and_then(Value::as_object) {
            bounds.extend(map.keys().cloned());
        }
    }
    if params.intersection(&bounds).next().is_some() {
        Err("invalid_params_binding".into())
    } else {
        Ok(())
    }
}

fn meet_params(sources: &[Value]) -> Result<Option<Value>, String> {
    let mut acc: Option<Map<String, Value>> = None;
    for source in sources {
        let Some(params) = source.get("params") else {
            continue;
        };
        if params.is_null() {
            return Err("invalid_params_null".into());
        }
        let Some(map) = params.as_object() else {
            return Err("invalid_params_binding".into());
        };
        if map.is_empty() {
            continue;
        }
        deny_empty_param_value(map)?;
        acc = Some(match acc {
            None => map.clone(),
            Some(prev) => meet_param_maps(&prev, map)?,
        });
    }
    Ok(acc.map(Value::Object))
}

fn deny_empty_param_value(map: &Map<String, Value>) -> Result<(), String> {
    fn walk(value: &Value) -> bool {
        match value {
            Value::Array(items) if items.is_empty() => true,
            Value::Object(obj) if obj.is_empty() => true,
            Value::Array(items) => items.iter().any(walk),
            Value::Object(obj) => obj.values().any(walk),
            _ => false,
        }
    }
    if map.values().any(walk) {
        Err("empty_bound_denies_class".into())
    } else {
        Ok(())
    }
}

fn meet_param_maps(
    left: &Map<String, Value>,
    right: &Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let mut keys: Vec<&String> = left.keys().chain(right.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut out = Map::new();
    for key in keys {
        match (left.get(key), right.get(key)) {
            (Some(a), Some(b)) => {
                out.insert(key.clone(), meet_param_values(a, b)?);
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

fn meet_param_values(left: &Value, right: &Value) -> Result<Value, String> {
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
        (Value::Array(a), Value::Array(b)) => {
            if a.is_empty() || b.is_empty() {
                return Err("empty_bound_denies_class".into());
            }
            let kept: Vec<Value> = a
                .iter()
                .filter(|item| b.iter().any(|other| json_eq(item, other)))
                .cloned()
                .collect();
            if kept.is_empty() {
                Err("no_overlap".into())
            } else {
                Ok(Value::Array(kept))
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() || a.keys().any(|key| !b.contains_key(key)) {
                return Err("no_overlap".into());
            }
            let mut out = Map::new();
            for (key, value) in a {
                out.insert(key.clone(), meet_param_values(value, &b[key])?);
            }
            Ok(Value::Object(out))
        }
        (Value::Null, _) | (_, Value::Null) => Err("invalid_params_null".into()),
        _ => Err("no_overlap".into()),
    }
}

fn parse_bound_map(value: &Value) -> Result<BTreeMap<String, Bound>, String> {
    if value.is_null() {
        return Err("invalid_params_null".into());
    }
    let Some(map) = value.as_object() else {
        return Err("invalid_params_binding".into());
    };
    let mut out = BTreeMap::new();
    for (key, raw) in map {
        out.insert(key.clone(), parse_bound(raw)?);
    }
    Ok(out)
}

fn parse_bound(value: &Value) -> Result<Bound, String> {
    let Some(map) = value.as_object() else {
        return Err("invalid_params_binding".into());
    };
    let mut bound = Bound {
        min: None,
        max: None,
        step: None,
        enumeration: None,
        min_items: None,
        max_items: None,
        nested: None,
        optional: false,
    };
    for (key, raw) in map {
        match key.as_str() {
            "min" => bound.min = Some(require_number(raw)?),
            "max" => bound.max = Some(require_number(raw)?),
            "step" => bound.step = Some(require_number(raw)?),
            "enum" => {
                let Some(items) = raw.as_array() else {
                    return Err("invalid_params_binding".into());
                };
                if items
                    .iter()
                    .any(|item| item.is_object() || item.is_array() || item.is_null())
                {
                    return Err("invalid_params_binding".into());
                }
                bound.enumeration = Some(items.clone());
            }
            "min_items" => bound.min_items = Some(require_u64_value(raw)?),
            "max_items" => bound.max_items = Some(require_u64_value(raw)?),
            "nested" => {
                if !raw.is_object() {
                    return Err("invalid_params_binding".into());
                }
                bound.nested = Some(parse_bound_map(raw)?);
            }
            "optional" => {
                let Some(flag) = raw.as_bool() else {
                    return Err("invalid_params_binding".into());
                };
                bound.optional = flag;
            }
            _ => return Err("invalid_params_binding".into()),
        }
    }
    let families = [
        bound.min.is_some() || bound.max.is_some() || bound.step.is_some(),
        bound.enumeration.is_some() || bound.min_items.is_some() || bound.max_items.is_some(),
        bound.nested.is_some(),
    ]
    .into_iter()
    .filter(|on| *on)
    .count();
    if families > 1 {
        return Err("invalid_params_binding".into());
    }
    if let (Some(min), Some(max)) = (&bound.min, &bound.max) {
        if as_f64(min)? > as_f64(max)? {
            return Err("invalid_params_binding".into());
        }
    }
    if let Some(step) = &bound.step {
        if as_f64(step)? <= 0.0 {
            return Err("invalid_params_binding".into());
        }
    }
    if let (Some(min), Some(max)) = (&bound.min_items, &bound.max_items) {
        if as_u64(min)? > as_u64(max)? {
            return Err("invalid_params_binding".into());
        }
    }
    Ok(bound)
}

fn require_number(value: &Value) -> Result<Value, String> {
    as_f64(value)?;
    Ok(value.clone())
}

fn require_u64_value(value: &Value) -> Result<Value, String> {
    as_u64(value)?;
    Ok(value.clone())
}

fn contains_empty_enum(bound: &Bound) -> bool {
    if bound.enumeration.as_ref().is_some_and(Vec::is_empty) {
        return true;
    }
    bound
        .nested
        .as_ref()
        .is_some_and(|nested| nested.values().any(contains_empty_enum))
}

fn meet_bound_maps(maps: &[BTreeMap<String, Bound>]) -> Result<BTreeMap<String, Bound>, String> {
    let mut acc: Option<BTreeMap<String, Bound>> = None;
    for map in maps {
        acc = Some(match acc {
            None => map.clone(),
            Some(prev) => {
                let mut keys = BTreeSet::new();
                keys.extend(prev.keys().cloned());
                keys.extend(map.keys().cloned());
                let mut next = BTreeMap::new();
                for key in keys {
                    match (prev.get(&key), map.get(&key)) {
                        (Some(left), Some(right)) => {
                            next.insert(key, meet_bound(left, right)?);
                        }
                        (Some(left), None) => {
                            next.insert(key, left.clone());
                        }
                        (None, Some(right)) => {
                            next.insert(key, right.clone());
                        }
                        (None, None) => {}
                    }
                }
                next
            }
        });
    }
    Ok(acc.unwrap_or_default())
}

fn meet_bound(left: &Bound, right: &Bound) -> Result<Bound, String> {
    if contains_empty_enum(left) || contains_empty_enum(right) {
        return Err("empty_bound_denies_class".into());
    }
    // `{}` is the identity for value families. It does not say `optional: true`,
    // so a key it declares stays required unless every source opts in.
    if family(left) == Family::Empty {
        let mut out = right.clone();
        out.optional = left.optional && right.optional;
        return Ok(out);
    }
    if family(right) == Family::Empty {
        let mut out = left.clone();
        out.optional = left.optional && right.optional;
        return Ok(out);
    }
    if family(left) != family(right) {
        return Err("invalid_params_binding".into());
    }
    let optional = left.optional && right.optional;
    if family(left) == Family::Numeric {
        meet_numeric(left, right, optional)
    } else if family(left) == Family::Enum {
        meet_enum_bound(left, right, optional)
    } else {
        meet_nested_bound(left, right, optional)
    }
}

fn family(bound: &Bound) -> Family {
    if bound.nested.is_some() {
        Family::Nested
    } else if bound.enumeration.is_some() || bound.min_items.is_some() || bound.max_items.is_some()
    {
        Family::Enum
    } else if bound.min.is_some() || bound.max.is_some() || bound.step.is_some() {
        Family::Numeric
    } else {
        Family::Empty
    }
}

fn meet_numeric(left: &Bound, right: &Bound, optional: bool) -> Result<Bound, String> {
    let min = match (&left.min, &right.min) {
        (Some(a), Some(b)) => Some(tighter_min(a, b)?),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    let max = match (&left.max, &right.max) {
        (Some(a), Some(b)) => Some(tighter_max(a, b)?),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    let step = match (&left.step, &right.step) {
        (Some(a), Some(b)) => Some(coarser_step(a, b)?),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    if let (Some(min), Some(max)) = (&min, &max) {
        if as_f64(min)? > as_f64(max)? {
            return Err("no_overlap".into());
        }
    }
    Ok(Bound {
        min,
        max,
        step,
        enumeration: None,
        min_items: None,
        max_items: None,
        nested: None,
        optional,
    })
}

fn meet_enum_bound(left: &Bound, right: &Bound, optional: bool) -> Result<Bound, String> {
    let enumeration = match (&left.enumeration, &right.enumeration) {
        (Some(a), Some(b)) => {
            let kept: Vec<Value> = a
                .iter()
                .filter(|item| b.iter().any(|other| json_eq(item, other)))
                .cloned()
                .collect();
            if kept.is_empty() {
                return Err("no_overlap".into());
            }
            Some(kept)
        }
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    let min_items = match (&left.min_items, &right.min_items) {
        (Some(a), Some(b)) => Some(tighter_min(a, b)?),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    let max_items = match (&left.max_items, &right.max_items) {
        (Some(a), Some(b)) => Some(tighter_max(a, b)?),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    if let (Some(min), Some(max)) = (&min_items, &max_items) {
        if as_u64(min)? > as_u64(max)? {
            return Err("no_overlap".into());
        }
    }
    Ok(Bound {
        min: None,
        max: None,
        step: None,
        enumeration,
        min_items,
        max_items,
        nested: None,
        optional,
    })
}

fn meet_nested_bound(left: &Bound, right: &Bound, optional: bool) -> Result<Bound, String> {
    let left_map = left.nested.as_ref().ok_or("invalid_params_binding")?;
    let right_map = right.nested.as_ref().ok_or("invalid_params_binding")?;
    if left_map.len() != right_map.len() || left_map.keys().any(|key| !right_map.contains_key(key)) {
        return Err("no_overlap".into());
    }
    let mut nested = BTreeMap::new();
    for (key, child) in left_map {
        nested.insert(key.clone(), meet_bound(child, &right_map[key])?);
    }
    Ok(Bound {
        min: None,
        max: None,
        step: None,
        enumeration: None,
        min_items: None,
        max_items: None,
        nested: Some(nested),
        optional,
    })
}

fn tighter_min(left: &Value, right: &Value) -> Result<Value, String> {
    if as_f64(left)? >= as_f64(right)? {
        Ok(left.clone())
    } else {
        Ok(right.clone())
    }
}

fn tighter_max(left: &Value, right: &Value) -> Result<Value, String> {
    if as_f64(left)? <= as_f64(right)? {
        Ok(left.clone())
    } else {
        Ok(right.clone())
    }
}

fn coarser_step(left: &Value, right: &Value) -> Result<Value, String> {
    let a = as_f64(left)?;
    let b = as_f64(right)?;
    if integer_multiple(a, b) {
        Ok(left.clone())
    } else if integer_multiple(b, a) {
        Ok(right.clone())
    } else {
        Err("invalid_params_binding".into())
    }
}

fn integer_multiple(value: f64, step: f64) -> bool {
    if step <= 0.0 || !value.is_finite() || !step.is_finite() {
        return false;
    }
    let quotient = value / step;
    quotient == quotient.floor() && quotient * step == value
}

fn emit_bound_map(map: &BTreeMap<String, Bound>) -> Value {
    let mut out = Map::new();
    for (key, bound) in map {
        out.insert(key.clone(), emit_bound(bound));
    }
    Value::Object(out)
}

fn emit_bound(bound: &Bound) -> Value {
    let mut out = Map::new();
    if let Some(value) = &bound.min {
        out.insert("min".into(), value.clone());
    }
    if let Some(value) = &bound.max {
        out.insert("max".into(), value.clone());
    }
    if let Some(value) = &bound.step {
        out.insert("step".into(), value.clone());
    }
    if let Some(items) = &bound.enumeration {
        out.insert("enum".into(), Value::Array(items.clone()));
    }
    if let Some(value) = &bound.min_items {
        out.insert("min_items".into(), value.clone());
    }
    if let Some(value) = &bound.max_items {
        out.insert("max_items".into(), value.clone());
    }
    if let Some(nested) = &bound.nested {
        out.insert("nested".into(), emit_bound_map(nested));
    }
    if bound.optional {
        out.insert("optional".into(), Value::Bool(true));
    }
    Value::Object(out)
}

fn materialize_defaults(
    grant: &Value,
    request: &Value,
    defaults: Option<&Value>,
) -> Result<Value, String> {
    let mut params = match request.get("params") {
        None => Value::Object(Map::new()),
        Some(Value::Object(map)) => Value::Object(map.clone()),
        Some(other) => return Ok(other.clone()),
    };
    let Some(Value::Object(defs)) = defaults else {
        return Ok(params);
    };
    let mut declared = BTreeSet::new();
    if let Some(map) = grant.get("params").and_then(Value::as_object) {
        if !map.is_empty() {
            declared.extend(map.keys().cloned());
        }
    }
    if let Some(map) = grant.get("param_bounds").and_then(Value::as_object) {
        declared.extend(map.keys().cloned());
    }
    let Some(map) = params.as_object_mut() else {
        return Ok(params);
    };
    for (key, value) in defs {
        if map.contains_key(key) || !declared.contains(key) {
            continue;
        }
        map.insert(key.clone(), value.clone());
    }
    Ok(params)
}

fn revision_code(vector: &Value) -> Option<String> {
    let declared = vector.get("clc_revision").and_then(Value::as_str)?;
    grammar::check_clc_revision(declared)
        .err()
        .map(|reason| reason.code())
}

fn judge_meet(vector: &Value, met: &Meet) -> Result<String, String> {
    let expect = vector.get("expect").cloned().unwrap_or(Value::Null);
    let mut problems = Vec::new();
    match expect.get("param_bounds") {
        Some(expected) => match &met.param_bounds {
            Some(actual) if values_eq(actual, expected) => {}
            Some(actual) => problems.push(format!("param_bounds {actual} != {expected}")),
            None => problems.push("param_bounds missing".into()),
        },
        None => {
            if met.param_bounds.is_some() {
                problems.push("unexpected param_bounds".into());
            }
        }
    }
    match expect.get("params") {
        Some(expected) => match &met.params {
            Some(actual) if values_eq(actual, expected) => {}
            Some(actual) => problems.push(format!("params {actual} != {expected}")),
            None => problems.push("params missing".into()),
        },
        None => {
            if met.params.is_some() {
                problems.push("unexpected params".into());
            }
        }
    }
    if expect.get("reason").is_some() {
        problems.push("meet succeeded".into());
    }
    if problems.is_empty() {
        Ok("meet".into())
    } else {
        Err(problems.join("; "))
    }
}

fn judge_decision(
    vector: &Value,
    verdict: &str,
    reason: Option<&str>,
    unresolved: Option<&[String]>,
) -> Result<String, String> {
    let expect = vector.get("expect").cloned().unwrap_or(Value::Null);
    if expect.is_null() {
        return Ok(verdict.to_string());
    }
    let mut problems = Vec::new();
    if let Some(expected) = expect.get("verdict").and_then(Value::as_str) {
        if expected != verdict {
            problems.push(format!("verdict {verdict} != {expected}"));
        }
    }
    match expect.get("reason") {
        Some(Value::String(expected)) => {
            if reason != Some(expected.as_str()) {
                problems.push(format!("reason {} != {expected}", reason.unwrap_or("")));
            }
        }
        Some(Value::Null) => {
            if reason.is_some() {
                problems.push(format!("reason {} != null", reason.unwrap_or("")));
            }
        }
        Some(other) => problems.push(format!("reason shape {other}")),
        None => {}
    }
    if let Some(list) = expect.get("unresolved").and_then(Value::as_array) {
        let expected = string_list(Some(&Value::Array(list.clone())));
        let actual = unresolved.unwrap_or(&[]);
        if actual != expected.as_slice() {
            problems.push(format!("unresolved {actual:?} != {expected:?}"));
        }
    }
    if problems.is_empty() {
        Ok(match reason {
            Some(text) if !text.is_empty() => format!("{verdict} {text}"),
            _ => verdict.to_string(),
        })
    } else {
        Err(problems.join("; "))
    }
}

fn judge_contains(vector: &Value, contains: bool, reason: Option<&str>) -> Result<String, String> {
    let expect = vector.get("expect").cloned().unwrap_or(Value::Null);
    let mut problems = Vec::new();
    if let Some(flag) = expect.get("contains").and_then(Value::as_bool) {
        if flag != contains {
            problems.push(format!("contains {contains} != {flag}"));
        }
    }
    if let Some(expected) = expect.get("reason").and_then(Value::as_str) {
        if reason != Some(expected) {
            problems.push(format!("reason {} != {expected}", reason.unwrap_or("")));
        }
    } else if reason.is_some() && expect.get("reason").is_some() {
        problems.push(format!("reason {}", reason.unwrap_or("")));
    }
    if problems.is_empty() {
        Ok(if contains {
            "contains=true".into()
        } else {
            format!("contains=false {}", reason.unwrap_or(""))
        })
    } else {
        Err(problems.join("; "))
    }
}

fn check_declaration(declaration: &Value) -> Result<(), String> {
    let Some(map) = declaration.as_object() else {
        return Err("invalid_declaration".into());
    };
    for key in map.keys() {
        if !matches!(
            key.as_str(),
            "@version" | "scheme" | "declaration_id" | "clc_revision" | "capabilities" | "digest"
        ) {
            return Err("invalid_declaration".into());
        }
    }
    if map.get("@version").and_then(Value::as_str) != Some("CLC-SCHEME-v1") {
        return Err("invalid_declaration".into());
    }
    match map.get("scheme").and_then(Value::as_str) {
        Some(text) if !text.is_empty() => {}
        _ => return Err("invalid_declaration".into()),
    }
    match map.get("declaration_id").and_then(Value::as_str) {
        Some(text) if !text.is_empty() => {}
        _ => return Err("invalid_declaration".into()),
    }
    match map.get("clc_revision").and_then(Value::as_str) {
        Some(text) if !text.is_empty() => {}
        _ => return Err("invalid_declaration".into()),
    }
    let Some(capabilities) = map.get("capabilities").and_then(Value::as_object) else {
        return Err("invalid_declaration".into());
    };
    if capabilities.is_empty() {
        return Err("invalid_declaration".into());
    }
    for capability in capabilities.values() {
        check_capability(capability)?;
    }
    if let Some(digest) = map.get("digest") {
        let Some(text) = digest.as_str() else {
            return Err("invalid_declaration".into());
        };
        let mut body = map.clone();
        body.remove("digest");
        let canonical = jcs(&Value::Object(body));
        let hash = Sha256::digest(canonical.as_bytes());
        let computed = format!("sha256:{}", hex_encode(hash.as_ref()));
        if text != computed {
            return Err("declaration_digest_mismatch".into());
        }
    }
    Ok(())
}

fn check_capability(capability: &Value) -> Result<(), String> {
    let Some(map) = capability.as_object() else {
        return Err("invalid_declaration".into());
    };
    if map.len() != 1 || !map.contains_key("params") {
        return Err("invalid_declaration".into());
    }
    let Some(params) = map.get("params").and_then(Value::as_object) else {
        return Err("invalid_declaration".into());
    };
    if params.is_empty() {
        return Err("invalid_declaration".into());
    }
    for spec in params.values() {
        check_domain(spec)?;
    }
    Ok(())
}

fn check_domain(spec: &Value) -> Result<(), String> {
    let Some(map) = spec.as_object() else {
        return Err("invalid_declaration".into());
    };
    let Some(domain) = map.get("domain").and_then(Value::as_str) else {
        return Err("invalid_declaration".into());
    };
    let known = matches!(
        domain,
        "bound" | "enum" | "quantity" | "subdomain" | "path-prefix" | "decimal-string"
    );
    if !known {
        return Err("unsupported_domain".into());
    }
    for key in map.keys() {
        let allowed = key == "domain" || (domain == "quantity" && key == "unit");
        if !allowed {
            return Err("invalid_declaration".into());
        }
    }
    if domain == "quantity" {
        match map.get("unit").and_then(Value::as_str) {
            Some(unit) if !unit.is_empty() => {}
            _ => return Err("invalid_declaration".into()),
        }
    }
    Ok(())
}

fn decimal_within(grant: &str, op: &str) -> Result<(), String> {
    let grant = parse_decimal(grant).ok_or_else(|| "invalid_params_binding".to_string())?;
    let op = parse_decimal(op).ok_or_else(|| "invalid_params_binding".to_string())?;
    if compare_decimal(&op, &grant) != Ordering::Greater {
        Ok(())
    } else {
        Err("params_exceed_grant".into())
    }
}

struct Decimal {
    neg: bool,
    coef: Vec<u8>,
    exp: i32,
}

fn parse_decimal(input: &str) -> Option<Decimal> {
    if input.is_empty()
        || input
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '_' || ch == '\u{00a0}')
    {
        return None;
    }
    let (neg, rest) = match input.as_bytes().first()? {
        b'+' => (false, &input[1..]),
        b'-' => (true, &input[1..]),
        _ => (false, input),
    };
    if rest.is_empty() {
        return None;
    }
    let (body, exp_extra) = split_exponent(rest)?;
    if body.is_empty() || body.starts_with('.') || body.ends_with('.') {
        return None;
    }
    let (int_part, frac_part) = if let Some((int_part, frac_part)) = body.split_once('.') {
        if int_part.is_empty() || frac_part.is_empty() {
            return None;
        }
        (int_part, frac_part)
    } else {
        (body, "")
    };
    if !int_part.bytes().all(|b| b.is_ascii_digit()) || !frac_part.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let exp = exp_extra.checked_sub(i32::try_from(frac_part.len()).ok()?)?;
    let mut digits = String::new();
    digits.push_str(int_part);
    digits.push_str(frac_part);
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        return Some(Decimal {
            neg: false,
            coef: vec![0],
            exp: 0,
        });
    }
    Some(Decimal {
        neg,
        coef: trimmed.bytes().map(|b| b - b'0').collect(),
        exp,
    })
}

fn split_exponent(input: &str) -> Option<(&str, i32)> {
    let pos = input.bytes().position(|b| b == b'e' || b == b'E');
    let Some(pos) = pos else {
        return Some((input, 0));
    };
    let body = &input[..pos];
    let exp = &input[pos + 1..];
    if exp.is_empty() {
        return None;
    }
    let (sign, digits) = match exp.as_bytes().first()? {
        b'+' => (1, &exp[1..]),
        b'-' => (-1, &exp[1..]),
        _ => (1, exp),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) || digits.len() > 9 {
        return None;
    }
    let magnitude: i32 = digits.parse().ok()?;
    Some((body, sign * magnitude))
}

fn compare_decimal(left: &Decimal, right: &Decimal) -> Ordering {
    if left.coef == [0] && right.coef == [0] {
        return Ordering::Equal;
    }
    if left.neg != right.neg {
        return if left.neg {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let abs = compare_abs(left, right);
    if left.neg {
        abs.reverse()
    } else {
        abs
    }
}

fn compare_abs(left: &Decimal, right: &Decimal) -> Ordering {
    let (mut a, mut b) = (left.coef.clone(), right.coef.clone());
    if left.exp > right.exp {
        let shift = usize::try_from(left.exp - right.exp).unwrap_or(usize::MAX);
        if shift > 10_000 {
            return Ordering::Greater;
        }
        a.extend(std::iter::repeat(0).take(shift));
    } else if right.exp > left.exp {
        let shift = usize::try_from(right.exp - left.exp).unwrap_or(usize::MAX);
        if shift > 10_000 {
            return Ordering::Less;
        }
        b.extend(std::iter::repeat(0).take(shift));
    }
    match a.len().cmp(&b.len()) {
        Ordering::Equal => a.cmp(&b),
        other => other,
    }
}

fn subdomain_within(grant: &str, op: &str) -> Result<(), String> {
    let grant = normalize_host(grant)?;
    let op = normalize_host(op)?;
    if op == grant || op.ends_with(&format!(".{grant}")) {
        Ok(())
    } else {
        Err("params_exceed_grant".into())
    }
}

fn normalize_host(input: &str) -> Result<String, String> {
    if input
        .chars()
        .any(|ch| ch.is_whitespace() || ch == '\u{00a0}' || ch.is_control())
    {
        return Err("invalid_params_binding".into());
    }
    let stripped = input.strip_suffix('.').unwrap_or(input);
    if stripped.is_empty()
        || stripped.starts_with('.')
        || stripped.ends_with('.')
        || stripped.split('.').any(str::is_empty)
    {
        return Err("invalid_params_binding".into());
    }
    Ok(stripped.to_ascii_lowercase())
}

fn path_within(grant: &str, op: &str) -> Result<(), String> {
    validate_path(grant)?;
    validate_path(op)?;
    if op == grant || op.starts_with(&format!("{grant}/")) {
        Ok(())
    } else {
        Err("params_exceed_grant".into())
    }
}

fn validate_path(input: &str) -> Result<(), String> {
    if input.is_empty() || input.chars().any(|ch| ch.is_whitespace() || ch == '\u{00a0}') {
        return Err("invalid_params_binding".into());
    }
    if !input.starts_with('/') {
        return Err("invalid_params_binding".into());
    }
    let mut segments = input.split('/');
    if segments.next() != Some("") {
        return Err("invalid_params_binding".into());
    }
    let mut saw = false;
    for segment in segments {
        saw = true;
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err("invalid_params_binding".into());
        }
    }
    if saw {
        Ok(())
    } else {
        Err("invalid_params_binding".into())
    }
}

struct Amount {
    value: i128,
    unit: String,
    scale: u32,
}

fn quantity_params(
    grant_params: Option<&Value>,
    op_params: Option<&Value>,
    contains: bool,
) -> Result<(), String> {
    let grant = object_map(grant_params)?;
    let op = object_map(op_params)?;
    let mut keys: Vec<&String> = grant.keys().chain(op.keys()).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        match (grant.get(key), op.get(key)) {
            (Some(left), Some(right)) => compare_quantity_value(left, right, contains)?,
            (Some(_), None) => {
                return Err(if contains {
                    "params_not_narrower".into()
                } else {
                    "params_missing".into()
                });
            }
            (None, Some(_)) => {
                return Err(if contains {
                    "params_not_narrower".into()
                } else {
                    "undeclared_param".into()
                });
            }
            (None, None) => {}
        }
    }
    Ok(())
}

fn object_map(value: Option<&Value>) -> Result<Map<String, Value>, String> {
    match value {
        None => Ok(Map::new()),
        Some(Value::Object(map)) => Ok(map.clone()),
        Some(Value::Null) => Err("invalid_params_null".into()),
        Some(_) => Err("invalid_params_binding".into()),
    }
}

fn compare_quantity_value(grant: &Value, op: &Value, contains: bool) -> Result<(), String> {
    let grant_amount = is_amount(grant);
    let op_amount = is_amount(op);
    if grant_amount != op_amount {
        return Err(if contains {
            "params_not_narrower".into()
        } else {
            "invalid_params_binding".into()
        });
    }
    if grant_amount {
        let grant = parse_amount(grant)?;
        let op = parse_amount(op)?;
        if grant.unit != op.unit {
            return Err(if contains {
                "params_not_narrower".into()
            } else {
                "unit_mismatch".into()
            });
        }
        let target = grant.scale.max(op.scale);
        let grant_units = rescale(grant.value, grant.scale, target)?;
        let op_units = rescale(op.value, op.scale, target)?;
        if op_units <= grant_units {
            Ok(())
        } else {
            Err(if contains {
                "params_not_narrower".into()
            } else {
                "params_exceed_grant".into()
            })
        }
    } else {
        let wrapped_grant = Value::Object(Map::from_iter([("k".into(), grant.clone())]));
        let wrapped_op = Value::Object(Map::from_iter([("k".into(), op.clone())]));
        parameters::entail_parameters(Some(&wrapped_grant), None, Some(&wrapped_op)).map(|_| ()).map_err(
            |reason| {
                if contains {
                    "params_not_narrower".into()
                } else {
                    reason.code()
                }
            },
        )
    }
}

fn is_amount(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|map| map.contains_key("value") && map.contains_key("unit"))
}

fn parse_amount(value: &Value) -> Result<Amount, String> {
    let Some(map) = value.as_object() else {
        return Err("invalid_params_binding".into());
    };
    for key in map.keys() {
        if key != "value" && key != "unit" && key != "scale" {
            return Err("invalid_params_binding".into());
        }
    }
    let unit = map
        .get("unit")
        .and_then(Value::as_str)
        .filter(|unit| !unit.is_empty())
        .ok_or_else(|| "invalid_params_binding".to_string())?
        .to_string();
    let number = map
        .get("value")
        .ok_or_else(|| "invalid_params_binding".to_string())?;
    let value = json_i128(number)?;
    let scale = match map.get("scale") {
        None => 0,
        Some(raw) => {
            let scale = json_i128(raw)?;
            if !(0..=18).contains(&scale) {
                return Err("invalid_params_binding".into());
            }
            u32::try_from(scale).unwrap_or(0)
        }
    };
    Ok(Amount { value, unit, scale })
}

fn json_i128(value: &Value) -> Result<i128, String> {
    let Some(number) = value.as_i64() else {
        let Some(number) = value.as_u64() else {
            return Err("invalid_params_binding".into());
        };
        return i128::try_from(number).map_err(|_| "invalid_params_binding".into());
    };
    Ok(i128::from(number))
}

fn rescale(value: i128, from: u32, to: u32) -> Result<i128, String> {
    if to < from {
        return Err("invalid_params_binding".into());
    }
    let mut acc = value;
    for _ in from..to {
        acc = acc.checked_mul(10).ok_or("invalid_params_binding")?;
    }
    Ok(acc)
}

fn symmetry_match(vector: &Value) -> String {
    if vector.get("no_evidence").and_then(Value::as_bool) == Some(true) {
        return "NO_EVIDENCE".into();
    }
    let type_name = vector
        .get("type")
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if let Some(observed) = vector.get("observed_type").and_then(Value::as_str) {
        if observed != type_name {
            return "INDETERMINATE".into();
        }
    }
    let suite = vector.get("suite").and_then(Value::as_str).unwrap_or("");
    if let Some(observed) = vector.get("observed_suite").and_then(Value::as_str) {
        if observed != suite {
            return "INDETERMINATE".into();
        }
    }
    let Some(observed) = vector.get("observed").and_then(Value::as_object) else {
        return "NO_EVIDENCE".into();
    };
    let Some(material) = vector
        .get("type")
        .and_then(|value| value.get("material"))
        .and_then(Value::as_array)
    else {
        return "action_not_matchable".into();
    };
    let Some(operation) = vector
        .get("authorization")
        .and_then(|value| value.get("operation"))
        .and_then(|value| value.get("params"))
        .and_then(Value::as_object)
    else {
        return "action_not_matchable".into();
    };
    for key in material.iter().filter_map(Value::as_str) {
        let Some(got) = observed.get(key) else {
            return "action_not_matchable".into();
        };
        let Some(expected) = operation.get(key) else {
            return "action_not_matchable".into();
        };
        if !json_eq(expected, got) {
            return "NOT_EQUIVALENT".into();
        }
    }
    "MATCH".into()
}

fn jcs(value: &Value) -> String {
    let mut out = String::new();
    write_jcs(value, &mut out);
    out
}

fn write_jcs(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => out.push_str(&jcs_number(number)),
        Value::String(text) => push_json_string(out, text),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_jcs(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|left, right| utf16_units(left).cmp(&utf16_units(right)));
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                push_json_string(out, key);
                out.push(':');
                write_jcs(&map[*key], out);
            }
            out.push('}');
        }
    }
}

fn jcs_number(number: &Number) -> String {
    number.to_string()
}

fn push_json_string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn utf16_units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn as_f64(value: &Value) -> Result<f64, String> {
    value
        .as_f64()
        .filter(|number| number.is_finite())
        .ok_or_else(|| "invalid_params_number".to_string())
}

fn as_u64(value: &Value) -> Result<u64, String> {
    value
        .as_u64()
        .ok_or_else(|| "invalid_params_binding".to_string())
}

fn json_eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
            (Some(p), Some(q)) => p == q && p.is_finite(),
            _ => false,
        },
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| json_eq(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(key, value)| b.get(key).is_some_and(|other| json_eq(value, other)))
        }
        _ => false,
    }
}

fn values_eq(left: &Value, right: &Value) -> bool {
    json_eq(left, right)
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
