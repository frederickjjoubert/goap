use std::collections::HashMap;

use goap::prelude::*;

fn main() {
    // Initial state - empty plot with basic resources
    let mut initial_state = WorldState::new();
    initial_state.set("metal", StateVar::I64(0)); // Basic building material
    initial_state.set("energy", StateVar::I64(0)); // Power resource
    initial_state.set("components", StateVar::I64(0)); // Crafted from metal
    initial_state.set("battery_charge", StateVar::I64(0)); // Stored energy
    initial_state.set("defense_rating", StateVar::I64(0)); // Base defense level
    initial_state.set("has_mine", StateVar::Bool(false));
    initial_state.set("has_factory", StateVar::Bool(false));
    initial_state.set("has_solar", StateVar::Bool(false));
    initial_state.set("has_battery", StateVar::Bool(false));
    initial_state.set("has_walls", StateVar::Bool(false));
    initial_state.set("has_turrets", StateVar::Bool(false));

    // Goal state - functional base with power and defenses
    let mut goal_state = WorldState::new();
    goal_state.set("energy", StateVar::I64(50)); // Sustainable power generation
    goal_state.set("battery_charge", StateVar::I64(100)); // Energy storage
    goal_state.set("defense_rating", StateVar::I64(75)); // Adequate defense
    goal_state.set("has_walls", StateVar::Bool(true)); // Basic defense
    goal_state.set("has_turrets", StateVar::Bool(true)); // Active defense

    let goal = Goal::new("build_defensive_base", goal_state);

    // Create planner
    let mut planner = Planner::new();

    // Action: Build Mining Facility
    let mut build_mine_effects = HashMap::new();
    build_mine_effects.insert(
        "has_mine".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    build_mine_effects.insert("metal".to_string(), StateOperation::Add(20)); // Initial metal bonus
    let build_mine = Action::new("build_mine", 3.0, WorldState::new(), build_mine_effects);

    // Action: Mine Resources
    let mut mine_effects = HashMap::new();
    mine_effects.insert("metal".to_string(), StateOperation::Add(15));
    let mut mine_conditions = WorldState::new();
    mine_conditions.set("has_mine", StateVar::Bool(true));
    let mine_resources = Action::new("mine_resources", 2.0, mine_conditions, mine_effects);

    // Action: Build Factory
    let mut build_factory_effects = HashMap::new();
    build_factory_effects.insert(
        "has_factory".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    build_factory_effects.insert("metal".to_string(), StateOperation::Subtract(20));
    let mut build_factory_conditions = WorldState::new();
    build_factory_conditions.set("metal", StateVar::I64(20));
    let build_factory = Action::new(
        "build_factory",
        4.0,
        build_factory_conditions,
        build_factory_effects,
    );

    // Action: Craft Components
    let mut craft_components_effects = HashMap::new();
    craft_components_effects.insert("components".to_string(), StateOperation::Add(10));
    craft_components_effects.insert("metal".to_string(), StateOperation::Subtract(5));
    let mut craft_components_conditions = WorldState::new();
    craft_components_conditions.set("has_factory", StateVar::Bool(true));
    craft_components_conditions.set("metal", StateVar::I64(5));
    let craft_components = Action::new(
        "craft_components",
        2.0,
        craft_components_conditions,
        craft_components_effects,
    );

    // Action: Build Solar Panels
    let mut build_solar_effects = HashMap::new();
    build_solar_effects.insert(
        "has_solar".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    build_solar_effects.insert("energy".to_string(), StateOperation::Add(30));
    build_solar_effects.insert("components".to_string(), StateOperation::Subtract(5));
    let mut build_solar_conditions = WorldState::new();
    build_solar_conditions.set("components", StateVar::I64(5));
    let build_solar = Action::new(
        "build_solar_panels",
        3.0,
        build_solar_conditions,
        build_solar_effects,
    );

    // Action: Build Battery
    let mut build_battery_effects = HashMap::new();
    build_battery_effects.insert(
        "has_battery".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    build_battery_effects.insert("components".to_string(), StateOperation::Subtract(8));
    let mut build_battery_conditions = WorldState::new();
    build_battery_conditions.set("components", StateVar::I64(8));
    let build_battery = Action::new(
        "build_battery",
        3.0,
        build_battery_conditions,
        build_battery_effects,
    );

    // Action: Charge Battery
    let mut charge_battery_effects = HashMap::new();
    charge_battery_effects.insert("battery_charge".to_string(), StateOperation::Add(50));
    charge_battery_effects.insert("energy".to_string(), StateOperation::Subtract(25));
    let mut charge_battery_conditions = WorldState::new();
    charge_battery_conditions.set("has_battery", StateVar::Bool(true));
    charge_battery_conditions.set("has_solar", StateVar::Bool(true));
    charge_battery_conditions.set("energy", StateVar::I64(25));
    let charge_battery = Action::new(
        "charge_battery",
        2.0,
        charge_battery_conditions,
        charge_battery_effects,
    );

    // Action: Build Walls
    let mut build_walls_effects = HashMap::new();
    build_walls_effects.insert(
        "has_walls".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    build_walls_effects.insert("defense_rating".to_string(), StateOperation::Add(30));
    build_walls_effects.insert("metal".to_string(), StateOperation::Subtract(15));
    let mut build_walls_conditions = WorldState::new();
    build_walls_conditions.set("metal", StateVar::I64(15));
    let build_walls = Action::new(
        "build_walls",
        4.0,
        build_walls_conditions,
        build_walls_effects,
    );

    // Action: Build Turrets
    let mut build_turrets_effects = HashMap::new();
    build_turrets_effects.insert(
        "has_turrets".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    build_turrets_effects.insert("defense_rating".to_string(), StateOperation::Add(45));
    build_turrets_effects.insert("components".to_string(), StateOperation::Subtract(10));
    let mut build_turrets_conditions = WorldState::new();
    build_turrets_conditions.set("components", StateVar::I64(10));
    build_turrets_conditions.set("has_walls", StateVar::Bool(true));
    build_turrets_conditions.set("energy", StateVar::I64(20));
    let build_turrets = Action::new(
        "build_turrets",
        5.0,
        build_turrets_conditions,
        build_turrets_effects,
    );

    // Add all actions to planner
    planner.add_action(build_mine);
    planner.add_action(mine_resources);
    planner.add_action(build_factory);
    planner.add_action(craft_components);
    planner.add_action(build_solar);
    planner.add_action(build_battery);
    planner.add_action(charge_battery);
    planner.add_action(build_walls);
    planner.add_action(build_turrets);

    // Store a copy of initial state for later comparison
    let initial_state_copy = initial_state.clone();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal);
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
        current_state = action.execute(&current_state);
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
