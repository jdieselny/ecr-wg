//! Authorization constraints (§8.1, §8.3, §8.4).
//!
//! Core evaluates `max_rows`. `time` and `network` are recognized and
//! residualized. A scheme or type that names telemetry or a sentinel is an
//! out-of-band obligation and residualizes the same way. Every other
//! pair is `unknown_constraint`.

use crate::error::Reason;
use crate::lattice::LatticeState;
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

const CORE_SCHEME: &str = "varwof/constraint-v1";

static TIMESTAMP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(Z|[+-](\d{2}):(\d{2}))$",
    )
    .expect("timestamp pattern")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintOutcome {
    pub unresolved: Vec<String>,
    pub rejection: Option<Reason>,
}

/// Evaluate a grant's constraints against an operation. The first failure wins.
pub fn evaluate_constraints(constraints: &[String], op_params: Option<&Value>) -> ConstraintOutcome {
    let mut unresolved = Vec::new();
    for constraint in constraints {
        match classify(constraint, op_params) {
            Class::Satisfied => {}
            Class::Residual => unresolved.push(constraint.clone()),
            Class::Reject(reason) => {
                return ConstraintOutcome {
                    unresolved: Vec::new(),
                    rejection: Some(reason),
                };
            }
        }
    }
    unresolved.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    unresolved.dedup();
    ConstraintOutcome {
        unresolved,
        rejection: None,
    }
}

/// Core clock for a `time:window` obligation. `None` when this constraint
/// is not a core time window (the consumer then owns the status).
pub fn clock_status(constraint: &str, now: &str) -> Result<Option<LatticeState>, Reason> {
    let tod = parse_timestamp(now).ok_or(Reason::InvalidTimestamp)?;
    let Some((scheme, ty, rest)) = split_constraint(constraint) else {
        return Ok(None);
    };
    if scheme != CORE_SCHEME || ty != "time" {
        return Ok(None);
    }
    let segments = parse_window(&rest).map_err(|_| Reason::InvalidConstraint)?;
    let inside = segments.iter().any(|seg| tod >= seg.0 && tod < seg.1);
    Ok(Some(if inside {
        LatticeState::Satisfied
    } else {
        LatticeState::Violated
    }))
}

pub fn constraint_type_name(constraint: &str) -> String {
    split_constraint(constraint)
        .map(|(_, ty, _)| ty)
        .unwrap_or_else(|| "constraint".to_string())
}

pub fn timestamp_ok(now: &str) -> bool {
    parse_timestamp(now).is_some()
}

enum Class {
    Satisfied,
    Residual,
    Reject(Reason),
}

fn classify(constraint: &str, op_params: Option<&Value>) -> Class {
    let Some((scheme, ty, rest)) = split_constraint(constraint) else {
        return Class::Reject(Reason::UnknownConstraint);
    };
    if scheme == CORE_SCHEME {
        return match ty.as_str() {
            "max_rows" => match eval_max_rows(&rest, op_params) {
                Ok(()) => Class::Satisfied,
                Err(reason) => Class::Reject(reason),
            },
            "time" => match parse_window(&rest) {
                Ok(_) => Class::Residual,
                Err(reason) => Class::Reject(reason),
            },
            "network" => match parse_cidrs(&rest) {
                Ok(_) => Class::Residual,
                Err(reason) => Class::Reject(reason),
            },
            _ => Class::Reject(Reason::UnknownConstraint),
        };
    }
    if is_oob(&scheme, &ty) {
        Class::Residual
    } else {
        Class::Reject(Reason::UnknownConstraint)
    }
}

fn is_oob(scheme: &str, ty: &str) -> bool {
    let scheme = scheme.to_ascii_lowercase();
    let ty = ty.to_ascii_lowercase();
    scheme.contains("telemetry")
        || scheme.contains("sentinel")
        || ty == "telemetry"
        || ty == "sentinel"
}

fn eval_max_rows(rest: &str, op_params: Option<&Value>) -> Result<(), Reason> {
    let bound = parse_max_rows_token(rest).ok_or(Reason::InvalidConstraint)?;
    let Some(Value::Object(map)) = op_params else {
        return Err(Reason::Violated("max_rows".to_string()));
    };
    let Some(value) = map.get("max_rows") else {
        return Err(Reason::Violated("max_rows".to_string()));
    };
    let Value::Number(number) = value else {
        return Err(Reason::Violated("max_rows".to_string()));
    };
    let Some(got) = number.as_f64() else {
        return Err(Reason::Violated("max_rows".to_string()));
    };
    if !got.is_finite() || got < 0.0 || got.fract() != 0.0 || got > bound as f64 {
        return Err(Reason::Violated("max_rows".to_string()));
    }
    Ok(())
}

