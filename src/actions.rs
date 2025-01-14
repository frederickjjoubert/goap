use crate::state::{StateOperation, WorldState};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Action {
    pub name: String,
    pub cost: f64,
    pub preconditions: WorldState,
    pub effects: HashMap<String, StateOperation>,
}

impl Action {
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

    pub fn execute(&self, state: &WorldState) -> WorldState {
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
        let result_state = action.execute(&initial_state);

        assert_eq!(result_state.get("has_planks").unwrap().as_f64(), Some(1.0));
        assert_eq!(result_state.get("has_wood").unwrap().as_f64(), Some(0.0));
        assert_eq!(result_state.get("has_tools").unwrap().as_f64(), Some(1.0));
    }
}
