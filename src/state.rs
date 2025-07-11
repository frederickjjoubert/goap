use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::Hash;

/// Errors that can occur when working with state variables.
#[derive(Debug, PartialEq, Eq)]
pub enum StateError {
    /// The requested state variable was not found
    VarNotFound(String),
    /// The state variable exists but is not of the expected type
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

/// Represents the state of the world as a collection of named variables.
/// Each variable has a name (string key) and a typed value (StateVar).
/// States are used to represent the current world state, goal states, and action preconditions.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct State {
    /// The variables that make up this state, indexed by name
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
    /// Creates a new StateBuilder for constructing a state with the fluent interface.
    pub fn new() -> StateBuilder {
        StateBuilder::new()
    }

    /// Creates an empty state with no variables.
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

    /// Checks if this state satisfies all the conditions in the given state.
    /// For boolean and string variables, values must match exactly.
    /// For numeric variables, this state's value must be >= the required value.
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

    /// Applies a set of state operations to this state, modifying it in place.
    /// Operations can set variables to new values, add to numeric variables, or subtract from them.
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

    /// Merges another state into this one, overwriting any existing variables with the same name.
    pub fn merge(&mut self, other: &State) {
        for (key, value) in &other.vars {
            self.vars.insert(key.clone(), value.clone());
        }
    }
}

/// Builder for constructing states with a fluent interface.
/// Use `State::new()` to create a new builder.
pub struct StateBuilder {
    /// The variables being built
    vars: HashMap<String, StateVar>,
}

impl StateBuilder {
    /// Creates a new empty StateBuilder.
    pub fn new() -> Self {
        StateBuilder {
            vars: HashMap::new(),
        }
    }

    /// Sets a variable to the given value. This method accepts any type that can be converted to a StateVar.
    pub fn set<T: IntoStateVar>(mut self, key: &str, value: T) -> Self {
        self.vars.insert(key.to_string(), value.into_state_var());
        self
    }

    /// Builds the final State from the configured builder.
    pub fn build(self) -> State {
        State { vars: self.vars }
    }
}

impl Default for StateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Variable types that can be stored in the world state.
/// Each variant represents a different data type that can be used in state variables.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StateVar {
    /// Boolean value (true/false)
    Bool(bool),
    /// 64-bit signed integer
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
    /// Note that arithmetic operations (Add/Subtract) can be performed using convenience methods.
    F64(i64),
    /// String/text value for names, locations, enum values, etc.
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
    pub fn from_f64(value: f64) -> Self {
        // Convert to fixed point with 3 decimal places
        StateVar::F64((value * 1000.0).round() as i64)
    }

    /// Converts an F64 StateVar back to a floating point value.
    /// Returns None if the StateVar is not an F64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            StateVar::F64(value) => Some(*value as f64 / 1000.0),
            _ => None,
        }
    }

    /// Extracts the value as an i64.
    /// Returns None if the StateVar is not an I64.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            StateVar::I64(value) => Some(*value),
            _ => None,
        }
    }

    /// Extracts the value as an i32.
    /// Returns None if the StateVar is not an I64 or if the value doesn't fit in an i32.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            StateVar::I64(value) => (*value).try_into().ok(),
            _ => None,
        }
    }

    /// Extracts the value as a bool.
    /// Returns None if the StateVar is not a Bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            StateVar::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Extracts the value as a String reference.
    /// Returns None if the StateVar is not a String.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            StateVar::String(value) => Some(value),
            _ => None,
        }
    }

    /// Calculates the distance between two StateVar values.
    /// This is used by the planner's heuristic function to estimate cost.
    /// For booleans and strings, distance is 0 if equal, 1 if different.
    /// For numbers, distance is the absolute difference.
    /// Panics if the StateVar types don't match.
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

/// Trait for types that can be converted to StateVar.
/// This trait is implemented for all common types (bool, integers, floats, strings)
/// and can be implemented for custom enum types.
pub trait IntoStateVar {
    /// Converts this value into a StateVar.
    fn into_state_var(self) -> StateVar;
}

/// Trait for types that can be extracted from StateVar with proper error handling.
/// This trait is implemented for all common types and provides type-safe extraction.
pub trait TryFromStateVar: Sized {
    /// Attempts to extract a value of this type from a StateVar.
    /// Returns an error if the StateVar is not of the expected type.
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

/// Marker trait for enum types that should be stored as strings in the state.
/// Implement this trait on your enum types to enable them to be used as state variables.
/// Your enum must also implement `Display` to convert to string representation.
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

/// Operations that can be performed on state variables.
/// These operations are used in action effects to modify the world state.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StateOperation {
    /// Set a variable to a specific value
    Set(StateVar),
    /// Add a value to a numeric variable (for integers and fixed-point floats)
    Add(i64),
    /// Subtract a value from a numeric variable (for integers and fixed-point floats)
    Subtract(i64),
}

impl StateOperation {
    /// Creates a Set operation that will set the value to the given i64 value.
    pub fn set_i64(value: i64) -> Self {
        StateOperation::Set(StateVar::I64(value))
    }

    /// Creates an Add operation that will add the given i64 value.
    pub fn add_i64(value: i64) -> Self {
        StateOperation::Add(value)
    }

    /// Creates a Subtract operation that will subtract the given i64 value.
    pub fn subtract_i64(value: i64) -> Self {
        StateOperation::Subtract(value)
    }

    /// Creates a Set operation that will set the value to the given f64 value.
    /// The value will be converted to fixed point with 3 decimal places.
    pub fn set_f64(value: f64) -> Self {
        StateOperation::Set(StateVar::from_f64(value))
    }

    /// Creates an Add operation that will add the given f64 value.
    /// The value will be converted to fixed point with 3 decimal places.
    pub fn add_f64(value: f64) -> Self {
        StateOperation::Add((value * 1000.0).round() as i64)
    }

    /// Creates a Subtract operation that will subtract the given f64 value.
    /// The value will be converted to fixed point with 3 decimal places.
    pub fn subtract_f64(value: f64) -> Self {
        StateOperation::Subtract((value * 1000.0).round() as i64)
    }
}
