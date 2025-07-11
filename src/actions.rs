use crate::state::{IntoStateVar, State, StateOperation};
use std::collections::HashMap;
use std::fmt;

/// Represents an action that can be performed to change the world state.
/// Actions have preconditions that must be satisfied before they can be executed,
/// and effects that modify the world state when executed.
#[derive(Clone, Debug)]
pub struct Action {
    /// The name of the action
    pub name: String,
    /// The cost of executing this action (used for pathfinding optimization)
    pub cost: f64,
    /// The state conditions that must be met before this action can be executed
    pub preconditions: State,
    /// The state changes that occur when this action is executed
    pub effects: HashMap<String, StateOperation>,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action '{}' (cost: {:.1})", self.name, self.cost)?;

        if !self.preconditions.vars.is_empty() {
            write!(f, "\n  Preconditions:")?;
            for (key, value) in &self.preconditions.vars {
                write!(f, "\n    - {key}: {value}")?;
            }
        }

        if !self.effects.is_empty() {
            write!(f, "\n  Effects:")?;
            for (key, operation) in &self.effects {
                match operation {
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
    /// Creates a new ActionBuilder for constructing an action with the given name.
    pub fn new(name: &str) -> ActionBuilder {
        ActionBuilder::new(name)
    }

    /// Creates an action directly from its component parts.
    /// This is useful when you have the preconditions and effects already constructed.
    pub fn from_parts(
        name: &str,
        cost: f64,
        preconditions: State,
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
        state.satisfies(&self.preconditions)
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
/// Use `Action::new(name)` to create a new builder.
pub struct ActionBuilder {
    /// The name of the action being built
    name: String,
    /// The cost of the action (defaults to 1.0)
    cost: f64,
    /// The preconditions that must be met
    preconditions: State,
    /// The effects that will be applied
    effects: HashMap<String, StateOperation>,
}

impl ActionBuilder {
    /// Creates a new ActionBuilder with the given name and default values.
    pub fn new(name: &str) -> Self {
        ActionBuilder {
            name: name.to_string(),
            cost: 1.0, // Default cost
            preconditions: State::empty(),
            effects: HashMap::new(),
        }
    }

    /// Sets the cost of this action.
    pub fn cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }

    /// Adds a precondition that must be satisfied before this action can be executed.
    fn precondition<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.preconditions.set(key, value.into_state_var());
        self
    }

    /// Adds an effect that sets a state variable to a specific value.
    fn effect_set_to<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::Set(value.into_state_var()));
        self
    }

    /// Adds an effect that adds an integer value to a state variable.
    fn effect_add_int(mut self, key: &str, value: i64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::add_i64(value));
        self
    }

    /// Adds an effect that subtracts an integer value from a state variable.
    fn effect_subtract_int(mut self, key: &str, value: i64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::subtract_i64(value));
        self
    }

    /// Adds an effect that adds a floating-point value to a state variable.
    fn effect_add_float(mut self, key: &str, value: f64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::add_f64(value));
        self
    }

    /// Adds an effect that subtracts a floating-point value from a state variable.
    fn effect_subtract_float(mut self, key: &str, value: f64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::subtract_f64(value));
        self
    }

    /// Adds a precondition that must be satisfied before this action can be executed.
    /// This is an alias for the internal `precondition` method.
    pub fn requires<T: IntoStateVar>(self, key: &str, value: T) -> Self {
        self.precondition(key, value)
    }

    /// Adds an effect that sets a state variable to a specific value.
    /// This is an alias for the internal `effect_set_to` method.
    pub fn sets<T: IntoStateVar>(self, key: &str, value: T) -> Self {
        self.effect_set_to(key, value)
    }

    /// Adds an effect that adds a numeric value to a state variable.
    /// Works with both integers and floats.
    pub fn adds<T: NumericValue>(self, key: &str, value: T) -> Self {
        value.add_to_action_builder(self, key)
    }

    /// Adds an effect that subtracts a numeric value from a state variable.
    /// Works with both integers and floats.
    pub fn subtracts<T: NumericValue>(self, key: &str, value: T) -> Self {
        value.subtract_from_action_builder(self, key)
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

/// Trait for numeric values that can be added or subtracted in action effects.
/// This trait is implemented for i32, i64, f32, and f64.
pub trait NumericValue {
    /// Adds this numeric value to the specified state variable in the action builder.
    fn add_to_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder;
    /// Subtracts this numeric value from the specified state variable in the action builder.
    fn subtract_from_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder;
}

impl NumericValue for i64 {
    fn add_to_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_add_int(key, self)
    }

    fn subtract_from_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_subtract_int(key, self)
    }
}

impl NumericValue for f64 {
    fn add_to_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_add_float(key, self)
    }

    fn subtract_from_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_subtract_float(key, self)
    }
}

impl NumericValue for i32 {
    fn add_to_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_add_int(key, self as i64)
    }

    fn subtract_from_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_subtract_int(key, self as i64)
    }
}

impl NumericValue for f32 {
    fn add_to_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_add_float(key, self as f64)
    }

    fn subtract_from_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder {
        builder.effect_subtract_float(key, self as f64)
    }
}
