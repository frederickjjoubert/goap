#[cfg(test)]
mod tests {
    use goap::prelude::*;
    use std::collections::HashMap;

    // Tests for StateVar distance calculations

    /// Test StateVar distance calculation for bool values
    /// Validates: Bool distance is 0 for same values, 1 for different values
    /// Failure: Boolean distance calculation logic is broken
    #[test]
    fn test_state_var_bool_distance() {
        assert_eq!(StateVar::Bool(true).distance(&StateVar::Bool(true)).unwrap(), 0);
        assert_eq!(StateVar::Bool(false).distance(&StateVar::Bool(false)).unwrap(), 0);
        assert_eq!(StateVar::Bool(true).distance(&StateVar::Bool(false)).unwrap(), 1);
        assert_eq!(StateVar::Bool(false).distance(&StateVar::Bool(true)).unwrap(), 1);
    }

    /// Test StateVar distance calculation for i64 values
    /// Validates: I64 distance is absolute difference between values
    /// Failure: Integer distance calculation logic is broken
    #[test]
    fn test_state_var_i64_distance() {
        assert_eq!(StateVar::I64(5).distance(&StateVar::I64(5)).unwrap(), 0);
        assert_eq!(StateVar::I64(10).distance(&StateVar::I64(5)).unwrap(), 5);
        assert_eq!(StateVar::I64(5).distance(&StateVar::I64(10)).unwrap(), 5);
        assert_eq!(StateVar::I64(-5).distance(&StateVar::I64(5)).unwrap(), 10);
    }

    /// Test StateVar distance calculation for f64 values
    /// Validates: F64 distance is absolute difference between fixed-point values
    /// Failure: Float distance calculation logic is broken
    #[test]
    fn test_state_var_f64_distance() {
        // Test fixed point number distance (3 decimal places)
        // 1.5 is stored as 1500
        assert_eq!(StateVar::F64(1500).distance(&StateVar::F64(1500)).unwrap(), 0);
        // 1.5 to 2.5 = distance of 1000 (1.0)
        assert_eq!(StateVar::F64(1500).distance(&StateVar::F64(2500)).unwrap(), 1000);
        // 2.5 to 1.5 = distance of 1000 (1.0)
        assert_eq!(StateVar::F64(2500).distance(&StateVar::F64(1500)).unwrap(), 1000);
        // -1.5 to 1.5 = distance of 3000 (3.0)
        assert_eq!(StateVar::F64(-1500).distance(&StateVar::F64(1500)).unwrap(), 3000);
        // Test small decimal differences
        // 1.001 to 1.002 = distance of 1 (0.001)
        assert_eq!(StateVar::F64(1001).distance(&StateVar::F64(1002)).unwrap(), 1);
    }

    /// Test StateVar distance calculation for string values
    /// Validates: String distance is 0 for same values, 1 for different values
    /// Failure: String distance calculation logic is broken
    #[test]
    fn test_state_var_string_distance() {
        assert_eq!(
            StateVar::String("A".to_string()).distance(&StateVar::String("A".to_string())).unwrap(),
            0
        );
        assert_eq!(
            StateVar::String("A".to_string()).distance(&StateVar::String("B".to_string())).unwrap(),
            1
        );
    }

    /// Test StateVar distance calculation returns error for mismatched types
    /// Validates: Distance calculation returns error when comparing different StateVar types
    /// Failure: Type mismatch detection is broken
    #[test]
    fn test_state_var_distance_type_mismatch() {
        let result = StateVar::Bool(true).distance(&StateVar::I64(5));
        assert!(result.is_err());
        match result.unwrap_err() {
            StateError::InvalidVarType { expected, .. } => {
                assert_eq!(expected, "matching types for distance calculation");
            }
            _ => panic!("Expected InvalidVarType error"),
        }
    }

