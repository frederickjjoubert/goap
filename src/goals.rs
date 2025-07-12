use crate::state::{IntoStateVar, State};
use std::fmt;

/// A goal is a desired state of the world that an agent wants to achieve.
#[derive(Clone, Debug)]
pub struct Goal {
    /// The name of the goal.
    pub name: String,
    /// The desired state of the world that this goal represents.
    pub desired_state: State,
    /// The priority of this goal. Higher values indicate higher priority.
    pub priority: u16,
}

impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Goal '{}' (priority: {})", self.name, self.priority)?;
        if !self.desired_state.vars.is_empty() {
            write!(f, " desired state:")?;
            for (key, value) in &self.desired_state.vars {
                write!(f, "\n  - {key}: {value}")?;
            }
        }
        Ok(())
    }
}

impl Goal {
    /// Creates a new GoalBuilder for constructing a goal with the given name.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(name: &str) -> GoalBuilder {
        GoalBuilder::new(name)
    }

    /// Creates a goal directly from its component parts.
    /// This is useful when you have the desired state already constructed.
    pub fn from_state(name: &str, desired_state: State, priority: u16) -> Self {
        Goal {
            name: name.to_string(),
            desired_state,
            priority,
        }
    }

    /// Checks if this goal is satisfied by the given state.
    /// Returns true if the state meets all requirements of the desired state.
    pub fn is_satisfied(&self, state: &State) -> bool {
        state.satisfies(&self.desired_state)
    }
}

/// Builder for constructing goals with a fluent interface.
/// Use `Goal::new(name)` to create a new builder.
pub struct GoalBuilder {
    /// The name of the goal being built
    name: String,
    /// The desired state that must be achieved
    desired_state: State,
    /// The priority of the goal (defaults to 1)
    priority: u16,
}

impl GoalBuilder {
    /// Creates a new GoalBuilder with the given name and default values.
    pub fn new(name: &str) -> Self {
        GoalBuilder {
            name: name.to_string(),
            desired_state: State::empty(),
            priority: 1,
        }
    }

    /// Sets the priority of this goal. Higher values indicate higher priority.
    pub fn priority(mut self, priority: u16) -> Self {
        self.priority = priority;
        self
    }

    /// Adds a requirement that must be satisfied for this goal to be considered achieved.
    /// This method accepts any type that can be converted to a StateVar.
    pub fn requires<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.desired_state.set(key, value.into_state_var());
        self
    }

    /// Builds the final Goal from the configured builder.
    pub fn build(self) -> Goal {
        Goal {
            name: self.name,
            desired_state: self.desired_state,
            priority: self.priority,
        }
    }
}
