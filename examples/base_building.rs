use goap::prelude::*;

fn main() {
    // Initial state - empty plot with basic resources
    let initial_state = WorldState::builder()
        .int("metal", 0) // Basic building material
        .int("energy", 0) // Power resource
        .int("components", 0) // Crafted from metal
        .int("battery_charge", 0) // Stored energy
        .int("defense_rating", 0) // Base defense level
        .bool("has_mine", false)
        .bool("has_factory", false)
        .bool("has_solar", false)
        .bool("has_battery", false)
        .bool("has_walls", false)
        .bool("has_turrets", false)
        .build();

    // Goal state - functional base with power and defenses
    let goal = Goal::builder("build_defensive_base")
        .require_int("energy", 50) // Sustainable power generation
        .require_int("battery_charge", 100) // Energy storage
        .require_int("defense_rating", 75) // Adequate defense
        .require_bool("has_walls", true) // Basic defense
        .require_bool("has_turrets", true) // Active defense
        .build();

    // Action: Build Mining Facility
    let build_mine = Action::builder("build_mine")
        .cost(3.0)
        .effect_set_to("has_mine", true)
        .effect_add_int("metal", 20) // Initial metal bonus
        .build();

    // Action: Mine Resources
    let mine_resources = Action::builder("mine_resources")
        .cost(2.0)
        .precondition("has_mine", true)
        .effect_add_int("metal", 15)
        .build();

    // Action: Build Factory
    let build_factory = Action::builder("build_factory")
        .cost(4.0)
        .precondition("metal", 20)
        .effect_set_to("has_factory", true)
        .effect_subtract_int("metal", 20)
        .build();

    // Action: Craft Components
    let craft_components = Action::builder("craft_components")
        .cost(2.0)
        .precondition("has_factory", true)
        .precondition("metal", 5)
        .effect_add_int("components", 10)
        .effect_subtract_int("metal", 5)
        .build();

    // Action: Build Solar Panels
    let build_solar = Action::builder("build_solar_panels")
        .cost(3.0)
        .precondition("components", 5)
        .effect_set_to("has_solar", true)
        .effect_add_int("energy", 30)
        .effect_subtract_int("components", 5)
        .build();

    // Action: Build Battery
    let build_battery = Action::builder("build_battery")
        .cost(3.0)
        .precondition("components", 8)
        .effect_set_to("has_battery", true)
        .effect_subtract_int("components", 8)
        .build();

    // Action: Charge Battery
    let charge_battery = Action::builder("charge_battery")
        .cost(2.0)
        .precondition("has_battery", true)
        .precondition("has_solar", true)
        .precondition("energy", 25)
        .effect_add_int("battery_charge", 50)
        .effect_subtract_int("energy", 25)
        .build();

    // Action: Build Walls
    let build_walls = Action::builder("build_walls")
        .cost(4.0)
        .precondition("metal", 15)
        .effect_set_to("has_walls", true)
        .effect_add_int("defense_rating", 30)
        .effect_subtract_int("metal", 15)
        .build();

    // Action: Build Turrets
    let build_turrets = Action::builder("build_turrets")
        .cost(5.0)
        .precondition("components", 10)
        .precondition("has_walls", true)
        .precondition("energy", 20)
        .effect_set_to("has_turrets", true)
        .effect_add_int("defense_rating", 45)
        .effect_subtract_int("components", 10)
        .build();

    // Collect all actions
    let actions = vec![
        build_mine,
        mine_resources,
        build_factory,
        craft_components,
        build_solar,
        build_battery,
        charge_battery,
        build_walls,
        build_turrets,
    ];

    // Store a copy of initial state for later comparison
    let initial_state_copy = initial_state.clone();

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal, &actions);
    assert!(
        plan_result.is_some(),
        "Expected to find a valid plan for base building"
    );

    let (actions, total_cost) = plan_result.unwrap();

    println!("\nBase Building Plan found with cost {}", total_cost);
    for action in &actions {
        println!("- {} (cost: {})", action.name, action.cost);
    }

    // Verify the plan contains all necessary actions (in any order)
    let action_names: Vec<_> = actions.iter().map(|a| a.name.as_str()).collect();

    // Verify resource gathering chain
    assert!(
        action_names.contains(&"build_mine"),
        "Plan should include building a mine"
    );
    assert!(
        action_names.contains(&"mine_resources"),
        "Plan should include mining resources"
    );

    // Verify manufacturing chain
    assert!(
        action_names.contains(&"build_factory"),
        "Plan should include building a factory"
    );
    assert!(
        action_names.contains(&"craft_components"),
        "Plan should include crafting components"
    );

    // Verify power infrastructure
    assert!(
        action_names.contains(&"build_solar_panels"),
        "Plan should include building solar panels"
    );
    assert!(
        action_names.contains(&"build_battery"),
        "Plan should include building battery"
    );
    assert!(
        action_names.contains(&"charge_battery"),
        "Plan should include charging battery"
    );

    // Verify defense construction
    assert!(
        action_names.contains(&"build_walls"),
        "Plan should include building walls"
    );
    assert!(
        action_names.contains(&"build_turrets"),
        "Plan should include building turrets"
    );

    // Simulate plan execution to verify final state
    let mut current_state = initial_state_copy;

    println!("\nSimulating plan execution:");
    for action in &actions {
        current_state = action.apply_effect(&current_state);
        println!("After {}: ", action.name);
        if let Some(StateVar::I64(metal)) = current_state.get("metal") {
            println!("  Metal: {}", metal);
        }
        if let Some(StateVar::I64(energy)) = current_state.get("energy") {
            println!("  Energy: {}", energy);
        }
        if let Some(StateVar::I64(components)) = current_state.get("components") {
            println!("  Components: {}", components);
        }
        if let Some(StateVar::I64(defense)) = current_state.get("defense_rating") {
            println!("  Defense Rating: {}", defense);
        }
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );

    // Verify specific final state conditions
    if let Some(StateVar::I64(final_defense)) = current_state.get("defense_rating") {
        assert!(*final_defense >= 75, "Defense rating should be at least 75");
    }
    if let Some(StateVar::I64(final_energy)) = current_state.get("energy") {
        assert!(
            *final_energy >= 50,
            "Energy production should be at least 50"
        );
    }
    if let Some(StateVar::I64(final_battery)) = current_state.get("battery_charge") {
        assert!(
            *final_battery >= 100,
            "Battery charge should be at least 100"
        );
    }

    println!("\nFinal base state verification:");
    println!("Energy Production: {:?}", current_state.get("energy"));
    println!("Battery Charge: {:?}", current_state.get("battery_charge"));
    println!("Defense Rating: {:?}", current_state.get("defense_rating"));
    println!("Has Walls: {:?}", current_state.get("has_walls"));
    println!("Has Turrets: {:?}", current_state.get("has_turrets"));
}