    /// Test StateVar distance calculation returns error for f64/i64 mismatch
    /// Validates: F64 and I64 are treated as different types for distance calculation
    /// Failure: F64/I64 type distinction is broken
    #[test]
    fn test_state_var_f64_i64_distance_mismatch() {
        let result = StateVar::F64(1500).distance(&StateVar::I64(1500));
        assert!(result.is_err());
        match result.unwrap_err() {
            StateError::InvalidVarType { expected, .. } => {
                assert_eq!(expected, "matching types for distance calculation");
            }
            _ => panic!("Expected InvalidVarType error"),
        }
    }

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
        let mut state = State::empty();

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

    // Tests for State satisfaction logic

    /// Test State satisfies with bool requirements
    /// Validates: Bool requirements are satisfied when state matches exactly
    /// Failure: Boolean satisfaction logic is broken
    #[test]
    fn test_state_satisfies_bool() {
        let mut state = State::empty();
        state.set("bool_var", StateVar::Bool(true));

        let mut conditions = State::empty();
        conditions.set("bool_var", StateVar::Bool(true));
        assert!(state.satisfies(&conditions));

        conditions.set("bool_var", StateVar::Bool(false));
        assert!(!state.satisfies(&conditions));
    }

    /// Test State satisfies with i64 requirements
    /// Validates: I64 requirements are satisfied when state value >= requirement
    /// Failure: Integer satisfaction logic is broken
    #[test]
    fn test_state_satisfies_i64() {
        let mut state = State::empty();
        state.set("int_var", StateVar::I64(42));

        let mut conditions = State::empty();
        // Test exact match
        conditions.set("int_var", StateVar::I64(42));
        assert!(state.satisfies(&conditions));

        // Test greater than requirement
        conditions.set("int_var", StateVar::I64(40));
        assert!(state.satisfies(&conditions));

        // Test less than requirement
        conditions.set("int_var", StateVar::I64(50));
        assert!(!state.satisfies(&conditions));
    }

    /// Test State satisfies with f64 requirements
    /// Validates: F64 requirements are satisfied when state value >= requirement
    /// Failure: Float satisfaction logic is broken
    #[test]
    fn test_state_satisfies_f64() {
        let mut state = State::empty();
        state.set("float_var", StateVar::F64(1500)); // 1.5

        let mut conditions = State::empty();
        // Test exact match
        conditions.set("float_var", StateVar::F64(1500));
        assert!(state.satisfies(&conditions));

        // Test greater than requirement
        conditions.set("float_var", StateVar::F64(1000)); // 1.0
        assert!(state.satisfies(&conditions));

        // Test less than requirement
        conditions.set("float_var", StateVar::F64(2000)); // 2.0
        assert!(!state.satisfies(&conditions));
    }

    /// Test State satisfies with string requirements
    /// Validates: String requirements are satisfied when state matches exactly
    /// Failure: String satisfaction logic is broken
    #[test]
    fn test_state_satisfies_string() {
        let mut state = State::empty();
        state.set("string_var", StateVar::String("test".to_string()));

        let mut conditions = State::empty();
        // Test exact match
        conditions.set("string_var", StateVar::String("test".to_string()));
        assert!(state.satisfies(&conditions));

        // Test mismatch
        conditions.set("string_var", StateVar::String("other".to_string()));
        assert!(!state.satisfies(&conditions));
    }

    /// Test State satisfies with missing variables
    /// Validates: Satisfaction fails when required variables are missing
    /// Failure: Missing variable detection is broken
    #[test]
    fn test_state_satisfies_missing_variable() {
        let mut state = State::empty();
        state.set("existing_var", StateVar::Bool(true));

        let mut conditions = State::empty();
        conditions.set("nonexistent_var", StateVar::Bool(true));
        assert!(!state.satisfies(&conditions));
    }

    /// Test State satisfies with type mismatches
    /// Validates: Satisfaction fails when variable types don't match
    /// Failure: Type mismatch detection is broken
    #[test]
    fn test_state_satisfies_type_mismatch() {
        let mut state = State::empty();
        state.set("var", StateVar::Bool(true));

        let mut conditions = State::empty();
        conditions.set("var", StateVar::I64(1));
        assert!(!state.satisfies(&conditions));
    }

    // Tests for State apply operations

