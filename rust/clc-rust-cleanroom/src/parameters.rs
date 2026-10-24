//! Parameter entailment (§6.2, §6.5) and delegation narrowing (§13.4.3).
//!
//! A params number is an upper bound. Lower bounds, step, and cardinality
//! live on `param_bounds`. Key closure for Contains is symmetric: the child
//! key set equals the parent key set. Complex domains (filesystem paths,
//! URLs, shell commands) do not get a scalar pass — they return a boundary
//! notice citing `draft-correctover-gg-mcp-parameter-risk`.

use crate::error::Reason;
use serde_json::{Map, Number, Value};
use std::collections::{BTreeMap, HashSet};

pub const PARAMETER_RISK_DRAFT: &str = "draft-correctover-gg-mcp-parameter-risk";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryDomain {
    Filesystem,
    Url,
    Shell,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BoundaryNotice {
    pub key: String,
    pub domain: BoundaryDomain,
    pub citation: String,
}

#[derive(Debug, Clone)]
struct BoundSpec {
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    enumeration: Option<Vec<Value>>,
    min_items: Option<u64>,
    max_items: Option<u64>,
    nested: Option<BTreeMap<String, BoundSpec>>,
    optional: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Empty,
    Numeric,
    Enum,
    Nested,
}

impl BoundSpec {
    fn family(&self) -> Family {
        if self.nested.is_some() {
            Family::Nested
        } else if self.enumeration.is_some() || self.min_items.is_some() || self.max_items.is_some()
        {
            Family::Enum
        } else if self.min.is_some() || self.max.is_some() || self.step.is_some() {
            Family::Numeric
        } else {
            Family::Empty
        }
    }
}

/// Text entry point (§6.2 steps 1–7).
///
/// A leading BOM is a number-shape refusal and outranks size. Size is the
/// RFC 8785 canonical UTF-8 length. Duplicate keys follow size. Number
/// shape, including an integer outside the I-JSON safe range and an
/// unpaired surrogate, follows duplicates.
pub fn prepare_params(raw: &str) -> Result<Value, Reason> {
    let flags = analyze_text(raw);
    if flags.bom {
        return Err(Reason::InvalidParamsNumber);
    }
    if flags.too_big || flags.too_deep {
        return Err(Reason::InvalidParamsSize);
    }
    if flags.duplicate {
        return Err(Reason::InvalidParamsDuplicateKey);
    }
    if flags.bad_number || flags.surrogate || flags.syntax {
        return Err(Reason::InvalidParamsNumber);
    }
    let value: Value = serde_json::from_str(raw).map_err(|_| Reason::InvalidParamsNumber)?;
    if !value.is_object() {
        return Err(Reason::InvalidParamsNumber);
    }
    Ok(value)
}

/// Operation-within-grant (§6.2 / §6.3 / §6.5). `None` params means the field is absent.
pub fn entail_parameters(
    grant_params: Option<&Value>,
    grant_bounds: Option<&Value>,
    op_params: Option<&Value>,
) -> Result<Vec<BoundaryNotice>, Reason> {
    check_container(grant_params)?;
    check_container(op_params)?;
    let bounds = parse_bounds(grant_bounds)?;
    let gmap = object_map(grant_params);
    reject_same_key(&gmap, &bounds)?;
    if let Some(map) = gmap.as_ref() {
        scan_empty_and_null(map)?;
    }
    if let Some(Value::Object(map)) = op_params {
        scan_nulls(map)?;
    }

    let declared = declared_keys(&gmap, &bounds);
    let notices = boundary_notices_of(op_params.into_iter().chain(grant_params));
    if declared.is_empty() {
        return Ok(notices);
    }

    let op_map = match op_params {
        None => return Err(Reason::ParamsMissing),
        Some(Value::Object(map)) => map,
        Some(_) => return Err(Reason::InvalidParamsNumber),
    };

    if missing_required(&declared, &gmap, &bounds, op_map) {
        return Err(Reason::ParamsMissing);
    }
    if op_map.keys().any(|k| !declared.contains(k)) {
        return Err(Reason::UndeclaredParam);
    }

    let mut enum_err: Option<Reason> = None;
    let mut bound_err: Option<Reason> = None;
    let mut keys: Vec<&String> = declared.iter().collect();
    keys.sort();
    for key in keys {
        let result = if let Some(bound) = bounds.get(key) {
            let Some(value) = op_map.get(key) else {
                continue;
            };
            check_bound_value(bound, value)
        } else if let Some(map) = gmap.as_ref() {
            value_within(&map[key], &op_map[key])
        } else {
            Ok(())
        };
        if let Err(reason) = result {
            if is_enum_layer(&reason) {
                if enum_err.is_none() {
                    enum_err = Some(reason);
                }
            } else if bound_err.is_none() {
                bound_err = Some(reason);
            }
        }
    }
    if let Some(reason) = enum_err.or(bound_err) {
        return Err(reason);
    }
    Ok(notices)
}

/// Layer-1 parameter grammar for a grant: shape, size, nulls, and binding.
/// An empty array is not a grammar error; Contains reports it as `params_not_narrower`.
pub fn validate_grant_params(params: Option<&Value>, bounds: Option<&Value>) -> Result<(), Reason> {
    check_container(params)?;
    let parsed = parse_bounds(bounds)?;
    let map = object_map(params);
    reject_same_key(&map, &parsed)?;
    if let Some(map) = map.as_ref() {
        scan_nulls(map)?;
    }
    Ok(())
}

/// Child grant within parent grant (§13.4.3). Failures are `params_not_narrower`
/// unless the binding itself is malformed.
pub fn narrow_parameters(
    parent_params: Option<&Value>,
    parent_bounds: Option<&Value>,
    child_params: Option<&Value>,
    child_bounds: Option<&Value>,
) -> Result<Vec<BoundaryNotice>, Reason> {
    check_container(parent_params)?;
    check_container(child_params)?;
    let parent_b = parse_bounds(parent_bounds)?;
    let child_b = parse_bounds(child_bounds)?;
    let pmap = object_map(parent_params);
    let cmap = object_map(child_params);
    reject_same_key(&pmap, &parent_b)?;
    reject_same_key(&cmap, &child_b)?;
    if let Some(map) = pmap.as_ref() {
        scan_nulls(map)?;
    }
    if let Some(map) = cmap.as_ref() {
        scan_nulls(map)?;
    }

    let notices = boundary_notices_of(parent_params.into_iter().chain(child_params));
    let parent_keys = declared_keys(&pmap, &parent_b);
    let child_keys = declared_keys(&cmap, &child_b);
    if parent_keys.is_empty() {
        return Ok(notices);
    }
    if child_keys.is_empty() || parent_keys != child_keys {
        return Err(Reason::ParamsNotNarrower);
    }
    for key in &parent_keys {
        let parent_site_bounds = parent_b.contains_key(key);
        let child_site_bounds = child_b.contains_key(key);
        if parent_site_bounds != child_site_bounds {
            return Err(Reason::ParamsNotNarrower);
        }
        if parent_site_bounds {
            bound_within(&parent_b[key], &child_b[key])?;
        } else {
            let parent_v = &pmap.as_ref().unwrap()[key];
            let child_v = &cmap.as_ref().unwrap()[key];
            value_narrower(parent_v, child_v)?;
        }
    }
    Ok(notices)
}

pub fn boundary_notices_of<'a>(values: impl IntoIterator<Item = &'a Value>) -> Vec<BoundaryNotice> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for value in values {
        walk_boundary("", value, &mut out, &mut seen);
    }
    out
}

