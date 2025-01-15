use std::collections::HashMap;
use std::hash::Hash;

// Variable types that can be stored in state
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StateVar {
    Bool(bool),
    I64(i64),
    /// F64 values are stored as fixed-point numbers with 3 decimal places of precision.
    /// This means that floating point values are multiplied by 1000 and stored as integers.
    /// For example:
    /// - 1.5 is stored as 1500
    /// - 0.001 is stored as 1
    /// - -1.5 is stored as -1500
    ///
    /// This approach is used because:
    /// 1. We need exact equality comparison for the GOAP algorithm
    /// 2. We need to implement Hash and Eq traits
    /// 3. Floating point numbers don't support exact equality or hashing
    ///
    /// To make this API more ergonomic, use the helper methods:
    /// - `StateVar::from_f64(1.5)` to create a value
    /// - `state_var.as_f64()` to read a value
    ///
    /// Note that arithmetic operations (Add/Subtract) can be performed using convenience methods:
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use goap::prelude::*;
    /// # let mut state = WorldState::new();
    /// # state.set("value", StateVar::from_f64(1.5));
    /// let mut changes = HashMap::new();
    /// changes.insert("value".to_string(), StateOperation::add_f64(0.5)); // Add 0.5
    /// state.apply(&changes);
    /// assert_eq!(state.get("value").and_then(|v| v.as_f64()), Some(2.0));
    /// ```
    F64(i64),
    Enum(String),
}

impl StateVar {
    /// Creates a new F64 StateVar from a floating point value.
    /// The value will be rounded to 3 decimal places.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let value = StateVar::from_f64(1.5);
    /// assert_eq!(value, StateVar::F64(1500));
    ///
    /// // Handles rounding
    /// let value = StateVar::from_f64(1.2345);
    /// assert_eq!(value.as_f64(), Some(1.235));
    /// ```
    pub fn from_f64(value: f64) -> Self {
        // Convert to fixed point with 3 decimal places
        StateVar::F64((value * 1000.0).round() as i64)
    }

    /// Converts an F64 StateVar back to a floating point value.
    /// Returns None if the StateVar is not an F64.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let value = StateVar::from_f64(1.5);
    /// assert_eq!(value.as_f64(), Some(1.5));
    ///
    /// // Returns None for other types
    /// assert_eq!(StateVar::Bool(true).as_f64(), None);
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            StateVar::F64(value) => Some(*value as f64 / 1000.0),
            _ => None,
        }
    }

    pub fn distance(&self, other: &StateVar) -> u64 {
        match (self, other) {
            (StateVar::Bool(a), StateVar::Bool(b)) => {
                if a == b {
                    0
                } else {
                    1
                }
            }
            (StateVar::I64(a), StateVar::I64(b)) => (*a - *b).unsigned_abs(),
            (StateVar::F64(a), StateVar::F64(b)) => (*a - *b).unsigned_abs(),
            (StateVar::Enum(a), StateVar::Enum(b)) => {
                if a == b {
                    0
                } else {
                    1
                }
            }
            _ => panic!("Cannot calculate distance between different StateVar types"),
        }
    }
}

// From implementations for common types
impl From<bool> for StateVar {
    fn from(value: bool) -> Self {
        StateVar::Bool(value)
    }
}

impl From<i64> for StateVar {
    fn from(value: i64) -> Self {
        StateVar::I64(value)
    }
}

impl From<f64> for StateVar {
    fn from(value: f64) -> Self {
        StateVar::from_f64(value)
    }
}

impl From<String> for StateVar {
    fn from(value: String) -> Self {
        StateVar::Enum(value)
    }
}

impl From<&str> for StateVar {
    fn from(value: &str) -> Self {
        StateVar::Enum(value.to_string())
    }
}

// Add these convenience implementations for smaller integer types
impl From<i32> for StateVar {
    fn from(value: i32) -> Self {
        StateVar::I64(value as i64)
    }
}

