# Goal-Oriented Action Planning (GOAP)

A general purpose Goal-Oriented Action Planning (GOAP) library written in Rust.

- Goals are a desired state of the world that an agent wants to achieve.
- Actions are the atomic operations that can be performed in the world, requiring preconditions and having effects.
- State is a collection of variables that represent the current state of the world.

Simply define goals, actions, and the world state, and the planner will find the cheapest sequence of actions that achieves the goal. The planner is A* with an admissible heuristic derived from your action set (it prices both the effects a goal needs and the chain of preconditions that enables them), so the plan it returns is guaranteed optimal for the actions and costs you gave it. Inside a `plan` call the state is compiled to a compact indexed form once, so the search never hashes or copies variable names, and the same problem always produces the same plan.

## Usage

### Conditions

Goal requirements and action preconditions are both *conditions* on state variables. A plain value means **equals**. Every other comparison is spelled out with a helper from the prelude:

```rust
use goap::prelude::*;

let goal = Goal::builder("ready")
    .requires("has_tools", true)                 // == true
    .requires("location", "workshop")            // == "workshop"
    .requires("level", 5)                        // == 5
    .requires("gold", at_least(100))             // >= 100
    .requires("alert_level", at_most(0))         // <= 0
    .requires("health", more_than(0))            // >  0
    .requires("weight", less_than(10.0))         // <  10.0
    .requires("temperature", between(20.0, 24.0)) // 20.0 <= t <= 24.0 (inclusive)
    .requires("weather", not("storm"))           // != "storm"
    .build();
```

`at_least`, `at_most`, `more_than`, `less_than` and `between` only accept numeric values; passing a `bool` or a string is a compile error. `between` accepts its bounds in either order. Only one condition is kept per variable name: a second `requires` on the same key replaces the first.

### Goals

Goals define a desired state. Create goals using the builder pattern:

```rust
use goap::prelude::*;

let goal = Goal::builder("craft_item")
    .requires("has_tools", true)            // Boolean requirement
    .requires("materials", at_least(5))     // Integer requirement
    .requires("energy", at_least(50.0))     // Float requirement
    .requires("location", "workshop")       // String requirement
    .build();
```

### Actions

Actions are operations that change the world state. Each action has preconditions, effects, and a cost:

```rust
use goap::prelude::*;

let craft = Action::builder("craft_item")
    .cost(2.0)  // Action cost
    // Preconditions are conditions, exactly like goal requirements
    .requires("has_tools", true)         // Boolean precondition
    .requires("materials", at_least(5))  // Integer precondition
    .requires("energy", at_least(50.0))  // Float precondition
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

Effect rules:

- `sets` replaces the variable with a value of any type, creating it if needed.
- `adds` / `subtracts` only accept numbers, and the **delta's type must match the variable's type**: an integer delta for an `i64` variable, a float delta for an `f64` variable. `adds("energy", 5)` on a float variable is a mistake, and `Planner::plan` reports it as `IncompatibleStateTypes` before searching.
- A variable that is missing from the state counts as zero when added to or subtracted from, so counters such as `items_crafted` need no initial value.
- Arithmetic saturates at the `i64` bounds instead of overflowing.
- One effect is kept per variable: a second `sets`/`adds`/`subtracts` on the same key replaces the first, exactly like `requires`.

Costs are compared at a resolution of 0.001 (`cost(1.23456)` is stored as `1.235`), and the planner sums them exactly, so ten actions of cost `0.1` give a plan cost of exactly `1.0`.

### State

State represents the current state of the world:

```rust
use goap::prelude::*;

let state = State::builder()
    // All possible state types
    .set("has_tools", true)              // Boolean state
    .set("materials", 10)                // Integer state
    .set("energy", 100.0)                // Float state
    .set("location", "workshop")         // String state
    .build();

// Typed and untyped reads; the variable map itself is private.
let energy: Option<f64> = state.get("energy");
let raw: Option<&StateVar> = state.get_var("energy");
assert!(state.contains("materials") && state.len() == 4);

