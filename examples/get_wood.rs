use std::collections::HashMap;

use goap::prelude::*;

fn main() {
    // Create initial state
    let mut initial_state = WorldState::new();
    initial_state.set("has_wood", StateVar::Bool(false));
    initial_state.set("has_axe", StateVar::Bool(true));
    initial_state.set("at_tree", StateVar::Bool(false));

    // Create goal state
    let mut goal_state = WorldState::new();
    goal_state.set("has_wood", StateVar::Bool(true));
    let goal = Goal::new("gather_wood", goal_state);

    // Create actions
    let mut move_to_tree_effects = HashMap::new();
    move_to_tree_effects.insert(
        "at_tree".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let move_action = Action::new(
        "move_to_tree",
        1.0,
        WorldState::new(), // No preconditions
        move_to_tree_effects,
    );

    let mut chop_tree_effects = HashMap::new();
    chop_tree_effects.insert(
        "has_wood".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut chop_conditions = WorldState::new();
    chop_conditions.set("has_axe", StateVar::Bool(true));
    chop_conditions.set("at_tree", StateVar::Bool(true));
    let chop_action = Action::new("chop_tree", 2.0, chop_conditions, chop_tree_effects);

    // Create planner and add actions
    let mut planner = Planner::new();
    planner.add_action(move_action);
    planner.add_action(chop_action);

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal);
    assert!(plan_result.is_some(), "Expected to find a valid plan");

    let (actions, total_cost) = plan_result.unwrap();
    assert_eq!(total_cost, 3.0, "Expected total cost to be 3.0");

    let expected_actions = vec!["move_to_tree", "chop_tree"];
    assert_eq!(
        actions.len(),
        expected_actions.len(),
        "Plan length doesn't match expected length"
    );

    for (action, expected_name) in actions.iter().zip(expected_actions.iter()) {
        assert_eq!(
            action.name, *expected_name,
            "Expected action {} but got {}",
            expected_name, action.name
        );
    }

    println!("Found plan with cost {}", total_cost);
    for action in actions {
        println!("- {}", action.name);
    }
}
