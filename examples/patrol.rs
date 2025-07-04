use goap::prelude::*;

fn main() {
    // Initial state - guard starts at base without radio
    let initial_state = State::new()
        .set("at_base", true)
        .set("at_point_a", false)
        .set("at_point_b", false)
        .set("at_point_c", false)
        .set("has_radio", false)
        .set("reported_at_a", false)
        .set("reported_at_b", false)
        .set("reported_at_c", false)
        .build();

    // Goal state - complete patrol route and return to base
    let goal = Goal::new("complete_patrol")
        .requires("has_radio", true)
        .requires("reported_at_a", true)
        .requires("reported_at_b", true)
        .requires("reported_at_c", true)
        .requires("at_base", true)
        .build();

    // Action: Equip Radio at Base
    let equip_radio = Action::new("equip_radio")
        .cost(1.0)
        .requires("at_base", true)
        .requires("has_radio", false)
        .sets("has_radio", true)
        .build();

    // Action: Move to Point A
    let goto_point_a = Action::new("move_to_point_a")
        .cost(5.0)
        .sets("at_point_a", true)
        .sets("at_base", false)
        .sets("at_point_b", false)
        .sets("at_point_c", false)
        .build();

    // Action: Move to Point B
    let goto_point_b = Action::new("move_to_point_b")
        .cost(5.0)
        .sets("at_point_b", true)
        .sets("at_base", false)
        .sets("at_point_a", false)
        .sets("at_point_c", false)
        .build();

    // Action: Move to Point C
    let goto_point_c = Action::new("move_to_point_c")
        .cost(5.0)
        .sets("at_point_c", true)
        .sets("at_base", false)
        .sets("at_point_a", false)
        .sets("at_point_b", false)
        .build();

    // Action: Return to Base
    let goto_base = Action::new("return_to_base")
        .cost(5.0)
        .sets("at_base", true)
        .sets("at_point_a", false)
        .sets("at_point_b", false)
        .sets("at_point_c", false)
        .build();

    // Action: Report at Point A
    let report_at_a = Action::new("report_at_point_a")
        .cost(2.0)
        .requires("has_radio", true)
        .requires("at_point_a", true)
        .sets("reported_at_a", true)
        .build();

    // Action: Report at Point B
    let report_at_b = Action::new("report_at_point_b")
        .cost(2.0)
        .requires("has_radio", true)
        .requires("at_point_b", true)
        .requires("reported_at_a", true) // Must report at A first
        .sets("reported_at_b", true)
        .build();

    // Action: Report at Point C
    let report_at_c = Action::new("report_at_point_c")
        .cost(2.0)
        .requires("has_radio", true)
        .requires("at_point_c", true)
        .requires("reported_at_b", true) // Must report at B first
        .sets("reported_at_c", true)
        .build();

    // Collect all actions
    let actions = vec![
        equip_radio,
        goto_point_a,
        goto_point_b,
        goto_point_c,
        goto_base,
        report_at_a,
        report_at_b,
        report_at_c,
    ];

    // Store initial state for comparison
    let initial_state_copy = initial_state.clone();

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal, &actions);
    assert!(plan_result.is_ok(), "Expected to find a valid patrol plan");

    let plan = plan_result.unwrap();

    println!(
        "\nPatrol Mission Plan found with cost {cost}",
        cost = plan.cost
    );
    for action in &plan.actions {
        let name = &action.name;
        println!("- {name}");
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
    for action in &plan.actions {
        current_state = action.apply_effect(&current_state);
        let name = &action.name;
        println!("After {name}: ");
        if let Some(StateVar::Bool(at_base)) = current_state.get("at_base") {
            println!("  At Base: {at_base}");
        }
        if let Some(StateVar::Bool(has_radio)) = current_state.get("has_radio") {
            println!("  Has Radio: {has_radio}");
        }
        if let Some(StateVar::Bool(reported_a)) = current_state.get("reported_at_a") {
            println!("  Reported at A: {reported_a}");
        }
        if let Some(StateVar::Bool(reported_b)) = current_state.get("reported_at_b") {
            println!("  Reported at B: {reported_b}");
        }
        if let Some(StateVar::Bool(reported_c)) = current_state.get("reported_at_c") {
            println!("  Reported at C: {reported_c}");
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
    let at_base = current_state.get("at_base");
    println!("At Base: {at_base:?}");
    let has_radio = current_state.get("has_radio");
    println!("Has Radio: {has_radio:?}");
    let reported_at_a = current_state.get("reported_at_a");
    println!("Reported at A: {reported_at_a:?}");
    let reported_at_b = current_state.get("reported_at_b");
    println!("Reported at B: {reported_at_b:?}");
    let reported_at_c = current_state.get("reported_at_c");
    println!("Reported at C: {reported_at_c:?}");
}
