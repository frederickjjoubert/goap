use goap::prelude::*;

fn main() {
    // Initial state
    let initial_state = WorldState::builder()
        .int("wood_logs", 0)
        .int("planks", 0)
        .bool("has_saw", false)
        .int("coins", 10)
        .bool("at_store", false)
        .bool("at_forest", false)
        .build();

    // Goal: Have at least 5 planks
    let goal = Goal::builder("craft_planks")
        .require_int("planks", 5)
        .build();

    // Action: Go to store
    let goto_store = Action::builder("goto_store")
        .cost(1.0)
        .effect_set_to("at_store", true)
        .effect_set_to("at_forest", false)
        .build();

    // Action: Buy saw
    let buy_saw = Action::builder("buy_saw")
        .cost(1.0)
        .precondition("at_store", true)
        .precondition("coins", 5)
        .effect_set_to("has_saw", true)
        .effect_subtract_int("coins", 5)
        .build();

    // Action: Go to forest
    let goto_forest = Action::builder("goto_forest")
        .cost(1.0)
        .effect_set_to("at_forest", true)
        .effect_set_to("at_store", false)
        .build();

    // Action: Gather wood
    let gather_wood = Action::builder("gather_wood")
        .cost(2.0)
        .precondition("at_forest", true)
        .effect_add_int("wood_logs", 3)
        .build();

    // Action: Craft planks
    let craft_planks = Action::builder("craft_planks")
        .cost(2.0)
        .precondition("wood_logs", 3)
        .precondition("has_saw", true)
        .effect_set_to("planks", 6)
        .effect_set_to("wood_logs", 0)
        .build();

    // Collect all actions
    let actions = vec![goto_store, buy_saw, goto_forest, gather_wood, craft_planks];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);

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