fn walk_boundary(
    key: &str,
    value: &Value,
    out: &mut Vec<BoundaryNotice>,
    seen: &mut HashSet<(String, u8)>,
) {
    match value {
        Value::Object(map) => {
            for (k, child) in map {
                if let Some(domain) = domain_for(k, child) {
                    push_notice(k, domain, out, seen);
                }
                let next = if key.is_empty() {
                    k.clone()
                } else {
                    format!("{key}.{k}")
                };
                walk_boundary(&next, child, out, seen);
            }
        }
        Value::Array(items) => {
            for child in items {
                walk_boundary(key, child, out, seen);
            }
        }
        Value::String(_) => {
            if !key.is_empty() {
                if let Some(domain) = domain_for(key, value) {
                    push_notice(key, domain, out, seen);
                }
            }
        }
        _ => {}
    }
}

fn push_notice(
    key: &str,
    domain: BoundaryDomain,
    out: &mut Vec<BoundaryNotice>,
    seen: &mut HashSet<(String, u8)>,
) {
    let tag = match domain {
        BoundaryDomain::Filesystem => 0,
        BoundaryDomain::Url => 1,
        BoundaryDomain::Shell => 2,
    };
    if seen.insert((key.to_string(), tag)) {
        out.push(BoundaryNotice {
            key: key.to_string(),
            domain,
            citation: PARAMETER_RISK_DRAFT.to_string(),
        });
    }
}

fn domain_for(key: &str, value: &Value) -> Option<BoundaryDomain> {
    let key_l = key.rsplit('.').next().unwrap_or(key);
    if let Value::String(text) = value {
        if looks_like_url(text) {
            return Some(BoundaryDomain::Url);
        }
        if looks_like_path(text) {
            return Some(BoundaryDomain::Filesystem);
        }
        if looks_like_shell(key_l, text) {
            return Some(BoundaryDomain::Shell);
        }
    }
    match key_l {
        "path" | "file" | "filepath" | "file_path" | "directory" | "dir" | "filename" => {
            Some(BoundaryDomain::Filesystem)
        }
        "url" | "uri" | "href" | "endpoint" => Some(BoundaryDomain::Url),
        "command" | "cmd" | "shell" | "argv" => Some(BoundaryDomain::Shell),
        _ => None,
    }
}

fn looks_like_url(text: &str) -> bool {
    let Some(idx) = text.find("://") else {
        return false;
    };
    let scheme = &text[..idx];
    !scheme.is_empty()
        && scheme
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
}

fn looks_like_path(text: &str) -> bool {
    text.starts_with('/')
        || text.starts_with('\\')
        || text.starts_with("~/")
        || text.contains("../")
        || text.contains("..\\")
        || (text.len() >= 3
            && text.as_bytes()[0].is_ascii_alphabetic()
            && text.as_bytes()[1] == b':'
            && (text.as_bytes()[2] == b'\\' || text.as_bytes()[2] == b'/'))
}

fn looks_like_shell(key: &str, text: &str) -> bool {
    matches!(key, "command" | "cmd" | "shell" | "argv" | "args")
        || text.contains("&&")
        || text.contains("||")
        || text.contains("$(")
        || text.contains('`')
        || text.contains('\n')
}

fn check_container(value: Option<&Value>) -> Result<(), Reason> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_null() {
        return Err(Reason::InvalidParamsNull);
    }
    if !value.is_object() {
        return Err(Reason::InvalidParamsNumber);
    }
    if value_depth(value) > 32 || json_len(value) > 512 {
        return Err(Reason::InvalidParamsSize);
    }
    reject_unsafe_integers(value)?;
    Ok(())
}

fn object_map(value: Option<&Value>) -> Option<Map<String, Value>> {
    match value {
        Some(Value::Object(map)) => Some(map.clone()),
        _ => None,
    }
}

fn declared_keys(params: &Option<Map<String, Value>>, bounds: &BTreeMap<String, BoundSpec>) -> HashSet<String> {
    let mut keys = HashSet::new();
    if let Some(map) = params {
        // `{}` declares nothing. A non-empty object declares its keys.
        if !map.is_empty() {
            keys.extend(map.keys().cloned());
        }
    }
    keys.extend(bounds.keys().cloned());
    keys
}

fn reject_same_key(params: &Option<Map<String, Value>>, bounds: &BTreeMap<String, BoundSpec>) -> Result<(), Reason> {
    if let Some(map) = params {
        if map.keys().any(|k| bounds.contains_key(k)) {
            return Err(Reason::InvalidParamsBinding);
        }
    }
    Ok(())
}

fn scan_empty_and_null(map: &Map<String, Value>) -> Result<(), Reason> {
    if contains_empty_bound(map) {
        return Err(Reason::EmptyBoundDeniesClass);
    }
    scan_nulls(map)
}

