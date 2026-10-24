//! Capability identifier grammar (§3) and path coverage (§6.1, §13.4.2).
//!
//! Wildcard-shape checks run before the generic grammar check. The only
//! v1 wildcard is a complete trailing segment `*`, and it matches one or
//! more remaining segments, never zero.

use crate::error::Reason;

/// Parsed capability identifier. Segments do not include a trailing `*`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityId {
    pub raw: String,
    pub scheme: String,
    pub segments: Vec<String>,
    pub wildcard: bool,
    /// `scheme:action-class` — the first two colon-delimited segments (§9.1).
    pub namespace: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverError {
    DifferentNamespace,
    LiteralMismatch,
    WildcardRequiresTrailingSegment,
}

/// Parse `vendor/product-vMAJOR:action[:segment]…` with an optional trailing `:*`.
pub fn parse_capability_id(raw: &str) -> Result<CapabilityId, Reason> {
    if raw.is_empty() {
        return Err(Reason::MissingCapabilityId);
    }
    if forbidden_wildcard(raw) {
        return Err(Reason::UnsupportedWildcard);
    }
    let (scheme, action) = split_scheme(raw)?;
    let wildcard = action.ends_with(":*") || action == "*";
    let body = if wildcard {
        action.trim_end_matches(":*").trim_end_matches('*')
    } else {
        action
    };
    if wildcard && body.is_empty() {
        // `scheme:*` has the legal star shape but no action segment.
        return Err(Reason::InvalidCapabilityId);
    }
    if body.is_empty() {
        return Err(Reason::InvalidCapabilityId);
    }
    let mut segments = Vec::new();
    for segment in body.split(':') {
        if !valid_segment(segment) {
            return Err(Reason::InvalidCapabilityId);
        }
        segments.push(segment.to_string());
    }
    let namespace = format!("{}:{}", scheme, segments[0]);
    Ok(CapabilityId {
        raw: raw.to_string(),
        scheme,
        segments,
        wildcard,
        namespace,
    })
}

/// §6.1 / §13.4.2 path coverage. Parameters are not consulted.
pub fn covers(grant: &CapabilityId, child: &CapabilityId) -> Result<(), CoverError> {
    if grant.namespace != child.namespace {
        return Err(CoverError::DifferentNamespace);
    }
    if !grant.wildcard {
        if !child.wildcard && grant.segments == child.segments {
            return Ok(());
        }
        return Err(CoverError::LiteralMismatch);
    }
    if !child.segments.starts_with(&grant.segments) {
        return Err(CoverError::LiteralMismatch);
    }
    if child.wildcard {
        // Identical or more-specific wildcard prefixes denote coverage.
        if child.segments.len() >= grant.segments.len() {
            return Ok(());
        }
        return Err(CoverError::WildcardRequiresTrailingSegment);
    }
    if child.segments.len() > grant.segments.len() {
        return Ok(());
    }
    Err(CoverError::WildcardRequiresTrailingSegment)
}

fn forbidden_wildcard(raw: &str) -> bool {
    if raw.contains(['{', '}', '[', ']']) || raw.contains("**") {
        return true;
    }
    if !raw.contains('*') {
        return false;
    }
    match raw.rsplit_once(':') {
        Some((head, "*")) => head.contains('*'),
        _ => true,
    }
}

fn split_scheme(raw: &str) -> Result<(String, &str), Reason> {
    let slash = raw.find('/').ok_or(Reason::InvalidCapabilityId)?;
    let vendor = &raw[..slash];
    if !valid_name(vendor) {
        return Err(Reason::InvalidCapabilityId);
    }
    let rest = &raw[slash + 1..];
    let scheme_end = rest.find(':').ok_or(Reason::InvalidCapabilityId)?;
    let product_ver = &rest[..scheme_end];
    let (product, major) = split_product_version(product_ver).ok_or(Reason::InvalidCapabilityId)?;
    if !valid_name(product) || product.contains("-v") {
        return Err(Reason::InvalidCapabilityId);
    }
    if major.is_empty() || !major.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Reason::InvalidCapabilityId);
    }
    let scheme = format!("{vendor}/{product}-v{major}");
    let action = &rest[scheme_end + 1..];
    if action.is_empty() {
        return Err(Reason::InvalidCapabilityId);
    }
    Ok((scheme, action))
}

/// Last `-v` whose remainder is all digits. A product that itself
/// contains the two-character sequence `-v` is out of grammar (§3).
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
        if search > rest.len() {
            break;
        }
    }
    let pos = found?;
    let product = &rest[..pos];
    let major = &rest[pos + 2..];
    if product.is_empty() || product.contains("-v") {
        return None;
    }
    Some((product, major))
}

