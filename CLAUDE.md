# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Goal Oriented Action Planning (GOAP) library written in Rust. GOAP is an AI planning technique where:

- **Goals** define desired world states (e.g., "has_wood = true", "gold >= 100")
- **Actions** are operations with preconditions and effects (e.g., "chop_tree" requires "has_axe" and "at_tree")
- **State** represents current world variables (bools, ints, floats, strings, enums)
- **Conditions** are typed comparisons (`==`, `!=`, `>=`, `<=`, `>`, `<`, range) used by goals and preconditions
- **Planner** uses A* search with an admissible heuristic, so returned plans are cost-optimal

## Common Commands

### Building and Testing

```bash
# Build the library
cargo build

# Run all tests (unit tests in tests/ directory)
cargo test

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Build and run examples (17 available)
cargo run --example get_wood
cargo run --example navigation_2d
cargo run --example combat_strategy
cargo run --example enum_usage
cargo run --example mutable_state
cargo run --example mutatable_actions      # API design example
cargo run --example mutatable_goals        # API design example
```

### Development Commands

```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Build documentation
cargo doc --open
```

## Architecture

### Core Components

- **`src/lib.rs`**: Module exports with comprehensive documentation
- **`src/prelude.rs`**: Convenience imports for users
- **`src/state.rs`**: World state representation with typed variables and ergonomic APIs
- **`src/conditions.rs`**: `Condition` / `Conditions` types, the `IntoCondition` trait and the comparison helpers (`at_least`, `between`, ...)
- **`src/goals.rs`**: Goal definitions with requirements
- **`src/actions.rs`**: Action definitions with preconditions and effects
- **`src/planner.rs`**: A* search algorithm with robust error handling

### Key Types

- **`State`**: Map of named variables behind a typed API (`set`, `get::<T>`, `get_var`, `contains`, `remove`, `len`, `is_empty`, `iter`) plus the standard collection traits (`IntoIterator`, `Extend`, `FromIterator`). The map itself is private so validated states cannot be edited behind the planner's back. `State::new()` is empty; `State::builder()` is the fluent builder
- **`StateVar`**: Enum supporting Bool, I64, F64 (fixed-point), and String types. `from_f64` panics on NaN / infinite / out-of-range input; `try_from_f64` returns `StateError::UnrepresentableFloat`. `state_type()` gives the **`StateType`** (`Bool`, `I64`, `F64`, `String`), used by validation and errors
- **`EnumStateVar`**: marker trait (`Display + FromStr`) for user enums stored as strings; `state.get::<MyEnum>` parses them back
- **`Condition`**: One comparison on one variable (`Equals`, `NotEquals`, `AtLeast`, `AtMost`, `MoreThan`, `LessThan`, `Between`)
- **`Conditions`**: Named collection of conditions, one per variable; used for goal requirements and action preconditions
- **`Goal`**: Contains `conditions` that must all be satisfied, and a `priority` read by `Planner::plan_by_priority`. `Goal::new(name)` is an empty goal; `Goal::builder(name)` is the builder
- **`Action`**: Contains `preconditions` (a `Conditions`), effects (`HashMap<String, StateOperation>`, one per variable), and cost. `Action::new(name)` is a bare action (cost 1.0); `Action::builder(name)` is the builder
- **`StateOperation`**: One effect: `Set(StateVar)`, `Add(StateVar)` or `Subtract(StateVar)`. Deltas are typed (`I64` or `F64`) and must match the variable they change
- **`Plan`**: Sequence of actions with total cost. The cost is summed exactly in thousandths inside the planner (private `Cost` newtype in `planner.rs`) and exposed as `f64`
- **`Planner`**: Configuration-only planner (expansion budget). `plan(&State, &Goal, &[Action])` for one goal; `plan_by_priority(&State, &[Goal], &[Action])` tries goals from the highest priority down and returns the index of the first with a plan

### Current API Design

#### Unified State API (Ergonomic)

```rust
// Direct mutation with type safety
let mut state = State::new();
state.set("health", 100i32);
state.set("has_weapon", true);
state.set("position", 10.5f64);
state.set("location", "town");

// Type-safe retrieval with Option<T>
let health: Option<i32> = state.get("health");
let health = state.get::<i32>("health").unwrap_or(100);
```

#### Builder Pattern API (Current)

```rust
// States
let state = State::builder()
    .set("has_wood", false)
    .set("gold", 100)
    .build();

// Goals. A plain value means equality; comparisons use the prelude helpers.
let goal = Goal::builder("get_rich")
    .requires("gold", at_least(1000))
    .requires("location", "bank")
    .requires("debt", at_most(0))
    .priority(100)
    .build();

// Actions
let action = Action::builder("mine_gold")
    .cost(2.0)
    .requires("has_pickaxe", true)
    .requires("stamina", more_than(0))
    .sets("gold", 50)
    .adds("experience", 10)
    .build();
```