fn scan_nulls(map: &Map<String, Value>) -> Result<(), Reason> {
    fn walk(value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::Array(items) => items.iter().any(walk),
            Value::Object(map) => map.values().any(walk),
            _ => false,
        }
    }
    if map.values().any(walk) {
        Err(Reason::InvalidParamsNull)
    } else {
        Ok(())
    }
}

fn contains_empty_bound(map: &Map<String, Value>) -> bool {
    fn walk(value: &Value) -> bool {
        match value {
            Value::Array(items) if items.is_empty() => true,
            Value::Object(obj) if obj.is_empty() => true,
            Value::Array(items) => items.iter().any(walk),
            Value::Object(obj) => obj.values().any(walk),
            _ => false,
        }
    }
    map.values().any(walk)
}

fn missing_required(
    declared: &HashSet<String>,
    params: &Option<Map<String, Value>>,
    bounds: &BTreeMap<String, BoundSpec>,
    op: &Map<String, Value>,
) -> bool {
    declared.iter().any(|key| {
        if op.contains_key(key) {
            return false;
        }
        if let Some(bound) = bounds.get(key) {
            return !bound.optional;
        }
        params.as_ref().is_some_and(|map| map.contains_key(key))
    })
}

fn is_enum_layer(reason: &Reason) -> bool {
    matches!(
        reason,
        Reason::NotInEnum | Reason::ParamsCardinality | Reason::EmptyBoundDeniesClass
    )
}

fn value_within(grant: &Value, op: &Value) -> Result<(), Reason> {
    if grant.is_null() || op.is_null() {
        return Err(Reason::InvalidParamsNull);
    }
    match grant {
        Value::Number(limit) => {
            let limit = finite(limit).ok_or(Reason::InvalidParamsNumber)?;
            let Value::Number(got) = op else {
                return Err(Reason::ParamsExceedGrant);
            };
            let got = finite(got).ok_or(Reason::InvalidParamsNumber)?;
            if got <= limit {
                Ok(())
            } else {
                Err(Reason::ParamsExceedGrant)
            }
        }
        Value::String(expected) => match op {
            Value::String(got) if got == expected => Ok(()),
            _ => Err(Reason::ParamsExceedGrant),
        },
        Value::Bool(expected) => match op {
            Value::Bool(got) if got == expected => Ok(()),
            _ => Err(Reason::ParamsExceedGrant),
        },
        Value::Array(allowed) => {
            if allowed.is_empty() {
                return Err(Reason::EmptyBoundDeniesClass);
            }
            let ok = match op {
                Value::Array(got) => got.iter().all(|item| allowed.iter().any(|m| json_eq(m, item))),
                other => allowed.iter().any(|m| json_eq(m, other)),
            };
            if ok {
                Ok(())
            } else {
                Err(Reason::NotInEnum)
            }
        }
        Value::Object(expected) => {
            if expected.is_empty() {
                return Err(Reason::EmptyBoundDeniesClass);
            }
            let Value::Object(got) = op else {
                return Err(Reason::ParamsExceedGrant);
            };
            if expected.keys().any(|k| !got.contains_key(k)) {
                return Err(Reason::ParamsMissing);
            }
            if got.keys().any(|k| !expected.contains_key(k)) {
                return Err(Reason::UndeclaredParam);
            }
            for (key, bound) in expected {
                value_within(bound, &got[key])?;
            }
            Ok(())
        }
        Value::Null => Err(Reason::InvalidParamsNull),
    }
}

fn value_narrower(parent: &Value, child: &Value) -> Result<(), Reason> {
    match parent {
        Value::Number(limit) => {
            let limit = finite(limit).ok_or(Reason::InvalidParamsNumber)?;
            let Value::Number(got) = child else {
                return Err(Reason::ParamsNotNarrower);
            };
            let got = finite(got).ok_or(Reason::InvalidParamsNumber)?;
            if got <= limit {
                Ok(())
            } else {
                Err(Reason::ParamsNotNarrower)
            }
        }
        Value::String(expected) => match child {
            Value::String(got) if got == expected => Ok(()),
            _ => Err(Reason::ParamsNotNarrower),
        },
        Value::Bool(expected) => match child {
            Value::Bool(got) if got == expected => Ok(()),
            _ => Err(Reason::ParamsNotNarrower),
        },
        Value::Array(allowed) => {
            if allowed.is_empty() {
                return Err(Reason::ParamsNotNarrower);
            }
            match child {
                Value::Array(got) => {
                    if got.iter().all(|item| allowed.iter().any(|m| json_eq(m, item))) {
                        Ok(())
                    } else {
                        Err(Reason::ParamsNotNarrower)
                    }
                }
                other if allowed.iter().any(|m| json_eq(m, other)) => Ok(()),
                _ => Err(Reason::ParamsNotNarrower),
            }
        }
        Value::Object(expected) => {
            let Value::Object(got) = child else {
                return Err(Reason::ParamsNotNarrower);
            };
            if expected.len() != got.len() || expected.keys().any(|k| !got.contains_key(k)) {
                return Err(Reason::ParamsNotNarrower);
            }
            for (key, bound) in expected {
                value_narrower(bound, &got[key])?;
            }
            Ok(())
        }
        _ => Err(Reason::ParamsNotNarrower),
    }
}

fn check_bound_value(bound: &BoundSpec, value: &Value) -> Result<(), Reason> {
    match bound.family() {
        Family::Empty => Ok(()),
        Family::Numeric => check_numeric(bound, value),
        Family::Enum => check_enum(bound, value),
        Family::Nested => check_nested(bound, value),
    }
}

fn check_numeric(bound: &BoundSpec, value: &Value) -> Result<(), Reason> {
    let Value::Number(number) = value else {
        return Err(Reason::ParamsExceedGrant);
    };
    let got = finite(number).ok_or(Reason::InvalidParamsNumber)?;
    if bound.min.is_some_and(|min| got < min) || bound.max.is_some_and(|max| got > max) {
        return Err(Reason::ParamsOutOfRange);
    }
    if bound.step.is_some_and(|step| !integer_multiple(got, step)) {
        return Err(Reason::ParamsNotMultiple);
    }
    Ok(())
}

