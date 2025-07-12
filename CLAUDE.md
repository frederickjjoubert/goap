# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Goal Oriented Action Planning (GOAP) library written in Rust. GOAP is an AI planning technique where:

- **Goals** define desired world states (e.g., "has_wood = true")
- **Actions** are operations with preconditions and effects (e.g., "chop_tree" requires "has_axe" and "at_tree")
- **State** represents current world variables (bools, ints, floats, strings, enums)
- **Planner** uses A* search to find optimal action sequences

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

# Build and run examples (14+ available)
cargo run --example get_wood
cargo run --example navigation_2d
cargo run --example combat_strategy
cargo run --example enum_usage
cargo run --example modifiable_state_usage
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
- **`src/goals.rs`**: Goal definitions with requirements
- **`src/actions.rs`**: Action definitions with preconditions and effects
- **`src/planner.rs`**: A* search algorithm with robust error handling

### Key Types

- **`State`**: HashMap-like structure with type-safe get/set operations
- **`StateVar`**: Enum supporting Bool, I64, F64 (fixed-point), and String types
- **`Goal`**: Contains requirements that must be satisfied
- **`Action`**: Contains preconditions, effects, and cost
- **`Plan`**: Sequence of actions with total cost
- **`Planner`**: Stateless planner with proper error handling

### Current API Design

#### Unified State API (Ergonomic)

```rust
// Direct mutation with type safety
let mut state = State::empty();
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
let state = State::new()
    .set("has_wood", false)
    .set("gold", 100)
    .build();

// Goals
let goal = Goal::new("get_rich")
    .requires("gold", 1000)
    .priority(100)
    .build();

// Actions
let action = Action::new("mine_gold")
    .cost(2.0)
    .requires("has_pickaxe", true)
    .sets("gold", 50)
    .adds("experience", 10)
    .build();
```

### Error Handling

The library uses robust error handling:

- **`StateError`**: For state variable type mismatches and missing variables
- **`PlannerError`**: For planning failures and incompatible state types
- **No panics**: All potential failure points return `Result` types
- **Type safety**: Distance calculations and heuristics handle type mismatches gracefully

### Fixed-Point Arithmetic

F64 values use fixed-point arithmetic with 3 decimal places for exact equality:

- Stored as `i64` values multiplied by 1000
- Supports `Hash` and `Eq` traits for use in planning algorithms
- Transparent to users through conversion methods

## Examples Structure

Examples in `examples/` demonstrate different use cases:

### Core Examples

- **Resource gathering**: `get_wood.rs`, `get_planks.rs`, `resource_management.rs`
- **Navigation**: `navigation_2d.rs`, `patrol.rs`
- **Combat and strategy**: `combat_strategy.rs`, `stealth_mission.rs`
- **Management systems**: `base_building.rs`, `rpg_party.rs`

### Advanced Examples

- **Enum usage**: `enum_usage.rs`, `simple_enum.rs`
- **State management**: `modifiable_state_usage.rs`, `temperature_control.rs`
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
cargo test planner          # Planner tests including edge cases
cargo test state            # State API tests
```

## Development Notes

### Code Quality

- **No external dependencies**: Pure Rust implementation
- **Rust 2024 edition**: Uses latest language features
- **Comprehensive documentation**: All public APIs documented with examples
- **No unsafe code**: Memory-safe implementation throughout

### Error Handling Philosophy

- **No panics in production code**: All failures return proper `Result` types
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
let mut action = Action::empty("attack");
action.set_cost(3.0);
action.add_precondition("has_weapon", true);
action.add_effect_set("enemy_health", 0);
action.remove_precondition("old_requirement");

let stamina_req = action.get_precondition::<i32>("stamina").unwrap_or(10);
```

### Proposed Goal API

```rust
let mut goal = Goal::empty("survive");
goal.set_priority(100);
goal.add_requirement("health", 50);
goal.update_requirement("health", 75);

let completion = goal.calculate_completion_percentage(&current_state);
let unmet = goal.get_unmet_requirements(&current_state);
```

These APIs would provide the same ergonomic experience as the current State API while enabling runtime modification and introspection of Actions and Goals.