fn parse_max_rows_token(token: &str) -> Option<u64> {
    if token.is_empty() || token.starts_with('+') || token.starts_with('-') {
        return None;
    }
    if token.len() > 1 && token.starts_with('0') {
        return None;
    }
    if !token.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    token.parse().ok()
}

fn parse_window(rest: &str) -> Result<Vec<(u32, u32)>, Reason> {
    let json = rest
        .strip_prefix("window:")
        .ok_or(Reason::InvalidConstraint)?;
    let value: Value = serde_json::from_str(json).map_err(|_| Reason::InvalidConstraint)?;
    let Value::Array(items) = value else {
        return Err(Reason::InvalidConstraint);
    };
    if items.is_empty() || items.len() > 32 {
        return Err(Reason::InvalidConstraint);
    }
    let mut segments = Vec::with_capacity(items.len());
    for item in items {
        let Value::Object(obj) = item else {
            return Err(Reason::InvalidConstraint);
        };
        if obj.len() != 2 || !obj.contains_key("start") || !obj.contains_key("end") {
            return Err(Reason::InvalidConstraint);
        }
        let start_raw = obj["start"].as_str().ok_or(Reason::InvalidConstraint)?;
        let end_raw = obj["end"].as_str().ok_or(Reason::InvalidConstraint)?;
        let start = parse_hms(start_raw, false).ok_or(Reason::InvalidConstraint)?;
        let end = parse_hms(end_raw, true).ok_or(Reason::InvalidConstraint)?;
        if start >= end || (start == 0 && end == 86_400) {
            return Err(Reason::InvalidConstraint);
        }
        segments.push((start, end));
    }
    if !windows_ordered(&segments) {
        return Err(Reason::InvalidConstraint);
    }
    Ok(segments)
}

/// Ascending by `(startSec, endSec)`, non-overlapping on the linear day.
/// One descent is legal: a segment that ends at 86400 followed by a
/// segment that starts at 00:00 (the §8.1 / D22 midnight split).
fn windows_ordered(segments: &[(u32, u32)]) -> bool {
    let mut saw_wrap = false;
    for i in 0..segments.len() {
        for j in (i + 1)..segments.len() {
            let (a0, a1) = segments[i];
            let (b0, b1) = segments[j];
            if a0 < b1 && b0 < a1 {
                return false;
            }
        }
        if i + 1 == segments.len() {
            break;
        }
        let (a0, a1) = segments[i];
        let (b0, _b1) = segments[i + 1];
        if b0 > a0 {
            continue;
        }
        if !saw_wrap && a1 == 86_400 && b0 == 0 {
            saw_wrap = true;
            continue;
        }
        return false;
    }
    true
}

fn parse_hms(raw: &str, end: bool) -> Option<u32> {
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 2 && parts.len() != 3 {
        return None;
    }
    if parts.iter().any(|part| part.len() != 2) {
        return None;
    }
    let hour: u32 = parts[0].parse().ok()?;
    let minute: u32 = parts[1].parse().ok()?;
    let second: u32 = if parts.len() == 3 {
        parts[2].parse().ok()?
    } else {
        0
    };
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    if end && hour == 0 && minute == 0 && second == 0 {
        return Some(86_400);
    }
    Some(hour * 3600 + minute * 60 + second)
}

fn parse_cidrs(rest: &str) -> Result<Vec<String>, Reason> {
    let json = rest.strip_prefix("cidr:").ok_or(Reason::InvalidConstraint)?;
    let value: Value = serde_json::from_str(json).map_err(|_| Reason::InvalidConstraint)?;
    let Value::Array(items) = value else {
        return Err(Reason::InvalidConstraint);
    };
    if items.is_empty() || items.len() > 32 {
        return Err(Reason::InvalidConstraint);
    }
    let mut cidrs = Vec::with_capacity(items.len());
    for item in items {
        let text = item.as_str().ok_or(Reason::InvalidConstraint)?;
        if !valid_cidr(text) {
            return Err(Reason::InvalidConstraint);
        }
        cidrs.push(text.to_string());
    }
    Ok(cidrs)
}

#[cfg(test)]
pub fn ipv4_cidr_contains(outer: &str, inner: &str) -> bool {
    let (outer_net, outer_bits) = ipv4_network(outer).unwrap_or((0, 99));
    let (inner_net, inner_bits) = match ipv4_network(inner) {
        Some(parsed) => parsed,
        None => return false,
    };
    if outer_bits > 32 || inner_bits < outer_bits {
        return false;
    }
    let shift = 32 - outer_bits;
    (outer_net >> shift) == (inner_net >> shift)
}

