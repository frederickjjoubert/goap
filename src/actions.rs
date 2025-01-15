use crate::state::{StateOperation, StateVar, WorldState};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Action {
    pub name: String,
    pub cost: f64,
    pub preconditions: WorldState,
    pub effects: HashMap<String, StateOperation>,
}

pub struct ActionBuilder {
    name: String,
    cost: f64,
    preconditions: WorldState,
    effects: HashMap<String, StateOperation>,
}

impl ActionBuilder {
    pub fn new(name: &str) -> Self {
        ActionBuilder {
            name: name.to_string(),
            cost: 1.0, // Default cost
            preconditions: WorldState::new(),
            effects: HashMap::new(),
        }
    }

    pub fn cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }

    pub fn precondition<T: Into<StateVar>>(mut self, key: &str, value: T) -> Self {
        self.preconditions.set(key, value.into());
        self
    }

    pub fn effect_set_to<T: Into<StateVar>>(mut self, key: &str, value: T) -> Self {
        self.effects
            .insert(key.to_string(), StateOperation::Set(value.into()));
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

    pub fn build(self) -> Action {
        Action {
            name: self.name,
            cost: self.cost,
            preconditions: self.preconditions,
            effects: self.effects,
        }
    }
}

impl Action {
    pub fn builder(name: &str) -> ActionBuilder {
        ActionBuilder::new(name)
    }

    pub fn new(
        name: &str,
        cost: f64,
        preconditions: WorldState,
        effects: HashMap<String, StateOperation>,
    ) -> Self {
        Action {
            name: name.to_string(),
            cost,
            preconditions,
            effects,
        }
    }

    pub fn can_execute(&self, state: &WorldState) -> bool {
        state.satisfies(&self.preconditions)
    }

