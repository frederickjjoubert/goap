# Goal-Oriented Action Planning (GOAP)

A general purpose Goal-Oriented Action Planning (GOAP) library written in Rust.

- Goals are a desired state of the world that an agent wants to achieve.
- Actions are the atomic operations that can be performed in the world, requiring preconditions and having effects.
- State is a collection of variables that represent the current state of the world.

Simply define goals, actions, and the world state, and the planner will find the best possible path of actions to achieve the goal.

## Usage

### Goals

Goals define a desired state. Create goals using the builder pattern:

```rust
use goap::prelude::*;

let goal = Goal::new("craft_item")
    // All possible requirement types
    .requires("has_tools", true)         // Boolean requirement
    .requires("materials", 5)            // Integer requirement (satisfied by >= 5)
    .requires("energy", 50.0)            // Float requirement (satisfied by >= 50.0)
    .requires("location", "workshop")    // String requirement (exact match)
    .build();
```

### Actions

Actions are operations that change the world state. Each action has preconditions, effects, and a cost:

```rust
use goap::prelude::*;

let craft = Action::new("craft_item")
    .cost(2.0)  // Action cost
    // All possible precondition types
    .requires("has_tools", true)         // Boolean precondition
    .requires("materials", 5)            // Integer precondition
    .requires("energy", 50.0)            // Float precondition
    .requires("location", "workshop")    // String precondition
    // All possible effect types
    .sets("has_item", true)              // Set boolean
    .sets("materials", 0)                // Set integer
    .sets("energy", 25.0)                // Set float
    .sets("location", "workshop")        // Set string
    .adds("items_crafted", 1)            // Add to integer
    .subtracts("materials", 5)           // Subtract from integer
    .adds("experience", 10.0)            // Add to float
    .subtracts("energy", 25.0)           // Subtract from float
    .build();
```

### State

State represents the current state of the world:

```rust
use goap::prelude::*;

let state = State::new()
    // All possible state types
    .set("has_tools", true)              // Boolean state
    .set("materials", 10)                // Integer state
    .set("energy", 100.0)                // Float state
    .set("location", "workshop")         // String state
    .build();
```

### Planning

Use the planner to find a sequence of actions to achieve a goal:

```rust
use goap::prelude::*;

// Create initial state
let initial_state = State::new()
    .set("has_wood", false)
    .set("has_axe", true)
    .set("at_tree", false)
    .build();

// Create goal
let goal = Goal::new("gather_wood")
    .requires("has_wood", true)
    .build();

// Create actions
let move_to_tree = Action::new("move_to_tree")
    .sets("at_tree", true)
    .build();

let chop_tree = Action::new("chop_tree")
    .cost(2.0)
    .requires("has_axe", true)
    .requires("at_tree", true)
    .sets("has_wood", true)
    .build();

// Find plan
let actions = vec![move_to_tree, chop_tree];
let planner = Planner::new();
let plan = planner.plan(initial_state, &goal, &actions).unwrap();

println!("Found plan with cost {}", plan.cost);
for action in &plan.actions {
    println!("- {}", action.name);
}
```

## Examples

See the examples directory for complete usage examples including:

- Resource gathering and crafting
- Character and party management
- Navigation and pathfinding
- Combat and strategy
- Base building and management

## License

This project is triple-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-Apache-2.0)  
- [BSD 3-Clause License](LICENSE-BSD-3-Clause)

You may use this software under the terms of any of these licenses.
