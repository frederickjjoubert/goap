use crate::state::{State, StateOperation, IntoStateVar};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Action {
    pub name: String,
    pub cost: f64,
    pub preconditions: State,
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
    pub fn new(name: &str) -> ActionBuilder {
        ActionBuilder::new(name)
    }

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

    pub fn builder(name: &str) -> ActionBuilder {
        ActionBuilder::new(name)
    }

    pub fn can_execute(&self, state: &State) -> bool {
        state.satisfies(&self.preconditions)
    }

    pub fn apply_effect(&self, state: &State) -> State {
        let mut new_state = state.clone();
        new_state.apply(&self.effects);
        new_state
    }
}

pub struct ActionBuilder {
    name: String,
    cost: f64,
    preconditions: State,
    effects: HashMap<String, StateOperation>,
}

impl ActionBuilder {
    pub fn new(name: &str) -> Self {
        ActionBuilder {
            name: name.to_string(),
            cost: 1.0, // Default cost
            preconditions: State::empty(),
            effects: HashMap::new(),
        }
    }

    pub fn cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }

    pub fn precondition<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.preconditions.set(key, value.into_state_var());
        self
    }

    pub fn effect_set_to<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::Set(value.into_state_var()));
        self
    }

    pub fn effect_add_int(mut self, key: &str, value: i64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::add_i64(value));
        self
    }

    pub fn effect_subtract_int(mut self, key: &str, value: i64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::subtract_i64(value));
        self
    }

    pub fn effect_add_float(mut self, key: &str, value: f64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::add_f64(value));
        self
    }

    pub fn effect_subtract_float(mut self, key: &str, value: f64) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::subtract_f64(value));
        self
    }

    /// Alias for precondition
    pub fn requires<T: IntoStateVar>(self, key: &str, value: T) -> Self {
        self.precondition(key, value)
    }

    /// Alias for effect_set_to
    pub fn sets<T: IntoStateVar>(self, key: &str, value: T) -> Self {
        self.effect_set_to(key, value)
    }

    /// Add value - works with both integers and floats
    pub fn adds<T: NumericValue>(self, key: &str, value: T) -> Self {
        value.add_to_action_builder(self, key)
    }

    /// Subtract value - works with both integers and floats  
    pub fn subtracts<T: NumericValue>(self, key: &str, value: T) -> Self {
        value.subtract_from_action_builder(self, key)
    }

    pub fn build(self) -> Action {
        Action {
            name: self.name,
            cost: self.cost,
            preconditions: self.preconditions,
            effects: self.effects,
        }
    }
}

// Trait for values that can be added/subtracted
pub trait NumericValue {
    fn add_to_action_builder(self, builder: ActionBuilder, key: &str) -> ActionBuilder;
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
