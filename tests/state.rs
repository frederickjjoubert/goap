#[cfg(test)]
mod tests {
    use goap::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn test_state_var_distance() {
        // Test boolean distance
        assert_eq!(StateVar::Bool(true).distance(&StateVar::Bool(true)), 0);
        assert_eq!(StateVar::Bool(false).distance(&StateVar::Bool(false)), 0);
        assert_eq!(StateVar::Bool(true).distance(&StateVar::Bool(false)), 1);
        assert_eq!(StateVar::Bool(false).distance(&StateVar::Bool(true)), 1);

        // Test i64 distance
        assert_eq!(StateVar::I64(5).distance(&StateVar::I64(5)), 0);
        assert_eq!(StateVar::I64(10).distance(&StateVar::I64(5)), 5);
        assert_eq!(StateVar::I64(5).distance(&StateVar::I64(10)), 5);
        assert_eq!(StateVar::I64(-5).distance(&StateVar::I64(5)), 10);

        // Test enum distance
        assert_eq!(
            StateVar::String("A".to_string()).distance(&StateVar::String("A".to_string())),
            0
        );
        assert_eq!(
            StateVar::String("A".to_string()).distance(&StateVar::String("B".to_string())),
            1
        );
    }

    #[test]
    #[should_panic(expected = "Cannot calculate distance between different StateVar types")]
    fn test_state_var_distance_panic() {
        StateVar::Bool(true).distance(&StateVar::I64(5));
    }

    #[test]
    fn test_world_state_basic_operations() {
        let mut state = State::new();

        // Test set and get
        state.set("bool_var", StateVar::Bool(true));
        state.set("int_var", StateVar::I64(42));
        state.set("enum_var", StateVar::String("test".to_string()));

        assert_eq!(state.get("bool_var"), Some(&StateVar::Bool(true)));
        assert_eq!(state.get("int_var"), Some(&StateVar::I64(42)));
        assert_eq!(
            state.get("enum_var"),
            Some(&StateVar::String("test".to_string()))
        );
        assert_eq!(state.get("nonexistent"), None);
    }

    #[test]
    fn test_world_state_satisfies() {
        let mut state = State::new();
        state.set("bool_var", StateVar::Bool(true));
        state.set("int_var", StateVar::I64(42));
        state.set("enum_var", StateVar::String("test".to_string()));

        // Test exact matches
        let mut conditions = State::new();
        conditions.set("bool_var", StateVar::Bool(true));
        assert!(state.satisfies(&conditions));

        // Test integer greater than
        conditions.set("int_var", StateVar::I64(40));
        assert!(state.satisfies(&conditions));

        // Test failure cases
        conditions.set("bool_var", StateVar::Bool(false));
        assert!(!state.satisfies(&conditions));

        conditions = State::new();
        conditions.set("nonexistent", StateVar::Bool(true));
        assert!(!state.satisfies(&conditions));
    }

    #[test]
    fn test_world_state_apply() {
        let mut state = State::new();
        state.set("bool_var", StateVar::Bool(false));
        state.set("int_var", StateVar::I64(10));
        state.set("enum_var", StateVar::String("old".to_string()));

        let mut changes = HashMap::new();
        changes.insert(
            "bool_var".to_string(),
            StateOperation::Set(StateVar::Bool(true)),
        );
        changes.insert("int_var".to_string(), StateOperation::Add(5));
        changes.insert(
            "enum_var".to_string(),
            StateOperation::Set(StateVar::String("new".to_string())),
        );

        state.apply(&changes);

        assert_eq!(state.get("bool_var"), Some(&StateVar::Bool(true)));
        assert_eq!(state.get("int_var"), Some(&StateVar::I64(15)));
        assert_eq!(
            state.get("enum_var"),
            Some(&StateVar::String("new".to_string()))
        );

        // Test subtraction
        let mut changes = HashMap::new();
        changes.insert("int_var".to_string(), StateOperation::Subtract(3));
        state.apply(&changes);
        assert_eq!(state.get("int_var"), Some(&StateVar::I64(12)));
    }

