use goap::prelude::*;

fn main() {
    // Initial state - starting conditions for the stealth mission
    let initial_state = State::new()
        .set("player_detected", false)
        .set("guard_alert_level", 0)
        .set("security_cameras_active", true)
        .set("has_disguise", false)
        .set("has_security_keycard", false)
        .set("target_location_accessible", false)
        .set("guard_patrol_memorized", false)
        .set("hacking_tools_available", false)
        .set("emp_charges", 0)
        .set("distractions_available", 0)
        .set("current_noise_level", 0)
        .set("current_location", "entrance")
        .build();

    // Goal state - successfully infiltrate without being detected
    let goal = Goal::new("stealth_infiltration")
        .requires("current_location", "vault")
        .requires("player_detected", false)
        .requires("target_location_accessible", true)
        .requires("guard_alert_level", 0)
        .build();

    // Action: Observe Guard Patterns
    let observe_guards = Action::new("observe_guard_patterns")
        .cost(2.0)
        .requires("player_detected", false)
        .sets("guard_patrol_memorized", true)
        .build();

    // Action: Acquire Disguise
    let acquire_disguise = Action::new("acquire_disguise")
        .cost(3.0)
        .requires("player_detected", false)
        .requires("current_location", "entrance")
        .sets("has_disguise", true)
        .build();

    // Action: Obtain Hacking Tools
    let obtain_tools = Action::new("obtain_hacking_tools")
        .cost(2.0)
        .sets("hacking_tools_available", true)
        .build();

    // Action: Hack Security System
    let hack_security = Action::new("hack_security")
        .cost(4.0)
        .requires("hacking_tools_available", true)
        .requires("player_detected", false)
        .sets("security_cameras_active", false)
        .sets("has_security_keycard", true)
        .build();

    // Action: Create Distraction
    let create_distraction = Action::new("create_distraction")
        .cost(1.0)
        .adds("distractions_available", 1)
        .adds("current_noise_level", 2)
        .build();

    // Action: Move to Security Room
    let move_to_security = Action::new("move_to_security_room")
        .cost(2.0)
        .requires("player_detected", false)
        .requires("guard_patrol_memorized", true)
        .sets("current_location", "security_room")
        .build();

    // Action: Move to Vault
    let move_to_vault = Action::new("move_to_vault")
        .cost(3.0)
        .requires("current_location", "security_room")
        .requires("has_security_keycard", true)
        .requires("player_detected", false)
        .sets("current_location", "vault")
        .sets("target_location_accessible", true)
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
        plan_result.is_ok(),
        "Expected to find a valid plan for stealth mission"
    );

    let plan = plan_result.unwrap();

    println!(
        "\nStealth Mission Plan found with cost {cost}",
        cost = plan.cost
    );
    for action in &plan.actions {
        println!(
            "- {name} (cost: {cost})",
            name = action.name,
            cost = action.cost
        );
    }

    // Verify the plan contains necessary actions
    let action_names: Vec<_> = plan.actions.iter().map(|a| a.name.as_str()).collect();

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
    for action in &plan.actions {
        current_state = action.apply_effect(&current_state);
        let name = &action.name;
        println!("After {name}: ");
        if let Some(detected) = current_state.get::<bool>("player_detected") {
            println!("  Player Detected: {detected}");
        }
        if let Some(alert) = current_state.get::<i64>("guard_alert_level") {
            println!("  Guard Alert Level: {alert}");
        }
        if let Some(location) = current_state.get::<String>("current_location") {
            println!("  Current Location: {location}");
        }
        if let Some(accessible) = current_state.get::<bool>("target_location_accessible") {
            println!("  Target Location Accessible: {accessible}");
        }
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );

    // Verify specific final conditions
    if let Some(detected) = current_state.get::<bool>("player_detected") {
        assert!(!detected, "Player should not be detected");
    }
    if let Some(alert) = current_state.get::<i64>("guard_alert_level") {
        assert!(alert == 0, "Guard alert level should be 0");
    }
    if let Some(location) = current_state.get::<String>("current_location") {
        assert!(location == "vault", "Should reach the vault");
    }
    if let Some(accessible) = current_state.get::<bool>("target_location_accessible") {
        assert!(accessible, "Target location should be accessible");
    }

    println!("\nFinal mission state verification:");
    let location = current_state.get::<String>("current_location");
    println!("Location: {location:?}");
    let target_accessible = current_state.get::<bool>("target_location_accessible");
    println!("Target Accessible: {target_accessible:?}");
    let player_detected = current_state.get::<bool>("player_detected");
    println!("Player Detected: {player_detected:?}");
    let guard_alert_level = current_state.get::<i64>("guard_alert_level");
    println!("Guard Alert Level: {guard_alert_level:?}");
}
