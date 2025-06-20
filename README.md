# Goal Oriented Action Planning (GOAP)

A general purpose Goal Oriented Action Planning (GOAP) framework written in Rust.

- Goals are a desired state of the world that an agent wants to achieve.
- Actions are the atomic operations that can be performed in the world, requiring preconditions and having effects.
- State is a collection of variables that represent the current state of the world.

Simply define goals, actions, and the world state, and the planner will find the best possible path of actions to achieve the goal.

## Usage

### Goals

Goals define a desired state. Create goals using the builder pattern:

```rust
use goap::prelude::*;

let goal = Goal::builder("craft_item")
    // All possible requirement types
    .require_bool("has_tools", true)         // Boolean requirement
    .require_int("materials", 5)             // Integer requirement (satisfied by >= 5)
    .require_float("energy", 50.0)           // Float requirement (satisfied by >= 50.0)
    .require_enum("location", "workshop")    // Enum/String requirement (exact match)
    .build();
```

### Actions

Actions are operations that change the world state. Each action has preconditions, effects, and a cost:

```rust
use goap::prelude::*;

let craft = Action::builder("craft_item")
    .cost(2.0)  // Action cost
    // All possible precondition types
    .precondition("has_tools", true)         // Boolean precondition
    .precondition("materials", 5)            // Integer precondition
    .precondition("energy", 50.0)            // Float precondition
    .precondition("location", "workshop")    // Enum/String precondition
    // All possible effect types
    .effect_set_to("has_item", true)         // Set boolean
    .effect_set_to("materials", 0)           // Set integer
    .effect_set_to("energy", 25.0)           // Set float
    .effect_set_to("location", "workshop")   // Set enum/string
    .effect_add_int("items_crafted", 1)      // Add to integer
    .effect_subtract_int("materials", 5)      // Subtract from integer
    .effect_add_float("experience", 10.0)     // Add to float
    .effect_subtract_float("energy", 25.0)    // Subtract from float
    .build();
```

### State

State represents the current state of the world:

```rust
use goap::prelude::*;

let state = State::builder()
    // All possible state types
    .bool("has_tools", true)             // Boolean state
    .int("materials", 10)                // Integer state
    .float("energy", 100.0)              // Float state
    .enum_val("location", "workshop")    // Enum/String state
    .build();
```

See the examples directory for complete usage examples including:

- Resource gathering and crafting
- Character and party management
- Navigation and pathfinding
- Combat and strategy
- Base building and management
