use crate::conditions::Conditions;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::Hash;

/// Errors that can occur when working with state variables.
#[derive(Debug, PartialEq)]
pub enum StateError {
    /// The state variable exists but is not of the requested type.
    InvalidVarType {
        /// Name of the variable.
        variable: String,
        /// The Rust type that was asked for (`"i32"`, `"bool"`, an enum's
        /// type name, ...).
        expected: &'static str,
        /// The type the variable actually has.
        found: StateType,
    },
    /// A float cannot be stored as a fixed-point `F64`: it is NaN, infinite,
    /// or outside [`StateVar::MIN_F64`]`..=`[`StateVar::MAX_F64`].
    UnrepresentableFloat(f64),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::InvalidVarType {
                variable,
                expected,
                found,
            } => write!(
                f,
                "State variable '{variable}' is not of type {expected} (it is {found})"
            ),
            StateError::UnrepresentableFloat(value) if value.is_nan() => {
                write!(f, "cannot store NaN as a state value")
            }
            StateError::UnrepresentableFloat(value) => write!(
                f,
                "cannot store {value:e} as a state value: it is outside the representable range \
                 {} to {}",
                StateVar::MIN_F64,
                StateVar::MAX_F64
            ),
        }
    }
}

impl Error for StateError {}

/// Represents the state of the world as a collection of named variables.
/// Each variable has a name (string key) and a typed value (StateVar).
/// States are used to represent the current world state, goal states, and action preconditions.
///
/// The variables are only reachable through the typed API ([`State::set`],
/// [`State::get`], [`State::get_var`], [`State::iter`], ...), so a state the
/// planner has validated cannot be edited behind its back:
///
/// ```compile_fail
/// use goap::prelude::*;
/// let mut state = State::builder().set("x", 5).build();
/// state.vars.insert("x".to_string(), StateVar::Bool(true)); // error: private field
/// ```
///
/// `State` also implements the standard collection traits: iterate it with
/// a `for` loop, merge another state into it with [`Extend::extend`], or
/// build one from `(name, value)` pairs with [`FromIterator`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct State {
    /// The variables that make up this state, indexed by name
    vars: HashMap<String, StateVar>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "empty state")?;
        } else {
            writeln!(f, "State:")?;
            let mut entries: Vec<_> = self.vars.iter().collect();
            entries.sort_by_key(|(key, _)| *key);
            for (key, value) in entries {
                writeln!(f, "  - {key}: {value}")?;
            }
        }
        Ok(())
    }
}

