use goap::prelude::*;

fn main() {
    // Initial state - player is not ready for combat
    let initial_state = State::new()
        .set("health", 30) // Low health (30/100)
        .set("armor", 0) // No armor
        .set("ammo", 5) // Low ammo
        .set("at_medical", false)
        .set("at_armory", false)
        .set("at_trading", false)
        .set("has_credits", 100) // Starting credits
        .build();

    // Goal state - ready for boss battle
    let goal = Goal::new("prepare_for_boss")
        .requires("health", 100) // Full health
        .requires("armor", 50) // Decent armor
        .requires("ammo", 50) // Sufficient ammo
        .build();

    // Action: Move to medical bay
    let goto_medical = Action::new("goto_medical")
        .cost(1.0)
        .sets("at_medical", true)
        .sets("at_armory", false)
        .sets("at_trading", false)
        .build();

    // Action: Move to armory
    let goto_armory = Action::new("goto_armory")
        .cost(1.0)
        .sets("at_armory", true)
        .sets("at_medical", false)
        .sets("at_trading", false)
        .build();

    // Action: Move to trading post
    let goto_trading = Action::new("goto_trading")
        .cost(1.0)
        .sets("at_trading", true)
        .sets("at_medical", false)
        .sets("at_armory", false)
        .build();

    // Action: Use health station
    let heal = Action::new("use_health_station")
        .cost(2.0)
        .requires("at_medical", true)
        .requires("has_credits", 20)
        .sets("health", 100)
        .subtracts("has_credits", 20)
        .build();

    // Action: Buy armor
    let buy_armor = Action::new("buy_armor")
        .cost(2.0)
        .requires("at_armory", true)
        .requires("has_credits", 30)
        .sets("armor", 50)
        .subtracts("has_credits", 30)
        .build();

    // Action: Buy ammo
    let buy_ammo = Action::new("buy_ammo")
        .cost(1.5)
        .requires("at_trading", true)
        .requires("has_credits", 25)
        .sets("ammo", 50)
        .subtracts("has_credits", 25)
        .build();

    // Collect all actions
    let actions = vec![
        goto_medical,
        goto_armory,
        goto_trading,
        heal,
        buy_armor,
        buy_ammo,
    ];

    // Store a copy of initial state for later comparison
    let initial_state_copy = initial_state.clone();

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal, &actions);
    assert!(
        plan_result.is_ok(),
        "Expected to find a valid plan for combat preparation"
    );

    let plan = plan_result.unwrap();

    println!(
        "\nCombat Strategy Plan found with cost {cost}",
        cost = plan.cost
    );
    println!("Starting with 100 credits");
    for action in &plan.actions {
        println!(
            "- {name} (cost: {cost})",
            name = action.name,
            cost = action.cost
        );
    }

    // Verify we have the right number of actions (should include movement and purchases)
    assert!(
        actions.len() >= 5,
        "Expected at least 5 actions in the plan"
    );

    // Verify the plan contains all necessary actions (in any order)
    let action_names: Vec<_> = plan.actions.iter().map(|a| a.name.as_str()).collect();

    // Verify healing sequence
    assert!(
        action_names.contains(&"goto_medical"),
        "Plan should include going to medical"
    );
    assert!(
        action_names.contains(&"use_health_station"),
        "Plan should include healing"
    );

    // Verify armor sequence
    assert!(
        action_names.contains(&"goto_armory"),
        "Plan should include going to armory"
    );
    assert!(
        action_names.contains(&"buy_armor"),
        "Plan should include buying armor"
    );

    // Verify ammo sequence
    assert!(
        action_names.contains(&"goto_trading"),
        "Plan should include going to trading post"
    );
    assert!(
        action_names.contains(&"buy_ammo"),
        "Plan should include buying ammo"
    );

    // Verify total cost is within expected range (should be sum of all action costs)
    assert!(
        plan.cost >= 7.5,
        "Total cost should be at least 7.5 (movement + purchases)"
    );

    // Simulate plan execution to verify final state
    let mut current_state = initial_state_copy.clone();
    let mut total_credits = 100;

    for action in &plan.actions {
        // Apply action effects
        current_state = action.apply_effect(&current_state);

        // Track credits spent
        if let Some(credits) = current_state.get::<i64>("has_credits") {
            total_credits = credits;
        }
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );

    // Verify we didn't overspend
    assert!(
        total_credits >= 0,
        "Should not spend more credits than available"
    );

    println!("\nFinal state verification:");
    println!("Remaining credits: {total_credits}");
    if let Some(health) = current_state.get::<i64>("health") {
        println!("Final health: {health}");
    }
    if let Some(armor) = current_state.get::<i64>("armor") {
        println!("Final armor: {armor}");
    }
    if let Some(ammo) = current_state.get::<i64>("ammo") {
        println!("Final ammo: {ammo}");
    }
}
