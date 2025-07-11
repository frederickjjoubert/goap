/// Convenient imports for common GOAP usage patterns.
///
/// This module re-exports the most commonly used types and traits from the library,
/// allowing users to import everything they need with `use goap::prelude::*;`.
///

/// Action-related types for defining what agents can do
pub use crate::actions::{Action, NumericValue};
/// Goal-related types for defining what agents want to achieve
pub use crate::goals::Goal;
/// Planning-related types for finding sequences of actions
pub use crate::planner::{Plan, Planner, PlannerError};
/// State-related types for representing the world state
pub use crate::state::{
    EnumStateVar, IntoStateVar, State, StateError, StateOperation, StateVar, TryFromStateVar,
};