// Manual Hash implementation. HashMap iteration order is arbitrary, so each
// (key, value) pair is hashed independently and the results are combined with
// a commutative operation. This avoids allocating and sorting the keys on
// every hash, which matters because the planner hashes every state it visits.
impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hasher;
        let mut combined: u64 = 0;
        for (key, value) in &self.vars {
            let mut h = std::hash::DefaultHasher::new();
            key.hash(&mut h);
            value.hash(&mut h);
            combined = combined.wrapping_add(h.finish());
        }
        state.write_usize(self.vars.len());
        state.write_u64(combined);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Creates an empty state with no variables.
    pub fn new() -> Self {
        State {
            vars: HashMap::new(),
        }
    }

    /// Creates a [`StateBuilder`] for constructing a state with the fluent
    /// interface:
    ///
    /// ```
    /// use goap::prelude::*;
    /// let state = State::builder().set("gold", 5).set("armed", true).build();
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn builder() -> StateBuilder {
        StateBuilder::new()
    }

    /// Sets `key` to `value`, replacing any previous value of any type.
    /// Accepts every type that implements [`IntoStateVar`].
    ///
    /// # Panics
    ///
    /// Panics if `value` is a float that is NaN, infinite or outside
    /// [`StateVar::MIN_F64`]`..=`[`StateVar::MAX_F64`] (see
    /// [`StateVar::from_f64`]). For floats you do not control, convert first:
    ///
    /// ```
    /// use goap::prelude::*;
    /// # fn run(reading: f64) -> Result<(), StateError> {
    /// let mut state = State::new();
    /// state.set("temperature", StateVar::try_from_f64(reading)?);
    /// # Ok(()) }
    /// # assert!(run(21.5).is_ok());
    /// # assert!(run(f64::NAN).is_err());
    /// ```
    pub fn set<T: IntoStateVar>(&mut self, key: &str, value: T) {
        self.vars.insert(key.to_string(), value.into_state_var());
    }

    /// Typed read: `None` if the variable is missing or has another type.
    /// Usage: `let x: Option<i32> = state.get("x");`
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: TryFromStateVar,
    {
        self.get_var(key)
            .and_then(|var| T::try_from_state_var(var, key).ok())
    }

    /// Untyped read: the stored [`StateVar`], or `None` if the variable is
    /// missing.
    pub fn get_var(&self, key: &str) -> Option<&StateVar> {
        self.vars.get(key)
    }

    /// True if the state has a variable named `key`, of any type.
    pub fn contains(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    /// Removes and returns the variable named `key`, if any.
    pub fn remove(&mut self, key: &str) -> Option<StateVar> {
        self.vars.remove(key)
    }

    /// Number of variables.
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// True if the state has no variables.
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// Iterates over `(variable name, value)` pairs in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &StateVar)> {
        self.vars.iter()
    }

    /// True if this state satisfies every condition in `conditions`.
    /// See [`Conditions::is_met_by`].
    pub fn meets(&self, conditions: &Conditions) -> bool {
        conditions.is_met_by(self)
    }

    /// Applies a set of state operations to this state, modifying it in place.
    ///
    /// - `Set` replaces the variable (creating it if needed), whatever its type.
    /// - `Add` / `Subtract` need a numeric variable of the **same type as the
    ///   delta**: an `I64` delta for an `i64` variable, an `F64` delta for an
    ///   `f64` variable. A missing variable counts as zero and is created with
    ///   the delta's type. A variable of a different type, or a non-numeric
    ///   delta, leaves the state unchanged. Arithmetic saturates at the `i64`
    ///   bounds instead of overflowing.
    ///
    /// `Planner::plan` rejects mismatched effect types before searching, so the
    /// "left unchanged" case only arises when `apply` is called directly.
    pub fn apply(&mut self, changes: &HashMap<String, StateOperation>) {
        for (key, operation) in changes {
            match operation {
                StateOperation::Set(value) => {
                    self.vars.insert(key.clone(), value.clone());
                }
                StateOperation::Add(delta) | StateOperation::Subtract(delta) => {
                    let subtract = matches!(operation, StateOperation::Subtract(_));
                    let next = match (self.vars.get(key), delta) {
                        (None, StateVar::I64(d)) => Some(StateVar::I64(shift(0, *d, subtract))),
                        (Some(StateVar::I64(c)), StateVar::I64(d)) => {
                            Some(StateVar::I64(shift(*c, *d, subtract)))
                        }
                        (None, StateVar::F64(d)) => Some(StateVar::F64(shift(0, *d, subtract))),
                        (Some(StateVar::F64(c)), StateVar::F64(d)) => {
                            Some(StateVar::F64(shift(*c, *d, subtract)))
                        }
                        // Type mismatch or non-numeric delta: no effect.
                        _ => None,
                    };
                    if let Some(value) = next {
                        self.vars.insert(key.clone(), value);
                    }
                }
            }
        }
    }
}

impl IntoIterator for State {
    type Item = (String, StateVar);
    type IntoIter = std::collections::hash_map::IntoIter<String, StateVar>;

    fn into_iter(self) -> Self::IntoIter {
        self.vars.into_iter()
    }
}

impl<'a> IntoIterator for &'a State {
    type Item = (&'a String, &'a StateVar);
    type IntoIter = std::collections::hash_map::Iter<'a, String, StateVar>;

    fn into_iter(self) -> Self::IntoIter {
        self.vars.iter()
    }
}

/// Inserts every pair, replacing existing variables of any type (the same
/// rule as [`State::set`]). Merging two states is `a.extend(b)`.
impl<K: Into<String>, V: IntoStateVar> Extend<(K, V)> for State {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.vars.insert(key.into(), value.into_state_var());
        }
    }
}

