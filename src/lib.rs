//! A Goal Oriented Action Planning (GOAP) library for Rust.
//!
//! This library provides a framework for AI agents to plan sequences of actions
//! to achieve desired goals. GOAP is useful for game AI, robotics, and other
//! applications where an agent needs to reason about actions and their effects.
//!
//! ## Core Concepts
//!
//! - **State**: Represents the current state of the world using typed variables
//! - **Conditions**: Comparisons (`==`, `>=`, `<=`, ranges, ...) on state variables
//! - **Goals**: Define desired world states that the agent wants to achieve
//! - **Actions**: Operations that can change the world state, with preconditions and effects
//! - **Planner**: Uses A* search to find optimal sequences of actions to achieve goals,
//!   with an expansion budget so planning always terminates

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

/// Actions module - defines actions that can be performed to change state
pub mod actions;
// `conditions` carries its own module documentation (`//!`); an outer doc
// comment here would make rustdoc resolve its intra-doc links in this scope.
pub mod conditions;
/// Goals module - defines goals that agents want to achieve
pub mod goals;
/// Planner module - implements A* search for finding action sequences
pub mod planner;
/// Prelude module - convenient imports for common use cases
pub mod prelude;
/// State module - represents world state using typed variables
pub mod state;