impl From<i16> for StateVar {
    fn from(value: i16) -> Self {
        StateVar::I64(value as i64)
    }
}

impl From<i8> for StateVar {
    fn from(value: i8) -> Self {
        StateVar::I64(value as i64)
    }
}

// First, let's define how a state variable can be modified
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StateOperation {
    Set(StateVar),
    Add(i64),      // Add value (for integers and fixed-point)
    Subtract(i64), // Subtract value (for integers and fixed-point)
}

impl StateOperation {
    /// Creates a Set operation that will set the value to the given i64 value.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let op = StateOperation::set_i64(100); // Will set the value to 100
    /// assert_eq!(op, StateOperation::Set(StateVar::I64(100)));
    /// ```
    pub fn set_i64(value: i64) -> Self {
        StateOperation::Set(StateVar::I64(value))
    }

    /// Creates an Add operation that will add the given i64 value.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let op = StateOperation::add_i64(50); // Will add 50 to the value
    /// assert_eq!(op, StateOperation::Add(50));
    /// ```
    pub fn add_i64(value: i64) -> Self {
        StateOperation::Add(value)
    }

    /// Creates a Subtract operation that will subtract the given i64 value.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let op = StateOperation::subtract_i64(50); // Will subtract 50 from the value
    /// assert_eq!(op, StateOperation::Subtract(50));
    /// ```
    pub fn subtract_i64(value: i64) -> Self {
        StateOperation::Subtract(value)
    }

    /// Creates a Set operation that will set the value to the given f64 value.
    /// The value will be converted to fixed point with 3 decimal places.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let op = StateOperation::set_f64(1.5); // Will set the value to 1.5
    /// assert_eq!(op, StateOperation::Set(StateVar::F64(1500)));
    /// ```
    pub fn set_f64(value: f64) -> Self {
        StateOperation::Set(StateVar::from_f64(value))
    }

    /// Creates an Add operation that will add the given f64 value.
    /// The value will be converted to fixed point with 3 decimal places.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let op = StateOperation::add_f64(1.5); // Will add 1.5 to the value
    /// assert_eq!(op, StateOperation::Add(1500));
    /// ```
    pub fn add_f64(value: f64) -> Self {
        StateOperation::Add((value * 1000.0).round() as i64)
    }

    /// Creates a Subtract operation that will subtract the given f64 value.
    /// The value will be converted to fixed point with 3 decimal places.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let op = StateOperation::subtract_f64(1.5); // Will subtract 1.5 from the value
    /// assert_eq!(op, StateOperation::Subtract(1500));
    /// ```
    pub fn subtract_f64(value: f64) -> Self {
        StateOperation::Subtract((value * 1000.0).round() as i64)
    }
}

// State representation as a collection of named variables
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WorldState {
    pub vars: HashMap<String, StateVar>,
}