    /// Test State apply with Set operations
    /// Validates: Set operations properly replace variable values
    /// Failure: Set operation logic is broken
    #[test]
    fn test_state_apply_set_operations() {
        let mut state = State::empty();
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
        let mut state = State::empty();
        state.set("int_var", StateVar::I64(10));

        let mut changes = HashMap::new();
        changes.insert("int_var".to_string(), StateOperation::Add(5));
        state.apply(&changes);

        assert_eq!(state.get::<i64>("int_var"), Some(15));
    }

    /// Test State apply with Add operations on f64
    /// Validates: Add operations properly increment f64 values
    /// Failure: F64 Add operation logic is broken
    #[test]
    fn test_state_apply_add_f64() {
        let mut state = State::empty();
        state.set("float_var", StateVar::F64(1500)); // 1.5

        let mut changes = HashMap::new();
        changes.insert("float_var".to_string(), StateOperation::Add(500)); // Add 0.5
        state.apply(&changes);

        assert_eq!(state.get::<f64>("float_var"), Some(2.0));
    }

    /// Test State apply with Subtract operations on i64
    /// Validates: Subtract operations properly decrement i64 values
    /// Failure: I64 Subtract operation logic is broken
    #[test]
    fn test_state_apply_subtract_i64() {
        let mut state = State::empty();
        state.set("int_var", StateVar::I64(15));

        let mut changes = HashMap::new();
        changes.insert("int_var".to_string(), StateOperation::Subtract(3));
        state.apply(&changes);

        assert_eq!(state.get::<i64>("int_var"), Some(12));
    }

    /// Test State apply with Subtract operations on f64
    /// Validates: Subtract operations properly decrement f64 values
    /// Failure: F64 Subtract operation logic is broken
    #[test]
    fn test_state_apply_subtract_f64() {
        let mut state = State::empty();
        state.set("float_var", StateVar::F64(2000)); // 2.0

        let mut changes = HashMap::new();
        changes.insert("float_var".to_string(), StateOperation::Subtract(750)); // Subtract 0.75
        state.apply(&changes);

        assert_eq!(state.get::<f64>("float_var"), Some(1.25));
    }

    /// Test State apply operations on missing variables
    /// Validates: Operations on missing variables are safely ignored
    /// Failure: Missing variable handling in apply is broken
    #[test]
    fn test_state_apply_missing_variable() {
        let mut state = State::empty();

        let mut changes = HashMap::new();
        changes.insert("nonexistent".to_string(), StateOperation::Add(5));
        state.apply(&changes);

        // Should not create the variable or crash
        assert_eq!(state.get::<i64>("nonexistent"), None);
    }

    // Tests for State merge operations

    /// Test State merge functionality
    /// Validates: Merge properly combines states with override behavior
    /// Failure: State merge logic is broken
    #[test]
    fn test_state_merge() {
        let mut state1 = State::empty();
        state1.set("var1", StateVar::Bool(true));
        state1.set("var2", StateVar::I64(10));

        let mut state2 = State::empty();
        state2.set("var2", StateVar::I64(20)); // Should override
        state2.set("var3", StateVar::String("test".to_string()));

        state1.merge(&state2);

        assert_eq!(state1.get::<bool>("var1"), Some(true));
        assert_eq!(state1.get::<i64>("var2"), Some(20)); // Overridden
        assert_eq!(state1.get::<String>("var3"), Some("test".to_string()));
    }

    // Tests for State hash consistency

    /// Test State hash consistency regardless of insertion order
    /// Validates: Hash is consistent regardless of variable insertion order
    /// Failure: State hash implementation is broken
    #[test]
    fn test_state_hash_consistency() {
        let mut state1 = State::empty();
        state1.set("a", StateVar::Bool(true));
        state1.set("b", StateVar::I64(10));

        let mut state2 = State::empty();
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
        let mut state = State::empty();
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
        let mut state = State::empty();
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
        let mut state = State::empty();
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
        let mut state = State::empty();
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
        let state = State::new().set("has_key", true).build();
        assert_eq!(state.get::<bool>("has_key"), Some(true));
    }

