use std::collections::HashMap;

use goap::prelude::*;

fn main() {
    // Initial state - guard starts at base without radio
    let mut initial_state = WorldState::new();
    initial_state.set("at_base", StateVar::Bool(true));
    initial_state.set("at_point_a", StateVar::Bool(false));
    initial_state.set("at_point_b", StateVar::Bool(false));
    initial_state.set("at_point_c", StateVar::Bool(false));
    initial_state.set("has_radio", StateVar::Bool(false));
    initial_state.set("reported_at_a", StateVar::Bool(false));
    initial_state.set("reported_at_b", StateVar::Bool(false));
    initial_state.set("reported_at_c", StateVar::Bool(false));

    // Goal state - complete patrol route and return to base
    let mut goal_state = WorldState::new();
    goal_state.set("has_radio", StateVar::Bool(true));
    goal_state.set("reported_at_a", StateVar::Bool(true));
    goal_state.set("reported_at_b", StateVar::Bool(true));
    goal_state.set("reported_at_c", StateVar::Bool(true));
    goal_state.set("at_base", StateVar::Bool(true));

    let goal = Goal::new("complete_patrol", goal_state);

    // Create planner
    let mut planner = Planner::new();

    // Action: Equip Radio at Base
    let mut equip_radio_effects = HashMap::new();
    equip_radio_effects.insert(
        "has_radio".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut equip_radio_conditions = WorldState::new();
    equip_radio_conditions.set("at_base", StateVar::Bool(true));
    equip_radio_conditions.set("has_radio", StateVar::Bool(false));
    let equip_radio = Action::new(
        "equip_radio",
        1.0,
        equip_radio_conditions,
        equip_radio_effects,
    );

    // Action: Move to Point A
    let mut goto_a_effects = HashMap::new();
    goto_a_effects.insert(
        "at_point_a".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_a_effects.insert(
        "at_base".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_a_effects.insert(
        "at_point_b".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_a_effects.insert(
        "at_point_c".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_point_a = Action::new("move_to_point_a", 5.0, WorldState::new(), goto_a_effects);

    // Action: Move to Point B
    let mut goto_b_effects = HashMap::new();
    goto_b_effects.insert(
        "at_point_b".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_b_effects.insert(
        "at_base".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_b_effects.insert(
        "at_point_a".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_b_effects.insert(
        "at_point_c".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_point_b = Action::new("move_to_point_b", 5.0, WorldState::new(), goto_b_effects);

    // Action: Move to Point C
    let mut goto_c_effects = HashMap::new();
    goto_c_effects.insert(
        "at_point_c".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_c_effects.insert(
        "at_base".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_c_effects.insert(
        "at_point_a".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_c_effects.insert(
        "at_point_b".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_point_c = Action::new("move_to_point_c", 5.0, WorldState::new(), goto_c_effects);

    // Action: Return to Base
    let mut goto_base_effects = HashMap::new();
    goto_base_effects.insert(
        "at_base".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    goto_base_effects.insert(
        "at_point_a".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_base_effects.insert(
        "at_point_b".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    goto_base_effects.insert(
        "at_point_c".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    let goto_base = Action::new("return_to_base", 5.0, WorldState::new(), goto_base_effects);

    // Action: Report at Point A
    let mut report_a_effects = HashMap::new();
    report_a_effects.insert(
        "reported_at_a".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut report_a_conditions = WorldState::new();
    report_a_conditions.set("has_radio", StateVar::Bool(true));
    report_a_conditions.set("at_point_a", StateVar::Bool(true));
    let report_at_a = Action::new(
        "report_at_point_a",
        2.0,
        report_a_conditions,
        report_a_effects,
    );

    // Action: Report at Point B
    let mut report_b_effects = HashMap::new();
    report_b_effects.insert(
        "reported_at_b".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut report_b_conditions = WorldState::new();
    report_b_conditions.set("has_radio", StateVar::Bool(true));
    report_b_conditions.set("at_point_b", StateVar::Bool(true));
    report_b_conditions.set("reported_at_a", StateVar::Bool(true)); // Must report at A first
    let report_at_b = Action::new(
        "report_at_point_b",
        2.0,
        report_b_conditions,
        report_b_effects,
    );

    // Action: Report at Point C
    let mut report_c_effects = HashMap::new();
    report_c_effects.insert(
        "reported_at_c".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut report_c_conditions = WorldState::new();
    report_c_conditions.set("has_radio", StateVar::Bool(true));
    report_c_conditions.set("at_point_c", StateVar::Bool(true));
    report_c_conditions.set("reported_at_b", StateVar::Bool(true)); // Must report at B first
    let report_at_c = Action::new(
        "report_at_point_c",
        2.0,
        report_c_conditions,
        report_c_effects,
    );

    // Add all actions to planner
    planner.add_action(equip_radio);
    planner.add_action(goto_point_a);
    planner.add_action(goto_point_b);
    planner.add_action(goto_point_c);
    planner.add_action(goto_base);
    planner.add_action(report_at_a);
    planner.add_action(report_at_b);
    planner.add_action(report_at_c);

    // Store initial state for comparison
    let initial_state_copy = initial_state.clone();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal);
    assert!(
        plan_result.is_some(),
        "Expected to find a valid patrol plan"
    );

    let (actions, total_cost) = plan_result.unwrap();

    println!("\nPatrol Mission Plan found with cost {}", total_cost);
    for action in &actions {
        println!("- {}", action.name);
    }

    // Verify the plan contains necessary actions
    let action_names: Vec<_> = actions.iter().map(|a| a.name.as_str()).collect();

    // Verify equipment phase
    assert!(
        action_names.contains(&"equip_radio"),
        "Plan should include equipping radio"
    );

    // Verify patrol sequence
    assert!(
        action_names.contains(&"move_to_point_a"),
        "Plan should include moving to point A"
    );
    assert!(
        action_names.contains(&"report_at_point_a"),
        "Plan should include reporting at point A"
    );
    assert!(
        action_names.contains(&"move_to_point_b"),
        "Plan should include moving to point B"
    );
    assert!(
        action_names.contains(&"report_at_point_b"),
        "Plan should include reporting at point B"
    );
    assert!(
        action_names.contains(&"move_to_point_c"),
        "Plan should include moving to point C"
    );
    assert!(
        action_names.contains(&"report_at_point_c"),
        "Plan should include reporting at point C"
    );
    assert!(
        action_names.contains(&"return_to_base"),
        "Plan should include returning to base"
    );

    // Simulate plan execution to verify final state
    let mut current_state = initial_state_copy;

    println!("\nSimulating plan execution:");
    for action in &actions {
        current_state = action.execute(&current_state);
        println!("After {}: ", action.name);
        if let Some(StateVar::Bool(at_base)) = current_state.get("at_base") {
            println!("  At Base: {}", at_base);
        }
        if let Some(StateVar::Bool(has_radio)) = current_state.get("has_radio") {
            println!("  Has Radio: {}", has_radio);
        }
        if let Some(StateVar::Bool(reported_a)) = current_state.get("reported_at_a") {
            println!("  Reported at A: {}", reported_a);
        }
        if let Some(StateVar::Bool(reported_b)) = current_state.get("reported_at_b") {
            println!("  Reported at B: {}", reported_b);
        }
        if let Some(StateVar::Bool(reported_c)) = current_state.get("reported_at_c") {
            println!("  Reported at C: {}", reported_c);
        }
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );

    // Verify specific final conditions
    if let Some(StateVar::Bool(at_base)) = current_state.get("at_base") {
        assert!(*at_base, "Guard should end at base");
    }
    if let Some(StateVar::Bool(has_radio)) = current_state.get("has_radio") {
        assert!(*has_radio, "Guard should have radio");
    }
    if let Some(StateVar::Bool(reported_a)) = current_state.get("reported_at_a") {
        assert!(*reported_a, "Guard should have reported at point A");
    }
    if let Some(StateVar::Bool(reported_b)) = current_state.get("reported_at_b") {
        assert!(*reported_b, "Guard should have reported at point B");
    }
    if let Some(StateVar::Bool(reported_c)) = current_state.get("reported_at_c") {
        assert!(*reported_c, "Guard should have reported at point C");
    }

    println!("\nFinal patrol state verification:");
    println!("At Base: {:?}", current_state.get("at_base"));
    println!("Has Radio: {:?}", current_state.get("has_radio"));
    println!("Reported at A: {:?}", current_state.get("reported_at_a"));
    println!("Reported at B: {:?}", current_state.get("reported_at_b"));
    println!("Reported at C: {:?}", current_state.get("reported_at_c"));
}
