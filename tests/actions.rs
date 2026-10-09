#[cfg(test)]
mod tests {
    use goap::prelude::*;
    use std::collections::HashMap;

    fn create_test_state() -> State {
        let mut state = State::new();
        state.set("has_wood", StateVar::from_f64(1.0));
        state.set("has_tools", StateVar::from_f64(1.0));
        state
    }

    fn create_test_action() -> Action {
        let mut preconditions = Conditions::new();
        preconditions.set("has_wood", 1.0);
        preconditions.set("has_tools", 1.0);

        let mut effects = HashMap::new();
        effects.insert("has_planks".to_string(), StateOperation::set(1.0));
        effects.insert("has_wood".to_string(), StateOperation::set(0.0));

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
        assert_eq!(
            action.preconditions.get("has_wood"),
            Some(&Condition::Equals(1.0.into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("has_tools"),
            Some(&Condition::Equals(1.0.into_state_var()))
        );
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
        let mut invalid_state = State::new();
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
        let action = Action::builder("test_action")
            .requires("has_key", true)
            .requires("door_locked", false)
            .build();

        assert_eq!(
            action.preconditions.get("has_key"),
            Some(&Condition::Equals(true.into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("door_locked"),
            Some(&Condition::Equals(false.into_state_var()))
        );
    }

    /// Test ActionBuilder with boolean effects using Set operation
    /// Validates: Bool values can be set as action effects
    /// Failure: Builder doesn't handle bool effects correctly
    #[test]
    fn test_builder_bool_effects() {
        let action = Action::builder("unlock_door")
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
        let action = Action::builder("buy_item")
            .requires("gold", 100)
            .requires("level", 5)
            .build();

        assert_eq!(
            action.preconditions.get("gold"),
            Some(&Condition::Equals(100.into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("level"),
            Some(&Condition::Equals(5.into_state_var()))
        );
    }

    /// Test ActionBuilder with integer Set effects
    /// Validates: I64 values can be set as action effects
    /// Failure: Builder doesn't handle integer Set effects correctly
    #[test]
    fn test_builder_i64_set_effects() {
        let action = Action::builder("set_gold")
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
        let action = Action::builder("gain_resources")
            .adds("gold", 50)
            .adds("experience", 100)
            .build();

        if let StateOperation::Add(value) = action.effects.get("gold").unwrap() {
            assert_eq!(*value, StateVar::I64(50));
        } else {
            panic!("Expected Add operation for gold");
        }

        if let StateOperation::Add(value) = action.effects.get("experience").unwrap() {
            assert_eq!(*value, StateVar::I64(100));
        } else {
            panic!("Expected Add operation for experience");
        }
    }

    /// Test ActionBuilder with integer Subtract effects
    /// Validates: I64 values can be subtracted using Subtract operation
    /// Failure: Builder doesn't handle integer Subtract effects correctly
    #[test]
    fn test_builder_i64_subtract_effects() {
        let action = Action::builder("spend_resources")
            .subtracts("gold", 25)
            .subtracts("energy", 10)
            .build();

        if let StateOperation::Subtract(value) = action.effects.get("gold").unwrap() {
            assert_eq!(*value, StateVar::I64(25));
        } else {
            panic!("Expected Subtract operation for gold");
        }

        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, StateVar::I64(10));
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
        let action = Action::builder("precise_action")
            .requires("health", 75.5)
            .requires("speed", 2.25)
            .build();

        assert_eq!(
            action.preconditions.get("health"),
            Some(&Condition::Equals(75.5.into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("speed"),
            Some(&Condition::Equals(2.25.into_state_var()))
        );
    }

    /// Test ActionBuilder with float Set effects
    /// Validates: F64 values can be set as action effects
    /// Failure: Builder doesn't handle float Set effects correctly
    #[test]
    fn test_builder_f64_set_effects() {
        let action = Action::builder("restore_health")
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
        let action = Action::builder("boost_stats")
            .adds("speed", 1.5)
            .adds("strength", 2.25)
            .build();

        if let StateOperation::Add(value) = action.effects.get("speed").unwrap() {
            assert_eq!(*value, StateVar::F64(1500));
        } else {
            panic!("Expected Add operation for speed");
        }

        if let StateOperation::Add(value) = action.effects.get("strength").unwrap() {
            assert_eq!(*value, StateVar::F64(2250));
        } else {
            panic!("Expected Add operation for strength");
        }
    }

    /// Test ActionBuilder with float Subtract effects
    /// Validates: F64 values can be subtracted using Subtract operation
    /// Failure: Builder doesn't handle float Subtract effects correctly
    #[test]
    fn test_builder_f64_subtract_effects() {
        let action = Action::builder("drain_stats")
            .subtracts("energy", 10.5)
            .subtracts("stamina", 5.75)
            .build();

        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, StateVar::F64(10500));
        } else {
            panic!("Expected Subtract operation for energy");
        }

        if let StateOperation::Subtract(value) = action.effects.get("stamina").unwrap() {
            assert_eq!(*value, StateVar::F64(5750));
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
        let action = Action::builder("enter_area")
            .requires("location", "town")
            .requires("state", "peaceful")
            .build();

        assert_eq!(
            action.preconditions.get("location"),
            Some(&Condition::Equals("town".into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("state"),
            Some(&Condition::Equals("peaceful".into_state_var()))
        );
    }

    /// Test ActionBuilder with string/enum Set effects
    /// Validates: String values can be set as action effects
    /// Failure: Builder doesn't handle string Set effects correctly
    #[test]
    fn test_builder_string_effects() {
        let action = Action::builder("travel")
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
        let action = Action::builder("expensive_action").cost(15.5).build();

        assert_eq!(action.cost, 15.5);
    }

    /// Test ActionBuilder default cost
    /// Validates: Actions have default cost of 1.0 when not specified
    /// Failure: Default cost is not set correctly
    #[test]
    fn test_builder_default_cost() {
        let action = Action::builder("default_cost_action").build();
        assert_eq!(action.cost, 1.0);
    }

    /// BUG-013: the builder rounds to the planner's resolution of 0.001, so
    /// `action.cost` is exactly what the planner will use.
    #[test]
    fn test_builder_cost_rounds_to_thousandths() {
        assert_eq!(Action::builder("a").cost(1.23456).build().cost, 1.235);
        assert_eq!(Action::builder("a").cost(0.0004).build().cost, 0.0);
        assert_eq!(Action::builder("a").cost(0.1).build().cost, 0.1);
        assert_eq!(Action::builder("a").cost(2.5).build().cost, 2.5);
        // Invalid costs pass through unchanged for the planner to reject.
        assert!(Action::builder("a").cost(f64::NAN).build().cost.is_nan());
        assert_eq!(Action::builder("a").cost(-1.0).build().cost, -1.0);
        assert_eq!(
            Action::builder("a").cost(f64::INFINITY).build().cost,
            f64::INFINITY
        );
    }

    // Tests for mixed type scenarios

    /// Test ActionBuilder with mixed precondition types
    /// Validates: Multiple different StateVar types can be used as preconditions
    /// Failure: Builder doesn't handle mixed precondition types correctly
    #[test]
    fn test_builder_mixed_preconditions() {
        let action = Action::builder("complex_action")
            .requires("has_key", true)
            .requires("gold", 100)
            .requires("health", 75.5)
            .requires("location", "castle")
            .build();

        assert_eq!(
            action.preconditions.get("has_key"),
            Some(&Condition::Equals(true.into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("gold"),
            Some(&Condition::Equals(100.into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("health"),
            Some(&Condition::Equals(75.5.into_state_var()))
        );
        assert_eq!(
            action.preconditions.get("location"),
            Some(&Condition::Equals("castle".into_state_var()))
        );
    }

    /// Test ActionBuilder with comparison preconditions
    /// Validates: at_least / at_most / between / not can be used as preconditions
    /// Failure: Builder does not accept Condition values
    #[test]
    fn test_builder_comparison_preconditions() {
        let action = Action::builder("buy_sword")
            .requires("gold", at_least(50))
            .requires("weight", at_most(10.0))
            .requires("level", between(3, 7))
            .requires("location", not("dungeon"))
            .build();

        assert_eq!(action.preconditions.get("gold"), Some(&at_least(50)));
        assert_eq!(action.preconditions.get("weight"), Some(&at_most(10.0)));
        assert_eq!(action.preconditions.get("level"), Some(&between(3, 7)));
        assert_eq!(action.preconditions.get("location"), Some(&not("dungeon")));

        let ok = State::builder()
            .set("gold", 60)
            .set("weight", 9.5)
            .set("level", 5)
            .set("location", "town")
            .build();
        assert!(action.can_execute(&ok));

        let broke = State::builder()
            .set("gold", 49)
            .set("weight", 9.5)
            .set("level", 5)
            .set("location", "town")
            .build();
        assert!(!action.can_execute(&broke));
    }

    /// Test ActionBuilder with mixed effect types
    /// Validates: Multiple different StateOperation types can be used as effects
    /// Failure: Builder doesn't handle mixed effect types correctly
    #[test]
    fn test_builder_mixed_effects() {
        let action = Action::builder("complex_effects")
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
            assert_eq!(*value, StateVar::I64(50));
        } else {
            panic!("Expected Add operation for experience");
        }

        // Check i64 subtract
        if let StateOperation::Subtract(value) = action.effects.get("energy").unwrap() {
            assert_eq!(*value, StateVar::I64(25));
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
            assert_eq!(*value, StateVar::F64(1500));
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
    // Tests for typed deltas, display and duplicate keys (BUG-005/007/008)

    /// Every numeric literal type produces a delta of the matching StateVar
    /// type; smaller integer and float types widen.
    #[test]
    fn test_builder_delta_types_follow_the_literal() {
        let action = Action::builder("typed")
            .adds("a", 1i32)
            .adds("b", 2i64)
            .adds("c", 1.5f32)
            .adds("d", 2.5f64)
            .subtracts("e", 3i8)
            .subtracts("f", 0.25)
            .build();
        assert_eq!(action.effects["a"], StateOperation::Add(StateVar::I64(1)));
        assert_eq!(action.effects["b"], StateOperation::Add(StateVar::I64(2)));
        assert_eq!(
            action.effects["c"],
            StateOperation::Add(StateVar::F64(1500))
        );
        assert_eq!(
            action.effects["d"],
            StateOperation::Add(StateVar::F64(2500))
        );
        assert_eq!(
            action.effects["e"],
            StateOperation::Subtract(StateVar::I64(3))
        );
        assert_eq!(
            action.effects["f"],
            StateOperation::Subtract(StateVar::F64(250))
        );
    }

    /// BUG-007: float deltas display as decimals, not fixed-point units, and
    /// effects print in key order.
    #[test]
    fn test_action_display_shows_typed_effects_in_key_order() {
        let action = Action::builder("heat")
            .cost(2.0)
            .subtracts("power", 12.5)
            .adds("temp", 1.5)
            .sets("heater_on", true)
            .subtracts("gold", 5)
            .build();
        let text = action.to_string();
        let expected = "Action \x27heat\x27 (cost: 2.0)\n  Effects:\n    - Subtract 5 from gold\n    - Set heater_on to true\n    - Subtract 12.500 from power\n    - Add 1.500 to temp";
        assert_eq!(text, expected);
    }

    /// BUG-008: one effect per key; the last call wins, like `requires`.
    #[test]
    fn test_builder_last_effect_per_key_wins() {
        let action = Action::builder("dup").sets("x", 1).adds("x", 2).build();
        assert_eq!(action.effects.len(), 1);
        assert_eq!(action.effects["x"], StateOperation::Add(StateVar::I64(2)));

        let state = State::builder().set("x", 10).build();
        assert_eq!(action.apply_effect(&state).get::<i64>("x"), Some(12));
    }

    /// `apply_effect` on a missing counter creates it (BUG-009) and on a
    /// mismatched type leaves it alone (BUG-005).
    #[test]
    fn test_apply_effect_missing_and_mismatched_variables() {
        let action = Action::builder("craft")
            .adds("items_crafted", 1)
            .adds("temp", 5) // i64 delta
            .build();
        let state = State::builder().set("temp", 20.0).build();
        let next = action.apply_effect(&state);
        assert_eq!(next.get::<i64>("items_crafted"), Some(1));
        assert_eq!(next.get::<f64>("temp"), Some(20.0));
    }

    // Plan 005 (BUG-018, BUG-023, BUG-025)

    /// `Action::new` is a bare action; `Action::builder` builds.
    #[test]
    fn test_action_new_defaults() {
        let action = Action::new("wait");
        assert_eq!(action.name, "wait");
        assert_eq!(action.cost, 1.0);
        assert!(action.preconditions.is_empty());
        assert!(action.effects.is_empty());
        assert!(action.can_execute(&State::new()));
        assert_eq!(action.apply_effect(&State::new()), State::new());

        let mut editable = Action::new("mine");
        editable.cost = 2.0;
        editable.preconditions.set("has_pick", true);
        editable
            .effects
            .insert("gold".to_string(), StateOperation::add(5));
        assert_eq!(
            editable,
            Action::builder("mine")
                .cost(2.0)
                .requires("has_pick", true)
                .adds("gold", 5)
                .build()
        );
    }

    /// BUG-023: `Action` compares by value; `cost` takes part.
    #[test]
    fn test_action_equality() {
        let a = Action::builder("a")
            .cost(2.0)
            .requires("x", 1)
            .sets("y", true)
            .build();
        let b = Action::builder("a")
            .cost(2.0)
            .requires("x", 1)
            .sets("y", true)
            .build();
        assert_eq!(a, b);
        assert_ne!(
            a,
            Action::builder("a")
                .cost(2.5)
                .requires("x", 1)
                .sets("y", true)
                .build()
        );
        assert_ne!(
            a,
            Action::builder("b")
                .cost(2.0)
                .requires("x", 1)
                .sets("y", true)
                .build()
        );
        assert_ne!(a, Action::builder("a").cost(2.0).sets("y", true).build());
        assert_ne!(
            a,
            Action::builder("a")
                .cost(2.0)
                .requires("x", 1)
                .sets("y", false)
                .build()
        );
        let template = Action::builder("t").cost(3.0);
        assert_eq!(template.clone().build(), template.build());
    }

    /// BUG-025: `adds` / `subtracts` accept a `StateVar` delta, so a computed
    /// float has a fallible route.
    #[test]
    fn test_action_builder_accepts_state_var_delta() {
        fn build(delta: f64) -> Result<Action, StateError> {
            let delta = StateVar::try_from_f64(delta)?;
            Ok(Action::builder("adjust")
                .adds("temperature", delta.clone())
                .subtracts("fuel", delta)
                .build())
        }
        let action = build(1.5).unwrap();
        assert_eq!(action.effects["temperature"], StateOperation::add(1.5));
        assert_eq!(action.effects["fuel"], StateOperation::subtract(1.5));
        assert!(matches!(
            build(f64::NAN),
            Err(StateError::UnrepresentableFloat(_))
        ));
        assert!(matches!(
            build(f64::INFINITY),
            Err(StateError::UnrepresentableFloat(_))
        ));
    }
}
