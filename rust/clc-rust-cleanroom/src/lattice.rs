//! §8.5 partial order on residual-obligation status.
//!
//! Restrictiveness is the chain `Violated ≻ Satisfied ≻ Unknown`.
//! Join (`∨`) is the least upper bound (most restrictive source wins).
//! Meet (`∧`) is the greatest lower bound.

use serde::{Deserialize, Serialize};

/// Three-valued obligation status. Declaration order is restrictiveness,
/// so [`Ord::max`] is join and [`Ord::min`] is meet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatticeState {
    Unknown = 0,
    Satisfied = 1,
    Violated = 2,
}

impl LatticeState {
    /// Least upper bound. `Violated ∨ x = Violated`.
    /// `Satisfied ∨ Unknown = Satisfied`. `Unknown ∨ Unknown = Unknown`.
    pub fn join(self, other: Self) -> Self {
        self.max(other)
    }

    /// Greatest lower bound on the same chain.
    pub fn meet(self, other: Self) -> Self {
        self.min(other)
    }

    /// Fold join. An empty set of sources is `Unknown` (absent everywhere).
    pub fn collapse<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        states
            .into_iter()
            .reduce(Self::join)
            .unwrap_or(Self::Unknown)
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "unknown" => Some(Self::Unknown),
            "satisfied" => Some(Self::Satisfied),
            "violated" => Some(Self::Violated),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LatticeState::*;

    #[test]
    fn join_violated_absorbs_every_other_status() {
        assert_eq!(Violated.join(Violated), Violated);
        assert_eq!(Violated.join(Satisfied), Violated);
        assert_eq!(Satisfied.join(Violated), Violated);
        assert_eq!(Violated.join(Unknown), Violated);
        assert_eq!(Unknown.join(Violated), Violated);
    }

    #[test]
    fn join_satisfied_unknown_and_unknown_identity() {
        assert_eq!(Satisfied.join(Unknown), Satisfied);
        assert_eq!(Unknown.join(Satisfied), Satisfied);
        assert_eq!(Unknown.join(Unknown), Unknown);
        assert_eq!(Satisfied.join(Satisfied), Satisfied);
    }

    #[test]
    fn meet_is_the_greatest_lower_bound() {
        assert_eq!(Violated.meet(Satisfied), Satisfied);
        assert_eq!(Satisfied.meet(Unknown), Unknown);
        assert_eq!(Violated.meet(Unknown), Unknown);
        assert_eq!(Violated.meet(Violated), Violated);
        assert_eq!(Unknown.meet(Unknown), Unknown);
    }

    #[test]
    fn sentinel_collapse_is_deterministic() {
        // A single violated sentinel dominates satisfied and unknown sources.
        assert_eq!(
            super::LatticeState::collapse([Satisfied, Unknown, Violated, Satisfied]),
            Violated
        );
        assert_eq!(
            super::LatticeState::collapse([Unknown, Satisfied]),
            Satisfied
        );
        assert_eq!(super::LatticeState::collapse([Unknown, Unknown]), Unknown);
        assert_eq!(super::LatticeState::collapse([]), Unknown);
    }
}