#### Condition semantics (important)

- `requires(key, value)` with a plain value is **equality** for every type. `requires("x", 0)` means `x == 0`.
- Ordered comparisons are explicit: `at_least`, `at_most`, `more_than`, `less_than`, `between(lo, hi)` (inclusive, bounds in any order), and `not(value)` for `!=`.
- The ordered helpers are bounded by `NumericStateVar`, so `at_least(true)` does not compile.
- One condition per key. A second `requires` on the same key replaces the first.
- A type mismatch between a condition and the variable it names (e.g. `at_least(5)` on an `f64`) is never satisfied, and `Planner::plan` reports it as `IncompatibleStateTypes` before searching. A malformed condition (an ordered comparison on a non-numeric `StateVar`, or a `Between` with mixed bound types; see `Condition::is_well_formed`) is reported as `InvalidCondition`.
- `StateVar` implements `NumericStateVar`, so `at_least(StateVar::try_from_f64(x)?)` is the fallible route for computed floats.

#### Effect semantics (important)

- `sets(key, value)` replaces the variable, any type. `adds`/`subtracts` are bounded by `NumericStateVar`; the delta keeps the type of the literal (`adds("x", 5)` is an `I64` delta, `adds("x", 5.0)` an `F64` delta).
- `State::apply` only applies a delta to a variable of the same type. A missing variable counts as zero and is created with the delta's type. A mismatched type (or a hand-built non-numeric delta) leaves the variable unchanged. Arithmetic saturates at the `i64` bounds.
- `Planner::plan` rejects mismatched deltas (`IncompatibleStateTypes`) and non-numeric deltas (`InvalidEffect`) up front, so inside the search `apply` never hits the "unchanged" case.
- One effect per key; a second `sets`/`adds`/`subtracts` on the same key replaces the first.
- The reachability analysis and the heuristic both rely on "missing counts as zero" (`Possible.base` gets the typed zero; `ConditionInfo::lower_bound` prices steps from zero). Keep `State::apply` and those two in agreement.
- `ConditionInfo::lower_bound` caches a `fixed_bound` when no action steps the variable (the bound then depends only on `Set` effects); keep that shortcut valid if `lower_bound` learns a new state-dependent case.

#### Planner guarantees

