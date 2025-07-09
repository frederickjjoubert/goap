use goap::prelude::*;

fn main() {
    // Initial state - empty plot with basic resources
    let initial_state = State::new()
        .set("metal", 0) // Basic building material
        .set("energy", 0) // Power resource
        .set("components", 0) // Crafted from metal
        .set("battery_charge", 0) // Stored energy
        .set("defense_rating", 0) // Base defense level
        .set("has_mine", false)
        .set("has_factory", false)
        .set("has_solar", false)
        .set("has_battery", false)
        .set("has_walls", false)
        .set("has_turrets", false)
        .build();

    // Goal state - functional base with power and defenses
    let goal = Goal::new("build_defensive_base")
        .requires("energy", 50) // Sustainable power generation
        .requires("battery_charge", 100) // Energy storage
        .requires("defense_rating", 75) // Adequate defense
        .requires("has_walls", true) // Basic defense
        .requires("has_turrets", true) // Active defense
        .build();

    // Action: Build Mining Facility
    let build_mine = Action::new("build_mine")
        .cost(3.0)
        .sets("has_mine", true)
        .adds("metal", 20) // Initial metal bonus
        .build();

    // Action: Mine Resources
    let mine_resources = Action::new("mine_resources")
        .cost(2.0)
        .requires("has_mine", true)
        .adds("metal", 15)
        .build();

    // Action: Build Factory
    let build_factory = Action::new("build_factory")
        .cost(4.0)
        .requires("metal", 20)
        .sets("has_factory", true)
        .subtracts("metal", 20)
        .build();

    // Action: Craft Components
    let craft_components = Action::new("craft_components")
        .cost(2.0)
        .requires("has_factory", true)
        .requires("metal", 5)
        .adds("components", 10)
        .subtracts("metal", 5)
        .build();

    // Action: Build Solar Panels
    let build_solar = Action::new("build_solar_panels")
        .cost(3.0)
        .requires("components", 5)
        .sets("has_solar", true)
        .adds("energy", 30)
        .subtracts("components", 5)
        .build();

    // Action: Build Battery
    let build_battery = Action::new("build_battery")
        .cost(3.0)
        .requires("components", 8)
        .sets("has_battery", true)
        .subtracts("components", 8)
        .build();

    // Action: Charge Battery
    let charge_battery = Action::new("charge_battery")
        .cost(2.0)
        .requires("has_battery", true)
        .requires("has_solar", true)
        .requires("energy", 25)
        .adds("battery_charge", 50)
        .subtracts("energy", 25)
        .build();

    // Action: Build Walls
    let build_walls = Action::new("build_walls")
        .cost(4.0)
        .requires("metal", 15)
        .sets("has_walls", true)
        .adds("defense_rating", 30)
        .subtracts("metal", 15)
        .build();

    // Action: Build Turrets
    let build_turrets = Action::new("build_turrets")
        .cost(5.0)
        .requires("components", 10)
        .requires("has_walls", true)
        .requires("energy", 20)
        .sets("has_turrets", true)
        .adds("defense_rating", 45)
        .subtracts("components", 10)
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
        plan_result.is_ok(),
        "Expected to find a valid plan for base building"
    );

    let plan = plan_result.unwrap();

    println!(
        "\nBase Building Plan found with cost {cost}",
        cost = plan.cost
    );
    for action in &plan.actions {
        println!(
            "- {name} (cost: {cost})",
            name = action.name,
            cost = action.cost
        );
    }

    // Verify the plan contains all necessary actions (in any order)
    let action_names: Vec<_> = plan.actions.iter().map(|a| a.name.as_str()).collect();

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
    for action in &plan.actions {
        current_state = action.apply_effect(&current_state);
        println!("After {name}: ", name = action.name);
        if let Some(metal) = current_state.get::<i64>("metal") {
            println!("  Metal: {metal}");
        }
        if let Some(energy) = current_state.get::<i64>("energy") {
            println!("  Energy: {energy}");
        }
        if let Some(components) = current_state.get::<i64>("components") {
            println!("  Components: {components}");
        }
        if let Some(defense) = current_state.get::<i64>("defense_rating") {
            println!("  Defense Rating: {defense}");
        }
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );

    // Verify specific final state conditions
    if let Some(final_defense) = current_state.get::<i64>("defense_rating") {
        assert!(final_defense >= 75, "Defense rating should be at least 75");
    }
    if let Some(final_energy) = current_state.get::<i64>("energy") {
        assert!(final_energy >= 50, "Energy production should be at least 50");
    }
    if let Some(final_battery) = current_state.get::<i64>("battery_charge") {
        assert!(final_battery >= 100, "Battery charge should be at least 100");
    }

    println!("\nFinal base state verification:");
    let energy_prod = current_state.get::<i64>("energy");
    println!("Energy Production: {energy_prod:?}");
    let battery_charge = current_state.get::<i64>("battery_charge");
    println!("Battery Charge: {battery_charge:?}");
    let defense_rating = current_state.get::<i64>("defense_rating");
    println!("Defense Rating: {defense_rating:?}");
    let has_walls = current_state.get::<bool>("has_walls");
    println!("Has Walls: {has_walls:?}");
    let has_turrets = current_state.get::<bool>("has_turrets");
    println!("Has Turrets: {has_turrets:?}");
}
