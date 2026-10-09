use crate::conditions::{Conditions, IntoCondition};
use crate::state::{IntoStateVar, NumericStateVar, State, StateOperation};
use std::collections::HashMap;
use std::fmt;

/// Represents an action that can be performed to change the world state.
/// Actions have preconditions that must be satisfied before they can be executed,
/// and effects that modify the world state when executed.
///
/// `PartialEq` compares every field; there is no `Eq` because `cost` is an
/// `f64`.
#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    /// The name of the action
    pub name: String,
    /// The cost of executing this action (used for pathfinding optimization)
    pub cost: f64,
    /// The conditions that must be met before this action can be executed
    pub preconditions: Conditions,
    /// The state changes that occur when this action is executed
    pub effects: HashMap<String, StateOperation>,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action '{}' (cost: {:.1})", self.name, self.cost)?;

        if !self.preconditions.is_empty() {
            write!(f, "\n  Preconditions:")?;
            let mut preconditions: Vec<_> = self.preconditions.iter().collect();
            preconditions.sort_by_key(|(key, _)| *key);
            for (key, condition) in preconditions {
                write!(f, "\n    - {key} {condition}")?;
            }
        }

        if !self.effects.is_empty() {
            write!(f, "\n  Effects:")?;
            let mut keys: Vec<_> = self.effects.keys().collect();
            keys.sort();
            for key in keys {
                match &self.effects[key] {
                    StateOperation::Set(value) => write!(f, "\n    - Set {key} to {value}")?,
                    StateOperation::Add(value) => write!(f, "\n    - Add {value} to {key}")?,
                    StateOperation::Subtract(value) => {
                        write!(f, "\n    - Subtract {value} from {key}")?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Action {
    /// Creates an action with cost `1.0`, no preconditions and no effects.
    /// Fill it in through the public fields, or use [`Action::builder`].
    pub fn new(name: &str) -> Self {
        Action {
            name: name.to_string(),
            cost: 1.0,
            preconditions: Conditions::new(),
            effects: HashMap::new(),
        }
    }

    /// Creates an [`ActionBuilder`] for constructing an action with the
    /// fluent interface:
    ///
    /// ```
    /// use goap::prelude::*;
    /// let mine = Action::builder("mine").cost(2.0).adds("gold", 5).build();
    /// assert_eq!(mine.cost, 2.0);
    /// ```
    pub fn builder(name: &str) -> ActionBuilder {
        ActionBuilder::new(name)
    }

    /// Creates an action directly from its component parts.
    /// This is useful when you have the preconditions and effects already constructed.
    pub fn from_parts(
        name: &str,
        cost: f64,
        preconditions: Conditions,
        effects: HashMap<String, StateOperation>,
    ) -> Self {
        Action {
            name: name.to_string(),
            cost,
            preconditions,
            effects,
        }
    }

    /// Checks if this action can be executed given the current state.
    /// Returns true if all preconditions are satisfied.
    pub fn can_execute(&self, state: &State) -> bool {
        self.preconditions.is_met_by(state)
    }

    /// Applies this action's effects to the given state, returning a new state.
    /// This does not modify the original state.
    pub fn apply_effect(&self, state: &State) -> State {
        let mut new_state = state.clone();
        new_state.apply(&self.effects);
        new_state
    }
}

/// Builder for constructing actions with a fluent interface.
/// Use [`Action::builder`] to create one.
#[derive(Clone, Debug)]
pub struct ActionBuilder {
    /// The name of the action being built
    name: String,
    /// The cost of the action (defaults to 1.0)
    cost: f64,
    /// The preconditions that must be met
    preconditions: Conditions,
    /// The effects that will be applied
    effects: HashMap<String, StateOperation>,
}

impl ActionBuilder {
    /// Creates a new ActionBuilder with the given name and default values.
    pub fn new(name: &str) -> Self {
        ActionBuilder {
            name: name.to_string(),
            cost: 1.0, // Default cost
            preconditions: Conditions::new(),
            effects: HashMap::new(),
        }
    }

    /// Sets the cost of this action. The default is `1.0`.
    ///
    /// The planner compares costs at a resolution of 0.001, the same as
    /// `f64` state values, so the cost is rounded to the nearest thousandth
    /// here and `action.cost` is exactly the value the planner will use.
    /// Negative, NaN and infinite costs are stored as given and rejected by
    /// `Planner::plan` with `InvalidActionCost`.
    pub fn cost(mut self, cost: f64) -> Self {
        self.cost = (cost * 1000.0).round() / 1000.0;
        self
    }

    /// Adds a precondition that must be satisfied before this action can be executed.
    ///
    /// A plain value means the variable must **equal** that value. Use the
    /// helpers from the prelude for other comparisons:
    ///
    /// ```
    /// use goap::prelude::*;
    /// let action = Action::builder("buy_sword")
    ///     .requires("at_shop", true)            // equals
    ///     .requires("gold", at_least(50))       // >=
    ///     .subtracts("gold", 50)
    ///     .sets("has_sword", true)
    ///     .build();
    /// ```
    ///
    /// Only one condition is kept per key; a second call with the same key
    /// replaces the first.
    pub fn requires<T: IntoCondition>(mut self, key: &str, condition: T) -> Self {
        self.preconditions.set(key, condition);
        self
    }

    /// Adds an effect that sets a state variable to `value` when the action
    /// runs. The variable is created if it does not exist.
    ///
    /// One effect is kept per key: a second `sets`/`adds`/`subtracts` on the
    /// same key replaces the first, exactly like `requires`.
    pub fn sets<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::set(value));
        self
    }

    /// Adds an effect that adds `value` to a numeric state variable.
    ///
    /// The variable must have the same type as `value` (`i64` or `f64`);
    /// `Planner::plan` rejects a mismatch before searching. A variable that is
    /// missing from the state counts as zero, so counters need no initial
    /// value:
    ///
    /// ```
    /// use goap::prelude::*;
    /// let craft = Action::builder("craft").adds("items_crafted", 1).build();
    /// let after = craft.apply_effect(&State::new());
    /// assert_eq!(after.get::<i64>("items_crafted"), Some(1));
    /// ```
    ///
    /// Only numeric values are accepted; this does not compile:
    ///
    /// ```compile_fail
    /// use goap::prelude::*;
    /// let action = Action::builder("bad").adds("flag", true).build();
    /// ```
    ///
    /// One effect is kept per key; a second call on the same key replaces
    /// the first.
    pub fn adds<T: NumericStateVar>(mut self, key: &str, value: T) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::add(value));
        self
    }

    /// Adds an effect that subtracts `value` from a numeric state variable.
    /// Same rules as [`ActionBuilder::adds`].
    pub fn subtracts<T: NumericStateVar>(mut self, key: &str, value: T) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::subtract(value));
        self
    }

    /// Builds the final Action from the configured builder.
    pub fn build(self) -> Action {
        Action {
            name: self.name,
            cost: self.cost,
            preconditions: self.preconditions,
            effects: self.effects,
        }
    }
}
