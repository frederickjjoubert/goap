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
    pub fn new(name: &str) -> GoalBuilder {
        GoalBuilder::new(name)
    }

    pub fn from_state(name: &str, desired_state: State, priority: u16) -> Self {
        Goal {
            name: name.to_string(),
            desired_state,
            priority,
        }
    }

    pub fn is_satisfied(&self, state: &State) -> bool {
        state.satisfies(&self.desired_state)
    }
}

pub struct GoalBuilder {
    name: String,
    desired_state: State,
    priority: u16,
}

impl GoalBuilder {
    pub fn new(name: &str) -> Self {
        GoalBuilder {
            name: name.to_string(),
            desired_state: State::empty(),
            priority: 1,
        }
    }

    pub fn priority(mut self, priority: u16) -> Self {
        self.priority = priority;
        self
    }

    /// Unified method to require any value type
    pub fn requires<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.desired_state.set(key, value.into_state_var());
        self
    }

    pub fn build(self) -> Goal {
        Goal {
            name: self.name,
            desired_state: self.desired_state,
            priority: self.priority,
        }
    }
}