fn check_enum(bound: &BoundSpec, value: &Value) -> Result<(), Reason> {
    if let Some(members) = &bound.enumeration {
        let ok = match value {
            Value::Array(items) => items.iter().all(|item| members.iter().any(|m| json_eq(m, item))),
            other => members.iter().any(|m| json_eq(m, other)),
        };
        if !ok {
            return Err(Reason::NotInEnum);
        }
    }
    if bound.min_items.is_some() || bound.max_items.is_some() {
        let n = match value {
            Value::Array(items) => items.len() as u64,
            _ => 1,
        };
        if bound.min_items.is_some_and(|min| n < min) || bound.max_items.is_some_and(|max| n > max) {
            return Err(Reason::ParamsCardinality);
        }
    }
    Ok(())
}

fn check_nested(bound: &BoundSpec, value: &Value) -> Result<(), Reason> {
    let Some(nested) = &bound.nested else {
        return Ok(());
    };
    let Value::Object(got) = value else {
        return Err(Reason::ParamsExceedGrant);
    };
    if nested.keys().any(|k| !bound_optional(nested, k) && !got.contains_key(k)) {
        return Err(Reason::ParamsMissing);
    }
    if got.keys().any(|k| !nested.contains_key(k)) {
        return Err(Reason::UndeclaredParam);
    }
    for (key, child) in nested {
        if let Some(present) = got.get(key) {
            check_bound_value(child, present)?;
        }
    }
    Ok(())
}

fn bound_optional(nested: &BTreeMap<String, BoundSpec>, key: &str) -> bool {
    nested.get(key).is_some_and(|b| b.optional)
}

fn bound_within(parent: &BoundSpec, child: &BoundSpec) -> Result<(), Reason> {
    if !parent.optional && child.optional {
        return Err(Reason::ParamsNotNarrower);
    }
    match (parent.family(), child.family()) {
        (Family::Empty, _) => {}
        (family, other) if family != other => return Err(Reason::ParamsNotNarrower),
        _ => {}
    }
    if let Some(min) = parent.min {
        match child.min {
            Some(got) if got >= min => {}
            _ => return Err(Reason::ParamsNotNarrower),
        }
    }
    if let Some(max) = parent.max {
        match child.max {
            Some(got) if got <= max => {}
            _ => return Err(Reason::ParamsNotNarrower),
        }
    }
    if let Some(step) = parent.step {
        match child.step {
            Some(got) if integer_multiple(got, step) => {}
            _ => return Err(Reason::ParamsNotNarrower),
        }
    }
    if let Some(allowed) = &parent.enumeration {
        if allowed.is_empty() {
            return Err(Reason::ParamsNotNarrower);
        }
        match &child.enumeration {
            Some(got) if got.iter().all(|item| allowed.iter().any(|m| json_eq(m, item))) => {}
            _ => return Err(Reason::ParamsNotNarrower),
        }
    }
    if let Some(min) = parent.min_items {
        match child.min_items {
            Some(got) if got >= min => {}
            _ => return Err(Reason::ParamsNotNarrower),
        }
    }
    if let Some(max) = parent.max_items {
        match child.max_items {
            Some(got) if got <= max => {}
            _ => return Err(Reason::ParamsNotNarrower),
        }
    }
    if let Some(expected) = &parent.nested {
        let Some(got) = &child.nested else {
            return Err(Reason::ParamsNotNarrower);
        };
        if expected.len() != got.len() || expected.keys().any(|k| !got.contains_key(k)) {
            return Err(Reason::ParamsNotNarrower);
        }
        for (key, bound) in expected {
            bound_within(bound, &got[key])?;
        }
    }
    Ok(())
}

fn parse_bounds(value: Option<&Value>) -> Result<BTreeMap<String, BoundSpec>, Reason> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    if value.is_null() {
        return Err(Reason::InvalidParamsNull);
    }
    let Value::Object(map) = value else {
        return Err(Reason::InvalidParamsBinding);
    };
    if value_depth(value) > 32 || json_len(value) > 512 {
        return Err(Reason::InvalidParamsSize);
    }
    let mut out = BTreeMap::new();
    for (key, raw) in map {
        out.insert(key.clone(), parse_bound(raw)?);
    }
    Ok(out)
}

fn parse_bound(value: &Value) -> Result<BoundSpec, Reason> {
    let Value::Object(map) = value else {
        return Err(Reason::InvalidParamsBinding);
    };
    let mut bound = BoundSpec {
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
            "min" => bound.min = Some(require_finite(raw)?),
            "max" => bound.max = Some(require_finite(raw)?),
            "step" => bound.step = Some(require_finite(raw)?),
            "enum" => {
                let Value::Array(items) = raw else {
                    return Err(Reason::InvalidParamsBinding);
                };
                if items.iter().any(|item| item.is_object() || item.is_array() || item.is_null()) {
                    return Err(Reason::InvalidParamsBinding);
                }
                bound.enumeration = Some(items.clone());
            }
            "min_items" => bound.min_items = Some(require_u64(raw)?),
            "max_items" => bound.max_items = Some(require_u64(raw)?),
            "nested" => {
                let Value::Object(_) = raw else {
                    return Err(Reason::InvalidParamsBinding);
                };
                bound.nested = Some(parse_bounds(Some(raw))?);
            }
            "optional" => {
                let Value::Bool(flag) = raw else {
                    return Err(Reason::InvalidParamsBinding);
                };
                bound.optional = *flag;
            }
            _ => return Err(Reason::InvalidParamsBinding),
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
        return Err(Reason::InvalidParamsBinding);
    }
    if bound.min.is_some_and(|min| bound.max.is_some_and(|max| min > max)) {
        return Err(Reason::InvalidParamsBinding);
    }
    if bound.step.is_some_and(|step| step <= 0.0) {
        return Err(Reason::InvalidParamsBinding);
    }
    if bound
        .min_items
        .is_some_and(|min| bound.max_items.is_some_and(|max| min > max))
    {
        return Err(Reason::InvalidParamsBinding);
    }
    Ok(bound)
}

fn require_finite(value: &Value) -> Result<f64, Reason> {
    let Value::Number(number) = value else {
        return Err(Reason::InvalidParamsBinding);
    };
    finite(number).ok_or(Reason::InvalidParamsNumber)
}