/// Builds a state from `(name, value)` pairs; a later pair with the same
/// name replaces an earlier one.
///
/// ```
/// use goap::prelude::*;
/// let state: State = [("gold", 5), ("wood", 2)].into_iter().collect();
/// assert_eq!(state.get::<i64>("wood"), Some(2));
/// ```
impl<K: Into<String>, V: IntoStateVar> FromIterator<(K, V)> for State {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut state = State::new();
        state.extend(iter);
        state
    }
}

/// `current ± delta` in raw units, saturating at the `i64` bounds.
fn shift(current: i64, delta: i64, subtract: bool) -> i64 {
    if subtract {
        current.saturating_sub(delta)
    } else {
        current.saturating_add(delta)
    }
}

/// Builder for constructing states with a fluent interface.
/// Use [`State::builder`] to create one.
#[derive(Clone, Debug)]
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

/// The type of a [`StateVar`], without its value. Used by the planner's
/// type validation and in error messages; `Display` prints `bool`, `i64`,
/// `f64` or `string`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StateType {
    /// `StateVar::Bool`
    Bool,
    /// `StateVar::I64`
    I64,
    /// `StateVar::F64`
    F64,
    /// `StateVar::String`
    String,
}

impl fmt::Display for StateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StateType::Bool => "bool",
            StateType::I64 => "i64",
            StateType::F64 => "f64",
            StateType::String => "string",
        })
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

/// Thousandths of `value` as an `i64`, or `None` if the result does not fit:
/// NaN, infinite, or `|value|` beyond about 9.2e15. Shared by `F64` state
/// values and action costs so both round the same way.
pub(crate) fn fixed_point_from_f64(value: f64) -> Option<i64> {
    // NaN fails both comparisons. Within the bounds, `value * 1000.0` stays
    // strictly inside the i64 range even after float rounding, so the cast
    // never saturates.
    if (StateVar::MIN_F64..=StateVar::MAX_F64).contains(&value) {
        Some((value * 1000.0).round() as i64)
    } else {
        None
    }
}

impl StateVar {
    /// The largest value an `F64` can hold: the `i64` range in thousandths,
    /// less one float ulp so that the scaled value is always in range.
    pub const MAX_F64: f64 = 9_223_372_036_854_774.0;
    /// The smallest value an `F64` can hold.
    pub const MIN_F64: f64 = -9_223_372_036_854_774.0;

    /// Creates an `F64` from a float, rounded to the nearest thousandth.
    ///
    /// # Panics
    ///
    /// Panics if `value` is NaN, infinite or outside
    /// [`MIN_F64`](Self::MIN_F64)`..=`[`MAX_F64`](Self::MAX_F64). Such a value
    /// has no honest fixed-point representation, and storing something else
    /// in its place would make the planner reason about a world that does not
    /// exist. Use [`StateVar::try_from_f64`] when the input is not under your
    /// control. (This mirrors `std::time::Duration::from_secs_f64` and
    /// `try_from_secs_f64`.)
    pub fn from_f64(value: f64) -> Self {
        Self::try_from_f64(value)
            .unwrap_or_else(|e| panic!("StateVar::from_f64: {e}; use StateVar::try_from_f64"))
    }

    /// Creates an `F64` from a float, rounded to the nearest thousandth, or
    /// returns [`StateError::UnrepresentableFloat`] if `value` is NaN,
    /// infinite or outside [`MIN_F64`](Self::MIN_F64)`..=`[`MAX_F64`](Self::MAX_F64).
    ///
    /// ```
    /// use goap::prelude::*;
    /// assert_eq!(StateVar::try_from_f64(1.5), Ok(StateVar::F64(1500)));
    /// assert_eq!(StateVar::try_from_f64(0.0004), Ok(StateVar::F64(0)));
    /// assert!(StateVar::try_from_f64(f64::NAN).is_err());
    /// assert!(StateVar::try_from_f64(f64::INFINITY).is_err());
    /// assert!(StateVar::try_from_f64(1e16).is_err());
    /// ```
    pub fn try_from_f64(value: f64) -> Result<Self, StateError> {
        fixed_point_from_f64(value)
            .map(StateVar::F64)
            .ok_or(StateError::UnrepresentableFloat(value))
    }

