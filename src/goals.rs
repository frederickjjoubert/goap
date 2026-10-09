use crate::conditions::{Conditions, IntoCondition};
use crate::state::State;
use std::fmt;

/// A goal is a desired state of the world that an agent wants to achieve.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Goal {
    /// The name of the goal.
    pub name: String,
    /// The conditions that must all hold for this goal to be satisfied.
    pub conditions: Conditions,
    /// The priority of this goal. Higher values indicate higher priority.
    /// Read by [`Planner::plan_by_priority`](crate::planner::Planner::plan_by_priority),
    /// which tries goals from the highest priority down.
    pub priority: u16,
}

impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Goal '{}' (priority: {})", self.name, self.priority)?;
        if !self.conditions.is_empty() {
            write!(f, " requires:")?;
            let mut conditions: Vec<_> = self.conditions.iter().collect();
            conditions.sort_by_key(|(key, _)| *key);
            for (key, condition) in conditions {
                write!(f, "\n  - {key} {condition}")?;
            }
        }
        Ok(())
    }
}

impl Goal {
    /// Creates a goal with no conditions and the default priority of 1. Such
    /// a goal is satisfied by every state; add conditions through the public
    /// fields, or use [`Goal::builder`].
    pub fn new(name: &str) -> Self {
        Goal {
            name: name.to_string(),
            conditions: Conditions::new(),
            priority: 1,
        }
    }

    /// Creates a [`GoalBuilder`] for constructing a goal with the fluent
    /// interface:
    ///
    /// ```
    /// use goap::prelude::*;
    /// let goal = Goal::builder("rich").requires("gold", at_least(100)).build();
    /// assert_eq!(goal.conditions.len(), 1);
    /// ```
    pub fn builder(name: &str) -> GoalBuilder {
        GoalBuilder::new(name)
    }

    /// Creates a goal directly from its component parts.
    /// This is useful when you have the conditions already constructed.
    pub fn from_conditions(name: &str, conditions: Conditions, priority: u16) -> Self {
        Goal {
            name: name.to_string(),
            conditions,
            priority,
        }
    }

    /// Checks if this goal is satisfied by the given state.
    /// Returns true if the state meets every condition.
    pub fn is_satisfied(&self, state: &State) -> bool {
        self.conditions.is_met_by(state)
    }
}

/// Builder for constructing goals with a fluent interface.
/// Use [`Goal::builder`] to create one.
#[derive(Clone, Debug)]
pub struct GoalBuilder {
    /// The name of the goal being built
    name: String,
    /// The conditions that must be achieved
    conditions: Conditions,
    /// The priority of the goal (defaults to 1)
    priority: u16,
}

impl GoalBuilder {
    /// Creates a new GoalBuilder with the given name and default values.
    pub fn new(name: &str) -> Self {
        GoalBuilder {
            name: name.to_string(),
            conditions: Conditions::new(),
            priority: 1,
        }
    }

    /// Sets the priority of this goal. Higher values indicate higher priority.
    pub fn priority(mut self, priority: u16) -> Self {
        self.priority = priority;
        self
    }

    /// Adds a requirement that must be satisfied for this goal to be considered achieved.
    ///
    /// A plain value means the variable must **equal** that value. Use the
    /// helpers from the prelude for other comparisons:
    ///
    /// ```
    /// use goap::prelude::*;
    /// let goal = Goal::builder("rich")
    ///     .requires("has_wallet", true)          // equals
    ///     .requires("gold", at_least(100))       // >=
    ///     .requires("debt", at_most(0))          // <=
    ///     .build();
    /// ```
    ///
    /// Only one condition is kept per key; a second call with the same key
    /// replaces the first.
    pub fn requires<T: IntoCondition>(mut self, key: &str, condition: T) -> Self {
        self.conditions.set(key, condition);
        self
    }

    /// Builds the final Goal from the configured builder.
    pub fn build(self) -> Goal {
        Goal {
            name: self.name,
            conditions: self.conditions,
            priority: self.priority,
        }
    }
}