fn require_u64(value: &Value) -> Result<u64, Reason> {
    let Value::Number(number) = value else {
        return Err(Reason::InvalidParamsBinding);
    };
    number.as_u64().ok_or(Reason::InvalidParamsBinding)
}

fn finite(number: &Number) -> Option<f64> {
    number.as_f64().filter(|value| value.is_finite())
}

fn integer_multiple(value: f64, step: f64) -> bool {
    if step <= 0.0 || !value.is_finite() || !step.is_finite() {
        return false;
    }
    let quotient = value / step;
    quotient == quotient.floor() && quotient * step == value
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
        _ => false,
    }
}

fn value_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => 1 + map.values().map(value_depth).max().unwrap_or(0),
        Value::Array(items) => 1 + items.iter().map(value_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn json_len(value: &Value) -> usize {
    match value {
        Value::Null => 4,
        Value::Bool(true) => 4,
        Value::Bool(false) => 5,
        Value::Number(number) => jcs_number(number).len(),
        Value::String(text) => 2 + escaped_len(text),
        Value::Array(items) => {
            let body: usize = items.iter().map(json_len).sum();
            let commas = items.len().saturating_sub(1);
            2 + body + commas
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| utf16_units(a).cmp(&utf16_units(b)));
            let mut total = 2;
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    total += 1;
                }
                total += 2 + escaped_len(key) + 1 + json_len(&map[*key]);
            }
            total
        }
    }
}

fn escaped_len(text: &str) -> usize {
    text.chars()
        .map(|ch| match ch {
            '"' | '\\' | '\u{0008}' | '\u{000c}' => 2,
            '\n' | '\r' | '\t' => 2,
            ch if (ch as u32) < 0x20 => 6,
            ch => ch.len_utf8(),
        })
        .sum()
}

fn utf16_units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// I-JSON / RFC 8785 safe integer: `2^53 - 1`.
const SAFE_INT_MAX: i64 = 9_007_199_254_740_991;

fn jcs_number(number: &Number) -> String {
    if let Some(value) = number.as_i64() {
        return value.to_string();
    }
    if let Some(value) = number.as_u64() {
        return value.to_string();
    }
    if let Some(value) = number.as_f64() {
        if value.is_finite() {
            return jcs_f64(value);
        }
    }
    number.to_string()
}

/// ES6 `Number::toString`, which RFC 8785 uses for the canonical number.
fn jcs_f64(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let negative = value.is_sign_negative();
    let debug = format!("{:?}", value.abs());
    let (mantissa, exp) = split_exp(&debug);
    let (digits, n) = significand(mantissa, exp);
    let body = es6_format(&digits, n);
    if negative {
        format!("-{body}")
    } else {
        body
    }
}

fn number_is_unsafe_integer(number: &Number) -> bool {
    if let Some(value) = number.as_i64() {
        return value > SAFE_INT_MAX || value < -SAFE_INT_MAX;
    }
    if let Some(value) = number.as_u64() {
        return value > SAFE_INT_MAX as u64;
    }
    if let Some(value) = number.as_f64() {
        if !value.is_finite() {
            return true;
        }
        return value.fract() == 0.0 && value.abs() > SAFE_INT_MAX as f64;
    }
    true
}