    /// Converts an F64 StateVar back to a floating point value.
    /// Returns None if the StateVar is not an F64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            StateVar::F64(value) => Some(*value as f64 / 1000.0),
            _ => None,
        }
    }

    /// The raw integer payload of a numeric variable: the value itself for
    /// `I64`, the fixed-point units (thousandths) for `F64`. `None` for
    /// `Bool` and `String`.
    pub fn raw_numeric(&self) -> Option<i64> {
        match self {
            StateVar::I64(n) | StateVar::F64(n) => Some(*n),
            _ => None,
        }
    }

    /// Zero of the same numeric type as `self` (`I64(0)` or `F64(0)`), or
    /// `None` for non-numeric variables.
    pub fn numeric_zero(&self) -> Option<StateVar> {
        match self {
            StateVar::I64(_) => Some(StateVar::I64(0)),
            StateVar::F64(_) => Some(StateVar::F64(0)),
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

    /// The type of this variable.
    pub fn state_type(&self) -> StateType {
        match self {
            StateVar::Bool(_) => StateType::Bool,
            StateVar::I64(_) => StateType::I64,
            StateVar::F64(_) => StateType::F64,
            StateVar::String(_) => StateType::String,
        }
    }

    /// True for `I64` and `F64`: the types that ordered comparisons and
    /// `Add`/`Subtract` effects apply to.
    pub fn is_numeric(&self) -> bool {
        matches!(self, StateVar::I64(_) | StateVar::F64(_))
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

/// Panics on NaN, infinite or out-of-range input; see [`StateVar::from_f64`].
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
            variable: key.to_string(),
            expected: "i32",
            found: var.state_type(),
        })
    }
}

impl TryFromStateVar for i64 {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_i64().ok_or_else(|| StateError::InvalidVarType {
            variable: key.to_string(),
            expected: "i64",
            found: var.state_type(),
        })
    }
}

impl TryFromStateVar for bool {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_bool().ok_or_else(|| StateError::InvalidVarType {
            variable: key.to_string(),
            expected: "bool",
            found: var.state_type(),
        })
    }
}

impl TryFromStateVar for f64 {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_f64().ok_or_else(|| StateError::InvalidVarType {
            variable: key.to_string(),
            expected: "f64",
            found: var.state_type(),
        })
    }
}

