#[cfg(test)]
mod tests {
    use goap::prelude::*;

    // Tests for basic Goal creation and properties

    /// Test basic goal creation using new constructor and builder
    /// Validates: Goal name is properly stored and builder pattern works
    /// Failure: Goal constructor is broken or name not stored correctly
    #[test]
    fn test_goal_creation_basic() {
        let goal = Goal::new("test_goal").build();
        assert_eq!(goal.name, "test_goal");
    }

    /// Test goal creation using from_state constructor
    /// Validates: Alternative constructor properly creates goal with given state and priority
    /// Failure: from_state constructor is broken or parameters not stored correctly
    #[test]
    fn test_goal_creation_from_state() {
        let state = State::new().set("has_item", true).build();
        let goal = Goal::from_state("test_goal", state, 5);

        assert_eq!(goal.name, "test_goal");
        assert_eq!(goal.priority, 5);
        assert_eq!(
            goal.desired_state.get("has_item"),
            Some(&StateVar::Bool(true))
        );
    }

    // Tests for GoalBuilder - Bool requirements

    /// Test GoalBuilder with boolean requirements
    /// Validates: Bool values can be used as goal requirements
    /// Failure: Builder doesn't handle bool requirements correctly
    #[test]
    fn test_builder_bool_requirements() {
        let goal = Goal::new("bool_goal")
            .requires("has_key", true)
            .requires("door_locked", false)
            .build();

        assert_eq!(
            goal.desired_state.get("has_key"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(
            goal.desired_state.get("door_locked"),
            Some(&StateVar::Bool(false))
        );
    }

    // Tests for GoalBuilder - I64 requirements

    /// Test GoalBuilder with integer requirements
    /// Validates: I64 values can be used as goal requirements
    /// Failure: Builder doesn't handle integer requirements correctly
    #[test]
    fn test_builder_i64_requirements() {
        let goal = Goal::new("resource_goal")
            .requires("gold", 100)
            .requires("level", 5)
            .build();

        assert_eq!(goal.desired_state.get("gold"), Some(&StateVar::I64(100)));
        assert_eq!(goal.desired_state.get("level"), Some(&StateVar::I64(5)));
    }

    // Tests for GoalBuilder - F64 requirements

    /// Test GoalBuilder with float requirements
    /// Validates: F64 values can be used as goal requirements
    /// Failure: Builder doesn't handle float requirements correctly
    #[test]
    fn test_builder_f64_requirements() {
        let goal = Goal::new("precision_goal")
            .requires("health", 75.5)
            .requires("speed", 2.25)
            .build();

        assert_eq!(
            goal.desired_state.get("health"),
            Some(&StateVar::F64(75500))
        );
        assert_eq!(goal.desired_state.get("speed"), Some(&StateVar::F64(2250)));
    }

    // Tests for GoalBuilder - String requirements

    /// Test GoalBuilder with string/enum requirements
    /// Validates: String values can be used as goal requirements
    /// Failure: Builder doesn't handle string requirements correctly
    #[test]
    fn test_builder_string_requirements() {
        let goal = Goal::new("location_goal")
            .requires("location", "town")
            .requires("weather", "sunny")
            .build();

        assert_eq!(
            goal.desired_state.get("location"),
            Some(&StateVar::String("town".to_string()))
        );
        assert_eq!(
            goal.desired_state.get("weather"),
            Some(&StateVar::String("sunny".to_string()))
        );
    }

    // Tests for GoalBuilder - Priority functionality

    /// Test GoalBuilder priority setting
    /// Validates: Goal priority can be properly set via builder
    /// Failure: Builder doesn't handle priority setting correctly
    #[test]
    fn test_builder_priority() {
        let goal = Goal::new("high_priority_goal").priority(10).build();

        assert_eq!(goal.priority, 10);
    }

    /// Test GoalBuilder default priority
    /// Validates: Goals have default priority of 1 when not specified
    /// Failure: Default priority is not set correctly
    #[test]
    fn test_builder_default_priority() {
        let goal = Goal::new("default_priority_goal").build();
        assert_eq!(goal.priority, 1);
    }

    // Tests for GoalBuilder - Mixed requirements

    /// Test GoalBuilder with mixed requirement types
    /// Validates: Multiple different StateVar types can be used as requirements
    /// Failure: Builder doesn't handle mixed requirement types correctly
    #[test]
    fn test_builder_mixed_requirements() {
        let goal = Goal::new("complex_goal")
            .requires("has_key", true)
            .requires("gold", 100)
            .requires("health", 75.5)
            .requires("location", "castle")
            .priority(5)
            .build();

        assert_eq!(
            goal.desired_state.get("has_key"),
            Some(&StateVar::Bool(true))
        );
        assert_eq!(goal.desired_state.get("gold"), Some(&StateVar::I64(100)));
        assert_eq!(
            goal.desired_state.get("health"),
            Some(&StateVar::F64(75500))
        );
        assert_eq!(
            goal.desired_state.get("location"),
            Some(&StateVar::String("castle".to_string()))
        );
        assert_eq!(goal.priority, 5);
    }

    // Tests for Goal satisfaction logic - Bool satisfaction

    /// Test goal satisfaction with boolean requirements
    /// Validates: Bool goal requirements are satisfied when state matches exactly
    /// Failure: Boolean goal satisfaction logic is broken
    #[test]
    fn test_satisfaction_bool_exact_match() {
        let goal = Goal::new("bool_goal").requires("has_key", true).build();

        let state = State::new().set("has_key", true).build();
        assert!(goal.is_satisfied(&state));
    }

    /// Test goal dissatisfaction with boolean requirements
    /// Validates: Bool goal requirements are not satisfied when state doesn't match
    /// Failure: Boolean goal satisfaction fails to detect mismatches
    #[test]
    fn test_satisfaction_bool_mismatch() {
        let goal = Goal::new("bool_goal").requires("has_key", true).build();

        let state = State::new().set("has_key", false).build();
        assert!(!goal.is_satisfied(&state));
    }

    // Tests for Goal satisfaction logic - I64 satisfaction

    /// Test goal satisfaction with integer requirements (exact match)
    /// Validates: I64 goal requirements are satisfied when state matches exactly
    /// Failure: Integer goal satisfaction logic is broken for exact matches
    #[test]
    fn test_satisfaction_i64_exact_match() {
        let goal = Goal::new("resource_goal").requires("gold", 100).build();

        let state = State::new().set("gold", 100).build();
        assert!(goal.is_satisfied(&state));
    }

    /// Test goal satisfaction with integer requirements (exceeding)
    /// Validates: I64 goal requirements are satisfied when state exceeds requirement
    /// Failure: Integer goal satisfaction doesn't handle exceeding values correctly
    #[test]
    fn test_satisfaction_i64_exceeding() {
        let goal = Goal::new("resource_goal").requires("gold", 100).build();

        let state = State::new().set("gold", 150).build();
        assert!(goal.is_satisfied(&state));
    }

    /// Test goal dissatisfaction with integer requirements (insufficient)
    /// Validates: I64 goal requirements are not satisfied when state is insufficient
    /// Failure: Integer goal satisfaction fails to detect insufficient values
    #[test]
    fn test_satisfaction_i64_insufficient() {
        let goal = Goal::new("resource_goal").requires("gold", 100).build();

        let state = State::new().set("gold", 50).build();
        assert!(!goal.is_satisfied(&state));
    }

    // Tests for Goal satisfaction logic - F64 satisfaction

    /// Test goal satisfaction with float requirements (exact match)
    /// Validates: F64 goal requirements are satisfied when state matches exactly
    /// Failure: Float goal satisfaction logic is broken for exact matches
    #[test]
    fn test_satisfaction_f64_exact_match() {
        let goal = Goal::new("precision_goal").requires("health", 75.5).build();

        let state = State::new().set("health", 75.5).build();
        assert!(goal.is_satisfied(&state));
    }

    /// Test goal satisfaction with float requirements (exceeding)
    /// Validates: F64 goal requirements are satisfied when state exceeds requirement
    /// Failure: Float goal satisfaction doesn't handle exceeding values correctly
    #[test]
    fn test_satisfaction_f64_exceeding() {
        let goal = Goal::new("precision_goal").requires("health", 75.5).build();

        let state = State::new().set("health", 100.0).build();
        assert!(goal.is_satisfied(&state));
    }

    /// Test goal dissatisfaction with float requirements (insufficient)
    /// Validates: F64 goal requirements are not satisfied when state is insufficient
    /// Failure: Float goal satisfaction fails to detect insufficient values
    #[test]
    fn test_satisfaction_f64_insufficient() {
        let goal = Goal::new("precision_goal").requires("health", 75.5).build();

        let state = State::new().set("health", 50.0).build();
        assert!(!goal.is_satisfied(&state));
    }

    // Tests for Goal satisfaction logic - String satisfaction

    /// Test goal satisfaction with string requirements (exact match)
    /// Validates: String goal requirements are satisfied when state matches exactly
    /// Failure: String goal satisfaction logic is broken
    #[test]
    fn test_satisfaction_string_exact_match() {
        let goal = Goal::new("location_goal")
            .requires("location", "town")
            .build();

        let state = State::new().set("location", "town").build();
        assert!(goal.is_satisfied(&state));
    }

    /// Test goal dissatisfaction with string requirements (mismatch)
    /// Validates: String goal requirements are not satisfied when state doesn't match
    /// Failure: String goal satisfaction fails to detect mismatches
    #[test]
    fn test_satisfaction_string_mismatch() {
        let goal = Goal::new("location_goal")
            .requires("location", "town")
            .build();

        let state = State::new().set("location", "forest").build();
        assert!(!goal.is_satisfied(&state));
    }

    // Tests for Goal satisfaction logic - Multiple requirements

    /// Test goal satisfaction with multiple requirements (all satisfied)
    /// Validates: Goals with multiple requirements are satisfied when all conditions are met
    /// Failure: Multi-requirement goal satisfaction logic is broken
    #[test]
    fn test_satisfaction_multiple_requirements_all_met() {
        let goal = Goal::new("complex_goal")
            .requires("has_key", true)
            .requires("gold", 100)
            .requires("location", "castle")
            .build();

        let state = State::new()
            .set("has_key", true)
            .set("gold", 150)
            .set("location", "castle")
            .build();

        assert!(goal.is_satisfied(&state));
    }

    /// Test goal dissatisfaction with multiple requirements (partial satisfaction)
    /// Validates: Goals with multiple requirements are not satisfied when some conditions are unmet
    /// Failure: Multi-requirement goal satisfaction fails to detect partial satisfaction
    #[test]
    fn test_satisfaction_multiple_requirements_partial() {
        let goal = Goal::new("complex_goal")
            .requires("has_key", true)
            .requires("gold", 100)
            .requires("location", "castle")
            .build();

        let state = State::new()
            .set("has_key", true)
            .set("gold", 50) // Insufficient gold
            .set("location", "castle")
            .build();

        assert!(!goal.is_satisfied(&state));
    }

    // Tests for Goal satisfaction logic - Edge cases

    /// Test goal dissatisfaction with missing state variables
    /// Validates: Goals are not satisfied when required state variables are missing
    /// Failure: Goal satisfaction doesn't properly handle missing variables
    #[test]
    fn test_satisfaction_missing_variables() {
        let goal = Goal::new("missing_var_goal")
            .requires("has_key", true)
            .requires("gold", 100)
            .build();

        let state = State::new()
            .set("has_key", true)
            // Missing "gold" variable
            .build();

        assert!(!goal.is_satisfied(&state));
    }

    /// Test goal satisfaction with empty requirements
    /// Validates: Goals with no requirements are always satisfied
    /// Failure: Empty goal satisfaction logic is broken
    #[test]
    fn test_satisfaction_empty_requirements() {
        let goal = Goal::new("empty_goal").build();
        let state = State::new().set("random_var", true).build();

        assert!(goal.is_satisfied(&state));
    }

    /// Test goal satisfaction with empty state
    /// Validates: Goals with requirements are not satisfied by empty state
    /// Failure: Goal satisfaction with empty state is broken
    #[test]
    fn test_satisfaction_empty_state() {
        let goal = Goal::new("req_goal").requires("has_item", true).build();

        let empty_state = State::empty();
        assert!(!goal.is_satisfied(&empty_state));
    }
}
