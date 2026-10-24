//! Stable CLC reason codes (draft-wei-capability-language-core §9.1, §9.2, §13.5).
//!
//! A denial is a [`Reason`] on a decision. It is not a panic and not an I/O failure.

use thiserror::Error;

/// Normative reason code. Display text is the wire code.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Reason {
    #[error("missing_capability_id")]
    MissingCapabilityId,
    #[error("invalid_capability_id")]
    InvalidCapabilityId,
    #[error("unsupported_wildcard")]
    UnsupportedWildcard,
    #[error("different_namespace")]
    DifferentNamespace,
    #[error("literal_mismatch")]
    LiteralMismatch,
    #[error("wildcard_requires_trailing_segment")]
    WildcardRequiresTrailingSegment,
    #[error("capability_not_authorized")]
    CapabilityNotAuthorized,
    #[error("child_exceeds_parent")]
    ChildExceedsParent,
    #[error("params_not_narrower")]
    ParamsNotNarrower,
    #[error("params_missing")]
    ParamsMissing,
    #[error("undeclared_param")]
    UndeclaredParam,
    #[error("params_exceed_grant")]
    ParamsExceedGrant,
    #[error("params_out_of_range")]
    ParamsOutOfRange,
    #[error("params_not_multiple")]
    ParamsNotMultiple,
    #[error("not_in_enum")]
    NotInEnum,
    #[error("params_cardinality")]
    ParamsCardinality,
    #[error("empty_bound_denies_class")]
    EmptyBoundDeniesClass,
    #[error("invalid_params_null")]
    InvalidParamsNull,
    #[error("invalid_params_binding")]
    InvalidParamsBinding,
    #[error("invalid_params_number")]
    InvalidParamsNumber,
    #[error("invalid_params_size")]
    InvalidParamsSize,
    #[error("invalid_params_duplicate_key")]
    InvalidParamsDuplicateKey,
    #[error("unknown_constraint")]
    UnknownConstraint,
    #[error("invalid_constraint")]
    InvalidConstraint,
    #[error("invalid_resolution")]
    InvalidResolution,
    #[error("invalid_timestamp")]
    InvalidTimestamp,
    /// Declared `clc_revision` major is newer than this implementation (§12.1).
    #[error("unsupported_language_revision")]
    UnsupportedLanguageRevision,
    /// `{type}:violated` — core clock or a recognized evaluator (§8.1, §8.5).
    #[error("{0}:violated")]
    Violated(String),
}

impl Reason {
    pub fn code(&self) -> String {
        self.to_string()
    }
}