// Manual Hash implementation for WorldState
impl Hash for WorldState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Sort keys to ensure consistent hashing
        let mut keys: Vec<_> = self.vars.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(state);
            self.vars.get(key).unwrap().hash(state);
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            vars: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: StateVar) {
        self.vars.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&StateVar> {
        self.vars.get(key)
    }

    pub fn satisfies(&self, conditions: &WorldState) -> bool {
        for (key, value) in &conditions.vars {
            match self.vars.get(key) {
                Some(current_value) => {
                    match (current_value, value) {
                        (StateVar::Bool(cur), StateVar::Bool(req)) => {
                            if cur != req {
                                return false;
                            }
                        }
                        (StateVar::I64(cur), StateVar::I64(req)) => {
                            if cur < req {
                                return false;
                            }
                        }
                        (StateVar::F64(cur), StateVar::F64(req)) => {
                            if cur < req {
                                return false;
                            }
                        }
                        (StateVar::Enum(cur), StateVar::Enum(req)) => {
                            if cur != req {
                                return false;
                            }
                        }
                        _ => return false, // Mismatched types
                    }
                }
                None => return false,
            }
        }
        true
    }

    pub fn apply(&mut self, changes: &HashMap<String, StateOperation>) {
        for (key, operation) in changes {
            match operation {
                StateOperation::Set(value) => {
                    self.vars.insert(key.clone(), value.clone());
                }
                StateOperation::Add(amount) => match self.vars.get(key) {
                    Some(StateVar::I64(current)) => {
                        self.vars
                            .insert(key.clone(), StateVar::I64(current + amount));
                    }
                    Some(StateVar::F64(current)) => {
                        self.vars
                            .insert(key.clone(), StateVar::F64(current + amount));
                    }
                    _ => {}
                },
                StateOperation::Subtract(amount) => match self.vars.get(key) {
                    Some(StateVar::I64(current)) => {
                        self.vars
                            .insert(key.clone(), StateVar::I64(current - amount));
                    }
                    Some(StateVar::F64(current)) => {
                        self.vars
                            .insert(key.clone(), StateVar::F64(current - amount));
                    }
                    _ => {}
                },
            }
        }
    }

    pub fn merge(&mut self, other: &WorldState) {
        for (key, value) in &other.vars {
            self.vars.insert(key.clone(), value.clone());
        }
    }

    pub fn builder() -> WorldStateBuilder {
        WorldStateBuilder::new()
    }
}

pub struct WorldStateBuilder {
    vars: HashMap<String, StateVar>,
}

impl WorldStateBuilder {
    pub fn new() -> Self {
        WorldStateBuilder {
            vars: HashMap::new(),
        }
    }

    pub fn bool(mut self, key: &str, value: bool) -> Self {
        self.vars.insert(key.to_string(), StateVar::Bool(value));
        self
    }

    pub fn int(mut self, key: &str, value: i64) -> Self {
        self.vars.insert(key.to_string(), StateVar::I64(value));
        self
    }

    pub fn float(mut self, key: &str, value: f64) -> Self {
        self.vars.insert(key.to_string(), StateVar::from_f64(value));
        self
    }

    pub fn enum_val(mut self, key: &str, value: impl Into<String>) -> Self {
        self.vars
            .insert(key.to_string(), StateVar::Enum(value.into()));
        self
    }

    pub fn build(self) -> WorldState {
        WorldState { vars: self.vars }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            StateVar::Enum("A".to_string()).distance(&StateVar::Enum("A".to_string())),
            0
        );
        assert_eq!(
            StateVar::Enum("A".to_string()).distance(&StateVar::Enum("B".to_string())),
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
        let mut state = WorldState::new();

        // Test set and get
        state.set("bool_var", StateVar::Bool(true));
        state.set("int_var", StateVar::I64(42));
        state.set("enum_var", StateVar::Enum("test".to_string()));

        assert_eq!(state.get("bool_var"), Some(&StateVar::Bool(true)));
        assert_eq!(state.get("int_var"), Some(&StateVar::I64(42)));
        assert_eq!(
            state.get("enum_var"),
            Some(&StateVar::Enum("test".to_string()))
        );
        assert_eq!(state.get("nonexistent"), None);
    }

    #[test]
    fn test_world_state_satisfies() {
        let mut state = WorldState::new();
        state.set("bool_var", StateVar::Bool(true));
        state.set("int_var", StateVar::I64(42));
        state.set("enum_var", StateVar::Enum("test".to_string()));

        // Test exact matches
        let mut conditions = WorldState::new();
        conditions.set("bool_var", StateVar::Bool(true));
        assert!(state.satisfies(&conditions));

        // Test integer greater than
        conditions.set("int_var", StateVar::I64(40));
        assert!(state.satisfies(&conditions));

        // Test failure cases
        conditions.set("bool_var", StateVar::Bool(false));
        assert!(!state.satisfies(&conditions));

        conditions = WorldState::new();
        conditions.set("nonexistent", StateVar::Bool(true));
        assert!(!state.satisfies(&conditions));
    }