// Or start empty and mutate. `State` is also a collection: iterate it,
// merge another state in with `extend`, or `collect` one from pairs.
let mut current = State::new();
current.set("energy", 50.0);
current.extend(state);
assert_eq!(current.get::<i64>("materials"), Some(10));
```

Your own enums work as state values: implement `Display` (to store) and `FromStr` (to read back) plus the empty marker trait `EnumStateVar`, then `state.set("location", Location::Forest)` and `state.get::<Location>("location")` both work. See `examples/simple_enum.rs`.

Floats are stored as fixed point with three decimals, so `0.1 + 0.2 == 0.3` inside the planner. A float that cannot be stored this way (NaN, infinite, or beyond about ±9.2e15) is a programming error: `set` panics on it, the same way `Duration::from_secs_f64` does. For values you do not control, convert first and handle the error. The converted `StateVar` is accepted everywhere a number is, including `adds`, `subtracts` and the comparison helpers:

```rust
use goap::prelude::*;

fn record(state: &mut State, reading: f64) -> Result<(), StateError> {
    state.set("temperature", StateVar::try_from_f64(reading)?);
    Ok(())
}

fn heater(step: f64) -> Result<Action, StateError> {
    let step = StateVar::try_from_f64(step)?;
    Ok(Action::builder("heat").requires("temperature", at_most(step.clone())).adds("temperature", step).build())
}
```

### Planning

Use the planner to find a sequence of actions to achieve a goal:

```rust
use goap::prelude::*;

// Create initial state
let initial_state = State::builder()
    .set("has_wood", false)
    .set("has_axe", true)
    .set("at_tree", false)
    .build();

// Create goal
let goal = Goal::builder("gather_wood")
    .requires("has_wood", true)
    .build();

// Create actions
let move_to_tree = Action::builder("move_to_tree")
    .sets("at_tree", true)
    .build();

let chop_tree = Action::builder("chop_tree")
    .cost(2.0)
    .requires("has_axe", true)
    .requires("at_tree", true)
    .sets("has_wood", true)
    .build();

// Find plan
let actions = vec![move_to_tree, chop_tree];
let planner = Planner::new();
let plan = planner.plan(&initial_state, &goal, &actions).unwrap();

println!("Found plan with cost {}", plan.cost);
for action in &plan.actions {
    println!("- {}", action.name);
}
```

When an agent has several goals, `plan_by_priority` tries them from the highest `priority` down and returns the index of the first one that has a plan:

```rust
use goap::prelude::*;

let state = State::builder().set("hungry", true).set("gold", 0).build();
let goals = [
    Goal::builder("get_rich").priority(5).requires("gold", at_least(100)).build(),
    Goal::builder("eat").priority(10).requires("hungry", false).build(),
];
let actions = [Action::builder("eat").sets("hungry", false).build()];
let (index, plan) = Planner::new().plan_by_priority(&state, &goals, &actions).unwrap();
assert_eq!(goals[index].name, "eat");
```

`plan` returns an error instead of a plan when:

- any action has a negative, NaN, infinite or out-of-range cost (`PlannerError::InvalidActionCost { action, cost }`);
- a condition or effect uses a different type than the variable it refers to, for example an integer requirement or an integer `adds` on a float variable (`PlannerError::IncompatibleStateTypes { variable, expected, found, .. }`, with the origin of both mentions);
- a condition or effect is malformed: an ordered comparison on a non-numeric value, a `Between` with mixed bound types, or an `adds` of a non-numeric value (`PlannerError::InvalidCondition`, `PlannerError::InvalidEffect`). These can only be built by hand or by passing a `StateVar`; the typed helpers prevent them at compile time;
- the goal is proved unreachable (`PlannerError::NoPlanFound`). A reachability analysis runs before the search and catches goals no action can satisfy, including ones blocked by preconditions that can never hold;
- the search budget runs out (`PlannerError::SearchLimitExceeded { expanded }`). By default the planner expands at most 100 000 states before giving up, so an unreachable goal in an unbounded state space (for example an action that keeps adding gold) returns an error instead of running forever. Raise the budget with `Planner::new().max_expansions(n)` or remove it with `Planner::new().without_expansion_limit()`.

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
