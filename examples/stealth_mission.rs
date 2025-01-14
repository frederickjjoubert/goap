use goap::prelude::*;
use std::collections::HashMap;

fn main() {
    // Initial state - starting conditions for the stealth mission
    let mut initial_state = WorldState::new();
    initial_state.set("player_detected", StateVar::Bool(false));
    initial_state.set("guard_alert_level", StateVar::I64(0));
    initial_state.set("security_cameras_active", StateVar::Bool(true));
    initial_state.set("has_disguise", StateVar::Bool(false));
    initial_state.set("has_security_keycard", StateVar::Bool(false));
    initial_state.set("target_location_accessible", StateVar::Bool(false));
    initial_state.set("guard_patrol_memorized", StateVar::Bool(false));
    initial_state.set("hacking_tools_available", StateVar::Bool(false));
    initial_state.set("emp_charges", StateVar::I64(0));
    initial_state.set("distractions_available", StateVar::I64(0));
    initial_state.set("current_noise_level", StateVar::I64(0));
    initial_state.set("current_location", StateVar::Enum("entrance".to_string()));

    // Goal state - successfully infiltrate without being detected
    let mut goal_state = WorldState::new();
    goal_state.set("current_location", StateVar::Enum("vault".to_string()));
    goal_state.set("player_detected", StateVar::Bool(false));
    goal_state.set("target_location_accessible", StateVar::Bool(true));
    goal_state.set("guard_alert_level", StateVar::I64(0));

    let goal = Goal::new("stealth_infiltration", goal_state);

    // Create planner
    let mut planner = Planner::new();

    // Action: Observe Guard Patterns
    let mut observe_effects = HashMap::new();
    observe_effects.insert(
        "guard_patrol_memorized".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut observe_conditions = WorldState::new();
    observe_conditions.set("player_detected", StateVar::Bool(false));
    let observe_guards = Action::new(
        "observe_guard_patterns",
        2.0,
        observe_conditions,
        observe_effects,
    );

    // Action: Acquire Disguise
    let mut disguise_effects = HashMap::new();
    disguise_effects.insert(
        "has_disguise".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut disguise_conditions = WorldState::new();
    disguise_conditions.set("player_detected", StateVar::Bool(false));
    disguise_conditions.set("current_location", StateVar::Enum("entrance".to_string()));
    let acquire_disguise = Action::new(
        "acquire_disguise",
        3.0,
        disguise_conditions,
        disguise_effects,
    );

    // Action: Obtain Hacking Tools
    let mut hacking_effects = HashMap::new();
    hacking_effects.insert(
        "hacking_tools_available".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let obtain_tools = Action::new(
        "obtain_hacking_tools",
        2.0,
        WorldState::new(),
        hacking_effects,
    );

    // Action: Hack Security System
    let mut hack_effects = HashMap::new();
    hack_effects.insert(
        "security_cameras_active".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    hack_effects.insert(
        "has_security_keycard".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut hack_conditions = WorldState::new();
    hack_conditions.set("hacking_tools_available", StateVar::Bool(true));
    hack_conditions.set("player_detected", StateVar::Bool(false));
    let hack_security = Action::new("hack_security", 4.0, hack_conditions, hack_effects);

    // Action: Create Distraction
    let mut distract_effects = HashMap::new();
    distract_effects.insert("distractions_available".to_string(), StateOperation::Add(1));
    distract_effects.insert("current_noise_level".to_string(), StateOperation::Add(2));
    let create_distraction = Action::new(
        "create_distraction",
        1.0,
        WorldState::new(),
        distract_effects,
    );

    // Action: Move to Security Room
    let mut move_security_effects = HashMap::new();
    move_security_effects.insert(
        "current_location".to_string(),
        StateOperation::Set(StateVar::Enum("security_room".to_string())),
    );
    let mut move_security_conditions = WorldState::new();
    move_security_conditions.set("player_detected", StateVar::Bool(false));
    move_security_conditions.set("guard_patrol_memorized", StateVar::Bool(true));
    let move_to_security = Action::new(
        "move_to_security_room",
        2.0,
        move_security_conditions,
        move_security_effects,
    );

    // Action: Move to Vault
    let mut move_vault_effects = HashMap::new();
    move_vault_effects.insert(
        "current_location".to_string(),
        StateOperation::Set(StateVar::Enum("vault".to_string())),
    );
    move_vault_effects.insert(
        "target_location_accessible".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    let mut move_vault_conditions = WorldState::new();
    move_vault_conditions.set(
        "current_location",
        StateVar::Enum("security_room".to_string()),
    );
    move_vault_conditions.set("has_security_keycard", StateVar::Bool(true));
    move_vault_conditions.set("player_detected", StateVar::Bool(false));
    let move_to_vault = Action::new(
        "move_to_vault",
        3.0,
        move_vault_conditions,
        move_vault_effects,
    );

    // Add all actions to planner
    planner.add_action(observe_guards);
    planner.add_action(acquire_disguise);
    planner.add_action(obtain_tools);
    planner.add_action(hack_security);
    planner.add_action(create_distraction);
    planner.add_action(move_to_security);
    planner.add_action(move_to_vault);

    // Store a copy of initial state for later comparison
    let initial_state_copy = initial_state.clone();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal);
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
    let mut current_state = initial_state_copy;

    println!("\nSimulating plan execution:");
    for action in &actions {
        current_state = action.execute(&current_state);
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
