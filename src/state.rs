use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq)]
pub enum StateError {
    VarNotFound(String),
    InvalidVarType { var: String, expected: &'static str },
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::VarNotFound(var) => write!(f, "State variable '{var}' not found"),
            StateError::InvalidVarType { var, expected } => {
                write!(f, "State variable '{var}' is not of type {expected}")
            }
        }
    }
}

impl Error for StateError {}

// State representation as a collection of named variables
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct State {
    pub vars: HashMap<String, StateVar>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "empty state")?;
        } else {
            writeln!(f, "State:")?;
            for (key, value) in &self.vars {
                writeln!(f, "  - {key}: {value}")?;
            }
        }
        Ok(())
    }
}

// Manual Hash implementation for WorldState
impl Hash for State {
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

impl Default for State {
    fn default() -> Self {
        Self::empty()
    }
}

impl State {
    pub fn new() -> StateBuilder {
        StateBuilder::new()
    }

    pub fn empty() -> Self {
        State {
            vars: HashMap::new(),
        }
    }

    /// Generic set method that accepts any type implementing IntoStateVar
    pub fn set<T: IntoStateVar>(&mut self, key: &str, value: T) {
        self.vars.insert(key.to_string(), value.into_state_var());
    }

    /// Primary get method with type inference - returns None if key doesn't exist or type doesn't match
    /// Usage: let x: Option<i32> = state.get("x");
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: TryFromStateVar,
    {
        self.get_raw(key)
            .and_then(|var| T::try_from_state_var(var, key).ok())
    }

    /// Internal method to get raw StateVar - not exposed to library users
    fn get_raw(&self, key: &str) -> Option<&StateVar> {
        self.vars.get(key)
    }

    pub fn satisfies(&self, conditions: &State) -> bool {
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
                        (StateVar::String(cur), StateVar::String(req)) => {
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

    pub fn merge(&mut self, other: &State) {
        for (key, value) in &other.vars {
            self.vars.insert(key.clone(), value.clone());
        }
    }
}

pub struct StateBuilder {
    vars: HashMap<String, StateVar>,
}

impl StateBuilder {
    pub fn new() -> Self {
        StateBuilder {
            vars: HashMap::new(),
        }
    }

    /// Unified method to set any value type
    pub fn set<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.vars.insert(key.to_string(), value.into_state_var());
        self
    }

    pub fn build(self) -> State {
        State { vars: self.vars }
    }
}

impl Default for StateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// # let mut state = State::empty();
    /// # state.set("value", StateVar::from_f64(1.5));
    /// let mut changes = HashMap::new();
    /// changes.insert("value".to_string(), StateOperation::add_f64(0.5)); // Add 0.5
    /// state.apply(&changes);
    /// assert_eq!(state.get::<f64>("value"), Some(2.0));
    /// ```
    F64(i64),
    String(String),
}

impl fmt::Display for StateVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateVar::Bool(b) => write!(f, "{b}"),
            StateVar::I64(i) => write!(f, "{i}"),
            StateVar::F64(fp) => write!(f, "{:.3}", *fp as f64 / 1000.0),
            StateVar::String(s) => write!(f, "{s}"),
        }
    }
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

    /// Extracts the value as an i64.
    /// Returns None if the StateVar is not an I64.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let value = StateVar::I64(42);
    /// assert_eq!(value.as_i64(), Some(42));
    ///
    /// // Returns None for other types
    /// assert_eq!(StateVar::Bool(true).as_i64(), None);
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            StateVar::I64(value) => Some(*value),
            _ => None,
        }
    }

    /// Extracts the value as an i32.
    /// Returns None if the StateVar is not an I64 or if the value doesn't fit in an i32.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let value = StateVar::I64(42);
    /// assert_eq!(value.as_i32(), Some(42));
    ///
    /// // Returns None for other types
    /// assert_eq!(StateVar::Bool(true).as_i32(), None);
    /// ```
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            StateVar::I64(value) => (*value).try_into().ok(),
            _ => None,
        }
    }

    /// Extracts the value as a bool.
    /// Returns None if the StateVar is not a Bool.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let value = StateVar::Bool(true);
    /// assert_eq!(value.as_bool(), Some(true));
    ///
    /// // Returns None for other types
    /// assert_eq!(StateVar::I64(42).as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            StateVar::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Extracts the value as a String reference.
    /// Returns None if the StateVar is not a String.
    ///
    /// # Examples
    /// ```rust
    /// # use goap::prelude::*;
    /// let value = StateVar::String("hello".to_string());
    /// assert_eq!(value.as_string(), Some("hello"));
    ///
    /// // Returns None for other types
    /// assert_eq!(StateVar::Bool(true).as_string(), None);
    /// ```
    pub fn as_string(&self) -> Option<&str> {
        match self {
            StateVar::String(value) => Some(value),
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
            (StateVar::String(a), StateVar::String(b)) => {
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
        StateVar::String(value)
    }
}

impl From<&str> for StateVar {
    fn from(value: &str) -> Self {
        StateVar::String(value.to_string())
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

// Trait for types that can be converted to StateVar, including proper enum support
pub trait IntoStateVar {
    fn into_state_var(self) -> StateVar;
}

// Trait for types that can be extracted from StateVar with proper error handling
pub trait TryFromStateVar: Sized {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError>;
}

// Implementations for common types
impl TryFromStateVar for i32 {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_i32().ok_or_else(|| StateError::InvalidVarType {
            var: key.to_string(),
            expected: "i32",
        })
    }
}

impl TryFromStateVar for i64 {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_i64().ok_or_else(|| StateError::InvalidVarType {
            var: key.to_string(),
            expected: "i64",
        })
    }
}

impl TryFromStateVar for bool {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_bool().ok_or_else(|| StateError::InvalidVarType {
            var: key.to_string(),
            expected: "bool",
        })
    }
}

