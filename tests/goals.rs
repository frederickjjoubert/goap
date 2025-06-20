#[cfg(test)]
mod tests {
    use goap::prelude::*;

    #[test]
    fn test_goal_creation() {
        let goal = Goal::builder("get_equipment")
            .require_int("gold", 100)
            .require_bool("has_weapon", true)
            .require_enum("location", "shop")
            .priority(1)
            .build();

        assert_eq!(goal.name, "get_equipment");
        assert_eq!(goal.desired_state.get("gold"), Some(&StateVar::I64(100)));
        assert_eq!(
            goal.desired_state.get("has_weapon"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(
            goal.desired_state.get("location"),
            Some(&StateVar::String("shop".to_string()))
        );
    }

    #[test]
    fn test_goal_satisfaction_basic() {
        let goal = Goal::builder("earn_gold")
            .require_int("gold", 50)
            .priority(1)
            .build();

        // Test exact match
        let current_state = State::builder().int("gold", 50).build();
        assert!(goal.is_satisfied(&current_state));

        // Test exceeding goal
        let exceeding_state = State::builder().int("gold", 100).build();
        assert!(goal.is_satisfied(&exceeding_state));

        // Test not meeting goal
        let insufficient_state = State::builder().int("gold", 25).build();
        assert!(!goal.is_satisfied(&insufficient_state));
    }

    #[test]
    fn test_goal_satisfaction_multiple_conditions() {
        let goal = Goal::builder("complete_patrol")
            .require_bool("has_radio", true)
            .require_bool("at_base", true)
            .require_bool("reported_at_a", true)
            .priority(1)
            .build();

        // Test all conditions met
        let complete_state = State::builder()
            .bool("has_radio", true)
            .bool("at_base", true)
            .bool("reported_at_a", true)
            .build();
        assert!(goal.is_satisfied(&complete_state));

        // Test partial conditions
        let partial_state = State::builder()
            .bool("has_radio", true)
            .bool("at_base", true)
            .bool("reported_at_a", false)
            .build();
        assert!(!goal.is_satisfied(&partial_state));

        // Test missing condition
        let missing_state = State::builder()
            .bool("has_radio", true)
            .bool("at_base", true)
            .build();
        assert!(!goal.is_satisfied(&missing_state));
    }

    #[test]
    fn test_goal_satisfaction_with_f64() {
        let goal = Goal::builder("reach_destination")
            .require_float("speed", 1.5)
            .require_float("distance", 10.0)
            .priority(1)
            .build();

        // Test exact match
        let exact_state = State::builder()
            .float("speed", 1.5)
            .float("distance", 10.0)
            .build();
        assert!(goal.is_satisfied(&exact_state));

        // Test exceeding goal
        let exceeding_state = State::builder()
            .float("speed", 2.0)
            .float("distance", 15.0)
            .build();
        assert!(goal.is_satisfied(&exceeding_state));

        // Test not meeting goal
        let insufficient_state = State::builder()
            .float("speed", 1.0)
            .float("distance", 5.0)
            .build();
        assert!(!goal.is_satisfied(&insufficient_state));
    }

    #[test]
    fn test_goal_builder() {
        let goal = Goal::builder("reach_destination")
            .require_bool("has_fuel", true)
            .require_int("passengers", 4)
            .require_float("speed", 60.0)
            .require_enum("location", "airport")
            .priority(1)
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
            Some(&StateVar::String("airport".to_string()))
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
            .priority(1)
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
            Some(&StateVar::String("airport".to_string()))
        );
        assert_eq!(
            goal.desired_state.get("weather"),
            Some(&StateVar::String("clear".to_string()))
        );
    }
}
