#[cfg(test)]
mod tests {
    use goap::prelude::*;
    use std::collections::HashMap;

    fn create_test_state() -> State {
        let mut state = State::empty();
        state.set("has_wood", StateVar::from_f64(1.0));
        state.set("has_tools", StateVar::from_f64(1.0));
        state
    }

    fn create_test_action() -> Action {
        let mut preconditions = State::empty();
        preconditions.set("has_wood", StateVar::from_f64(1.0));
        preconditions.set("has_tools", StateVar::from_f64(1.0));

        let mut effects = HashMap::new();
        effects.insert("has_planks".to_string(), StateOperation::set_f64(1.0));
        effects.insert("has_wood".to_string(), StateOperation::set_f64(0.0));

        Action::from_parts("make_planks", 2.0, preconditions, effects)
    }

    // Tests for basic Action creation and properties

    /// Test basic action creation using from_parts constructor
    /// Validates: Action name, cost, preconditions, and effects are properly stored
    /// Failure: Action constructor is broken or data not stored correctly
    #[test]
    fn test_action_creation() {
        let action = create_test_action();
        assert_eq!(action.name, "make_planks");
        assert_eq!(action.cost, 2.0);
        assert_eq!(action.preconditions.get::<f64>("has_wood"), Some(1.0));
        assert_eq!(action.preconditions.get::<f64>("has_tools"), Some(1.0));
        if let StateOperation::Set(value) = action.effects.get("has_planks").unwrap() {
            assert_eq!(value.as_f64(), Some(1.0));
        } else {
            panic!("Expected Set operation");
        }
    }

    // Tests for Action execution logic

    /// Test action precondition checking with valid state
    /// Validates: can_execute returns true when all preconditions are satisfied
    /// Failure: Precondition checking logic is broken
    #[test]
    fn test_can_execute_with_valid_state() {
        let action = create_test_action();
        let valid_state = create_test_state();
        assert!(action.can_execute(&valid_state));
    }

    /// Test action precondition checking with invalid state  
    /// Validates: can_execute returns false when preconditions are not met
    /// Failure: Precondition checking fails to detect missing requirements
    #[test]
    fn test_can_execute_with_invalid_state() {
        let action = create_test_action();
        let mut invalid_state = State::empty();
        invalid_state.set("has_tools", StateVar::from_f64(1.0));
        assert!(!action.can_execute(&invalid_state));
    }

    /// Test action effect application to state
    /// Validates: Effects are properly applied and original state is preserved
    /// Failure: Effect application logic is broken or modifies wrong variables
    #[test]
    fn test_apply_effects() {
        let action = create_test_action();
        let initial_state = create_test_state();
        let result_state = action.apply_effect(&initial_state);

        assert_eq!(result_state.get::<f64>("has_planks"), Some(1.0));
        assert_eq!(result_state.get::<f64>("has_wood"), Some(0.0));
        assert_eq!(result_state.get::<f64>("has_tools"), Some(1.0));
    }

    // Tests for ActionBuilder - Bool preconditions and effects

    /// Test ActionBuilder with boolean preconditions
    /// Validates: Bool values can be used as action preconditions
    /// Failure: Builder doesn't handle bool preconditions correctly
    #[test]
    fn test_builder_bool_preconditions() {
        let action = Action::new("test_action")
            .requires("has_key", true)
            .requires("door_locked", false)
            .build();

        assert_eq!(action.preconditions.get::<bool>("has_key"), Some(true));
        assert_eq!(action.preconditions.get::<bool>("door_locked"), Some(false));
    }

    /// Test ActionBuilder with boolean effects using Set operation
    /// Validates: Bool values can be set as action effects
    /// Failure: Builder doesn't handle bool effects correctly
    #[test]
    fn test_builder_bool_effects() {
        let action = Action::new("unlock_door")
            .sets("door_unlocked", true)
            .sets("has_key", false)
            .build();

        if let StateOperation::Set(value) = action.effects.get("door_unlocked").unwrap() {
            assert_eq!(value, &StateVar::Bool(true));
        } else {
            panic!("Expected Set operation for door_unlocked");
        }

        if let StateOperation::Set(value) = action.effects.get("has_key").unwrap() {
            assert_eq!(value, &StateVar::Bool(false));
        } else {
            panic!("Expected Set operation for has_key");
        }
    }

    // Tests for ActionBuilder - I64 preconditions and effects

    /// Test ActionBuilder with integer preconditions
    /// Validates: I64 values can be used as action preconditions
    /// Failure: Builder doesn't handle integer preconditions correctly
    #[test]
    fn test_builder_i64_preconditions() {
        let action = Action::new("buy_item")
            .requires("gold", 100)
            .requires("level", 5)
            .build();

        assert_eq!(action.preconditions.get::<i64>("gold"), Some(100));
        assert_eq!(action.preconditions.get::<i64>("level"), Some(5));
    }

