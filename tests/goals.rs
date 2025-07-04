#[cfg(test)]
mod tests {
    use goap::prelude::*;

    #[test]
    fn test_goal_creation() {
        let goal = Goal::new("get_equipment")
            .requires("gold", 100)
            .requires("has_weapon", true)
            .requires("location", "shop")
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
        let goal = Goal::new("earn_gold")
            .requires("gold", 50)
            .priority(1)
            .build();

        // Test exact match
        let current_state = State::new().set("gold", 50).build();
        assert!(goal.is_satisfied(&current_state));

        // Test exceeding goal
        let exceeding_state = State::new().set("gold", 100).build();
        assert!(goal.is_satisfied(&exceeding_state));

        // Test not meeting goal
        let insufficient_state = State::new().set("gold", 25).build();
        assert!(!goal.is_satisfied(&insufficient_state));
    }

    #[test]
    fn test_goal_satisfaction_multiple_conditions() {
        let goal = Goal::new("complete_patrol")
            .requires("has_radio", true)
            .requires("at_base", true)
            .requires("reported_at_a", true)
            .priority(1)
            .build();

        // Test all conditions met
        let complete_state = State::new()
            .set("has_radio", true)
            .set("at_base", true)
            .set("reported_at_a", true)
            .build();
        assert!(goal.is_satisfied(&complete_state));

        // Test partial conditions
        let partial_state = State::new()
            .set("has_radio", true)
            .set("at_base", true)
            .set("reported_at_a", false)
            .build();
        assert!(!goal.is_satisfied(&partial_state));

        // Test missing condition
        let missing_state = State::new()
            .set("has_radio", true)
            .set("at_base", true)
            .build();
        assert!(!goal.is_satisfied(&missing_state));
    }

    #[test]
    fn test_goal_satisfaction_with_f64() {
        let goal = Goal::new("reach_destination")
            .requires("speed", 1.5)
            .requires("distance", 10.0)
            .priority(1)
            .build();

        // Test exact match
        let exact_state = State::new().set("speed", 1.5).set("distance", 10.0).build();
        assert!(goal.is_satisfied(&exact_state));

        // Test exceeding goal
        let exceeding_state = State::new().set("speed", 2.0).set("distance", 15.0).build();
        assert!(goal.is_satisfied(&exceeding_state));

        // Test not meeting goal
        let insufficient_state = State::new().set("speed", 1.0).set("distance", 5.0).build();
        assert!(!goal.is_satisfied(&insufficient_state));
    }

    #[test]
    fn test_goal_builder() {
        let goal = Goal::new("reach_destination")
            .requires("has_fuel", true)
            .requires("passengers", 4)
            .requires("speed", 60.0)
            .requires("location", "airport")
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
        let goal = Goal::new("complex_goal")
            .requires("has_fuel", true)
            .requires("engine_running", true)
            .requires("passengers", 4)
            .requires("luggage", 2)
            .requires("speed", 60.0)
            .requires("fuel_level", 75.5)
            .requires("location", "airport")
            .requires("weather", "clear")
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
