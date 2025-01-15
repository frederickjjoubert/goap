use crate::state::{StateVar, WorldState};

/// A goal is a desired state of the world that an agent wants to achieve.
#[derive(Clone, Debug)]
pub struct Goal {
    /// The name of the goal.
    pub name: String,
    /// The desired state of the world that this goal represents.
    pub desired_state: WorldState,
}

impl Goal {
    pub fn new(name: &str, desired_state: WorldState) -> Self {
        Goal {
            name: name.to_string(),
            desired_state,
        }
    }

    pub fn is_satisfied(&self, state: &WorldState) -> bool {
        state.satisfies(&self.desired_state)
    }

    pub fn builder(name: &str) -> GoalBuilder {
        GoalBuilder::new(name)
    }
}

pub struct GoalBuilder {
    name: String,
    desired_state: WorldState,
}

impl GoalBuilder {
    pub fn new(name: &str) -> Self {
        GoalBuilder {
            name: name.to_string(),
            desired_state: WorldState::new(),
        }
    }

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
        self.desired_state.set(key, StateVar::Enum(value.into()));
        self
    }

    pub fn build(self) -> Goal {
        Goal {
            name: self.name,
            desired_state: self.desired_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateVar;

    #[test]
    fn test_goal_creation() {
        let mut desired_state = WorldState::new();
        desired_state.set("gold", StateVar::I64(100));
        desired_state.set("has_weapon", StateVar::Bool(true));
        desired_state.set("location", StateVar::Enum("shop".to_string()));

        let goal = Goal::new("get_equipment", desired_state.clone());

        assert_eq!(goal.name, "get_equipment");
        assert_eq!(goal.desired_state.get("gold"), Some(&StateVar::I64(100)));
        assert_eq!(
            goal.desired_state.get("has_weapon"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(
            goal.desired_state.get("location"),
            Some(&StateVar::Enum("shop".to_string()))
        );
    }

    #[test]
    fn test_goal_satisfaction_basic() {
        let mut desired_state = WorldState::new();
        desired_state.set("gold", StateVar::I64(50));
        let goal = Goal::new("earn_gold", desired_state);

        // Test exact match
        let mut current_state = WorldState::new();
        current_state.set("gold", StateVar::I64(50));
        assert!(goal.is_satisfied(&current_state));

        // Test exceeding goal
        let mut exceeding_state = WorldState::new();
        exceeding_state.set("gold", StateVar::I64(100));
        assert!(goal.is_satisfied(&exceeding_state));

        // Test not meeting goal
        let mut insufficient_state = WorldState::new();
        insufficient_state.set("gold", StateVar::I64(25));
        assert!(!goal.is_satisfied(&insufficient_state));
    }

    #[test]
    fn test_goal_satisfaction_multiple_conditions() {
        let mut desired_state = WorldState::new();
        desired_state.set("has_radio", StateVar::Bool(true));
        desired_state.set("at_base", StateVar::Bool(true));
        desired_state.set("reported_at_a", StateVar::Bool(true));
        let goal = Goal::new("complete_patrol", desired_state);

        // Test all conditions met
        let mut complete_state = WorldState::new();
        complete_state.set("has_radio", StateVar::Bool(true));
        complete_state.set("at_base", StateVar::Bool(true));
        complete_state.set("reported_at_a", StateVar::Bool(true));
        assert!(goal.is_satisfied(&complete_state));

        // Test partial conditions
        let mut partial_state = WorldState::new();
        partial_state.set("has_radio", StateVar::Bool(true));
        partial_state.set("at_base", StateVar::Bool(true));
        partial_state.set("reported_at_a", StateVar::Bool(false));
        assert!(!goal.is_satisfied(&partial_state));

        // Test missing condition
        let mut missing_state = WorldState::new();
        missing_state.set("has_radio", StateVar::Bool(true));
        missing_state.set("at_base", StateVar::Bool(true));
        assert!(!goal.is_satisfied(&missing_state));
    }

    #[test]
    fn test_goal_satisfaction_with_f64() {
        let mut desired_state = WorldState::new();
        desired_state.set("speed", StateVar::from_f64(1.5));
        desired_state.set("distance", StateVar::from_f64(10.0));
        let goal = Goal::new("reach_destination", desired_state);

        // Test exact match
        let mut exact_state = WorldState::new();
        exact_state.set("speed", StateVar::from_f64(1.5));
        exact_state.set("distance", StateVar::from_f64(10.0));
        assert!(goal.is_satisfied(&exact_state));

        // Test exceeding goal
        let mut exceeding_state = WorldState::new();
        exceeding_state.set("speed", StateVar::from_f64(2.0));
        exceeding_state.set("distance", StateVar::from_f64(15.0));
        assert!(goal.is_satisfied(&exceeding_state));

        // Test not meeting goal
        let mut insufficient_state = WorldState::new();
        insufficient_state.set("speed", StateVar::from_f64(1.0));
        insufficient_state.set("distance", StateVar::from_f64(5.0));
        assert!(!goal.is_satisfied(&insufficient_state));
    }

    #[test]
    fn test_goal_builder() {
        let goal = Goal::builder("reach_destination")
            .require_bool("has_fuel", true)
            .require_int("passengers", 4)
            .require_float("speed", 60.0)
            .require_enum("location", "airport")
            .build();

        assert_eq!(goal.name, "reach_destination");
        assert_eq!(
            goal.desired_state.get("has_fuel"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(
            goal.desired_state.get("passengers"),
            Some(&StateVar::I64(4))
        );
        assert_eq!(goal.desired_state.get("speed"), Some(&StateVar::F64(60000))); // 60.0 stored as fixed point
        assert_eq!(
            goal.desired_state.get("location"),
            Some(&StateVar::Enum("airport".to_string()))
        );
    }

    #[test]
    fn test_goal_builder_chaining() {
        let goal = Goal::builder("complex_goal")
            // Chain multiple bools
            .require_bool("has_fuel", true)
            .require_bool("engine_running", true)
            // Chain multiple ints
            .require_int("passengers", 4)
            .require_int("luggage", 2)
            // Chain multiple floats
            .require_float("speed", 60.0)
            .require_float("fuel_level", 75.5)
            // Chain multiple enums
            .require_enum("location", "airport")
            .require_enum("weather", "clear")
            .build();

        // Test all values were set correctly
        assert_eq!(
            goal.desired_state.get("has_fuel"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(
            goal.desired_state.get("engine_running"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(
            goal.desired_state.get("passengers"),
            Some(&StateVar::I64(4))
        );
        assert_eq!(goal.desired_state.get("luggage"), Some(&StateVar::I64(2)));
        assert_eq!(goal.desired_state.get("speed"), Some(&StateVar::F64(60000)));
        assert_eq!(
            goal.desired_state.get("fuel_level"),
            Some(&StateVar::F64(75500))
        );
        assert_eq!(
            goal.desired_state.get("location"),
            Some(&StateVar::Enum("airport".to_string()))
        );
        assert_eq!(
            goal.desired_state.get("weather"),
            Some(&StateVar::Enum("clear".to_string()))
        );
    }
}