    /// Test ActionBuilder with integer Set effects
    /// Validates: I64 values can be set as action effects
    /// Failure: Builder doesn't handle integer Set effects correctly
    #[test]
    fn test_builder_i64_set_effects() {
        let action = Action::new("set_gold")
            .sets("gold", 200)
            .sets("level", 10)
            .build();

        if let StateOperation::Set(value) = action.effects.get("gold").unwrap() {
            assert_eq!(value, &StateVar::I64(200));
        } else {
            panic!("Expected Set operation for gold");
        }

        if let StateOperation::Set(value) = action.effects.get("level").unwrap() {
            assert_eq!(value, &StateVar::I64(10));
        } else {
            panic!("Expected Set operation for level");
        }
    }

    /// Test ActionBuilder with integer Add effects
    /// Validates: I64 values can be added using Add operation
    /// Failure: Builder doesn't handle integer Add effects correctly
    #[test]
    fn test_builder_i64_add_effects() {
        let action = Action::new("gain_resources")
            .adds("gold", 50)
            .adds("experience", 100)
            .build();

        if let StateOperation::Add(value) = action.effects.get("gold").unwrap() {
            assert_eq!(*value, 50);
        } else {
            panic!("Expected Add operation for gold");
        }

        if let StateOperation::Add(value) = action.effects.get("experience").unwrap() {
            assert_eq!(*value, 100);
        } else {
            panic!("Expected Add operation for experience");
        }
    }