- Plans are **cost-optimal**. The heuristic (`GoalAnalysis` in `planner.rs`) is derived from the action set and never overestimates. `tests/planner.rs` cross-checks it against uniform-cost search on 600 random problems; keep that test passing when touching the heuristic.
- The heuristic (plan 006) works on *facts*: every goal condition and every precondition of a live action. Per state it runs a small Dijkstra computing `h(f) = min over touchers a of H(pre(a)) + max(cost(a), effect_bound(f))` with `H = max` over an action's preconditions (an `h_max`), and combines goal facts by a greedy sum of bounds whose charged action sets (touchers for the effect bound, the transitive closure for `h`) are pairwise disjoint. `estimate` returning `None` means the goal is unreachable from that state and the node is pruned. The admissibility argument is in `docs/plans/006-precondition-aware-heuristic.md` section 2; read it before changing `estimate`.
- Costs have a resolution of 0.001, like `f64` state values. `ActionBuilder::cost` rounds to it; `Planner::plan` converts every cost to integer thousandths once and the search never touches an `f64` (heap ordering, `g`, the heuristic's `StepRate` fractions and `Plan::cost` are all exact). Negative, NaN, infinite or out-of-range action costs are rejected up front (`InvalidActionCost`).
- `plan` always returns. A relaxed reachability analysis (`Reachability` in `planner.rs`) runs first: goals it proves unreachable return `NoPlanFound` immediately, and actions that can never fire are dropped from the search. Everything else runs under an expansion budget (`Planner::max_expansions`, default `DEFAULT_MAX_EXPANSIONS` = 100 000) and returns `SearchLimitExceeded` when it is spent. `without_expansion_limit()` opts out; only safe when the reachable state space is finite.
- The reachability analysis is an over-approximation: it must never reject a solvable problem. The 300-problem property test in `tests/planner.rs` guards this too, because its reference search does not use `Planner`.

### Error Handling

The library uses robust error handling:

- **`StateError`**: `InvalidVarType { variable, expected, found }` (typed read of the wrong type) and `UnrepresentableFloat` (NaN, infinite or out-of-range float from `StateVar::try_from_f64`)
- **`PlannerError`**: `NoPlanFound` (proved impossible), `SearchLimitExceeded { expanded }` (gave up), `InvalidActionCost { action, cost }`, `IncompatibleStateTypes { variable, expected, expected_origin, found, found_origin }` (two mentions disagree on a variable's type; origins are `VariableOrigin`), `InvalidCondition { variable, origin, condition }`, `InvalidEffect { variable, action, operation }`. All fields are public so callers can match; `Display` gives the human message
- **No panics on valid input**: planning and state manipulation return `Result`/`Option`; numeric effects saturate rather than overflow. The one deliberate panic is the infallible float conversion (`StateVar::from_f64`, reached through `set`, `requires`, `adds`, ... with a float): a NaN, infinite or out-of-range float is a contract violation, exactly as in `std::time::Duration::from_secs_f64`. `StateVar::try_from_f64` is the fallible path
- **Type safety**: Distance calculations and heuristics handle type mismatches gracefully

### Fixed-Point Arithmetic

F64 values and action costs use fixed-point arithmetic with 3 decimal places for exact equality:

- Stored as `i64` values multiplied by 1000 (`fixed_point_from_f64` in `state.rs` is the single conversion; `Cost` in `planner.rs` reuses it)
- Representable range is `StateVar::MIN_F64..=StateVar::MAX_F64` (about ±9.2e15); anything else, and NaN, has no representation and is rejected (`try_from_f64`) or panics (`from_f64`)
- Supports `Hash` and `Eq` traits for use in planning algorithms
- Transparent to users through conversion methods

## Examples Structure

Examples in `examples/` demonstrate different use cases:

### Core Examples

- **Resource gathering**: `get_wood.rs`, `get_planks.rs`, `resource_management.rs`, `crafting_counters.rs` (counters that start absent, typed int/float effects)
- **Navigation**: `navigation_2d.rs`, `patrol.rs`
- **Combat and strategy**: `combat_strategy.rs`, `stealth_mission.rs`
- **Management systems**: `base_building.rs`, `rpg_party.rs`

### Advanced Examples

- **Enum usage**: `enum_usage.rs`, `simple_enum.rs`
- **State management**: `mutable_state.rs`, `temperature_control.rs`
- **Economic simulation**: `trading.rs`

### API Design Examples

- **`mutatable_actions.rs`**: Proposed ergonomic Action mutation API
- **`mutatable_goals.rs`**: Proposed ergonomic Goal mutation API

These design examples show how to extend the current ergonomic State API to Actions and Goals.

## Testing

Tests are organized in `tests/` directory with comprehensive coverage:

- **Unit tests**: Each module has extensive test coverage
- **Integration tests**: Complete workflow demonstrations
- **Edge case tests**: NaN handling, type mismatches, error conditions
- **Performance tests**: Large state spaces and complex planning scenarios

Run specific tests:

```bash
cargo test actions           # Action-related tests
cargo test conditions       # Condition / Conditions semantics
cargo test planner          # Planner tests including the optimality property test
cargo test state            # State API tests
```

## Bug tracking and plans

- `docs/bugs.md` tracks known bugs with status, reproduction and suggested fix. Update the status table when fixing one.
- `docs/plans/` holds design/implementation plans for larger changes, with before/after measurements.

## Development Notes

### Code Quality

- **No external dependencies**: Pure Rust implementation
- **Rust 2024 edition**: Uses latest language features
- **Comprehensive documentation**: All public APIs documented with examples
- **No unsafe code**: `#![forbid(unsafe_code)]` in `lib.rs`; `#![warn(missing_docs)]` and `#![warn(missing_debug_implementations)]` keep the public API documented and debuggable (`cargo clippy --all-targets` must stay at zero warnings)

### Error Handling Philosophy

- **No panics on valid input**: All failures return proper `Result` types; only unrepresentable floats passed to the infallible constructors panic (see Error Handling)
- **Graceful degradation**: Missing data returns `None`, not errors
- **Type safety**: Compile-time checking where possible
- **User-friendly errors**: Clear error messages with context

### API Design Principles

- **Ergonomic**: Easy-to-use APIs following Rust conventions
- **Type-safe**: Compile-time type checking with generic methods
- **Consistent**: Similar patterns across State, Goal, and Action APIs
- **Performance**: Zero-cost abstractions where possible
- **Extensible**: Trait-based design for custom types

## Proposed Future APIs

The library includes design examples for extending the ergonomic State API to Actions and Goals:

### Proposed Action API

```rust
let mut action = Action::new("attack");
action.set_cost(3.0);
action.add_precondition("has_weapon", true);
action.add_effect_set("enemy_health", 0);
action.remove_precondition("old_requirement");

let stamina_req = action.get_precondition::<i32>("stamina").unwrap_or(10);
```

### Proposed Goal API

```rust
let mut goal = Goal::new("survive");
goal.set_priority(100);
goal.add_requirement("health", 50);
goal.update_requirement("health", 75);

let completion = goal.calculate_completion_percentage(&current_state);
let unmet = goal.get_unmet_requirements(&current_state);
```

These APIs would provide the same ergonomic experience as the current State API while enabling runtime modification and introspection of Actions and Goals. Since plan 005, `Action::new(name)` and `Goal::new(name)` return bare values whose fields are public, so the same edits can already be made through `action.preconditions.set(..)`, `action.effects.insert(..)` and `goal.conditions.set(..)`.