    #[test]
    fn test_world_state_apply() {
        let mut state = WorldState::new();
        state.set("bool_var", StateVar::Bool(false));
        state.set("int_var", StateVar::I64(10));
        state.set("enum_var", StateVar::Enum("old".to_string()));

        let mut changes = HashMap::new();
        changes.insert(
            "bool_var".to_string(),
            StateOperation::Set(StateVar::Bool(true)),
        );
        changes.insert("int_var".to_string(), StateOperation::Add(5));
        changes.insert(
            "enum_var".to_string(),
            StateOperation::Set(StateVar::Enum("new".to_string())),
        );

        state.apply(&changes);

        assert_eq!(state.get("bool_var"), Some(&StateVar::Bool(true)));
        assert_eq!(state.get("int_var"), Some(&StateVar::I64(15)));
        assert_eq!(
            state.get("enum_var"),
            Some(&StateVar::Enum("new".to_string()))
        );

        // Test subtraction
        let mut changes = HashMap::new();
        changes.insert("int_var".to_string(), StateOperation::Subtract(3));
        state.apply(&changes);
        assert_eq!(state.get("int_var"), Some(&StateVar::I64(12)));
    }

    #[test]
    fn test_world_state_merge() {
        let mut state1 = WorldState::new();
        state1.set("var1", StateVar::Bool(true));
        state1.set("var2", StateVar::I64(10));

        let mut state2 = WorldState::new();
        state2.set("var2", StateVar::I64(20));
        state2.set("var3", StateVar::Enum("test".to_string()));

        state1.merge(&state2);

        assert_eq!(state1.get("var1"), Some(&StateVar::Bool(true)));
        assert_eq!(state1.get("var2"), Some(&StateVar::I64(20)));
        assert_eq!(
            state1.get("var3"),
            Some(&StateVar::Enum("test".to_string()))
        );
    }

    #[test]
    fn test_world_state_hash() {
        let mut state1 = WorldState::new();
        state1.set("a", StateVar::Bool(true));
        state1.set("b", StateVar::I64(10));

        let mut state2 = WorldState::new();
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
        let mut state = WorldState::new();

        // Test setting and getting F64 values
        state.set("speed", StateVar::F64(1500)); // 1.5
        state.set("height", StateVar::F64(2000)); // 2.0

        assert_eq!(state.get("speed"), Some(&StateVar::F64(1500)));
        assert_eq!(state.get("height"), Some(&StateVar::F64(2000)));

        // Test satisfies with F64 values
        let mut conditions;

        println!("\n=== Testing exact match ===");
        // Test exact match
        conditions = WorldState::new(); // Clear conditions
        conditions.set("speed", StateVar::F64(1500));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing greater than satisfies ===");
        // Test greater than satisfies
        conditions = WorldState::new(); // Clear conditions
        conditions.set("speed", StateVar::F64(1000)); // 1.0
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing less than fails ===");
        // Test less than fails
        conditions = WorldState::new(); // Clear conditions
        conditions.set("speed", StateVar::F64(2000)); // 2.0
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(!state.satisfies(&conditions));

        println!("\n=== Testing small decimal differences ===");
        // Test small decimal differences
        state.set("precise", StateVar::F64(1001)); // 1.001
        conditions = WorldState::new(); // Clear conditions
        conditions.set("precise", StateVar::F64(1000)); // 1.000
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        conditions = WorldState::new(); // Clear conditions
        conditions.set("precise", StateVar::F64(1002)); // 1.002
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(!state.satisfies(&conditions));
    }