fn reject_unsafe_integers(value: &Value) -> Result<(), Reason> {
    match value {
        Value::Number(number) if number_is_unsafe_integer(number) => Err(Reason::InvalidParamsNumber),
        Value::Array(items) => {
            for item in items {
                reject_unsafe_integers(item)?;
            }
            Ok(())
        }
        Value::Object(map) => {
            for child in map.values() {
                reject_unsafe_integers(child)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// `(negative, significand digits, ES6 n)`. A zero number has digits `"0"`.
fn number_shape(token: &str) -> (bool, String, i64) {
    let negative = token.starts_with('-');
    let body = if negative { &token[1..] } else { token };
    let (mantissa, exp) = split_exp(body);
    let (digits, n) = significand(mantissa, exp);
    (negative, digits, n)
}

fn split_exp(body: &str) -> (&str, i64) {
    let Some((mantissa, exp_text)) = body.split_once(['e', 'E']) else {
        return (body, 0);
    };
    (mantissa, parse_exp(exp_text))
}

fn parse_exp(text: &str) -> i64 {
    if let Ok(value) = text.parse::<i64>() {
        return value;
    }
    if text.trim_start_matches('+').starts_with('-') {
        i64::from(i32::MIN)
    } else {
        i64::from(i32::MAX)
    }
}

/// Shortest significand and the ES6 `n` (one more than `floor(log10)`).
fn significand(mantissa: &str, exp10: i64) -> (String, i64) {
    let (int_part, frac_part) = match mantissa.split_once('.') {
        Some((int_part, frac_part)) => (int_part, frac_part),
        None => (mantissa, ""),
    };
    let int_sig = int_part.trim_start_matches('0');
    let (digits, n) = if !int_sig.is_empty() {
        let mut digits = String::new();
        digits.push_str(int_sig);
        digits.push_str(frac_part);
        (digits, int_sig.len() as i64 + exp10)
    } else {
        let frac_leading = frac_part.chars().take_while(|ch| *ch == '0').count();
        let rest = &frac_part[frac_leading..];
        (rest.to_string(), exp10 - frac_leading as i64)
    };
    let digits = digits.trim_end_matches('0').to_string();
    if digits.is_empty() {
        ("0".into(), 1)
    } else {
        (digits, n)
    }
}

fn es6_format(digits: &str, n: i64) -> String {
    if digits == "0" {
        return "0".to_string();
    }
    let k = digits.len() as i64;
    if k <= n && n <= 21 {
        let mut out = digits.to_string();
        for _ in 0..(n - k) {
            out.push('0');
        }
        return out;
    }
    if (0 < n) && n <= 21 {
        let split = n as usize;
        return format!("{}.{}", &digits[..split], &digits[split..]);
    }
    if (-6 < n) && n <= 0 {
        let zeros = (-n) as usize;
        return format!("0.{}{digits}", "0".repeat(zeros));
    }
    let exp = n - 1;
    let exp_sign = if exp >= 0 { "+" } else { "-" };
    let exp_abs = exp.abs();
    if k == 1 {
        format!("{}e{exp_sign}{exp_abs}", &digits[..1])
    } else {
        format!("{}.{}e{exp_sign}{exp_abs}", &digits[..1], &digits[1..])
    }
}

fn integer_exceeds_safe(digits: &str, n: i64) -> bool {
    if digits == "0" {
        return false;
    }
    let k = digits.len() as i64;
    if n <= 0 || n < k {
        return false;
    }
    const BOUND: &str = "9007199254740991";
    if n > BOUND.len() as i64 {
        return true;
    }
    if n < BOUND.len() as i64 {
        return false;
    }
    let mut rendered = digits.to_string();
    while rendered.len() < BOUND.len() {
        rendered.push('0');
    }
    rendered.as_str() > BOUND
}

fn canonical_number(token: &str) -> Result<String, ()> {
    let (negative, digits, n) = number_shape(token);
    if overflows(token) || integer_exceeds_safe(&digits, n) {
        return Err(());
    }
    let body = es6_format(&digits, n);
    if negative && digits != "0" {
        Ok(format!("-{body}"))
    } else {
        Ok(body)
    }
}

fn decode_surrogate_pair(high: u32, low: u32) -> Option<char> {
    if !(0xD800..=0xDBFF).contains(&high) || !(0xDC00..=0xDFFF).contains(&low) {
        return None;
    }
    char::from_u32(0x10000 + (((high - 0xD800) << 10) | (low - 0xDC00)))
}

#[derive(Default)]
struct Flags {
    too_big: bool,
    too_deep: bool,
    duplicate: bool,
    bad_number: bool,
    surrogate: bool,
    syntax: bool,
    bom: bool,
    canon_len: usize,
}

fn analyze_text(raw: &str) -> Flags {
    let mut flags = Flags::default();
    if raw.starts_with('\u{FEFF}') {
        flags.bom = true;
        return flags;
    }
    if nesting_depth(raw) > 32 {
        flags.too_deep = true;
    }
    let bytes = raw.as_bytes();
    let (canon_len, trailing) = {
        let mut parser = Parser {
            bytes,
            index: 0,
            flags: &mut flags,
        };
        parser.skip_ws();
        if parser.peek() != Some(b'{') {
            (0, true)
        } else {
            let len = parser.parse_value(1);
            parser.skip_ws();
            (len, parser.index != bytes.len())
        }
    };
    flags.canon_len = canon_len;
    if trailing {
        flags.syntax = true;
    }
    if canon_len > 512 {
        flags.too_big = true;
    }
    flags
}

fn nesting_depth(raw: &str) -> usize {
    let mut depth: usize = 0;
    let mut max_depth: usize = 0;
    let mut in_string = false;
    let mut escape = false;
    for ch in raw.chars() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' | '[' => {
                depth += 1;
                max_depth = max_depth.max(depth);
            }
            '}' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    max_depth
}

struct Parser<'a> {
    bytes: &'a [u8],
    index: usize,
    flags: &'a mut Flags,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.index).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.index += 1;
        Some(byte)
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            self.index += 1;
        }
    }

    fn parse_value(&mut self, depth: usize) -> usize {
        if self.flags.syntax {
            return 0;
        }
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(depth),
            Some(b'[') => self.parse_array(depth),
            Some(b'"') => {
                let (text, extra) = self.parse_string();
                2 + escaped_len(&text) + extra
            }
            Some(b't') => {
                self.consume(b"true");
                4
            }
            Some(b'f') => {
                self.consume(b"false");
                5
            }
            Some(b'n') => {
                self.consume(b"null");
                4
            }
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            _ => {
                self.flags.syntax = true;
                0
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> usize {
        if depth > 32 {
            self.flags.too_deep = true;
        }
        self.bump();
        let mut keys = HashSet::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.bump();
            return 2;
        }
        let mut total = 2usize;
        let mut count = 0usize;
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                self.flags.syntax = true;
                return total;
            }
            let (key, extra) = self.parse_string();
            let key_len = 2 + escaped_len(&key) + extra;
            if !keys.insert(key) {
                self.flags.duplicate = true;
            }
            self.skip_ws();
            if self.bump() != Some(b':') {
                self.flags.syntax = true;
                return total;
            }
            let value_len = self.parse_value(depth + 1);
            if count > 0 {
                total += 1;
            }
            total += key_len + 1 + value_len;
            count += 1;
            if self.flags.syntax {
                return total;
            }
            self.skip_ws();
            match self.bump() {
                Some(b'}') => return total,
                Some(b',') => continue,
                _ => {
                    self.flags.syntax = true;
                    return total;
                }
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> usize {
        if depth > 32 {
            self.flags.too_deep = true;
        }
        self.bump();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.bump();
            return 2;
        }
        let mut total = 2usize;
        let mut count = 0usize;
        loop {
            let value_len = self.parse_value(depth + 1);
            if count > 0 {
                total += 1;
            }
            total += value_len;
            count += 1;
            if self.flags.syntax {
                return total;
            }
            self.skip_ws();
            match self.bump() {
                Some(b']') => return total,
                Some(b',') => continue,
                _ => {
                    self.flags.syntax = true;
                    return total;
                }
            }
        }
    }

    fn try_hex4(&mut self) -> Option<u32> {
        if self.index + 4 > self.bytes.len() {
            return None;
        }
        let hex = &self.bytes[self.index..self.index + 4];
        if !hex.iter().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let text = std::str::from_utf8(hex).ok()?;
        let code = u32::from_str_radix(text, 16).ok()?;
        self.index += 4;
        Some(code)
    }

    /// Decoded text plus extra octets for unpaired `\uXXXX` escapes.
    fn parse_string(&mut self) -> (String, usize) {
        self.bump();
        let mut out = String::new();
        let mut extra = 0usize;
        while let Some(byte) = self.bump() {
            match byte {
                b'"' => return (out, extra),
                b'\\' => match self.bump() {
                    Some(b'"') => out.push('"'),
                    Some(b'\\') => out.push('\\'),
                    Some(b'/') => out.push('/'),
                    Some(b'b') => out.push('\u{0008}'),
                    Some(b'f') => out.push('\u{000c}'),
                    Some(b'n') => out.push('\n'),
                    Some(b'r') => out.push('\r'),
                    Some(b't') => out.push('\t'),
                    Some(b'u') => {
                        let mut hex = [0u8; 4];
                        for slot in &mut hex {
                            match self.bump() {
                                Some(b) if b.is_ascii_hexdigit() => *slot = b,
                                _ => {
                                    self.flags.syntax = true;
                                    return (out, extra);
                                }
                            }
                        }
                        let text = std::str::from_utf8(&hex).unwrap_or("0000");
                        let code = u32::from_str_radix(text, 16).unwrap_or(0);
                        if (0xD800..=0xDBFF).contains(&code)
                            && self.bytes.get(self.index) == Some(&b'\\')
                            && self.bytes.get(self.index + 1) == Some(&b'u')
                        {
                            let saved = self.index;
                            self.index += 2;
                            if let Some(low) = self.try_hex4() {
                                if let Some(ch) = decode_surrogate_pair(code, low) {
                                    out.push(ch);
                                    continue;
                                }
                            }
                            self.index = saved;
                        }
                        if (0xD800..=0xDFFF).contains(&code) {
                            self.flags.surrogate = true;
                            extra += 6;
                        } else if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        } else {
                            self.flags.syntax = true;
                        }
                    }
                    _ => {
                        self.flags.syntax = true;
                        return (out, extra);
                    }
                },
                byte => match byte {
                    0x00..=0x1F => self.flags.syntax = true,
                    _ => {
                        self.index -= 1;
                        let rest = &self.bytes[self.index..];
                        let text = std::str::from_utf8(rest).unwrap_or("");
                        let ch = text.chars().next().unwrap_or('\u{FFFD}');
                        if ch == '\u{FFFD}' && !rest.starts_with(&[0xEF, 0xBF, 0xBD]) {
                            self.flags.syntax = true;
                            self.index += 1;
                            return (out, extra);
                        }
                        out.push(ch);
                        self.index += ch.len_utf8();
                    }
                },
            }
        }
        self.flags.syntax = true;
        (out, extra)
    }

    fn parse_number(&mut self) -> usize {
        let start = self.index;
        if self.peek() == Some(b'-') {
            self.bump();
        }
        if self.peek() == Some(b'0') {
            self.bump();
        } else if self.peek().is_some_and(|b| b.is_ascii_digit()) {
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.bump();
            }
        } else {
            self.flags.syntax = true;
            return 0;
        }
        if self.peek() == Some(b'.') {
            self.bump();
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.flags.syntax = true;
                return 0;
            }
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.bump();
            }
        }
        if self.peek().is_some_and(|b| b == b'e' || b == b'E') {
            self.bump();
            if self.peek().is_some_and(|b| b == b'+' || b == b'-') {
                self.bump();
            }
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.flags.syntax = true;
                return 0;
            }
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.bump();
            }
        }
        let token = std::str::from_utf8(&self.bytes[start..self.index]).unwrap_or("");
        match canonical_number(token) {
            Ok(rendered) => rendered.len(),
            Err(()) => {
                self.flags.bad_number = true;
                token.len()
            }
        }
    }

    fn consume(&mut self, literal: &[u8]) {
        if self.bytes[self.index..].starts_with(literal) {
            self.index += literal.len();
        } else {
            self.flags.syntax = true;
        }
    }
}

