/// A Goal Oriented Action Planning (GOAP) library for Rust.
///
/// This library provides a framework for AI agents to plan sequences of actions
/// to achieve desired goals. GOAP is useful for game AI, robotics, and other
/// applications where an agent needs to reason about actions and their effects.
///
/// ## Core Concepts
///
/// - **State**: Represents the current state of the world using typed variables
/// - **Goals**: Define desired world states that the agent wants to achieve
/// - **Actions**: Operations that can change the world state, with preconditions and effects
/// - **Planner**: Uses A* search to find optimal sequences of actions to achieve goals
///
/// ## Quick Start
///
/// ```rust
/// use goap::prelude::*;
///
/// // Create initial state
/// let state = State::new()
///     .set("has_wood", false)
///     .set("has_axe", true)
///     .build();
///
/// // Define goal
/// let goal = Goal::new("get_wood")
///     .requires("has_wood", true)
///     .build();
///
/// // Create action
/// let chop_wood = Action::new("chop_wood")
///     .requires("has_axe", true)
///     .sets("has_wood", true)
///     .build();
///
/// // Find plan
/// let planner = Planner::new();
/// let plan = planner.plan(state, &goal, &[chop_wood]).unwrap();
/// ```
///

/// Actions module - defines actions that can be performed to change state
pub mod actions;
/// Goals module - defines goals that agents want to achieve
pub mod goals;
/// Planner module - implements A* search for finding action sequences
pub mod planner;
/// Prelude module - convenient imports for common use cases
pub mod prelude;
/// State module - represents world state using typed variables
pub mod state;
