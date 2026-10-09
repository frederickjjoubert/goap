#[cfg(test)]
mod tests {
    use goap::prelude::*;
    use std::collections::HashMap;

    // Tests for StateVar conversion functionality

    /// Test StateVar f64 conversion methods
    /// Validates: from_f64 and as_f64 work correctly with rounding
    /// Failure: F64 conversion logic is broken
    #[test]
    fn test_state_var_f64_conversion() {
        // Test basic conversion
        let value = StateVar::from_f64(1.5);
        assert_eq!(value, StateVar::F64(1500));
        assert_eq!(value.as_f64(), Some(1.5));

        // Test rounding
        let value = StateVar::from_f64(1.2345);
        assert_eq!(value, StateVar::F64(1235)); // Should round to 1.235
        assert_eq!(value.as_f64(), Some(1.235));

        // Test negative values
        let value = StateVar::from_f64(-1.5);
        assert_eq!(value, StateVar::F64(-1500));
        assert_eq!(value.as_f64(), Some(-1.5));

        // Test zero
        let value = StateVar::from_f64(0.0);
        assert_eq!(value, StateVar::F64(0));
        assert_eq!(value.as_f64(), Some(0.0));

        // Test small decimals
        let value = StateVar::from_f64(0.001);
        assert_eq!(value, StateVar::F64(1));
        assert_eq!(value.as_f64(), Some(0.001));

        // Test non-F64 type
        assert_eq!(StateVar::Bool(true).as_f64(), None);
    }

    /// Test StateVar From trait implementations
    /// Validates: All From implementations work correctly for type conversion
    /// Failure: From trait implementations are broken
    #[test]
    fn test_state_var_from_implementations() {
        // Test bool conversion
        let bool_var: StateVar = true.into();
        assert_eq!(bool_var, StateVar::Bool(true));

        // Test i64 conversion
        let i64_var: StateVar = 42i64.into();
        assert_eq!(i64_var, StateVar::I64(42));

        // Test f64 conversion
        let f64_var: StateVar = 1.5f64.into();
        assert_eq!(f64_var, StateVar::F64(1500));

        // Test String conversion
        let string_var: StateVar = "test".to_string().into();
        assert_eq!(string_var, StateVar::String("test".to_string()));

        // Test str conversion
        let str_var: StateVar = "test".into();
        assert_eq!(str_var, StateVar::String("test".to_string()));

        // Test smaller integer conversions
        let i32_var: StateVar = 42i32.into();
        assert_eq!(i32_var, StateVar::I64(42));

        let i16_var: StateVar = 42i16.into();
        assert_eq!(i16_var, StateVar::I64(42));

        let i8_var: StateVar = 42i8.into();
        assert_eq!(i8_var, StateVar::I64(42));
    }

    // Tests for State basic operations

    /// Test State creation and basic variable operations
    /// Validates: State can store and retrieve variables of all types
    /// Failure: Basic state operations are broken
    #[test]
    fn test_state_basic_operations() {
        let mut state = State::new();

        // Test set and get
        state.set("bool_var", StateVar::Bool(true));
        state.set("int_var", StateVar::I64(42));
        state.set("float_var", StateVar::F64(1500)); // 1.5
        state.set("string_var", StateVar::String("test".to_string()));

        assert_eq!(state.get::<bool>("bool_var"), Some(true));
        assert_eq!(state.get::<i64>("int_var"), Some(42));
        assert_eq!(state.get::<f64>("float_var"), Some(1.5));
        assert_eq!(state.get::<String>("string_var"), Some("test".to_string()));
        assert_eq!(state.get::<bool>("nonexistent"), None);
    }

    // Tests for State satisfaction logic (see tests/conditions.rs for the
    // full Condition matrix)

    /// Test State::meets with equality conditions on every type
    /// Validates: Plain values are compared for exact equality
    /// Failure: Equality satisfaction logic is broken
    #[test]
    fn test_state_meets_equality() {
        let state = State::builder()
            .set("bool_var", true)
            .set("int_var", 42)
            .set("float_var", 1.5)
            .set("string_var", "test")
            .build();

        let mut conditions = Conditions::new();
        conditions.set("bool_var", true);
        conditions.set("int_var", 42);
        conditions.set("float_var", 1.5);
        conditions.set("string_var", "test");
        assert!(state.meets(&conditions));

        conditions.set("int_var", 40);
        assert!(!state.meets(&conditions), "42 is not equal to 40");
    }

