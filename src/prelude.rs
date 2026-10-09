//! Convenient imports for common GOAP usage patterns.
//!
//! This module re-exports the most commonly used types and traits from the library,
//! allowing users to import everything they need with `use goap::prelude::*;`.

/// Action-related types for defining what agents can do
pub use crate::actions::Action;
/// Condition types and the comparison helpers used with `requires`
pub use crate::conditions::{
    Condition, Conditions, Direction, IntoCondition, NumericStateVar, at_least, at_most, between,
    less_than, more_than, not,
};
/// Goal-related types for defining what agents want to achieve
pub use crate::goals::Goal;
/// Planning-related types for finding sequences of actions
pub use crate::planner::{DEFAULT_MAX_EXPANSIONS, Plan, Planner, PlannerError, VariableOrigin};
/// State-related types for representing the world state
pub use crate::state::{
    EnumStateVar, IntoStateVar, State, StateError, StateOperation, StateType, StateVar,
    TryFromStateVar,
};
