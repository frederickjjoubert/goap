use goap::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut planner = Planner::new();

    // Initial state: Room is 22.5°C, heater and cooler are off
    let mut initial_state = WorldState::new();
    initial_state.set("temperature", StateVar::from_f64(22.5));
    initial_state.set("heater_on", StateVar::Bool(false));
    initial_state.set("cooler_on", StateVar::Bool(false));
    initial_state.set("power_available", StateVar::from_f64(100.0)); // 100% power available

    // Goal: Get room to 24.0°C
    let mut goal_state = WorldState::new();
    goal_state.set("temperature", StateVar::from_f64(24.0));

    // Action: Turn on heater
    let mut turn_on_heater_conditions = WorldState::new();
    turn_on_heater_conditions.set("heater_on", StateVar::Bool(false));
    turn_on_heater_conditions.set("power_available", StateVar::from_f64(20.0)); // Need at least 20% power

    let mut turn_on_heater_effects = HashMap::new();
    turn_on_heater_effects.insert(
        "heater_on".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    turn_on_heater_effects.insert(
        "power_available".to_string(),
        StateOperation::subtract_f64(20.0),
    );

    let turn_on_heater = Action::new(
        "Turn on heater",
        1.0,
        turn_on_heater_conditions,
        turn_on_heater_effects,
    );

    // Action: Heat room (can be applied multiple times)
    let mut heat_room_conditions = WorldState::new();
    heat_room_conditions.set("heater_on", StateVar::Bool(true));

    let mut heat_room_effects = HashMap::new();
    heat_room_effects.insert(
        "temperature".to_string(),
        StateOperation::add_f64(0.5), // Increases temperature by 0.5°C
    );

    let heat_room = Action::new("Heat room", 2.0, heat_room_conditions, heat_room_effects);

    // Action: Turn on cooler
    let mut turn_on_cooler_conditions = WorldState::new();
    turn_on_cooler_conditions.set("cooler_on", StateVar::Bool(false));
    turn_on_cooler_conditions.set("power_available", StateVar::from_f64(30.0)); // Need at least 30% power

    let mut turn_on_cooler_effects = HashMap::new();
    turn_on_cooler_effects.insert(
        "cooler_on".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    turn_on_cooler_effects.insert(
        "power_available".to_string(),
        StateOperation::subtract_f64(30.0),
    );

    let turn_on_cooler = Action::new(
        "Turn on cooler",
        1.0,
        turn_on_cooler_conditions,
        turn_on_cooler_effects,
    );

    // Action: Cool room (can be applied multiple times)
    let mut cool_room_conditions = WorldState::new();
    cool_room_conditions.set("cooler_on", StateVar::Bool(true));

    let mut cool_room_effects = HashMap::new();
    cool_room_effects.insert(
        "temperature".to_string(),
        StateOperation::subtract_f64(1.0), // Decreases temperature by 1.0°C
    );

    let cool_room = Action::new("Cool room", 2.0, cool_room_conditions, cool_room_effects);

    // Add actions to planner
    planner.add_action(turn_on_heater);
    planner.add_action(heat_room);
    planner.add_action(turn_on_cooler);
    planner.add_action(cool_room);

    // Find plan
    println!("Planning to adjust room temperature...");
    println!("Initial state: {:?}", initial_state);
    println!("Goal state: {:?}", goal_state);

    match planner.plan(
        initial_state.clone(),
        &Goal::new("Adjust Temperature", goal_state),
    ) {
        Some((actions, cost)) => {
            println!(
                "\nFound plan with {} actions and cost {:.2}:",
                actions.len(),
                cost
            );
            let mut current_state = initial_state;

            for (i, action) in actions.iter().enumerate() {
                println!("\nStep {}: {}", i + 1, action.name);
                current_state.apply(&action.effects);

                if let Some(temp) = current_state.get("temperature").and_then(|v| v.as_f64()) {
                    println!("Temperature: {:.1}°C", temp);
                }
                if let Some(power) = current_state
                    .get("power_available")
                    .and_then(|v| v.as_f64())
                {
                    println!("Power available: {:.1}%", power);
                }
            }
        }
        None => println!("No plan found!"),
    }
}