    /// Test ActionBuilder with integer Subtract effects
    /// Validates: I64 values can be subtracted using Subtract operation
    /// Failure: Builder doesn't handle integer Subtract effects correctly
    #[test]
    fn test_builder_i64_subtract_effects() {
        let action = Action::new("spend_resources")
            .subtracts("gold", 25)
            .subtracts("energy", 10)
            .build();

        if let StateOperation::Subtract(value) = action.effects.get("gold").unwrap() {
            assert_eq!(*value, 25);
        } else {
            panic!("Expected Subtract operation for gold");
        }

        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, 10);
        } else {
            panic!("Expected Subtract operation for energy");
        }
    }

    // Tests for ActionBuilder - F64 preconditions and effects

    /// Test ActionBuilder with float preconditions
    /// Validates: F64 values can be used as action preconditions
    /// Failure: Builder doesn't handle float preconditions correctly
    #[test]
    fn test_builder_f64_preconditions() {
        let action = Action::new("precise_action")
            .requires("health", 75.5)
            .requires("speed", 2.25)
            .build();

        assert_eq!(action.preconditions.get::<f64>("health"), Some(75.5));
        assert_eq!(action.preconditions.get::<f64>("speed"), Some(2.25));
    }

    /// Test ActionBuilder with float Set effects
    /// Validates: F64 values can be set as action effects
    /// Failure: Builder doesn't handle float Set effects correctly
    #[test]
    fn test_builder_f64_set_effects() {
        let action = Action::new("restore_health")
            .sets("health", 100.0)
            .sets("mana", 50.75)
            .build();

        if let StateOperation::Set(value) = action.effects.get("health").unwrap() {
            assert_eq!(value, &StateVar::F64(100000));
        } else {
            panic!("Expected Set operation for health");
        }

        if let StateOperation::Set(value) = action.effects.get("mana").unwrap() {
            assert_eq!(value, &StateVar::F64(50750));
        } else {
            panic!("Expected Set operation for mana");
        }
    }

    /// Test ActionBuilder with float Add effects
    /// Validates: F64 values can be added using Add operation
    /// Failure: Builder doesn't handle float Add effects correctly
    #[test]
    fn test_builder_f64_add_effects() {
        let action = Action::new("boost_stats")
            .adds("speed", 1.5)
            .adds("strength", 2.25)
            .build();

        if let StateOperation::Add(value) = action.effects.get("speed").unwrap() {
            assert_eq!(*value, 1500);
        } else {
            panic!("Expected Add operation for speed");
        }

        if let StateOperation::Add(value) = action.effects.get("strength").unwrap() {
            assert_eq!(*value, 2250);
        } else {
            panic!("Expected Add operation for strength");
        }
    }

    /// Test ActionBuilder with float Subtract effects
    /// Validates: F64 values can be subtracted using Subtract operation
    /// Failure: Builder doesn't handle float Subtract effects correctly
    #[test]
    fn test_builder_f64_subtract_effects() {
        let action = Action::new("drain_stats")
            .subtracts("energy", 10.5)
            .subtracts("stamina", 5.75)
            .build();

        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, 10500);
        } else {
            panic!("Expected Subtract operation for energy");
        }

        if let StateOperation::Subtract(value) = action.effects.get("stamina").unwrap() {
            assert_eq!(*value, 5750);
        } else {
            panic!("Expected Subtract operation for stamina");
        }
    }

    // Tests for ActionBuilder - String/Enum preconditions and effects

    /// Test ActionBuilder with string/enum preconditions
    /// Validates: String values can be used as action preconditions
    /// Failure: Builder doesn't handle string preconditions correctly
    #[test]
    fn test_builder_string_preconditions() {
        let action = Action::new("enter_area")
            .requires("location", "town")
            .requires("state", "peaceful")
            .build();

        assert_eq!(action.preconditions.get::<String>("location"), Some("town".to_string()));
        assert_eq!(action.preconditions.get::<String>("state"), Some("peaceful".to_string()));
    }

    /// Test ActionBuilder with string/enum Set effects
    /// Validates: String values can be set as action effects
    /// Failure: Builder doesn't handle string Set effects correctly
    #[test]
    fn test_builder_string_effects() {
        let action = Action::new("travel")
            .sets("location", "forest")
            .sets("weather", "rainy")
            .build();

        if let StateOperation::Set(value) = action.effects.get("location").unwrap() {
            assert_eq!(value, &StateVar::String("forest".to_string()));
        } else {
            panic!("Expected Set operation for location");
        }

        if let StateOperation::Set(value) = action.effects.get("weather").unwrap() {
            assert_eq!(value, &StateVar::String("rainy".to_string()));
        } else {
            panic!("Expected Set operation for weather");
        }
    }

    // Tests for ActionBuilder - Cost functionality

    /// Test ActionBuilder cost setting
    /// Validates: Action cost can be properly set via builder
    /// Failure: Builder doesn't handle cost setting correctly
    #[test]
    fn test_builder_cost() {
        let action = Action::new("expensive_action").cost(15.5).build();

        assert_eq!(action.cost, 15.5);
    }

    /// Test ActionBuilder default cost
    /// Validates: Actions have default cost of 1.0 when not specified
    /// Failure: Default cost is not set correctly
    #[test]
    fn test_builder_default_cost() {
        let action = Action::new("default_cost_action").build();
        assert_eq!(action.cost, 1.0);
    }

    // Tests for mixed type scenarios

    /// Test ActionBuilder with mixed precondition types
    /// Validates: Multiple different StateVar types can be used as preconditions
    /// Failure: Builder doesn't handle mixed precondition types correctly
    #[test]
    fn test_builder_mixed_preconditions() {
        let action = Action::new("complex_action")
            .requires("has_key", true)
            .requires("gold", 100)
            .requires("health", 75.5)
            .requires("location", "castle")
            .build();

        assert_eq!(action.preconditions.get::<bool>("has_key"), Some(true));
        assert_eq!(action.preconditions.get::<i64>("gold"), Some(100));
        assert_eq!(action.preconditions.get::<f64>("health"), Some(75.5));
        assert_eq!(action.preconditions.get::<String>("location"), Some("castle".to_string()));
    }

    /// Test ActionBuilder with mixed effect types
    /// Validates: Multiple different StateOperation types can be used as effects
    /// Failure: Builder doesn't handle mixed effect types correctly
    #[test]
    fn test_builder_mixed_effects() {
        let action = Action::new("complex_effects")
            .sets("door_open", true)
            .sets("gold", 200)
            .adds("experience", 50)
            .subtracts("energy", 25)
            .sets("health", 100.0)
            .adds("speed", 1.5)
            .sets("location", "treasure_room")
            .build();

        // Check bool set
        if let StateOperation::Set(value) = action.effects.get("door_open").unwrap() {
            assert_eq!(value, &StateVar::Bool(true));
        } else {
            panic!("Expected Set operation for door_open");
        }

        // Check i64 set
        if let StateOperation::Set(value) = action.effects.get("gold").unwrap() {
            assert_eq!(value, &StateVar::I64(200));
        } else {
            panic!("Expected Set operation for gold");
        }

        // Check i64 add
        if let StateOperation::Add(value) = action.effects.get("experience").unwrap() {
            assert_eq!(*value, 50);
        } else {
            panic!("Expected Add operation for experience");
        }

        // Check i64 subtract
        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, 25);
        } else {
            panic!("Expected Subtract operation for energy");
        }

        // Check f64 set
        if let StateOperation::Set(value) = action.effects.get("health").unwrap() {
            assert_eq!(value, &StateVar::F64(100000));
        } else {
            panic!("Expected Set operation for health");
        }

        // Check f64 add
        if let StateOperation::Add(value) = action.effects.get("speed").unwrap() {
            assert_eq!(*value, 1500);
        } else {
            panic!("Expected Add operation for speed");
        }

        // Check string set
        if let StateOperation::Set(value) = action.effects.get("location").unwrap() {
            assert_eq!(value, &StateVar::String("treasure_room".to_string()));
        } else {
            panic!("Expected Set operation for location");
        }
    }
}
