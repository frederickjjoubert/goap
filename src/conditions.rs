//! Conditions describe what must be true of a [`State`] for a goal to be
//! satisfied or an action to be executable.
//!
//! A plain value passed to `requires` means **equality**:
//!
//! ```
//! use goap::prelude::*;
//!
//! let goal = Goal::builder("ready")
//!     .requires("has_axe", true)            // has_axe == true
//!     .requires("location", "forest")       // location == "forest"
//!     .requires("x", 8)                     // x == 8
//!     .build();
//! ```
//!
//! Every other comparison is spelled out with a helper:
//!
//! ```
//! use goap::prelude::*;
//!
//! let goal = Goal::builder("rich_and_safe")
//!     .requires("gold", at_least(100))
//!     .requires("alert_level", at_most(0))
//!     .requires("health", more_than(0))
//!     .requires("fuel", less_than(10.0))
//!     .requires("temperature", between(20.0, 24.0))
//!     .requires("location", not("vault"))
//!     .build();
//! ```
//!
//! Only one condition is stored per variable name. Calling `requires` twice
//! with the same key replaces the first condition. Use [`between`] to express
//! a range.

use crate::state::{IntoStateVar, State, StateType, StateVar};
// Defined next to `IntoStateVar`; re-exported here because the comparison
// helpers below are its main users.
pub use crate::state::NumericStateVar;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// A single requirement on one state variable.
///
/// `Equals` and `NotEquals` work for every variable type. The ordered
/// comparisons (`AtLeast`, `AtMost`, `MoreThan`, `LessThan`, `Between`) only
/// make sense for numeric variables (`I64` and `F64`); the helper functions
/// enforce this at compile time.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Condition {
    /// The variable must equal this value exactly.
    Equals(StateVar),
    /// The variable must not equal this value.
    NotEquals(StateVar),
    /// The variable must be greater than or equal to this value. Numeric only.
    AtLeast(StateVar),
    /// The variable must be less than or equal to this value. Numeric only.
    AtMost(StateVar),
    /// The variable must be strictly greater than this value. Numeric only.
    MoreThan(StateVar),
    /// The variable must be strictly less than this value. Numeric only.
    LessThan(StateVar),
    /// The variable must be within `[low, high]`, inclusive. Numeric only.
    /// The two bounds are always stored in ascending order.
    Between(StateVar, StateVar),
}

/// Which way a numeric variable must move to satisfy a condition.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    /// The condition is already satisfied.
    Satisfied,
    /// The variable must become larger.
    Increase,
    /// The variable must become smaller.
    Decrease,
    /// Either direction works (used by `NotEquals`), or the variable is not
    /// numeric and must simply be changed.
    Either,
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Equals(v) => write!(f, "== {v}"),
            Condition::NotEquals(v) => write!(f, "!= {v}"),
            Condition::AtLeast(v) => write!(f, ">= {v}"),
            Condition::AtMost(v) => write!(f, "<= {v}"),
            Condition::MoreThan(v) => write!(f, "> {v}"),
            Condition::LessThan(v) => write!(f, "< {v}"),
            Condition::Between(lo, hi) => write!(f, "in [{lo}, {hi}]"),
        }
    }
}

/// Raw integer payload of a numeric `StateVar` plus a tag for its kind, so
/// that I64 and F64 are never compared with each other.
fn numeric(v: &StateVar) -> Option<(bool, i64)> {
    match v {
        StateVar::I64(n) => Some((false, *n)),
        StateVar::F64(n) => Some((true, *n)),
        _ => None,
    }
}

/// Returns the raw values of `current` and `target` if both are numeric and
/// of the same kind.
fn numeric_pair(current: &StateVar, target: &StateVar) -> Option<(i64, i64)> {
    match (numeric(current), numeric(target)) {
        (Some((a_kind, a)), Some((b_kind, b))) if a_kind == b_kind => Some((a, b)),
        _ => None,
    }
}

impl Condition {
    /// The reference value of this condition. For `Between` this is the lower
    /// bound.
    pub fn value(&self) -> &StateVar {
        match self {
            Condition::Equals(v)
            | Condition::NotEquals(v)
            | Condition::AtLeast(v)
            | Condition::AtMost(v)
            | Condition::MoreThan(v)
            | Condition::LessThan(v)
            | Condition::Between(v, _) => v,
        }
    }

    /// The type of variable this condition applies to.
    pub fn state_type(&self) -> StateType {
        self.value().state_type()
    }

    /// Returns true if `current` is of the same type as this condition's
    /// value. A condition can never be met by a variable of another type.
    pub fn accepts_type(&self, current: &StateVar) -> bool {
        self.value().state_type() == current.state_type()
    }

    /// False for a condition that can never be satisfied because of its
    /// shape: an ordered comparison (`AtLeast`, `AtMost`, `MoreThan`,
    /// `LessThan`, `Between`) on a non-numeric value, or a `Between` whose
    /// bounds have different types. The helper functions only build
    /// well-formed conditions from numeric literals; this matters for
    /// hand-built conditions and for `StateVar` arguments. `Planner::plan`
    /// rejects malformed conditions with `InvalidCondition`.
    pub fn is_well_formed(&self) -> bool {
        match self {
            Condition::Equals(_) | Condition::NotEquals(_) => true,
            Condition::AtLeast(v)
            | Condition::AtMost(v)
            | Condition::MoreThan(v)
            | Condition::LessThan(v) => v.is_numeric(),
            Condition::Between(lo, hi) => lo.is_numeric() && lo.state_type() == hi.state_type(),
        }
    }