    /// Test StateBuilder set method with int
    /// Validates: Builder set method properly sets integer values
    /// Failure: StateBuilder set method is broken
    #[test]
    fn test_state_builder_int() {
        let state = State::new().set("gold", 100).build();
        assert_eq!(state.get::<i64>("gold"), Some(100));
    }

    /// Test StateBuilder set method with float
    /// Validates: Builder set method properly sets float values
    /// Failure: StateBuilder set method is broken
    #[test]
    fn test_state_builder_float() {
        let state = State::new().set("health", 75.5).build();
        assert_eq!(state.get::<f64>("health"), Some(75.5));
    }

    /// Test StateBuilder set method with enum/string
    /// Validates: Builder set method properly sets string values
    /// Failure: StateBuilder set method is broken
    #[test]
    fn test_state_builder_enum_val() {
        let state = State::new().set("location", "town").build();
        assert_eq!(state.get::<String>("location"), Some("town".to_string()));
    }

    /// Test StateBuilder unified set method
    /// Validates: Builder set method works with all value types via IntoStateVar
    /// Failure: StateBuilder unified set method is broken
    #[test]
    fn test_state_builder_unified_set() {
        let state = State::new()
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
        let state = State::new()
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

    // Tests for StateOperation helper methods

    /// Test StateOperation helper methods for i64
    /// Validates: set_i64, add_i64, subtract_i64 create correct operations
    /// Failure: StateOperation i64 helper methods are broken
    #[test]
    fn test_state_operation_i64_helpers() {
        assert_eq!(
            StateOperation::set_i64(100),
            StateOperation::Set(StateVar::I64(100))
        );
        assert_eq!(StateOperation::add_i64(50), StateOperation::Add(50));
        assert_eq!(
            StateOperation::subtract_i64(25),
            StateOperation::Subtract(25)
        );
    }

    /// Test StateOperation helper methods for f64
    /// Validates: set_f64, add_f64, subtract_f64 create correct operations with fixed-point conversion
    /// Failure: StateOperation f64 helper methods are broken
    #[test]
    fn test_state_operation_f64_helpers() {
        assert_eq!(
            StateOperation::set_f64(1.5),
            StateOperation::Set(StateVar::F64(1500))
        );
        assert_eq!(StateOperation::add_f64(0.5), StateOperation::Add(500));
        assert_eq!(
            StateOperation::subtract_f64(0.25),
            StateOperation::Subtract(250)
        );
    }

    /// Test StateOperation f64 helpers with rounding
    /// Validates: f64 helpers properly handle rounding to 3 decimal places
    /// Failure: StateOperation f64 rounding is broken
    #[test]
    fn test_state_operation_f64_rounding() {
        // Test rounding in set_f64
        assert_eq!(
            StateOperation::set_f64(1.2345),
            StateOperation::Set(StateVar::F64(1235))
        );

        // Test rounding in add_f64
        assert_eq!(StateOperation::add_f64(0.1234), StateOperation::Add(123));

        // Test rounding in subtract_f64
        assert_eq!(
            StateOperation::subtract_f64(0.6789),
            StateOperation::Subtract(679)
        );
    }

    /// Test complete f64 workflow with helper methods
    /// Validates: F64 operations work correctly when using helper methods throughout
    /// Failure: F64 helper method integration is broken
    #[test]
    fn test_state_f64_workflow_with_helpers() {
        let mut state = State::empty();

        // Test setting with helper
        state.set("value", StateVar::from_f64(1.5));

        let mut changes = HashMap::new();
        changes.insert("value".to_string(), StateOperation::add_f64(0.5)); // Add 0.5
        state.apply(&changes);
        assert_eq!(state.get::<f64>("value"), Some(2.0));

        // Test subtraction with helper
        changes.clear();
        changes.insert("value".to_string(), StateOperation::subtract_f64(0.75)); // Subtract 0.75
        state.apply(&changes);
        assert_eq!(state.get::<f64>("value"), Some(1.25));

        // Test set with helper
        changes.clear();
        changes.insert("value".to_string(), StateOperation::set_f64(3.04159)); // Should round to 3.042
        state.apply(&changes);
        assert_eq!(state.get::<f64>("value"), Some(3.042));
    }
}
