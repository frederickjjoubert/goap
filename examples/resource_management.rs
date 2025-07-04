use goap::prelude::*;

fn main() {
    // Initial state
    let initial_state = State::new()
        .set("wood_logs", 0)
        .set("planks", 0)
        .set("has_saw", false)
        .set("coins", 10)
        .set("at_store", false)
        .set("at_forest", false)
        .build();

    // Goal: Have at least 5 planks
    let goal = Goal::new("craft_planks").requires("planks", 5).build();

    // Action: Go to store
    let goto_store = Action::new("goto_store")
        .cost(1.0)
        .sets("at_store", true)
        .sets("at_forest", false)
        .build();

    // Action: Buy saw
    let buy_saw = Action::new("buy_saw")
        .cost(1.0)
        .requires("at_store", true)
        .requires("coins", 5)
        .sets("has_saw", true)
        .subtracts("coins", 5)
        .build();

    // Action: Go to forest
    let goto_forest = Action::new("goto_forest")
        .cost(1.0)
        .sets("at_forest", true)
        .sets("at_store", false)
        .build();

    // Action: Gather wood
    let gather_wood = Action::new("gather_wood")
        .cost(2.0)
        .requires("at_forest", true)
        .adds("wood_logs", 3)
        .build();

    // Action: Craft planks
    let craft_planks = Action::new("craft_planks")
        .cost(2.0)
        .requires("wood_logs", 3)
        .requires("has_saw", true)
        .sets("planks", 6)
        .sets("wood_logs", 0)
        .build();

    // Collect all actions
    let actions = vec![goto_store, buy_saw, goto_forest, gather_wood, craft_planks];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);

    // Assert that a plan was found
    assert!(plan_result.is_ok(), "Expected to find a valid plan");

    let plan = plan_result.unwrap();

    println!("\nActual plan found:");
    for action in &plan.actions {
        println!(
            "- {name} (cost: {cost})",
            name = action.name,
            cost = action.cost
        );
        let preconditions = &action.preconditions;
        println!("  Preconditions: {preconditions:?}");
        let effects = &action.effects;
        println!("  Effects: {effects:?}");
    }

    // Verify the total cost is what we expect (1 + 1 + 1 + 2 + 2 = 7)
    assert_eq!(plan.cost, 7.0, "Expected total cost to be 7.0");

    // Verify we have the right number of actions
    assert_eq!(plan.actions.len(), 5, "Expected 5 actions in the plan");

    // Verify the plan contains all the required actions (in any order)
    let action_names: Vec<_> = plan.actions.iter().map(|a| a.name.as_str()).collect();
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
    println!(
        "\nResource management plan found with cost {cost}",
        cost = plan.cost
    );
    for action in plan.actions {
        let name = &action.name;
        println!("- {name}");
    }
}