#[cfg(test)]
fn ipv4_network(cidr: &str) -> Option<(u32, u32)> {
    let (addr, prefix) = cidr.split_once('/')?;
    let bits: u32 = prefix.parse().ok()?;
    if bits > 32 || !valid_ipv4(addr) {
        return None;
    }
    let mut net = 0u32;
    for octet in addr.split('.') {
        net = (net << 8) | octet.parse::<u32>().ok()?;
    }
    Some((net, bits))
}

fn valid_cidr(text: &str) -> bool {
    let Some((addr, prefix)) = text.rsplit_once('/') else {
        return false;
    };
    if prefix.is_empty()
        || !prefix.bytes().all(|b| b.is_ascii_digit())
        || (prefix.len() > 1 && prefix.starts_with('0'))
    {
        return false;
    }
    let Ok(bits) = prefix.parse::<u32>() else {
        return false;
    };
    if addr.contains(':') {
        bits <= 128 && valid_ipv6(addr)
    } else {
        bits <= 32 && valid_ipv4(addr)
    }
}

fn valid_ipv4(addr: &str) -> bool {
    let octets: Vec<&str> = addr.split('.').collect();
    if octets.len() != 4 {
        return false;
    }
    octets.iter().all(|octet| {
        if octet.is_empty() || (octet.len() > 1 && octet.starts_with('0')) {
            return false;
        }
        octet
            .parse::<u32>()
            .is_ok_and(|value| value <= 255 && octet.bytes().all(|b| b.is_ascii_digit()))
    })
}

fn valid_ipv6(addr: &str) -> bool {
    if addr.is_empty() {
        return false;
    }
    let compressed = addr.contains("::");
    if compressed && addr.matches("::").count() != 1 {
        return false;
    }
    let (left, right) = if compressed {
        let (left, right) = addr.split_once("::").unwrap_or(("", ""));
        (left, right)
    } else {
        (addr, "")
    };
    let left_n = if left.is_empty() {
        0
    } else {
        match count_groups(left) {
            Some(n) => n,
            None => return false,
        }
    };
    let right_n = if right.is_empty() {
        0
    } else {
        match count_groups(right) {
            Some(n) => n,
            None => return false,
        }
    };
    let groups = left_n + right_n;
    if compressed {
        groups < 8
    } else {
        groups == 8
    }
}

fn count_groups(part: &str) -> Option<usize> {
    if part.is_empty() {
        return Some(0);
    }
    let bits: Vec<&str> = part.split(':').collect();
    let mut count = 0;
    for (index, group) in bits.iter().enumerate() {
        if group.is_empty() {
            return None;
        }
        if group.contains('.') {
            if index + 1 != bits.len() || !valid_ipv4(group) {
                return None;
            }
            count += 2;
        } else if group.len() > 4 || !group.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        } else {
            count += 1;
        }
    }
    Some(count)
}

fn split_constraint(raw: &str) -> Option<(String, String, String)> {
    let slash = raw.find('/')?;
    let vendor = &raw[..slash];
    if !name_ok(vendor) {
        return None;
    }
    let rest = &raw[slash + 1..];
    let colon = rest.find(':')?;
    let product_ver = &rest[..colon];
    let (product, major) = split_product_version(product_ver)?;
    if !name_ok(product) || product.contains("-v") || major.is_empty() {
        return None;
    }
    let scheme = format!("{vendor}/{product}-v{major}");
    let after = &rest[colon + 1..];
    if after.is_empty() {
        return None;
    }
    let (ty, tail) = match after.split_once(':') {
        Some((ty, tail)) => (ty, tail),
        None => (after, ""),
    };
    if !segment_ok(ty) {
        return None;
    }
    Some((scheme, ty.to_string(), tail.to_string()))
}

fn split_product_version(rest: &str) -> Option<(&str, &str)> {
    let mut search = 0;
    let mut found = None;
    while let Some(rel) = rest[search..].find("-v") {
        let pos = search + rel;
        let after = &rest[pos + 2..];
        if !after.is_empty() && after.bytes().all(|b| b.is_ascii_digit()) {
            found = Some(pos);
        }
        search = pos + 2;
    }
    let pos = found?;
    let product = &rest[..pos];
    let major = &rest[pos + 2..];
    if product.is_empty() || product.contains("-v") {
        return None;
    }
    Some((product, major))
}