fn overflows(token: &str) -> bool {
    token.parse::<f64>().ok().is_none_or(|value| !value.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obj(value: Value) -> Value {
        value
    }

    #[test]
    fn scalar_upper_bound_set_inclusion_and_key_closure() {
        let parent = obj(json!({"circuit_groups": [1, 2, 3], "duration_sec": 7200, "max_shed_kw": 10}));
        let child = obj(json!({"circuit_groups": [2], "duration_sec": 900, "max_shed_kw": 2.5}));
        assert!(narrow_parameters(Some(&parent), None, Some(&child), None).is_ok());

        let wider = obj(json!({"circuit_groups": [2], "duration_sec": 900, "max_shed_kw": 11}));
        assert_eq!(
            narrow_parameters(Some(&parent), None, Some(&wider), None).unwrap_err(),
            Reason::ParamsNotNarrower
        );

        let extra = obj(json!({"circuit_groups": [2], "duration_sec": 900, "max_shed_kw": 2.5, "note": "x"}));
        assert_eq!(
            narrow_parameters(Some(&parent), None, Some(&extra), None).unwrap_err(),
            Reason::ParamsNotNarrower
        );
        let dropped = obj(json!({"duration_sec": 900, "max_shed_kw": 2.5}));
        assert_eq!(
            narrow_parameters(Some(&parent), None, Some(&dropped), None).unwrap_err(),
            Reason::ParamsNotNarrower
        );
    }

    #[test]
    fn numeric_lower_bound_tightens_and_boolean_is_not_a_number() {
        let parent = json!({"load_kw": {"min": 0.0, "max": 10.0, "step": 1.0}});
        let child = json!({"load_kw": {"min": 2.0, "max": 8.0, "step": 2.0}});
        assert!(narrow_parameters(None, Some(&parent), None, Some(&child)).is_ok());

        let lowered = json!({"load_kw": {"min": -1.0, "max": 8.0, "step": 1.0}});
        assert_eq!(
            narrow_parameters(None, Some(&parent), None, Some(&lowered)).unwrap_err(),
            Reason::ParamsNotNarrower
        );
        // §13.4.3: child step must be an integer multiple of the parent
        // step. 3 is a coarser grid under step 1, so it narrows.
        let coarser = json!({"load_kw": {"min": 0.0, "max": 10.0, "step": 3.0}});
        assert!(narrow_parameters(None, Some(&parent), None, Some(&coarser)).is_ok());
        let parent_step2 = json!({"load_kw": {"min": 0.0, "max": 10.0, "step": 2.0}});
        let incommensurable = json!({"load_kw": {"min": 0.0, "max": 10.0, "step": 3.0}});
        assert_eq!(
            narrow_parameters(None, Some(&parent_step2), None, Some(&incommensurable)).unwrap_err(),
            Reason::ParamsNotNarrower
        );
        let finer = json!({"load_kw": {"min": 0.0, "max": 10.0, "step": 1.0}});
        assert_eq!(
            narrow_parameters(None, Some(&parent_step2), None, Some(&finer)).unwrap_err(),
            Reason::ParamsNotNarrower
        );

        let grant = json!({"armed": true});
        let op_true = json!({"armed": true});
        let op_num = json!({"armed": 1});
        assert!(entail_parameters(Some(&grant), None, Some(&op_true)).is_ok());
        assert!(entail_parameters(Some(&grant), None, Some(&op_num)).is_err());
    }

    #[test]
    fn enum_member_missing_key_and_undeclared_key() {
        let grant = json!({"station": [1, 2, 3]});
        assert!(entail_parameters(Some(&grant), None, Some(&json!({"station": 2}))).is_ok());
        assert_eq!(
            entail_parameters(Some(&grant), None, Some(&json!({"station": 9}))).unwrap_err(),
            Reason::NotInEnum
        );
        assert_eq!(
            entail_parameters(Some(&grant), None, Some(&json!({"station": [1], "extra": 1}))).unwrap_err(),
            Reason::UndeclaredParam
        );
        assert_eq!(
            entail_parameters(Some(&grant), None, Some(&json!({"extra": 1}))).unwrap_err(),
            Reason::ParamsMissing
        );
        assert_eq!(
            entail_parameters(Some(&json!({"tables": []})), None, Some(&json!({"tables": []}))).unwrap_err(),
            Reason::EmptyBoundDeniesClass
        );
    }

    #[test]
    fn boundary_notice_cites_parameter_risk_draft() {
        let params = json!({
            "path": "../secret",
            "url": "https://example.test/hook",
            "command": "sh -c true"
        });
        let notices = boundary_notices_of(std::iter::once(&params));
        assert!(notices.len() >= 3);
        assert!(notices.iter().all(|n| n.citation == PARAMETER_RISK_DRAFT));
        assert!(notices.iter().any(|n| n.domain == BoundaryDomain::Filesystem));
        assert!(notices.iter().any(|n| n.domain == BoundaryDomain::Url));
        assert!(notices.iter().any(|n| n.domain == BoundaryDomain::Shell));
    }

    #[test]
    fn raw_params_reject_size_duplicates_surrogates_and_over_precision() {
        let big = format!("{{\"k\":\"{}\"}}", "x".repeat(600));
        assert_eq!(prepare_params(&big).unwrap_err(), Reason::InvalidParamsSize);
        assert_eq!(
            prepare_params(r#"{"a":1,"a":2}"#).unwrap_err(),
            Reason::InvalidParamsDuplicateKey
        );
        assert_eq!(
            prepare_params(r#"{"x":"\ud800"}"#).unwrap_err(),
            Reason::InvalidParamsNumber
        );
        assert_eq!(
            prepare_params(r#"{"n":1e400}"#).unwrap_err(),
            Reason::InvalidParamsNumber
        );
        assert!(prepare_params(r#"{"limit":50}"#).is_ok());
    }

    #[test]
    fn jcs_number_spelling_and_safe_integer_bound() {
        let tiny = analyze_text(r#"{"n":1e-6}"#);
        assert!(!tiny.bad_number, "{:?}", tiny.canon_len);
        assert_eq!(tiny.canon_len, r#"{"n":0.000001}"#.len());
        let one = analyze_text(r#"{"n":1.0}"#);
        assert_eq!(one.canon_len, r#"{"n":1}"#.len());
        let pair = analyze_text("{\"s\":\"\\ud83d\\ude02\"}");
        assert!(!pair.surrogate);
        assert!(!pair.bad_number);
        assert_eq!(pair.canon_len, format!("{{\"s\":\"{}\"}}", '\u{1F602}').len());

        assert!(prepare_params(r#"{"n":9007199254740991}"#).is_ok());
        assert!(prepare_params(r#"{"n":4503599627370495.5}"#).is_ok());
        assert!(prepare_params(r#"{"n":5e-324}"#).is_ok());
        assert!(prepare_params(r#"{"n":-0}"#).is_ok());
        assert!(prepare_params(r#"{"n":1e3}"#).is_ok());
        assert!(prepare_params(r#"{"n":1.5e-3}"#).is_ok());
        assert!(prepare_params(r#"{"s":"\ud83d\ude02"}"#).is_ok());
        for token in [
            "9007199254740992",
            "9007199254740993",
            "9.007199254740993e15",
            "9007199254740993.0",
            "-9007199254740993",
            "1e100",
            "1.7976931348623157e308",
        ] {
            let raw = format!("{{\"n\":{token}}}");
            assert_eq!(
                prepare_params(&raw).unwrap_err(),
                Reason::InvalidParamsNumber,
                "{token}"
            );
        }
    }

    #[test]
    fn leading_bom_outranks_size() {
        let oversize = format!("\u{FEFF}{{\"s\":\"{}\"}}", "a".repeat(600));
        assert_eq!(
            prepare_params(&oversize).unwrap_err(),
            Reason::InvalidParamsNumber
        );
        assert_eq!(
            prepare_params("\u{FEFF}{\"n\":1}").unwrap_err(),
            Reason::InvalidParamsNumber
        );
    }

    #[test]
    fn decoded_jcs_length_matches_the_canonical_spelling() {
        let expanded = serde_json::from_str::<Value>(r#"{"n":1e-6}"#).unwrap();
        assert_eq!(json_len(&expanded), r#"{"n":0.000001}"#.len());
        let shrunk = serde_json::from_str::<Value>(r#"{"n":1.0}"#).unwrap();
        assert_eq!(json_len(&shrunk), r#"{"n":1}"#.len());
        let subnormal = serde_json::from_str::<Value>(r#"{"n":5e-324}"#).unwrap();
        assert_eq!(json_len(&subnormal), r#"{"n":5e-324}"#.len());
        assert!(check_container(Some(&subnormal)).is_ok());
        let over = serde_json::from_str::<Value>(r#"{"n":9007199254740992}"#).unwrap();
        assert_eq!(
            check_container(Some(&over)).unwrap_err(),
            Reason::InvalidParamsNumber
        );
        let edge = serde_json::from_str::<Value>(r#"{"n":9007199254740991}"#).unwrap();
        assert!(check_container(Some(&edge)).is_ok());
    }
}