impl TryFromStateVar for String {
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| StateError::InvalidVarType {
                variable: key.to_string(),
                expected: "string",
                found: var.state_type(),
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

/// Panics on NaN, infinite or out-of-range input; see [`StateVar::from_f64`].
impl IntoStateVar for f64 {
    fn into_state_var(self) -> StateVar {
        StateVar::from_f64(self)
    }
}

/// Panics on NaN, infinite or out-of-range input; see [`StateVar::from_f64`].
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

/// Marker trait for enum types stored as strings in the state.
///
/// Implement `Display` (used to store the value), `FromStr` (used to read it
/// back with [`State::get`]) and this empty trait:
///
/// ```
/// use goap::prelude::*;
/// use std::{fmt, str::FromStr};
///
/// #[derive(Debug, PartialEq)]
/// enum Location { Town, Forest }
///
/// impl fmt::Display for Location {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         f.write_str(match self { Location::Town => "town", Location::Forest => "forest" })
///     }
/// }
///
/// impl FromStr for Location {
///     type Err = ();
///     fn from_str(s: &str) -> Result<Self, ()> {
///         match s { "town" => Ok(Location::Town), "forest" => Ok(Location::Forest), _ => Err(()) }
///     }
/// }
///
/// impl EnumStateVar for Location {}
///
/// let mut state = State::new();
/// state.set("location", Location::Forest);
/// assert_eq!(state.get::<Location>("location"), Some(Location::Forest));
/// assert_eq!(state.get::<String>("location"), Some("forest".to_string()));
/// ```
pub trait EnumStateVar: fmt::Display + std::str::FromStr {}

// Blanket implementations for any type that implements the marker trait.
// They coexist with the concrete impls for `String` because `EnumStateVar`
// is this crate's trait: no other crate can implement it for `String`.
impl<T> IntoStateVar for T
where
    T: EnumStateVar,
{
    fn into_state_var(self) -> StateVar {
        StateVar::String(self.to_string())
    }
}

impl<T> TryFromStateVar for T
where
    T: EnumStateVar,
{
    fn try_from_state_var(var: &StateVar, key: &str) -> Result<Self, StateError> {
        var.as_string()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StateError::InvalidVarType {
                variable: key.to_string(),
                expected: std::any::type_name::<T>(),
                found: var.state_type(),
            })
    }
}

/// Marker for value types that produce a numeric `StateVar` (`I64` or
/// `F64`). The ordered comparison helpers (`at_least`, `between`, ...) and the
/// `adds`/`subtracts` effects are bounded by this trait, so `at_least(true)`
/// and `adds("x", "a")` are compile-time errors.
///
/// `StateVar` itself implements it so that a value converted with
/// [`StateVar::try_from_f64`] can be used where a NaN would otherwise panic:
///
/// ```
/// use goap::prelude::*;
/// # fn build(measured: f64) -> Result<Action, StateError> {
/// let delta = StateVar::try_from_f64(measured)?;
/// Ok(Action::builder("adjust")
///     .requires("temperature", at_most(delta.clone()))
///     .adds("temperature", delta)
///     .build())
/// # }
/// # assert!(build(1.5).is_ok());
/// # assert!(build(f64::NAN).is_err());
/// ```
///
/// A non-numeric `StateVar` passed this way is caught by `Planner::plan`
/// (`InvalidCondition` / `InvalidEffect`), not at compile time.
pub trait NumericStateVar: IntoStateVar {}

impl NumericStateVar for i8 {}
impl NumericStateVar for i16 {}
impl NumericStateVar for i32 {}
impl NumericStateVar for i64 {}
impl NumericStateVar for f32 {}
impl NumericStateVar for f64 {}
impl NumericStateVar for StateVar {}

/// Operations that can be performed on state variables.
/// These operations are used in action effects to modify the world state.
///
/// `Add` and `Subtract` carry a typed delta: an `I64` delta applies to `i64`
/// variables and an `F64` delta to `f64` variables. Build them with
/// [`StateOperation::add`] / [`StateOperation::subtract`] (or
/// `Action::builder(..).adds(..)`), which only accept numeric values.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StateOperation {
    /// Set a variable to a specific value
    Set(StateVar),
    /// Add a numeric delta to a variable of the same type. See [`State::apply`].
    Add(StateVar),
    /// Subtract a numeric delta from a variable of the same type. See [`State::apply`].
    Subtract(StateVar),
}

impl StateOperation {
    /// Creates a `Set` operation. Accepts any value that converts to a `StateVar`.
    pub fn set<T: IntoStateVar>(value: T) -> Self {
        StateOperation::Set(value.into_state_var())
    }

    /// Creates an `Add` operation. Floats are stored as fixed point with 3
    /// decimal places, so `add(0.5)` and `add(0.5004)` are the same operation.
    pub fn add<T: NumericStateVar>(value: T) -> Self {
        StateOperation::Add(value.into_state_var())
    }

    /// Creates a `Subtract` operation. Same conversion rules as [`StateOperation::add`].
    pub fn subtract<T: NumericStateVar>(value: T) -> Self {
        StateOperation::Subtract(value.into_state_var())
    }

    /// The value or delta this operation carries.
    pub fn value(&self) -> &StateVar {
        match self {
            StateOperation::Set(v) | StateOperation::Add(v) | StateOperation::Subtract(v) => v,
        }
    }

    /// The type of the value this operation carries.
    pub fn state_type(&self) -> StateType {
        self.value().state_type()
    }

    /// The delta of an `Add` or `Subtract` as a signed change in raw units
    /// (`Subtract(5)` is `-5`), or `None` for `Set` and for a non-numeric
    /// payload. `Subtract(i64::MIN)` saturates to `i64::MAX`.
    pub fn signed_delta(&self) -> Option<i64> {
        match self {
            StateOperation::Set(_) => None,
            StateOperation::Add(v) => v.raw_numeric(),
            StateOperation::Subtract(v) => v.raw_numeric().map(i64::saturating_neg),
        }
    }
}

impl fmt::Display for StateOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateOperation::Set(v) => write!(f, "set to {v}"),
            StateOperation::Add(v) => write!(f, "add {v}"),
            StateOperation::Subtract(v) => write!(f, "subtract {v}"),
        }
    }
}