fn valid_name(raw: &str) -> bool {
    !raw.is_empty()
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn valid_segment(raw: &str) -> bool {
    !raw.is_empty()
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// §12.1. A declared major newer than 1 is refused before evaluation.
/// `CLC-1`, `CLC-1.0`, and `CLC-1.19` are the same major. Absent or
/// unprefixed text is not a refusal — only a parsed major greater than 1.
pub fn check_clc_revision(declared: &str) -> Result<(), Reason> {
    let Some(rest) = declared.strip_prefix("CLC-") else {
        return Ok(());
    };
    let major_text = rest.split(['.', '-', '+']).next().unwrap_or("");
    let Ok(major) = major_text.parse::<u32>() else {
        return Ok(());
    };
    if major > 1 {
        Err(Reason::UnsupportedLanguageRevision)
    } else {
        Ok(())
    }
}

/// Namespace of a raw identifier, including the class-position star
/// `scheme:*` which [`parse_capability_id`] rejects. `None` when the
/// scheme or the first action segment is not well-formed.
fn class_namespace(raw: &str) -> Option<String> {
    let (scheme, action) = split_scheme(raw).ok()?;
    if action == "*" {
        return Some(format!("{scheme}:*"));
    }
    let first = action.split(':').next().unwrap_or("");
    if !valid_segment(first) {
        return None;
    }
    Some(format!("{scheme}:{first}"))
}

/// Two-identifier relation. Namespace equality is decided before wildcard
/// shape and the generic grammar, so `std/database-v1:*` against another
/// namespace is `different_namespace` even though that star is not a
/// parseable capability id on its own.
pub fn relate_identifiers(grant: &str, request: &str) -> Result<(), Reason> {
    if let (Some(grant_ns), Some(request_ns)) = (class_namespace(grant), class_namespace(request)) {
        if grant_ns != request_ns {
            return Err(Reason::DifferentNamespace);
        }
    }
    let grant_id = parse_capability_id(grant)?;
    let request_id = parse_capability_id(request)?;
    covers(&grant_id, &request_id).map_err(|err| match err {
        CoverError::DifferentNamespace => Reason::DifferentNamespace,
        CoverError::LiteralMismatch => Reason::LiteralMismatch,
        CoverError::WildcardRequiresTrailingSegment => Reason::WildcardRequiresTrailingSegment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_literal_and_trailing_wildcard() {
        let literal = parse_capability_id("std/database-v1:query:SELECT").unwrap();
        assert!(!literal.wildcard);
        assert_eq!(literal.namespace, "std/database-v1:query");
        assert_eq!(literal.segments, vec!["query", "SELECT"]);

        let wild = parse_capability_id("example/power-v1:curtail:*").unwrap();
        assert!(wild.wildcard);
        assert_eq!(wild.segments, vec!["curtail"]);
        assert_eq!(wild.namespace, "example/power-v1:curtail");
    }

    #[test]
    fn wildcard_matches_one_or_more_segments_on_a_boundary() {
        let grant = parse_capability_id("std/database-v1:query:*").unwrap();
        let one = parse_capability_id("std/database-v1:query:SELECT").unwrap();
        let deep = parse_capability_id("std/database-v1:query:SELECT:deep").unwrap();
        let empty = parse_capability_id("std/database-v1:query").unwrap();
        let lexical = parse_capability_id("std/database-v1:queryEXTRA").unwrap();
        assert!(covers(&grant, &one).is_ok());
        assert!(covers(&grant, &deep).is_ok());
        assert_eq!(
            covers(&grant, &empty),
            Err(CoverError::WildcardRequiresTrailingSegment)
        );
        assert_eq!(covers(&grant, &lexical), Err(CoverError::DifferentNamespace));
    }

    #[test]
    fn forbidden_wildcard_shapes_precede_generic_grammar() {
        for raw in [
            "std/database-v1:query:SEL*",
            "std/crm-v1:re*",
            "*",
            "*:query:SELECT",
            "std/database-v1:*:SELECT",
            "std/database-v1:query:**",
            "{a,b}",
            "std/database-v1:query:[a-z]",
        ] {
            assert_eq!(
                parse_capability_id(raw),
                Err(Reason::UnsupportedWildcard),
                "{raw}"
            );
        }
    }

    #[test]
    fn product_containing_dash_v_is_rejected() {
        assert_eq!(
            parse_capability_id("a/b-v1-v2:act"),
            Err(Reason::InvalidCapabilityId)
        );
        assert_eq!(
            parse_capability_id("std/database-v1:*"),
            Err(Reason::InvalidCapabilityId)
        );
    }

    #[test]
    fn literal_mismatch_and_equivalent_wildcards() {
        let select = parse_capability_id("std/database-v1:query:SELECT").unwrap();
        let insert = parse_capability_id("std/database-v1:query:INSERT").unwrap();
        assert_eq!(covers(&select, &insert), Err(CoverError::LiteralMismatch));

        let wild = parse_capability_id("example/power-v1:curtail:*").unwrap();
        assert!(covers(&wild, &wild).is_ok());
        let admin = parse_capability_id("std/database-v1:admin:DDL").unwrap();
        let query = parse_capability_id("std/database-v1:query:*").unwrap();
        assert_eq!(covers(&query, &admin), Err(CoverError::DifferentNamespace));
    }

    #[test]
    fn class_position_star_is_a_namespace_before_it_is_an_invalid_id() {
        assert_eq!(
            parse_capability_id("std/database-v1:*"),
            Err(Reason::InvalidCapabilityId)
        );
        assert_eq!(
            relate_identifiers("std/database-v1:*", "std/database-v1:query:SELECT"),
            Err(Reason::DifferentNamespace)
        );
        assert_eq!(
            relate_identifiers("std/database-v1:*", "std/database-v1:admin:DDL"),
            Err(Reason::DifferentNamespace)
        );
        assert!(relate_identifiers(
            "std/database-v1:query:*",
            "std/database-v1:query:SELECT"
        )
        .is_ok());
    }

    #[test]
    fn revision_major_above_one_is_unsupported() {
        assert!(check_clc_revision("CLC-1.19").is_ok());
        assert!(check_clc_revision("CLC-1.0").is_ok());
        assert!(check_clc_revision("CLC-1").is_ok());
        assert_eq!(
            check_clc_revision("CLC-2.0"),
            Err(Reason::UnsupportedLanguageRevision)
        );
    }
}