    /// Test State::meets with ordered numeric conditions
    /// Validates: at_least / at_most are honoured for I64 and F64
    /// Failure: Ordered comparison logic is broken
    #[test]
    fn test_state_meets_ordered() {
        let state = State::builder()
            .set("int_var", 42)
            .set("float_var", 1.5)
            .build();

        let mut conditions = Conditions::new();
        conditions.set("int_var", at_least(40));
        conditions.set("float_var", at_most(2.0));
        assert!(state.meets(&conditions));

        conditions.set("int_var", at_least(50));
        assert!(!state.meets(&conditions));
    }

    /// Test State::meets with missing variables
    /// Validates: Satisfaction fails when required variables are missing
    /// Failure: Missing variable detection is broken
    #[test]
    fn test_state_meets_missing_variable() {
        let state = State::builder().set("existing_var", true).build();
        let mut conditions = Conditions::new();
        conditions.set("nonexistent_var", true);
        assert!(!state.meets(&conditions));
    }

    /// Test State::meets with type mismatches
    /// Validates: Satisfaction fails when variable types don't match
    /// Failure: Type mismatch detection is broken
    #[test]
    fn test_state_meets_type_mismatch() {
        let state = State::builder().set("var", true).build();
        let mut conditions = Conditions::new();
        conditions.set("var", 1);
        assert!(!state.meets(&conditions));
    }

    // Tests for State apply operations

    /// Test State apply with Set operations
    /// Validates: Set operations properly replace variable values
    /// Failure: Set operation logic is broken
    #[test]
    fn test_state_apply_set_operations() {
        let mut state = State::new();
        state.set("bool_var", StateVar::Bool(false));
        state.set("int_var", StateVar::I64(10));
        state.set("string_var", StateVar::String("old".to_string()));

        let mut changes = HashMap::new();
        changes.insert(
            "bool_var".to_string(),
            StateOperation::Set(StateVar::Bool(true)),
        );
        changes.insert(
            "int_var".to_string(),
            StateOperation::Set(StateVar::I64(20)),
        );
        changes.insert(
            "string_var".to_string(),
            StateOperation::Set(StateVar::String("new".to_string())),
        );

        state.apply(&changes);

        assert_eq!(state.get::<bool>("bool_var"), Some(true));
        assert_eq!(state.get::<i64>("int_var"), Some(20));
        assert_eq!(state.get::<String>("string_var"), Some("new".to_string()));
    }

    /// Test State apply with Add operations on i64
    /// Validates: Add operations properly increment i64 values
    /// Failure: I64 Add operation logic is broken
    #[test]
    fn test_state_apply_add_i64() {
        let mut state = State::new();
        state.set("int_var", StateVar::I64(10));

        let mut changes = HashMap::new();
        changes.insert("int_var".to_string(), StateOperation::add(5));
        state.apply(&changes);

        assert_eq!(state.get::<i64>("int_var"), Some(15));
    }

    /// Test State apply with Add operations on f64
    /// Validates: Add operations properly increment f64 values
    /// Failure: F64 Add operation logic is broken
    #[test]
    fn test_state_apply_add_f64() {
        let mut state = State::new();
        state.set("float_var", StateVar::F64(1500)); // 1.5

        let mut changes = HashMap::new();
        changes.insert("float_var".to_string(), StateOperation::add(0.5));
        state.apply(&changes);

        assert_eq!(state.get::<f64>("float_var"), Some(2.0));
    }

    /// Test State apply with Subtract operations on i64
    /// Validates: Subtract operations properly decrement i64 values
    /// Failure: I64 Subtract operation logic is broken
    #[test]
    fn test_state_apply_subtract_i64() {
        let mut state = State::new();
        state.set("int_var", StateVar::I64(15));

        let mut changes = HashMap::new();
        changes.insert("int_var".to_string(), StateOperation::subtract(3));
        state.apply(&changes);

        assert_eq!(state.get::<i64>("int_var"), Some(12));
    }

