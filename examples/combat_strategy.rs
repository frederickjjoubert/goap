use goap::prelude::*;

fn main() {
    // Initial state - player is not ready for combat
    let initial_state = WorldState::builder()
        .int("health", 30) // Low health (30/100)
        .int("armor", 0) // No armor
        .int("ammo", 5) // Low ammo
        .bool("at_medical", false)
        .bool("at_armory", false)
        .bool("at_trading", false)
        .int("has_credits", 100) // Starting credits
        .build();

    // Goal state - ready for boss battle
    let goal = Goal::builder("prepare_for_boss")
        .require_int("health", 100) // Full health
        .require_int("armor", 50) // Decent armor
        .require_int("ammo", 50) // Sufficient ammo
        .build();

    // Action: Move to medical bay
    let goto_medical = Action::builder("goto_medical")
        .cost(1.0)
        .effect_set_to("at_medical", true)
        .effect_set_to("at_armory", false)
        .effect_set_to("at_trading", false)
        .build();

    // Action: Move to armory
    let goto_armory = Action::builder("goto_armory")
        .cost(1.0)
        .effect_set_to("at_armory", true)
        .effect_set_to("at_medical", false)
        .effect_set_to("at_trading", false)
        .build();

    // Action: Move to trading post
    let goto_trading = Action::builder("goto_trading")
        .cost(1.0)
        .effect_set_to("at_trading", true)
        .effect_set_to("at_medical", false)
        .effect_set_to("at_armory", false)
        .build();

    // Action: Use health station
    let heal = Action::builder("use_health_station")
        .cost(2.0)
        .precondition("at_medical", true)
        .precondition("has_credits", 20)
        .effect_set_to("health", 100)
        .effect_subtract_int("has_credits", 20)
        .build();

    // Action: Buy armor
    let buy_armor = Action::builder("buy_armor")
        .cost(2.0)
        .precondition("at_armory", true)
        .precondition("has_credits", 30)
        .effect_set_to("armor", 50)
        .effect_subtract_int("has_credits", 30)
        .build();

    // Action: Buy ammo
    let buy_ammo = Action::builder("buy_ammo")
        .cost(1.5)
        .precondition("at_trading", true)
        .precondition("has_credits", 25)
        .effect_set_to("ammo", 50)
        .effect_subtract_int("has_credits", 25)
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
        plan_result.is_some(),
        "Expected to find a valid plan for combat preparation"
    );

    let (actions, total_cost) = plan_result.unwrap();

    println!("\nCombat Strategy Plan found with cost {}", total_cost);
    println!("Starting with 100 credits");
    for action in &actions {
        println!("- {} (cost: {})", action.name, action.cost);
    }

    // Verify we have the right number of actions (should include movement and purchases)
    assert!(
        actions.len() >= 5,
        "Expected at least 5 actions in the plan"
    );

    // Verify the plan contains all necessary actions (in any order)
    let action_names: Vec<_> = actions.iter().map(|a| a.name.as_str()).collect();

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
        total_cost >= 7.5,
        "Total cost should be at least 7.5 (movement + purchases)"
    );

    // Simulate plan execution to verify final state
    let mut current_state = initial_state_copy.clone();
    let mut total_credits = 100;

    for action in &actions {
        // Apply action effects
        current_state = action.apply_effect(&current_state);

        // Track credits spent
        if let Some(StateVar::I64(credits)) = current_state.get("has_credits") {
            total_credits = *credits;
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
    println!("Remaining credits: {}", total_credits);
    if let Some(StateVar::I64(health)) = current_state.get("health") {
        println!("Final health: {}", health);
    }
    if let Some(StateVar::I64(armor)) = current_state.get("armor") {
        println!("Final armor: {}", armor);
    }
    if let Some(StateVar::I64(ammo)) = current_state.get("ammo") {
        println!("Final ammo: {}", ammo);
    }
}
