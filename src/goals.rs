use crate::state::{State, StateVar};
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
                write!(f, "\n  - {}: {}", key, value)?;
            }
        }
        Ok(())
    }
}

impl Goal {
    pub fn new(name: &str, desired_state: State, priority: u16) -> Self {
        Goal {
            name: name.to_string(),
            desired_state,
            priority,
        }
    }

    pub fn is_satisfied(&self, state: &State) -> bool {
        state.satisfies(&self.desired_state)
    }

    pub fn builder(name: &str) -> GoalBuilder {
        GoalBuilder::new(name)
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
            desired_state: State::new(),
            priority: 1,
        }
    }

    pub fn priority(mut self, priority: u16) -> Self {
        self.priority = priority;
        self
    }

    // TODO: Rename this to be clearer, like "wants"
    pub fn require_bool(mut self, key: &str, value: bool) -> Self {
        self.desired_state.set(key, StateVar::Bool(value));
        self
    }

    pub fn require_int(mut self, key: &str, value: i64) -> Self {
        self.desired_state.set(key, StateVar::I64(value));
        self
    }

    pub fn require_float(mut self, key: &str, value: f64) -> Self {
        self.desired_state.set(key, StateVar::from_f64(value));
        self
    }

    pub fn require_enum(mut self, key: &str, value: impl Into<String>) -> Self {
        self.desired_state.set(key, StateVar::String(value.into()));
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