fn name_ok(raw: &str) -> bool {
    !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn segment_ok(raw: &str) -> bool {
    !raw.is_empty()
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

fn parse_timestamp(now: &str) -> Option<u32> {
    let caps = TIMESTAMP.captures(now)?;
    let month: u32 = caps.get(2)?.as_str().parse().ok()?;
    let day: u32 = caps.get(3)?.as_str().parse().ok()?;
    let hour: u32 = caps.get(4)?.as_str().parse().ok()?;
    let minute: u32 = caps.get(5)?.as_str().parse().ok()?;
    let second: u32 = caps.get(6)?.as_str().parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 59
    {
        return None;
    }
    let local = hour * 3600 + minute * 60 + second;
    let offset = match caps.get(7)?.as_str() {
        "Z" => 0i32,
        text => {
            let sign = if text.starts_with('+') { 1i32 } else { -1 };
            let hh: i32 = caps.get(8)?.as_str().parse().ok()?;
            let mm: i32 = caps.get(9)?.as_str().parse().ok()?;
            if hh > 23 || mm > 59 {
                return None;
            }
            sign * (hh * 3600 + mm * 60)
        }
    };
    Some((local as i32 - offset).rem_euclid(86_400) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn max_rows_exact_bound_allows_and_outside_domain_violates() {
        let at = "varwof/constraint-v1:max_rows:10";
        let ok = evaluate_constraints(&[at.into()], Some(&json!({"max_rows": 10})));
        assert!(ok.rejection.is_none() && ok.unresolved.is_empty());

        let over = evaluate_constraints(&[at.into()], Some(&json!({"max_rows": 11})));
        assert_eq!(over.rejection, Some(Reason::Violated("max_rows".into())));

        let absent = evaluate_constraints(&[at.into()], Some(&json!({})));
        assert_eq!(absent.rejection, Some(Reason::Violated("max_rows".into())));

        let garbage = evaluate_constraints(&[at.into()], Some(&json!({"max_rows": "garbage"})));
        assert_eq!(garbage.rejection, Some(Reason::Violated("max_rows".into())));

        let fraction = evaluate_constraints(&[at.into()], Some(&json!({"max_rows": 1.5})));
        assert_eq!(fraction.rejection, Some(Reason::Violated("max_rows".into())));
    }

    #[test]
    fn time_and_network_grammar_and_residuals() {
        let cross = "varwof/constraint-v1:time:window:[{\"start\":\"22:00\",\"end\":\"06:00\"}]";
        let bad = evaluate_constraints(&[cross.into()], None);
        assert_eq!(bad.rejection, Some(Reason::InvalidConstraint));

        let split = "varwof/constraint-v1:time:window:[{\"start\":\"22:00\",\"end\":\"00:00\"},{\"start\":\"00:00\",\"end\":\"06:00\"}]";
        let time = evaluate_constraints(&[split.into()], None);
        assert!(time.rejection.is_none() && time.unresolved.len() == 1);

        let scalar = "varwof/constraint-v1:time:window:3600";
        assert_eq!(
            evaluate_constraints(&[scalar.into()], None).rejection,
            Some(Reason::InvalidConstraint)
        );

        let naked = "varwof/constraint-v1:network:cidr:[\"10.0.0.0\"]";
        assert_eq!(
            evaluate_constraints(&[naked.into()], None).rejection,
            Some(Reason::InvalidConstraint)
        );
        let cidr = "varwof/constraint-v1:network:cidr:[\"10.0.0.0/8\",\"2001:db8::/32\"]";
        let net = evaluate_constraints(&[cidr.into()], None);
        assert!(net.rejection.is_none() && net.unresolved == vec![cidr.to_string()]);

        let foreign = "foo/database-v1:max_rows:10";
        assert_eq!(
            evaluate_constraints(&[foreign.into()], None).rejection,
            Some(Reason::UnknownConstraint)
        );
    }

    #[test]
    fn clock_outside_window_is_violated_and_cidr_nesting_holds() {
        let window = "varwof/constraint-v1:time:window:[{\"end\":\"15:00\",\"start\":\"14:00\"}]";
        let status = clock_status(window, "2026-10-19T15:30:00Z").unwrap();
        assert_eq!(status, Some(LatticeState::Violated));
        let inside = clock_status(window, "2026-10-19T14:30:00Z").unwrap();
        assert_eq!(inside, Some(LatticeState::Satisfied));
        assert!(ipv4_cidr_contains("10.0.0.0/16", "10.0.1.0/24"));
        assert!(!ipv4_cidr_contains("10.0.1.0/24", "10.0.0.0/16"));
    }

    #[test]
    fn telemetry_and_sentinel_obligations_residualize() {
        let telemetry = "example/telemetry-v1:redundant_power_path";
        let outcome = evaluate_constraints(&[telemetry.into()], None);
        assert_eq!(outcome.unresolved, vec![telemetry.to_string()]);
        assert!(outcome.rejection.is_none());
    }
}