    /// Test State apply with Subtract operations on f64
    /// Validates: Subtract operations properly decrement f64 values
    /// Failure: F64 Subtract operation logic is broken
    #[test]
    fn test_state_apply_subtract_f64() {
        let mut state = State::new();
        state.set("float_var", StateVar::F64(2000)); // 2.0

        let mut changes = HashMap::new();
        changes.insert("float_var".to_string(), StateOperation::subtract(0.75));
        state.apply(&changes);

        assert_eq!(state.get::<f64>("float_var"), Some(1.25));
    }

    /// BUG-009: Add/Subtract on a missing variable treat it as zero and create
    /// it with the delta's type.
    #[test]
    fn test_state_apply_missing_variable_counts_as_zero() {
        let mut state = State::new();

        let mut changes = HashMap::new();
        changes.insert("items_crafted".to_string(), StateOperation::add(5));
        changes.insert("debt".to_string(), StateOperation::subtract(2.5));
        state.apply(&changes);

        assert_eq!(state.get::<i64>("items_crafted"), Some(5));
        assert_eq!(state.get::<f64>("debt"), Some(-2.5));

        // Applying again accumulates.
        state.apply(&changes);
        assert_eq!(state.get::<i64>("items_crafted"), Some(10));
        assert_eq!(state.get::<f64>("debt"), Some(-5.0));
    }

    // Tests for the collection traits (plan 005, BUG-026: replaces `merge`)

