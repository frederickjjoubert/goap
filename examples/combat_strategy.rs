use std::collections::HashMap;

use goap::prelude::*;

fn main() {
    // Initial state - player is not ready for combat
    let mut initial_state = WorldState::new();
    initial_state.set("health", StateVar::I64(30)); // Low health (30/100)
    initial_state.set("armor", StateVar::I64(0)); // No armor
    initial_state.set("ammo", StateVar::I64(5)); // Low ammo
    initial_state.set("at_medical", StateVar::Bool(false));
    initial_state.set("at_armory", StateVar::Bool(false));
    initial_state.set("at_trading", StateVar::Bool(false));
    initial_state.set("has_credits", StateVar::I64(100)); // Starting credits

    // Goal state - ready for boss battle
    let mut goal_state = WorldState::new();
    goal_state.set("health", StateVar::I64(100)); // Full health
    goal_state.set("armor", StateVar::I64(50)); // Decent armor
    goal_state.set("ammo", StateVar::I64(50)); // Sufficient ammo

    let goal = Goal::new("prepare_for_boss", goal_state);

    // Create actions
    let mut planner = Planner::new();

    // Action: Move to medical bay
    let mut goto_medical_effects = HashMap::new();
    goto_medical_effects.insert(
        "at_medical".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_medical_effects.insert(
        "at_armory".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_medical_effects.insert(
        "at_trading".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_medical = Action::new("goto_medical", 1.0, WorldState::new(), goto_medical_effects);

    // Action: Move to armory
    let mut goto_armory_effects = HashMap::new();
    goto_armory_effects.insert(
        "at_armory".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_armory_effects.insert(
        "at_medical".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_armory_effects.insert(
        "at_trading".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_armory = Action::new("goto_armory", 1.0, WorldState::new(), goto_armory_effects);

    // Action: Move to trading post
    let mut goto_trading_effects = HashMap::new();
    goto_trading_effects.insert(
        "at_trading".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_trading_effects.insert(
        "at_medical".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_trading_effects.insert(
        "at_armory".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_trading = Action::new("goto_trading", 1.0, WorldState::new(), goto_trading_effects);

    // Action: Use health station
    let mut heal_effects = HashMap::new();
    heal_effects.insert(
        "health".to_string(),
        StateOperation::Set(StateVar::I64(100)),
    );
    heal_effects.insert("has_credits".to_string(), StateOperation::Subtract(20));
    let mut heal_conditions = WorldState::new();
    heal_conditions.set("at_medical", StateVar::Bool(true));
    heal_conditions.set("has_credits", StateVar::I64(20));
    let heal = Action::new("use_health_station", 2.0, heal_conditions, heal_effects);

    // Action: Buy armor
    let mut buy_armor_effects = HashMap::new();
    buy_armor_effects.insert("armor".to_string(), StateOperation::Set(StateVar::I64(50)));
    buy_armor_effects.insert("has_credits".to_string(), StateOperation::Subtract(30));
    let mut buy_armor_conditions = WorldState::new();
    buy_armor_conditions.set("at_armory", StateVar::Bool(true));
    buy_armor_conditions.set("has_credits", StateVar::I64(30));
    let buy_armor = Action::new("buy_armor", 2.0, buy_armor_conditions, buy_armor_effects);

    // Action: Buy ammo
    let mut buy_ammo_effects = HashMap::new();
    buy_ammo_effects.insert("ammo".to_string(), StateOperation::Set(StateVar::I64(50)));
    buy_ammo_effects.insert("has_credits".to_string(), StateOperation::Subtract(25));
    let mut buy_ammo_conditions = WorldState::new();
    buy_ammo_conditions.set("at_trading", StateVar::Bool(true));
    buy_ammo_conditions.set("has_credits", StateVar::I64(25));
    let buy_ammo = Action::new("buy_ammo", 1.5, buy_ammo_conditions, buy_ammo_effects);

    // Add all actions to planner
    planner.add_action(goto_medical);
    planner.add_action(goto_armory);
    planner.add_action(goto_trading);
    planner.add_action(heal);
    planner.add_action(buy_armor);
    planner.add_action(buy_ammo);

    // Store a copy of initial state for later comparison
    let initial_state_copy = initial_state.clone();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal);
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
        current_state = action.execute(&current_state);

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