impl TryFromStateVar for f64 {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_f64().ok_or_else(|| StateError::InvalidVarType {
            var: key.to_string(),
            expected: "f64",
        })
    }
}

impl TryFromStateVar for String {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| StateError::InvalidVarType {
                var: key.to_string(),
                expected: "string",
            })
    }
}

// Implement for all existing types
impl IntoStateVar for bool {
    fn into_state_var(self) -> StateVar {
        StateVar::Bool(self)
    }
}

impl IntoStateVar for i64 {
    fn into_state_var(self) -> StateVar {
        StateVar::I64(self)
    }
}

impl IntoStateVar for i32 {
    fn into_state_var(self) -> StateVar {
        StateVar::I64(self as i64)
    }
}

impl IntoStateVar for i16 {
    fn into_state_var(self) -> StateVar {
        StateVar::I64(self as i64)
    }
}

impl IntoStateVar for i8 {
    fn into_state_var(self) -> StateVar {
        StateVar::I64(self as i64)
    }
}

impl IntoStateVar for f64 {
    fn into_state_var(self) -> StateVar {
        StateVar::from_f64(self)
    }
}

impl IntoStateVar for f32 {
    fn into_state_var(self) -> StateVar {
        StateVar::from_f64(self as f64)
    }
}

impl IntoStateVar for String {
    fn into_state_var(self) -> StateVar {
        StateVar::String(self)
    }
}

impl IntoStateVar for &str {
    fn into_state_var(self) -> StateVar {
        StateVar::String(self.to_string())
    }
}

impl IntoStateVar for StateVar {
    fn into_state_var(self) -> StateVar {
        self
    }
}

// Helper trait to identify types that should be treated as enums
pub trait EnumStateVar: fmt::Display {}

// Blanket implementation for any enum that implements Display and our marker trait
impl<T> IntoStateVar for T
where
    T: EnumStateVar,
{
    fn into_state_var(self) -> StateVar {
        StateVar::String(self.to_string())
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
