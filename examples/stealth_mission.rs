use goap::prelude::*;

fn main() {
    // Initial state - starting conditions for the stealth mission
    let initial_state = WorldState::builder()
        .bool("player_detected", false)
        .int("guard_alert_level", 0)
        .bool("security_cameras_active", true)
        .bool("has_disguise", false)
        .bool("has_security_keycard", false)
        .bool("target_location_accessible", false)
        .bool("guard_patrol_memorized", false)
        .bool("hacking_tools_available", false)
        .int("emp_charges", 0)
        .int("distractions_available", 0)
        .int("current_noise_level", 0)
        .enum_val("current_location", "entrance")
        .build();

    // Goal state - successfully infiltrate without being detected
    let goal = Goal::builder("stealth_infiltration")
        .require_enum("current_location", "vault")
        .require_bool("player_detected", false)
        .require_bool("target_location_accessible", true)
        .require_int("guard_alert_level", 0)
        .build();

    // Action: Observe Guard Patterns
    let observe_guards = Action::builder("observe_guard_patterns")
        .cost(2.0)
        .precondition("player_detected", false)
        .effect_set_to("guard_patrol_memorized", true)
        .build();

    // Action: Acquire Disguise
    let acquire_disguise = Action::builder("acquire_disguise")
        .cost(3.0)
        .precondition("player_detected", false)
        .precondition("current_location", "entrance")
        .effect_set_to("has_disguise", true)
        .build();

    // Action: Obtain Hacking Tools
    let obtain_tools = Action::builder("obtain_hacking_tools")
        .cost(2.0)
        .effect_set_to("hacking_tools_available", true)
        .build();

    // Action: Hack Security System
    let hack_security = Action::builder("hack_security")
        .cost(4.0)
        .precondition("hacking_tools_available", true)
        .precondition("player_detected", false)
        .effect_set_to("security_cameras_active", false)
        .effect_set_to("has_security_keycard", true)
        .build();

    // Action: Create Distraction
    let create_distraction = Action::builder("create_distraction")
        .cost(1.0)
        .effect_add_int("distractions_available", 1)
        .effect_add_int("current_noise_level", 2)
        .build();

    // Action: Move to Security Room
    let move_to_security = Action::builder("move_to_security_room")
        .cost(2.0)
        .precondition("player_detected", false)
        .precondition("guard_patrol_memorized", true)
        .effect_set_to("current_location", "security_room")
        .build();

    // Action: Move to Vault
    let move_to_vault = Action::builder("move_to_vault")
        .cost(3.0)
        .precondition("current_location", "security_room")
        .precondition("has_security_keycard", true)
        .precondition("player_detected", false)
        .effect_set_to("current_location", "vault")
        .effect_set_to("target_location_accessible", true)
        .build();

    // Collect all actions
    let actions = vec![
        observe_guards,
        acquire_disguise,
        obtain_tools,
        hack_security,
        create_distraction,
        move_to_security,
        move_to_vault,
    ];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);
    assert!(
        plan_result.is_some(),
        "Expected to find a valid plan for stealth mission"
    );

    let (actions, total_cost) = plan_result.unwrap();

    println!("\nStealth Mission Plan found with cost {}", total_cost);
    for action in &actions {
        println!("- {} (cost: {})", action.name, action.cost);
    }

    // Verify the plan contains necessary actions
    let action_names: Vec<_> = actions.iter().map(|a| a.name.as_str()).collect();

    // Verify preparation phase
    assert!(
        action_names.contains(&"observe_guard_patterns"),
        "Plan should include observing guard patterns"
    );
    assert!(
        action_names.contains(&"obtain_hacking_tools"),
        "Plan should include obtaining hacking tools"
    );

    // Verify infiltration phase
    assert!(
        action_names.contains(&"hack_security"),
        "Plan should include hacking security"
    );
    assert!(
        action_names.contains(&"move_to_security_room"),
        "Plan should include moving to security room"
    );
    assert!(
        action_names.contains(&"move_to_vault"),
        "Plan should include moving to vault"
    );

    // Simulate plan execution to verify final state
    let mut current_state = initial_state.clone();

    println!("\nSimulating plan execution:");
    for action in &actions {
        current_state = action.apply_effect(&current_state);
        println!("After {}: ", action.name);
        if let Some(StateVar::Bool(detected)) = current_state.get("player_detected") {
            println!("  Player Detected: {}", detected);
        }
        if let Some(StateVar::I64(alert)) = current_state.get("guard_alert_level") {
            println!("  Guard Alert Level: {}", alert);
        }
        if let Some(StateVar::Enum(location)) = current_state.get("current_location") {
            println!("  Current Location: {}", location);
        }
        if let Some(StateVar::Bool(accessible)) = current_state.get("target_location_accessible") {
            println!("  Target Location Accessible: {}", accessible);
        }
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );

    // Verify specific final conditions
    if let Some(StateVar::Bool(detected)) = current_state.get("player_detected") {
        assert!(!detected, "Player should not be detected");
    }
    if let Some(StateVar::I64(alert)) = current_state.get("guard_alert_level") {
        assert!(*alert == 0, "Guard alert level should be 0");
    }
    if let Some(StateVar::Enum(location)) = current_state.get("current_location") {
        assert!(location == "vault", "Should reach the vault");
    }
    if let Some(StateVar::Bool(accessible)) = current_state.get("target_location_accessible") {
        assert!(*accessible, "Target location should be accessible");
    }

    println!("\nFinal mission state verification:");
    println!("Location: {:?}", current_state.get("current_location"));
    println!(
        "Target Accessible: {:?}",
        current_state.get("target_location_accessible")
    );
    println!(
        "Player Detected: {:?}",
        current_state.get("player_detected")
    );
    println!(
        "Guard Alert Level: {:?}",
        current_state.get("guard_alert_level")
    );
}
