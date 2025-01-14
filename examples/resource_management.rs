use std::collections::HashMap;

use goap::prelude::*;

fn main() {
    // Initial state
    let mut initial_state = WorldState::new();
    initial_state.set("wood_logs", StateVar::I64(0));
    initial_state.set("planks", StateVar::I64(0));
    initial_state.set("has_saw", StateVar::Bool(false));
    initial_state.set("coins", StateVar::I64(10));
    initial_state.set("at_store", StateVar::Bool(false));
    initial_state.set("at_forest", StateVar::Bool(false));

    // Goal: Have at least 5 planks
    let mut goal_state = WorldState::new();
    goal_state.set("planks", StateVar::I64(5));
    let goal = Goal::new("craft_planks", goal_state);

    // Action: Go to store
    let mut goto_store_effects = HashMap::new();
    goto_store_effects.insert(
        "at_store".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_store_effects.insert(
        "at_forest".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_store_action = Action::new("goto_store", 1.0, WorldState::new(), goto_store_effects);

    // Action: Buy saw
    let mut buy_saw_effects = HashMap::new();
    buy_saw_effects.insert(
        "has_saw".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    buy_saw_effects.insert(
        "coins".to_string(),
        StateOperation::Subtract(5), // Subtract 5 coins instead of setting to 5
    );

    let mut buy_saw_conditions = WorldState::new();
    buy_saw_conditions.set("at_store", StateVar::Bool(true));
    buy_saw_conditions.set("coins", StateVar::I64(5));

    let buy_saw_action = Action::new("buy_saw", 1.0, buy_saw_conditions, buy_saw_effects);

    // Action: Go to forest
    let mut goto_forest_effects = HashMap::new();
    goto_forest_effects.insert(
        "at_forest".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_forest_effects.insert(
        "at_store".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_forest_action =
        Action::new("goto_forest", 1.0, WorldState::new(), goto_forest_effects);

    // Action: Gather wood
    let mut gather_wood_effects = HashMap::new();
    gather_wood_effects.insert(
        "wood_logs".to_string(),
        StateOperation::Add(3), // Add 3 logs instead of setting to 3
    );

    let mut gather_wood_conditions = WorldState::new();
    gather_wood_conditions.set("at_forest", StateVar::Bool(true));
    let gather_wood_action = Action::new(
        "gather_wood",
        2.0,
        gather_wood_conditions,
        gather_wood_effects,
    );

    // Action: Craft planks
    let mut craft_planks_effects = HashMap::new();
    craft_planks_effects.insert("planks".to_string(), StateOperation::Set(StateVar::I64(6)));
    craft_planks_effects.insert(
        "wood_logs".to_string(),
        StateOperation::Set(StateVar::I64(0)),
    );
    let mut craft_planks_conditions = WorldState::new();
    craft_planks_conditions.set("wood_logs", StateVar::I64(3)); // Need 3 logs
    craft_planks_conditions.set("has_saw", StateVar::Bool(true));
    let craft_planks_action = Action::new(
        "craft_planks",
        2.0,
        craft_planks_conditions,
        craft_planks_effects,
    );

    // Create planner and add actions
    let mut planner = Planner::new();
    planner.add_action(goto_store_action);
    planner.add_action(buy_saw_action);
    planner.add_action(goto_forest_action);
    planner.add_action(gather_wood_action);
    planner.add_action(craft_planks_action);

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal);

    // Assert that a plan was found
    assert!(plan_result.is_some(), "Expected to find a valid plan");

    let (actions, total_cost) = plan_result.unwrap();

    println!("\nActual plan found:");
    for action in &actions {
        println!("- {} (cost: {})", action.name, action.cost);
        println!("  Preconditions: {:?}", action.preconditions);
        println!("  Effects: {:?}", action.effects);
    }

    // Verify the total cost is what we expect (1 + 1 + 1 + 2 + 2 = 7)
    assert_eq!(total_cost, 7.0, "Expected total cost to be 7.0");

    // Verify we have the right number of actions
    assert_eq!(actions.len(), 5, "Expected 5 actions in the plan");

    // Verify the plan contains all the required actions (in any order)
    let action_names: Vec<_> = actions.iter().map(|a| a.name.as_str()).collect();
    assert!(
        action_names.contains(&"goto_store"),
        "Plan should include goto_store"
    );
    assert!(
        action_names.contains(&"buy_saw"),
        "Plan should include buy_saw"
    );
    assert!(
        action_names.contains(&"goto_forest"),
        "Plan should include goto_forest"
    );
    assert!(
        action_names.contains(&"gather_wood"),
        "Plan should include gather_wood"
    );
    assert!(
        action_names.contains(&"craft_planks"),
        "Plan should include craft_planks"
    );

    // Optional: Print the plan for debugging
    println!("\nResource management plan found with cost {}", total_cost);
    for action in actions {
        println!("- {}", action.name);
    }
}