    /// Does `current` satisfy this condition?
    ///
    /// A type mismatch (for example an `I64` condition checked against an
    /// `F64` variable) is never satisfied.
    pub fn is_met_by(&self, current: &StateVar) -> bool {
        match self {
            Condition::Equals(v) => current == v,
            Condition::NotEquals(v) => self.accepts_type(current) && current != v,
            Condition::AtLeast(v) => numeric_pair(current, v).is_some_and(|(c, t)| c >= t),
            Condition::AtMost(v) => numeric_pair(current, v).is_some_and(|(c, t)| c <= t),
            Condition::MoreThan(v) => numeric_pair(current, v).is_some_and(|(c, t)| c > t),
            Condition::LessThan(v) => numeric_pair(current, v).is_some_and(|(c, t)| c < t),
            Condition::Between(lo, hi) => {
                numeric_pair(current, lo).is_some_and(|(c, l)| c >= l)
                    && numeric_pair(current, hi).is_some_and(|(c, h)| c <= h)
            }
        }
    }

    /// How far `current` is from the nearest value that satisfies this
    /// condition, in the variable's native units. For `F64` variables the
    /// unit is one thousandth (the fixed-point resolution).
    ///
    /// Returns `0` when the condition is already met, `1` for any unmet
    /// `Bool`/`String` condition, and `None` on a type mismatch.
    pub fn distance(&self, current: &StateVar) -> Option<u64> {
        if !self.accepts_type(current) {
            return None;
        }
        if self.is_met_by(current) {
            return Some(0);
        }
        let dist = match self {
            Condition::Equals(v) => match numeric_pair(current, v) {
                Some((c, t)) => c.abs_diff(t),
                None => 1,
            },
            Condition::NotEquals(_) => 1,
            Condition::AtLeast(v) | Condition::MoreThan(v) => {
                let (c, t) = numeric_pair(current, v)?;
                let strict = matches!(self, Condition::MoreThan(_));
                t.abs_diff(c).saturating_add(u64::from(strict))
            }
            Condition::AtMost(v) | Condition::LessThan(v) => {
                let (c, t) = numeric_pair(current, v)?;
                let strict = matches!(self, Condition::LessThan(_));
                c.abs_diff(t).saturating_add(u64::from(strict))
            }
            Condition::Between(lo, hi) => {
                let (c, l) = numeric_pair(current, lo)?;
                let (_, h) = numeric_pair(current, hi)?;
                if c < l { l.abs_diff(c) } else { c.abs_diff(h) }
            }
        };
        Some(dist)
    }

    /// Which way `current` must move to satisfy this condition.
    pub fn direction(&self, current: &StateVar) -> Direction {
        if self.is_met_by(current) {
            return Direction::Satisfied;
        }
        match self {
            Condition::Equals(v) => match numeric_pair(current, v) {
                Some((c, t)) if c < t => Direction::Increase,
                Some(_) => Direction::Decrease,
                None => Direction::Either,
            },
            Condition::NotEquals(_) => Direction::Either,
            Condition::AtLeast(_) | Condition::MoreThan(_) => Direction::Increase,
            Condition::AtMost(_) | Condition::LessThan(_) => Direction::Decrease,
            Condition::Between(lo, _) => match numeric_pair(current, lo) {
                Some((c, l)) if c < l => Direction::Increase,
                _ => Direction::Decrease,
            },
        }
    }
}

/// A named collection of conditions, one per variable.
///
/// Used for both goal requirements and action preconditions. Build one
/// through [`GoalBuilder::requires`](crate::goals::GoalBuilder::requires) or
/// [`ActionBuilder::requires`](crate::actions::ActionBuilder::requires), or
/// directly with [`Conditions::new`] and [`Conditions::set`].
///
/// The map is private so that a condition set the planner has validated
/// cannot be edited behind its back; use [`Conditions::set`],
/// [`Conditions::get`], [`Conditions::remove`] and [`Conditions::iter`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Conditions {
    /// The conditions, indexed by variable name.
    vars: HashMap<String, Condition>,
}

impl Hash for Conditions {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<_> = self.vars.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(state);
            self.vars[key].hash(state);
        }
    }
}

impl fmt::Display for Conditions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            return write!(f, "no conditions");
        }
        let mut keys: Vec<_> = self.vars.keys().collect();
        keys.sort();
        for (i, key) in keys.into_iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{key} {}", self.vars[key])?;
        }
        Ok(())
    }
}

impl Conditions {
    /// Creates an empty set of conditions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces the condition on `key`.
    pub fn set<T: IntoCondition>(&mut self, key: &str, condition: T) {
        self.vars
            .insert(key.to_string(), condition.into_condition());
    }