    #[test]
    fn test_world_state_f64_hash_consistency() {
        let mut state1 = WorldState::new();
        state1.set("a", StateVar::F64(1500)); // 1.5
        state1.set("b", StateVar::F64(2500)); // 2.5

        let mut state2 = WorldState::new();
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
        let mut state = WorldState::new();

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
        let mut state = WorldState::new();

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
        let mut state = WorldState::new();

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
        state = WorldState::new(); // Clear state
        conditions = WorldState::new(); // Clear conditions
        state.set("zero", StateVar::F64(0));
        conditions.set("zero", StateVar::F64(0));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing positive satisfies zero ===");
        // Test positive satisfies zero
        state = WorldState::new(); // Clear state
        conditions = WorldState::new(); // Clear conditions
        state.set("pos", StateVar::F64(1)); // 0.001
        conditions.set("pos", StateVar::F64(0));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing negative doesn't satisfy zero ===");
        // Test negative doesn't satisfy zero
        state = WorldState::new(); // Clear state
        conditions = WorldState::new(); // Clear conditions
        state.set("neg", StateVar::F64(-1)); // -0.001
        conditions.set("neg", StateVar::F64(0));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(!state.satisfies(&conditions));

        println!("\n=== Testing exact decimal matches ===");
        // Test exact decimal matches
        state = WorldState::new(); // Clear state
        conditions = WorldState::new(); // Clear conditions
        state.set("exact", StateVar::F64(1234)); // 1.234
        conditions.set("exact", StateVar::F64(1234));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing very close values ===");
        // Test very close values
        state = WorldState::new(); // Clear state
        conditions = WorldState::new(); // Clear conditions
        state.set("close", StateVar::F64(1000)); // 1.000
        conditions.set("close", StateVar::F64(999)); // 0.999
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing boundary conditions ===");
        // Test boundary conditions
        state = WorldState::new(); // Clear state
        conditions = WorldState::new(); // Clear conditions
        state.set("boundary", StateVar::F64(1000)); // 1.000
        conditions.set("boundary", StateVar::F64(1001)); // 1.001
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
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
        let mut state = WorldState::new();

        // Test setting and getting F64 values using floating point
        state.set("speed", StateVar::from_f64(1.5));
        state.set("height", StateVar::from_f64(2.0));

        assert_eq!(state.get("speed"), Some(&StateVar::F64(1500)));
        assert_eq!(state.get("height"), Some(&StateVar::F64(2000)));

        // Test satisfies with F64 values
        let mut conditions;

        println!("\n=== Testing exact match ===");
        conditions = WorldState::new();
        conditions.set("speed", StateVar::from_f64(1.5));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing greater than satisfies ===");
        conditions = WorldState::new();
        conditions.set("speed", StateVar::from_f64(1.0));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        println!("\n=== Testing less than fails ===");
        conditions = WorldState::new();
        conditions.set("speed", StateVar::from_f64(2.0));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(!state.satisfies(&conditions));

        println!("\n=== Testing small decimal differences ===");
        state.set("precise", StateVar::from_f64(1.001));
        conditions = WorldState::new();
        conditions.set("precise", StateVar::from_f64(1.0));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(state.satisfies(&conditions));

        conditions = WorldState::new();
        conditions.set("precise", StateVar::from_f64(1.002));
        println!("State: {:?}", state);
        println!("Conditions: {:?}", conditions);
        assert!(!state.satisfies(&conditions));
    }

    #[test]
    fn test_world_state_f64_arithmetic_with_helpers() {
        let mut state = WorldState::new();

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
        changes.insert("value".to_string(), StateOperation::set_f64(3.14159)); // Should round to 3.142
        state.apply(&changes);
        assert_eq!(state.get("value").and_then(|v| v.as_f64()), Some(3.142));

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
        assert_eq!(string_var, StateVar::Enum("test".to_string()));

        // Test str conversion
        let str_var: StateVar = "test".into();
        assert_eq!(str_var, StateVar::Enum("test".to_string()));

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
        let state = WorldState::builder()
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
            Some(&StateVar::Enum("forest".to_string()))
        );
    }

    #[test]
    fn test_world_state_builder_chaining() {
        // Test that we can chain multiple values of the same type
        let state = WorldState::builder()
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
            Some(&StateVar::Enum("forest".to_string()))
        );
        assert_eq!(
            state.get("weather"),
            Some(&StateVar::Enum("sunny".to_string()))
        );
    }
}