    pub fn apply_effect(&self, state: &WorldState) -> WorldState {
        let mut new_state = state.clone();
        new_state.apply(&self.effects);
        new_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateVar;

    fn create_test_state() -> WorldState {
        let mut state = WorldState::new();
        state.set("has_wood", StateVar::from_f64(1.0));
        state.set("has_tools", StateVar::from_f64(1.0));
        state
    }

    fn create_test_action() -> Action {
        let mut preconditions = WorldState::new();
        preconditions.set("has_wood", StateVar::from_f64(1.0));
        preconditions.set("has_tools", StateVar::from_f64(1.0));

        let mut effects = HashMap::new();
        effects.insert("has_planks".to_string(), StateOperation::set_f64(1.0));
        effects.insert("has_wood".to_string(), StateOperation::set_f64(0.0));

        Action::new("make_planks", 2.0, preconditions, effects)
    }

    #[test]
    fn test_action_creation() {
        let action = create_test_action();
        assert_eq!(action.name, "make_planks");
        assert_eq!(action.cost, 2.0);
        assert_eq!(
            action.preconditions.get("has_wood").unwrap().as_f64(),
            Some(1.0)
        );
        assert_eq!(
            action.preconditions.get("has_tools").unwrap().as_f64(),
            Some(1.0)
        );
        if let StateOperation::Set(value) = action.effects.get("has_planks").unwrap() {
            assert_eq!(value.as_f64(), Some(1.0));
        } else {
            panic!("Expected Set operation");
        }
    }

    #[test]
    fn test_can_execute() {
        let action = create_test_action();
        let valid_state = create_test_state();
        assert!(action.can_execute(&valid_state));

        let mut invalid_state = WorldState::new();
        invalid_state.set("has_tools", StateVar::from_f64(1.0));
        assert!(!action.can_execute(&invalid_state));
    }

    #[test]
    fn test_execute() {
        let action = create_test_action();
        let initial_state = create_test_state();
        let result_state = action.apply_effect(&initial_state);

        assert_eq!(result_state.get("has_planks").unwrap().as_f64(), Some(1.0));
        assert_eq!(result_state.get("has_wood").unwrap().as_f64(), Some(0.0));
        assert_eq!(result_state.get("has_tools").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_action_builder() {
        let action = Action::builder("test_action")
            .cost(2.5)
            .precondition("has_wood", true)
            .precondition("energy", 10)
            .effect_set_to("has_planks", true)
            .effect_subtract_float("energy", 5.0)
            .build();

        assert_eq!(action.name, "test_action");
        assert_eq!(action.cost, 2.5);
        assert_eq!(
            action.preconditions.get("has_wood"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(action.preconditions.get("energy"), Some(&StateVar::I64(10)));

        // Check effects
        if let StateOperation::Set(value) = action.effects.get("has_planks").unwrap() {
            assert_eq!(value, &StateVar::Bool(true));
        } else {
            panic!("Expected Set operation");
        }

        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, 5000); // 5.0 converted to fixed point
        } else {
            panic!("Expected Subtract operation");
        }
    }

    #[test]
    fn test_action_builder_with_numeric_operations() {
        let action = Action::builder("complex_action")
            .cost(10.0)
            // Boolean preconditions and effects
            .precondition("has_wood", true)
            .effect_set_to("has_planks", true)
            // Integer operations
            .precondition("gold", 100)
            .effect_set_to("gold", 200) // Set to 200
            .effect_add_int("gold_add", 50) // Add 50
            .effect_subtract_int("items", 1) // Subtract 1
            // Float operations
            .precondition("health", 100.0)
            .effect_set_to("health", 150.0) // Set to 150.0
            .effect_add_float("speed", 1.5) // Add 1.5
            .effect_subtract_float("energy", 10.5) // Subtract 10.5
            // Enum operations
            .precondition("location", "forest")
            .effect_set_to("location", "town")
            .build();

        // Test cost
        assert_eq!(action.cost, 10.0);

        // Test boolean preconditions and effects
        assert_eq!(
            action.preconditions.get("has_wood"),
            Some(&StateVar::Bool(true))
        );
        if let StateOperation::Set(value) = action.effects.get("has_planks").unwrap() {
            assert_eq!(*value, StateVar::Bool(true));
        } else {
            panic!("Expected Set operation for has_planks");
        }

        // Test integer preconditions and effects
        assert_eq!(action.preconditions.get("gold"), Some(&StateVar::I64(100)));
        if let StateOperation::Set(value) = action.effects.get("gold").unwrap() {
            assert_eq!(*value, StateVar::I64(200));
        } else {
            panic!("Expected Set operation for gold");
        }
        if let StateOperation::Add(value) = action.effects.get("gold_add").unwrap() {
            assert_eq!(*value, 50);
        } else {
            panic!("Expected Add operation for gold_add");
        }
        if let StateOperation::Subtract(value) = action.effects.get("items").unwrap() {
            assert_eq!(*value, 1);
        } else {
            panic!("Expected Subtract operation for items");
        }

        // Test float preconditions and effects
        assert_eq!(
            action.preconditions.get("health"),
            Some(&StateVar::F64(100000)) // 100.0 converted to fixed point
        );
        if let StateOperation::Set(value) = action.effects.get("health").unwrap() {
            assert_eq!(*value, StateVar::F64(150000)); // 150.0 converted to fixed point
        } else {
            panic!("Expected Set operation for health");
        }
        if let StateOperation::Add(value) = action.effects.get("speed").unwrap() {
            assert_eq!(*value, 1500); // 1.5 converted to fixed point
        } else {
            panic!("Expected Add operation for speed");
        }
        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, 10500); // 10.5 converted to fixed point
        } else {
            panic!("Expected Subtract operation for energy");
        }

        // Test enum preconditions and effects
        assert_eq!(
            action.preconditions.get("location"),
            Some(&StateVar::Enum("forest".to_string()))
        );
        if let StateOperation::Set(value) = action.effects.get("location").unwrap() {
            assert_eq!(*value, StateVar::Enum("town".to_string()));
        } else {
            panic!("Expected Set operation for location");
        }
    }
}