    /// Returns the condition on `key`, if any.
    pub fn get(&self, key: &str) -> Option<&Condition> {
        self.vars.get(key)
    }

    /// Removes and returns the condition on `key`, if any.
    pub fn remove(&mut self, key: &str) -> Option<Condition> {
        self.vars.remove(key)
    }

    /// True if there is a condition on `key`.
    pub fn contains(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    /// True if there are no conditions.
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// Number of conditions.
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// Iterates over `(variable name, condition)` pairs in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Condition)> {
        self.vars.iter()
    }

    /// True if every condition is satisfied by `state`. A variable missing
    /// from `state` fails its condition.
    pub fn is_met_by(&self, state: &State) -> bool {
        self.vars.iter().all(|(key, cond)| {
            state
                .get_var(key)
                .is_some_and(|current| cond.is_met_by(current))
        })
    }

    /// Returns the names of the conditions that `state` does not satisfy,
    /// sorted alphabetically.
    pub fn unmet_by(&self, state: &State) -> Vec<String> {
        let mut unmet: Vec<String> = self
            .vars
            .iter()
            .filter(|(key, cond)| {
                !state
                    .get_var(key)
                    .is_some_and(|current| cond.is_met_by(current))
            })
            .map(|(key, _)| key.clone())
            .collect();
        unmet.sort();
        unmet
    }
}

/// Anything that can be turned into a [`Condition`].
///
/// Plain values (`bool`, integers, floats, strings, and `EnumStateVar` types)
/// become [`Condition::Equals`]. A `Condition` converts to itself, which is
/// how the helper functions (`at_least`, `between`, ...) plug into `requires`.
pub trait IntoCondition {
    /// Converts this value into a condition.
    fn into_condition(self) -> Condition;
}

impl IntoCondition for Condition {
    fn into_condition(self) -> Condition {
        self
    }
}

// This blanket impl coexists with the one above only because `Condition`
// does not implement `IntoStateVar`. Keep it that way.
impl<T: IntoStateVar> IntoCondition for T {
    fn into_condition(self) -> Condition {
        Condition::Equals(self.into_state_var())
    }
}

/// The variable must be greater than or equal to `value`.
pub fn at_least<T: NumericStateVar>(value: T) -> Condition {
    Condition::AtLeast(value.into_state_var())
}

/// The variable must be less than or equal to `value`.
pub fn at_most<T: NumericStateVar>(value: T) -> Condition {
    Condition::AtMost(value.into_state_var())
}

/// The variable must be strictly greater than `value`.
pub fn more_than<T: NumericStateVar>(value: T) -> Condition {
    Condition::MoreThan(value.into_state_var())
}

/// The variable must be strictly less than `value`.
pub fn less_than<T: NumericStateVar>(value: T) -> Condition {
    Condition::LessThan(value.into_state_var())
}

/// The variable must be within `[low, high]`, inclusive at both ends.
///
/// The bounds may be given in either order; `between(24.0, 20.0)` is the same
/// as `between(20.0, 24.0)`.
pub fn between<T: NumericStateVar>(low: T, high: T) -> Condition {
    let (lo, hi) = (low.into_state_var(), high.into_state_var());
    let swap = matches!(
        (&lo, &hi),
        (StateVar::I64(a), StateVar::I64(b)) | (StateVar::F64(a), StateVar::F64(b)) if a > b
    );
    if swap {
        Condition::Between(hi, lo)
    } else {
        Condition::Between(lo, hi)
    }
}

/// The variable must not equal `value`. Works for every variable type.
pub fn not<T: IntoStateVar>(value: T) -> Condition {
    Condition::NotEquals(value.into_state_var())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_values_mean_equals() {
        assert_eq!(5.into_condition(), Condition::Equals(StateVar::I64(5)));
        assert_eq!(
            true.into_condition(),
            Condition::Equals(StateVar::Bool(true))
        );
        assert_eq!(
            "x".into_condition(),
            Condition::Equals(StateVar::String("x".into()))
        );
        assert_eq!(1.5.into_condition(), Condition::Equals(StateVar::F64(1500)));
    }

    #[test]
    fn between_normalises_order() {
        assert_eq!(between(24.0, 20.0), between(20.0, 24.0));
        assert_eq!(
            between(5, 1),
            Condition::Between(StateVar::I64(1), StateVar::I64(5))
        );
    }

    #[test]
    fn display_is_readable() {
        assert_eq!(at_least(100).to_string(), ">= 100");
        assert_eq!(between(20.0, 24.0).to_string(), "in [20.000, 24.000]");
        assert_eq!(not("vault").to_string(), "!= vault");
        assert_eq!(true.into_condition().to_string(), "== true");
    }

    #[test]
    fn distance_zero_iff_met() {
        let conds = [
            at_least(10),
            at_most(10),
            more_than(10),
            less_than(10),
            between(5, 10),
            10.into_condition(),
            not(10),
        ];
        for c in &conds {
            for v in -3..15 {
                let var = StateVar::I64(v);
                assert_eq!(c.is_met_by(&var), c.distance(&var) == Some(0), "{c} vs {v}");
            }
        }
    }
}