    /// `extend` merges another state in, last write wins, any type.
    #[test]
    fn test_state_extend_and_from_iterator() {
        let mut state1 = State::new();
        state1.set("var1", StateVar::Bool(true));
        state1.set("var2", StateVar::I64(10));

        let state2 = State::builder()
            .set("var2", 20) // overrides
            .set("var3", "test")
            .build();

        state1.extend(state2);

        assert_eq!(state1.get::<bool>("var1"), Some(true));
        assert_eq!(state1.get::<i64>("var2"), Some(20));
        assert_eq!(state1.get::<String>("var3"), Some("test".to_string()));

        // Pairs of any `IntoStateVar` type; later pairs win.
        state1.extend([("var1", 1.5), ("var4", 2.5)]);
        assert_eq!(state1.get::<f64>("var1"), Some(1.5));
        assert_eq!(state1.get::<f64>("var4"), Some(2.5));

        let collected: State = [("a", 1), ("b", 2), ("a", 3)].into_iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected.get::<i64>("a"), Some(3));
        let from_strings: State = vec![("x".to_string(), StateVar::Bool(false))]
            .into_iter()
            .collect();
        assert_eq!(from_strings.get::<bool>("x"), Some(false));
    }

    /// Borrowed and owned iteration yield every variable once.
    #[test]
    fn test_state_into_iterator() {
        let state = State::builder().set("a", 1).set("b", true).build();
        let mut borrowed: Vec<(&String, &StateVar)> = (&state).into_iter().collect();
        borrowed.sort_by_key(|(key, _)| (*key).clone());
        assert_eq!(borrowed.len(), 2);
        assert_eq!(borrowed[0], (&"a".to_string(), &StateVar::I64(1)));

        let mut count = 0;
        for (key, value) in &state {
            assert_eq!(state.get_var(key), Some(value));
            count += 1;
        }
        assert_eq!(count, 2);

        let mut owned: Vec<(String, StateVar)> = state.into_iter().collect();
        owned.sort_by_key(|(key, _)| key.clone());
        assert_eq!(
            owned,
            vec![
                ("a".to_string(), StateVar::I64(1)),
                ("b".to_string(), StateVar::Bool(true)),
            ]
        );
    }

    // Tests for the accessors that replaced the public `vars` field (BUG-015)

    /// `get_var`, `contains`, `remove`, `len`, `is_empty` and `iter` agree
    /// with `set` and `get`.
    #[test]
    fn test_state_accessors() {
        let mut state = State::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert!(!state.contains("x"));
        assert_eq!(state.get_var("x"), None);

        state.set("x", 5);
        state.set("name", "bob");
        assert!(!state.is_empty());
        assert_eq!(state.len(), 2);
        assert!(state.contains("x"));
        assert_eq!(state.get_var("x"), Some(&StateVar::I64(5)));
        assert_eq!(state.get_var("name"), Some(&StateVar::String("bob".into())));

        let mut seen: Vec<(String, StateVar)> =
            state.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        seen.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            seen,
            vec![
                ("name".to_string(), StateVar::String("bob".into())),
                ("x".to_string(), StateVar::I64(5)),
            ]
        );

        assert_eq!(state.remove("x"), Some(StateVar::I64(5)));
        assert_eq!(state.remove("x"), None);
        assert!(!state.contains("x"));
        assert_eq!(state.len(), 1);
    }

    /// `Display` lists variables in key order, so output is deterministic.
    #[test]
    fn test_state_display_is_sorted() {
        let state = State::builder()
            .set("zeta", 1)
            .set("alpha", true)
            .set("mid", 2.5)
            .build();
        assert_eq!(
            state.to_string(),
            "State:\n  - alpha: true\n  - mid: 2.500\n  - zeta: 1\n"
        );
        assert_eq!(State::new().to_string(), "empty state");
    }

    // Tests for float conversion (BUG-014)

    /// `try_from_f64` rounds representable values and rejects the rest.
    #[test]
    fn test_state_var_try_from_f64_rejects_unrepresentable() {
        assert_eq!(StateVar::try_from_f64(1.5), Ok(StateVar::F64(1500)));
        assert_eq!(StateVar::try_from_f64(0.0), Ok(StateVar::F64(0)));
        assert_eq!(StateVar::try_from_f64(-0.0), Ok(StateVar::F64(0)));
        // The bounds themselves are representable and round-trip.
        let max = StateVar::try_from_f64(StateVar::MAX_F64).unwrap();
        assert_eq!(max.as_f64(), Some(StateVar::MAX_F64));
        let min = StateVar::try_from_f64(StateVar::MIN_F64).unwrap();
        assert_eq!(min.as_f64(), Some(StateVar::MIN_F64));
        // One ulp beyond is not.
        assert!(StateVar::try_from_f64(StateVar::MAX_F64 + 2.0).is_err());
        assert!(StateVar::try_from_f64(StateVar::MIN_F64 - 2.0).is_err());

        for bad in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            1e16,
            -1e16,
            1e300,
        ] {
            match StateVar::try_from_f64(bad) {
                Err(StateError::UnrepresentableFloat(v)) => {
                    assert!(v.is_nan() == bad.is_nan() && (v.is_nan() || v == bad));
                }
                other => panic!("{bad} should be rejected, got {other:?}"),
            }
        }

        // The error message names the problem.
        let err = StateVar::try_from_f64(f64::NAN).unwrap_err().to_string();
        assert!(err.contains("NaN"), "{err}");
        let err = StateVar::try_from_f64(1e16).unwrap_err().to_string();
        assert!(err.contains("range"), "{err}");
    }

    /// The infallible constructors panic instead of storing a wrong value.
    #[test]
    #[should_panic(expected = "try_from_f64")]
    fn test_state_var_from_f64_panics_on_nan() {
        let _ = StateVar::from_f64(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "try_from_f64")]
    fn test_state_set_panics_on_infinite_float() {
        let mut state = State::new();
        state.set("x", f64::INFINITY);
    }

    #[test]
    #[should_panic(expected = "try_from_f64")]
    fn test_state_builder_panics_on_out_of_range_float() {
        let _ = State::builder().set("x", 1e16).build();
    }

    /// Untrusted floats go through `try_from_f64`; `set` accepts the result.
    #[test]
    fn test_state_set_with_try_from_f64_is_fallible() {
        fn store(state: &mut State, value: f64) -> Result<(), StateError> {
            state.set("reading", StateVar::try_from_f64(value)?);
            Ok(())
        }
        let mut state = State::new();
        assert!(store(&mut state, 21.5).is_ok());
        assert_eq!(state.get::<f64>("reading"), Some(21.5));
        assert!(store(&mut state, f64::NAN).is_err());
        assert_eq!(state.get::<f64>("reading"), Some(21.5)); // unchanged
    }

    // Tests for State hash consistency

    /// Test State hash consistency regardless of insertion order
    /// Validates: Hash is consistent regardless of variable insertion order
    /// Failure: State hash implementation is broken
    #[test]
    fn test_state_hash_consistency() {
        let mut state1 = State::new();
        state1.set("a", StateVar::Bool(true));
        state1.set("b", StateVar::I64(10));

        let mut state2 = State::new();
        state2.set("b", StateVar::I64(10));
        state2.set("a", StateVar::Bool(true));

        // Hash should be the same regardless of insertion order
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        state1.hash(&mut hasher1);
        state2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // Tests for State helper methods

    /// Test State get method with bool pattern matching
    /// Validates: get method returns correct values and proper errors for bool
    /// Failure: get method pattern matching is broken
    #[test]
    fn test_state_get_bool() {
        let mut state = State::new();
        state.set("bool_var", StateVar::Bool(true));
        state.set("int_var", StateVar::I64(42));

        // Test successful retrieval
        assert_eq!(state.get::<bool>("bool_var"), Some(true));

        // Test variable not found
        assert_eq!(state.get::<bool>("nonexistent"), None);

        // Test wrong type
        assert_eq!(state.get::<i64>("int_var"), Some(42));
    }

    /// Test State get method with int pattern matching
    /// Validates: get method returns correct values and proper errors for int
    /// Failure: get method pattern matching is broken
    #[test]
    fn test_state_get_int() {
        let mut state = State::new();
        state.set("int_var", StateVar::I64(42));
        state.set("bool_var", StateVar::Bool(true));

        // Test successful retrieval
        assert_eq!(state.get::<i64>("int_var"), Some(42));

        // Test variable not found
        assert_eq!(state.get::<i64>("nonexistent"), None);

        // Test wrong type
        assert_eq!(state.get::<bool>("bool_var"), Some(true));
    }

    /// Test State get method with float pattern matching
    /// Validates: get method returns correct values and proper errors for float
    /// Failure: get method pattern matching is broken
    #[test]
    fn test_state_get_float() {
        let mut state = State::new();
        state.set("float_var", StateVar::F64(1500)); // 1.5
        state.set("bool_var", StateVar::Bool(true));

        // Test successful retrieval
        assert_eq!(state.get::<f64>("float_var"), Some(1.5));

        // Test variable not found
        assert_eq!(state.get::<f64>("nonexistent"), None);

        // Test wrong type
        assert_eq!(state.get::<bool>("bool_var"), Some(true));
    }

    /// Test State get method with enum/string pattern matching
    /// Validates: get method returns correct values and proper errors for enum/string
    /// Failure: get method pattern matching is broken
    #[test]
    fn test_state_get_enum() {
        let mut state = State::new();
        state.set("string_var", StateVar::String("test".to_string()));
        state.set("bool_var", StateVar::Bool(true));

        // Test successful retrieval
        assert_eq!(state.get::<String>("string_var"), Some("test".to_string()));

        // Test variable not found
        assert_eq!(state.get::<String>("nonexistent"), None);

        // Test wrong type
        assert_eq!(state.get::<bool>("bool_var"), Some(true));
    }

    // Tests for StateBuilder functionality

    /// Test StateBuilder set method with bool
    /// Validates: Builder set method properly sets boolean values
    /// Failure: StateBuilder set method is broken
    #[test]
    fn test_state_builder_bool() {
        let state = State::builder().set("has_key", true).build();
        assert_eq!(state.get::<bool>("has_key"), Some(true));
    }

    /// Test StateBuilder set method with int
    /// Validates: Builder set method properly sets integer values
    /// Failure: StateBuilder set method is broken
    #[test]
    fn test_state_builder_int() {
        let state = State::builder().set("gold", 100).build();
        assert_eq!(state.get::<i64>("gold"), Some(100));
    }

    /// Test StateBuilder set method with float
    /// Validates: Builder set method properly sets float values
    /// Failure: StateBuilder set method is broken
    #[test]
    fn test_state_builder_float() {
        let state = State::builder().set("health", 75.5).build();
        assert_eq!(state.get::<f64>("health"), Some(75.5));
    }

    /// Test StateBuilder set method with enum/string
    /// Validates: Builder set method properly sets string values
    /// Failure: StateBuilder set method is broken
    #[test]
    fn test_state_builder_enum_val() {
        let state = State::builder().set("location", "town").build();
        assert_eq!(state.get::<String>("location"), Some("town".to_string()));
    }

    /// Test StateBuilder unified set method
    /// Validates: Builder set method works with all value types via IntoStateVar
    /// Failure: StateBuilder unified set method is broken
    #[test]
    fn test_state_builder_unified_set() {
        let state = State::builder()
            .set("has_wood", true)
            .set("energy", 100)
            .set("temperature", 22.5)
            .set("location", "forest")
            .build();

        assert_eq!(state.get::<bool>("has_wood"), Some(true));
        assert_eq!(state.get::<i64>("energy"), Some(100));
        assert_eq!(state.get::<f64>("temperature"), Some(22.5));
        assert_eq!(state.get::<String>("location"), Some("forest".to_string()));
    }

    /// Test StateBuilder method chaining
    /// Validates: All builder methods can be chained together properly
    /// Failure: StateBuilder method chaining is broken
    #[test]
    fn test_state_builder_chaining() {
        let state = State::builder()
            .set("has_wood", true)
            .set("wood_count", 5)
            .set("health", 100.0)
            .set("location", "forest")
            .set("weather", "sunny")
            .build();

        assert_eq!(state.get::<bool>("has_wood"), Some(true));
        assert_eq!(state.get::<i64>("wood_count"), Some(5));
        assert_eq!(state.get::<f64>("health"), Some(100.0));
        assert_eq!(state.get::<String>("location"), Some("forest".to_string()));
        assert_eq!(state.get::<String>("weather"), Some("sunny".to_string()));
    }

    // Tests for StateOperation constructors and typed deltas

    /// `set`/`add`/`subtract` keep the type of the value they were given.
    #[test]
    fn test_state_operation_constructors_keep_types() {
        assert_eq!(
            StateOperation::set(100),
            StateOperation::Set(StateVar::I64(100))
        );
        assert_eq!(
            StateOperation::set(1.5),
            StateOperation::Set(StateVar::F64(1500))
        );
        assert_eq!(
            StateOperation::set("x"),
            StateOperation::Set(StateVar::String("x".to_string()))
        );
        assert_eq!(
            StateOperation::add(50),
            StateOperation::Add(StateVar::I64(50))
        );
        assert_eq!(
            StateOperation::add(50i32),
            StateOperation::Add(StateVar::I64(50))
        );
        assert_eq!(
            StateOperation::subtract(25),
            StateOperation::Subtract(StateVar::I64(25))
        );
        assert_eq!(
            StateOperation::add(0.5),
            StateOperation::Add(StateVar::F64(500))
        );
        assert_eq!(
            StateOperation::add(0.5f32),
            StateOperation::Add(StateVar::F64(500))
        );
        assert_eq!(
            StateOperation::subtract(0.25),
            StateOperation::Subtract(StateVar::F64(250))
        );
    }

    /// Float constructors round to 3 decimal places.
    #[test]
    fn test_state_operation_f64_rounding() {
        assert_eq!(
            StateOperation::set(1.2345),
            StateOperation::Set(StateVar::F64(1235))
        );
        assert_eq!(
            StateOperation::add(0.1234),
            StateOperation::Add(StateVar::F64(123))
        );
        assert_eq!(
            StateOperation::subtract(0.6789),
            StateOperation::Subtract(StateVar::F64(679))
        );
    }

    /// `signed_delta` normalises Add/Subtract to one signed change in raw
    /// units and is `None` for `Set` and for non-numeric payloads.
    #[test]
    fn test_state_operation_signed_delta() {
        assert_eq!(StateOperation::add(5).signed_delta(), Some(5));
        assert_eq!(StateOperation::subtract(5).signed_delta(), Some(-5));
        assert_eq!(StateOperation::add(-5).signed_delta(), Some(-5));
        assert_eq!(StateOperation::add(1.5).signed_delta(), Some(1500));
        assert_eq!(StateOperation::subtract(1.5).signed_delta(), Some(-1500));
        assert_eq!(StateOperation::set(5).signed_delta(), None);
        assert_eq!(
            StateOperation::Add(StateVar::Bool(true)).signed_delta(),
            None
        );
        assert_eq!(
            StateOperation::Subtract(StateVar::I64(i64::MIN)).signed_delta(),
            Some(i64::MAX)
        );
    }

    /// `state_type` and `Display` report the payload.
    #[test]
    fn test_state_operation_state_type_and_display() {
        assert_eq!(StateOperation::add(5).state_type(), StateType::I64);
        assert_eq!(StateOperation::add(5.0).state_type(), StateType::F64);
        assert_eq!(StateOperation::set(true).state_type(), StateType::Bool);
        assert_eq!(StateOperation::set("a").state_type(), StateType::String);
        assert_eq!(StateType::String.to_string(), "string");
        assert_eq!(StateType::F64.to_string(), "f64");
        assert_eq!(StateOperation::add(1.5).to_string(), "add 1.500");
        assert_eq!(StateOperation::subtract(2).to_string(), "subtract 2");
        assert_eq!(StateOperation::set("town").to_string(), "set to town");
    }

    /// Complete f64 workflow through the constructors.
    #[test]
    fn test_state_f64_workflow_with_constructors() {
        let mut state = State::new();
        state.set("value", StateVar::from_f64(1.5));

        let mut changes = HashMap::new();
        changes.insert("value".to_string(), StateOperation::add(0.5));
        state.apply(&changes);
        assert_eq!(state.get::<f64>("value"), Some(2.0));

        changes.clear();
        changes.insert("value".to_string(), StateOperation::subtract(0.75));
        state.apply(&changes);
        assert_eq!(state.get::<f64>("value"), Some(1.25));

        changes.clear();
        changes.insert("value".to_string(), StateOperation::set(3.04159)); // rounds to 3.042
        state.apply(&changes);
        assert_eq!(state.get::<f64>("value"), Some(3.042));
    }

    // Tests for the apply rules on mismatched, missing and extreme values

    /// BUG-005: a delta of the wrong type leaves the variable unchanged
    /// instead of applying the raw integer to the wrong representation.
    #[test]
    fn test_state_apply_mismatched_delta_type_is_ignored() {
        let mut state = State::builder().set("temp", 20.0).set("count", 1).build();

        let mut changes = HashMap::new();
        changes.insert("temp".to_string(), StateOperation::add(5)); // i64 onto f64
        changes.insert("count".to_string(), StateOperation::add(1.5)); // f64 onto i64
        state.apply(&changes);

        assert_eq!(state.get::<f64>("temp"), Some(20.0));
        assert_eq!(state.get::<i64>("count"), Some(1));

        // A hand-built non-numeric delta is also a no-op.
        changes.clear();
        changes.insert(
            "count".to_string(),
            StateOperation::Add(StateVar::Bool(true)),
        );
        state.apply(&changes);
        assert_eq!(state.get::<i64>("count"), Some(1));

        // Adding to a bool or string variable is a no-op too.
        let mut state = State::builder().set("flag", true).set("name", "x").build();
        changes.clear();
        changes.insert("flag".to_string(), StateOperation::add(1));
        changes.insert("name".to_string(), StateOperation::add(1));
        state.apply(&changes);
        assert_eq!(state.get::<bool>("flag"), Some(true));
        assert_eq!(state.get::<String>("name"), Some("x".to_string()));
    }

    /// BUG-006: arithmetic saturates instead of overflowing.
    #[test]
    fn test_state_apply_saturates_at_i64_bounds() {
        let mut state = State::builder()
            .set("max", i64::MAX)
            .set("min", i64::MIN)
            .set("fmax", StateVar::F64(i64::MAX))
            .set("fmin", StateVar::F64(i64::MIN))
            .build();

        let mut changes = HashMap::new();
        changes.insert("max".to_string(), StateOperation::add(1));
        changes.insert("min".to_string(), StateOperation::subtract(1));
        changes.insert("fmax".to_string(), StateOperation::add(0.001));
        changes.insert("fmin".to_string(), StateOperation::subtract(0.001));
        state.apply(&changes);

        assert_eq!(state.get::<i64>("max"), Some(i64::MAX));
        assert_eq!(state.get::<i64>("min"), Some(i64::MIN));
        assert_eq!(state.get_var("fmax"), Some(&StateVar::F64(i64::MAX)));
        assert_eq!(state.get_var("fmin"), Some(&StateVar::F64(i64::MIN)));

        // Subtracting a negative delta saturates upwards as well.
        changes.clear();
        changes.insert("max".to_string(), StateOperation::subtract(-1));
        state.apply(&changes);
        assert_eq!(state.get::<i64>("max"), Some(i64::MAX));
    }

    // Plan 005 (BUG-018, BUG-020, BUG-023, BUG-026)

    /// `State::new()` is the empty state; `State::builder()` is the builder.
    #[test]
    fn test_state_new_is_empty() {
        let state = State::new();
        assert!(state.is_empty());
        assert_eq!(state, State::default());
        assert_eq!(state, State::builder().build());
        let built = State::builder().set("x", 1).build();
        assert_eq!(built.len(), 1);
        let template = State::builder().set("x", 1);
        let a = template.clone().set("y", 2).build();
        let b = template.set("y", 3).build();
        assert_eq!(a.get::<i64>("y"), Some(2));
        assert_eq!(b.get::<i64>("y"), Some(3));
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum Location {
        Town,
        Forest,
    }

    impl std::fmt::Display for Location {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Location::Town => "town",
                Location::Forest => "forest",
            })
        }
    }

    impl std::str::FromStr for Location {
        type Err = ();
        fn from_str(s: &str) -> Result<Self, ()> {
            match s {
                "town" => Ok(Location::Town),
                "forest" => Ok(Location::Forest),
                _ => Err(()),
            }
        }
    }

    impl EnumStateVar for Location {}

    /// BUG-020: an `EnumStateVar` is stored as its `Display` string and read
    /// back through `FromStr`.
    #[test]
    fn test_state_enum_round_trip() {
        let mut state = State::builder().set("location", Location::Forest).build();
        assert_eq!(
            state.get_var("location"),
            Some(&StateVar::String("forest".into()))
        );
        assert_eq!(state.get::<Location>("location"), Some(Location::Forest));
        assert_eq!(state.get::<String>("location"), Some("forest".to_string()));

        state.set("location", Location::Town);
        assert_eq!(state.get::<Location>("location"), Some(Location::Town));

        // Through the trait directly, with the error shape.
        let err = Location::try_from_state_var(&StateVar::I64(3), "location").unwrap_err();
        assert_eq!(
            err,
            StateError::InvalidVarType {
                variable: "location".to_string(),
                expected: std::any::type_name::<Location>(),
                found: StateType::I64,
            }
        );
        assert!(err.to_string().contains("Location"), "{err}");
    }

    /// BUG-020: a string that does not parse, or a variable of another type,
    /// reads as `None`, like every other type mismatch.
    #[test]
    fn test_state_enum_read_fails_for_unknown_string_and_other_types() {
        let state = State::builder()
            .set("location", "moon")
            .set("count", 3)
            .set("flag", true)
            .build();
        assert_eq!(state.get::<Location>("location"), None);
        assert_eq!(state.get::<Location>("count"), None);
        assert_eq!(state.get::<Location>("flag"), None);
        assert_eq!(state.get::<Location>("missing"), None);
    }

    /// BUG-023 / BUG-017: `StateError::InvalidVarType` names the requested
    /// and the actual type.
    #[test]
    fn test_state_error_invalid_var_type_reports_both_types() {
        let err = bool::try_from_state_var(&StateVar::I64(1), "flag").unwrap_err();
        assert_eq!(
            err,
            StateError::InvalidVarType {
                variable: "flag".to_string(),
                expected: "bool",
                found: StateType::I64,
            }
        );
        assert_eq!(
            err.to_string(),
            "State variable 'flag' is not of type bool (it is i64)"
        );
    }

    /// `StateVar::state_type` and `is_numeric`.
    #[test]
    fn test_state_var_state_type_and_is_numeric() {
        assert_eq!(StateVar::Bool(true).state_type(), StateType::Bool);
        assert_eq!(StateVar::I64(1).state_type(), StateType::I64);
        assert_eq!(StateVar::from_f64(1.0).state_type(), StateType::F64);
        assert_eq!(StateVar::from("a").state_type(), StateType::String);
        assert!(StateVar::I64(1).is_numeric());
        assert!(StateVar::F64(1).is_numeric());
        assert!(!StateVar::Bool(true).is_numeric());
        assert!(!StateVar::from("a").is_numeric());
        assert_eq!(StateType::Bool.to_string(), "bool");
        assert_eq!(StateType::I64.to_string(), "i64");
    }
}