    #[test]
    fn test_world_state_merge() {
        let mut state1 = State::new();
        state1.set("var1", StateVar::Bool(true));
        state1.set("var2", StateVar::I64(10));

        let mut state2 = State::new();
        state2.set("var2", StateVar::I64(20));
        state2.set("var3", StateVar::String("test".to_string()));

        state1.merge(&state2);

        assert_eq!(state1.get("var1"), Some(&StateVar::Bool(true)));
        assert_eq!(state1.get("var2"), Some(&StateVar::I64(20)));
        assert_eq!(
            state1.get("var3"),
            Some(&StateVar::String("test".to_string()))
        );
    }

    #[test]
    fn test_world_state_hash() {
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

    #[test]
    fn test_state_var_f64_distance() {
        // Test fixed point number distance (3 decimal places)
        // 1.5 is stored as 1500
        assert_eq!(StateVar::F64(1500).distance(&StateVar::F64(1500)), 0);
        // 1.5 to 2.5 = distance of 1000 (1.0)
        assert_eq!(StateVar::F64(1500).distance(&StateVar::F64(2500)), 1000);
        // 2.5 to 1.5 = distance of 1000 (1.0)
        assert_eq!(StateVar::F64(2500).distance(&StateVar::F64(1500)), 1000);
        // -1.5 to 1.5 = distance of 3000 (3.0)
        assert_eq!(StateVar::F64(-1500).distance(&StateVar::F64(1500)), 3000);
        // Test small decimal differences
        // 1.001 to 1.002 = distance of 1 (0.001)
        assert_eq!(StateVar::F64(1001).distance(&StateVar::F64(1002)), 1);
    }

    #[test]
    #[should_panic(expected = "Cannot calculate distance between different StateVar types")]
    fn test_state_var_f64_distance_panic() {
        StateVar::F64(1500).distance(&StateVar::I64(1500));
    }

    #[test]
    fn test_world_state_f64_operations() {
        let mut state = State::new();

        // Test setting and getting F64 values
        state.set("speed", StateVar::F64(1500)); // 1.5
        state.set("height", StateVar::F64(2000)); // 2.0

        assert_eq!(state.get("speed"), Some(&StateVar::F64(1500)));
        assert_eq!(state.get("height"), Some(&StateVar::F64(2000)));

        // Test satisfies with F64 values
        let mut conditions;

        println!("\n=== Testing exact match ===");
        // Test exact match
        conditions = State::new(); // Clear conditions
        conditions.set("speed", StateVar::F64(1500));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing greater than satisfies ===");
        // Test greater than satisfies
        conditions = State::new(); // Clear conditions
        conditions.set("speed", StateVar::F64(1000)); // 1.0
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing less than fails ===");
        // Test less than fails
        conditions = State::new(); // Clear conditions
        conditions.set("speed", StateVar::F64(2000)); // 2.0
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(!state.satisfies(&conditions));

        println!("\n=== Testing small decimal differences ===");
        // Test small decimal differences
        state.set("precise", StateVar::F64(1001)); // 1.001
        conditions = State::new(); // Clear conditions
        conditions.set("precise", StateVar::F64(1000)); // 1.000
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        conditions = State::new(); // Clear conditions
        conditions.set("precise", StateVar::F64(1002)); // 1.002
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(!state.satisfies(&conditions));
    }

    #[test]
    fn test_world_state_f64_hash_consistency() {
        let mut state1 = State::new();
        state1.set("a", StateVar::F64(1500)); // 1.5
        state1.set("b", StateVar::F64(2500)); // 2.5

        let mut state2 = State::new();
        state2.set("b", StateVar::F64(2500)); // 2.5
        state2.set("a", StateVar::F64(1500)); // 1.5

        // Hash should be the same regardless of insertion order
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        state1.hash(&mut hasher1);
        state2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_world_state_f64_arithmetic() {
        let mut state = State::new();

        // Test addition
        state.set("value", StateVar::F64(1500)); // 1.5

        let mut changes = HashMap::new();
        changes.insert("value".to_string(), StateOperation::Add(500)); // Add 0.5
        state.apply(&changes);
        assert_eq!(state.get("value"), Some(&StateVar::F64(2000))); // Should be 2.0

        // Test subtraction
        changes.clear();
        changes.insert("value".to_string(), StateOperation::Subtract(750)); // Subtract 0.75
        state.apply(&changes);
        assert_eq!(state.get("value"), Some(&StateVar::F64(1250))); // Should be 1.25

        // Test small decimal arithmetic
        state.set("precise", StateVar::F64(1000)); // 1.000

        changes.clear();
        changes.insert("precise".to_string(), StateOperation::Add(1)); // Add 0.001
        state.apply(&changes);
        assert_eq!(state.get("precise"), Some(&StateVar::F64(1001))); // Should be 1.001

        changes.clear();
        changes.insert("precise".to_string(), StateOperation::Subtract(2)); // Subtract 0.002
        state.apply(&changes);
        assert_eq!(state.get("precise"), Some(&StateVar::F64(999))); // Should be 0.999

        // Test negative values
        state.set("negative", StateVar::F64(-1500)); // -1.5

        changes.clear();
        changes.insert("negative".to_string(), StateOperation::Add(2500)); // Add 2.5
        state.apply(&changes);
        assert_eq!(state.get("negative"), Some(&StateVar::F64(1000))); // Should be 1.0

        changes.clear();
        changes.insert("negative".to_string(), StateOperation::Subtract(3000)); // Subtract 3.0
        state.apply(&changes);
        assert_eq!(state.get("negative"), Some(&StateVar::F64(-2000))); // Should be -2.0
    }

    #[test]
    fn test_world_state_f64_edge_cases() {
        let mut state = State::new();

        // Test zero values
        state.set("zero", StateVar::F64(0));
        assert_eq!(state.get("zero"), Some(&StateVar::F64(0)));

        // Test adding to zero
        let mut changes = HashMap::new();
        changes.insert("zero".to_string(), StateOperation::Add(1)); // Add 0.001
        state.apply(&changes);
        assert_eq!(state.get("zero"), Some(&StateVar::F64(1))); // Should be 0.001

        // Test subtracting to zero
        state.set("almost_zero", StateVar::F64(1)); // 0.001
        changes.clear();
        changes.insert("almost_zero".to_string(), StateOperation::Subtract(1));
        state.apply(&changes);
        assert_eq!(state.get("almost_zero"), Some(&StateVar::F64(0)));

        // Test large numbers
        state.set("large", StateVar::F64(1_000_000_000)); // 1,000,000.000
        changes.clear();
        changes.insert("large".to_string(), StateOperation::Add(1)); // Add 0.001
        state.apply(&changes);
        assert_eq!(state.get("large"), Some(&StateVar::F64(1_000_000_001)));

        // Test crossing zero with operations
        state.set("cross_zero", StateVar::F64(1000)); // 1.000
        changes.clear();
        changes.insert("cross_zero".to_string(), StateOperation::Subtract(2000)); // Subtract 2.000
        state.apply(&changes);
        assert_eq!(state.get("cross_zero"), Some(&StateVar::F64(-1000))); // Should be -1.000

        changes.clear();
        changes.insert("cross_zero".to_string(), StateOperation::Add(1000)); // Add 1.000
        state.apply(&changes);
        assert_eq!(state.get("cross_zero"), Some(&StateVar::F64(0))); // Back to 0
    }

    #[test]
    fn test_world_state_f64_mixed_operations() {
        let mut state = State::new();

        // Test mixed operations in sequence
        state.set("mixed", StateVar::F64(1500)); // 1.500

        let mut changes = HashMap::new();
        // Add 0.1
        changes.insert("mixed".to_string(), StateOperation::Add(100));
        state.apply(&changes);

        // Subtract 0.05
        changes.clear();
        changes.insert("mixed".to_string(), StateOperation::Subtract(50));
        state.apply(&changes);

        // Add 0.005
        changes.clear();
        changes.insert("mixed".to_string(), StateOperation::Add(5));
        state.apply(&changes);

        // Should be 1.555
        assert_eq!(state.get("mixed"), Some(&StateVar::F64(1555)));

        // Test setting after operations
        changes.clear();
        changes.insert(
            "mixed".to_string(),
            StateOperation::Set(StateVar::F64(2000)),
        );
        state.apply(&changes);
        assert_eq!(state.get("mixed"), Some(&StateVar::F64(2000))); // Should be 2.000
    }

    #[test]
    fn test_world_state_f64_satisfies_edge_cases() {
        let mut state;
        let mut conditions;

        println!("\n=== Testing zero satisfies zero ===");
        // Test zero satisfies zero
        state = State::new(); // Clear state
        conditions = State::new(); // Clear conditions
        state.set("zero", StateVar::F64(0));
        conditions.set("zero", StateVar::F64(0));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing positive satisfies zero ===");
        // Test positive satisfies zero
        state = State::new(); // Clear state
        conditions = State::new(); // Clear conditions
        state.set("pos", StateVar::F64(1)); // 0.001
        conditions.set("pos", StateVar::F64(0));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing negative doesn't satisfy zero ===");
        // Test negative doesn't satisfy zero
        state = State::new(); // Clear state
        conditions = State::new(); // Clear conditions
        state.set("neg", StateVar::F64(-1)); // -0.001
        conditions.set("neg", StateVar::F64(0));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(!state.satisfies(&conditions));

        println!("\n=== Testing exact decimal matches ===");
        // Test exact decimal matches
        state = State::new(); // Clear state
        conditions = State::new(); // Clear conditions
        state.set("exact", StateVar::F64(1234)); // 1.234
        conditions.set("exact", StateVar::F64(1234));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing very close values ===");
        // Test very close values
        state = State::new(); // Clear state
        conditions = State::new(); // Clear conditions
        state.set("close", StateVar::F64(1000)); // 1.000
        conditions.set("close", StateVar::F64(999)); // 0.999
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing boundary conditions ===");
        // Test boundary conditions
        state = State::new(); // Clear state
        conditions = State::new(); // Clear conditions
        state.set("boundary", StateVar::F64(1000)); // 1.000
        conditions.set("boundary", StateVar::F64(1001)); // 1.001
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(!state.satisfies(&conditions));
    }

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

    #[test]
    fn test_world_state_f64_operations_with_conversion() {
        let mut state = State::new();

        // Test setting and getting F64 values using floating point
        state.set("speed", StateVar::from_f64(1.5));
        state.set("height", StateVar::from_f64(2.0));

        assert_eq!(state.get("speed"), Some(&StateVar::F64(1500)));
        assert_eq!(state.get("height"), Some(&StateVar::F64(2000)));

        // Test satisfies with F64 values
        let mut conditions;

        println!("\n=== Testing exact match ===");
        conditions = State::new();
        conditions.set("speed", StateVar::from_f64(1.5));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing greater than satisfies ===");
        conditions = State::new();
        conditions.set("speed", StateVar::from_f64(1.0));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing less than fails ===");
        conditions = State::new();
        conditions.set("speed", StateVar::from_f64(2.0));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(!state.satisfies(&conditions));

        println!("\n=== Testing small decimal differences ===");
        state.set("precise", StateVar::from_f64(1.001));
        conditions = State::new();
        conditions.set("precise", StateVar::from_f64(1.0));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(state.satisfies(&conditions));

        conditions = State::new();
        conditions.set("precise", StateVar::from_f64(1.002));
        println!("State: {state:?}");
        println!("Conditions: {conditions:?}");
        assert!(!state.satisfies(&conditions));
    }

    #[test]
    fn test_world_state_f64_arithmetic_with_helpers() {
        let mut state = State::new();

        // Test addition with helper
        state.set("value", StateVar::from_f64(1.5));

        let mut changes = HashMap::new();
        changes.insert("value".to_string(), StateOperation::add_f64(0.5)); // Add 0.5
        state.apply(&changes);
        assert_eq!(state.get("value").and_then(|v| v.as_f64()), Some(2.0));

        // Test subtraction with helper
        changes.clear();
        changes.insert("value".to_string(), StateOperation::subtract_f64(0.75)); // Subtract 0.75
        state.apply(&changes);
        assert_eq!(state.get("value").and_then(|v| v.as_f64()), Some(1.25));

        // Test set with helper
        changes.clear();
        changes.insert("value".to_string(), StateOperation::set_f64(4.56789)); // Should round to 4.568
        state.apply(&changes);
        assert_eq!(state.get("value").and_then(|v| v.as_f64()), Some(4.568));

        // Test small decimal arithmetic
        state.set("precise", StateVar::from_f64(1.0));

        changes.clear();
        changes.insert("precise".to_string(), StateOperation::add_f64(0.001)); // Add 0.001
        state.apply(&changes);
        assert_eq!(state.get("precise").and_then(|v| v.as_f64()), Some(1.001));

        changes.clear();
        changes.insert("precise".to_string(), StateOperation::subtract_f64(0.002)); // Subtract 0.002
        state.apply(&changes);
        assert_eq!(state.get("precise").and_then(|v| v.as_f64()), Some(0.999));

        // Test negative values
        state.set("negative", StateVar::from_f64(-1.5));

        changes.clear();
        changes.insert("negative".to_string(), StateOperation::add_f64(2.5)); // Add 2.5
        state.apply(&changes);
        assert_eq!(state.get("negative").and_then(|v| v.as_f64()), Some(1.0));

        changes.clear();
        changes.insert("negative".to_string(), StateOperation::set_f64(-4.567)); // Should round to -4.567
        state.apply(&changes);
        assert_eq!(state.get("negative").and_then(|v| v.as_f64()), Some(-4.567));

        // Test rounding behavior
        state.set("round", StateVar::from_f64(1.0));
        changes.clear();
        changes.insert("round".to_string(), StateOperation::add_f64(0.1234)); // Should round to 0.123
        state.apply(&changes);
        assert_eq!(state.get("round").and_then(|v| v.as_f64()), Some(1.123));
    }

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

    #[test]
    fn test_world_state_builder() {
        let state = State::builder()
            .bool("has_wood", true)
            .int("energy", 100)
            .float("temperature", 22.5)
            .enum_val("location", "forest")
            .build();

        // Test boolean value
        assert_eq!(state.get("has_wood"), Some(&StateVar::Bool(true)));

        // Test integer value
        assert_eq!(state.get("energy"), Some(&StateVar::I64(100)));

        // Test float value (remember it's stored as fixed point)
        assert_eq!(
            state.get("temperature"),
            Some(&StateVar::F64(22500)) // 22.5 * 1000
        );

        // Test enum value
        assert_eq!(
            state.get("location"),
            Some(&StateVar::String("forest".to_string()))
        );
    }

    #[test]
    fn test_world_state_builder_chaining() {
        // Test that we can chain multiple values of the same type
        let state = State::builder()
            .bool("has_wood", true)
            .bool("has_tools", false)
            .int("wood_count", 5)
            .int("tool_count", 0)
            .float("health", 100.0)
            .float("energy", 50.0)
            .enum_val("location", "forest")
            .enum_val("weather", "sunny")
            .build();

        assert_eq!(state.get("has_wood"), Some(&StateVar::Bool(true)));
        assert_eq!(state.get("has_tools"), Some(&StateVar::Bool(false)));
        assert_eq!(state.get("wood_count"), Some(&StateVar::I64(5)));
        assert_eq!(state.get("tool_count"), Some(&StateVar::I64(0)));
        assert_eq!(state.get("health"), Some(&StateVar::F64(100000)));
        assert_eq!(state.get("energy"), Some(&StateVar::F64(50000)));
        assert_eq!(
            state.get("location"),
            Some(&StateVar::String("forest".to_string()))
        );
        assert_eq!(
            state.get("weather"),
            Some(&StateVar::String("sunny".to_string()))
        );
    }
}
