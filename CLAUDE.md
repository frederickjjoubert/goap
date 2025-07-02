# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Goal Oriented Action Planning (GOAP) library written in Rust. GOAP is an AI planning technique where:

- **Goals** define desired world states (e.g., "has_wood = true")
- **Actions** are operations with preconditions and effects (e.g., "chop_tree" requires "has_axe" and "at_tree")
- **State** represents current world variables (bools, ints, floats, enums)
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

# Build and run examples
cargo run --example get_wood
cargo run --example navigation_2d
cargo run --example combat_strategy
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

- **`src/lib.rs`**: Module exports and entry point
- **`src/prelude.rs`**: Convenience imports for users
- **`src/state.rs`**: World state representation with typed variables (`StateVar` enum)
- **`src/goals.rs`**: Goal definitions with requirements
- **`src/actions.rs`**: Action definitions with preconditions and effects
- **`src/planner.rs`**: A* search algorithm implementation

### Key Types

- **`State`**: HashMap of variable names to `StateVar` values
- **`StateVar`**: Enum supporting Bool, Int, Float, and Enum types
- **`Goal`**: Contains requirements that must be satisfied
- **`Action`**: Contains preconditions, effects, and cost
- **`Plan`**: Sequence of actions with total cost
- **`Planner`**: Stateless planner that generates plans

### Builder Pattern Usage

All main types use builder patterns:

```rust
let state = State::builder().bool("var", true).build();
let goal = Goal::builder("name").require_bool("var", true).build();
let action = Action::builder("name").cost(1.0).precondition("var", true).build();
```

## Examples Structure

Examples in `examples/` demonstrate different use cases:

- Resource gathering (`get_wood.rs`, `get_planks.rs`)
- Navigation (`navigation_2d.rs`)
- Combat and strategy (`combat_strategy.rs`, `stealth_mission.rs`)
- Management systems (`resource_management.rs`, `base_building.rs`)

## Testing

Tests are organized in `tests/` directory mirroring `src/` structure:

- Unit tests for each module
- Integration tests showing complete workflows
- Run specific module tests: `cargo test actions` or `cargo test planner`

## Development Notes

- No external dependencies (pure Rust)
- Uses Rust 2024 edition
- Library follows standard Rust conventions
- All types implement Debug, Display where appropriate
- Error handling uses custom error types (`StateError`, `PlannerError`)
