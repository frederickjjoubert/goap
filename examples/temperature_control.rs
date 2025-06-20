use goap::prelude::*;

fn main() {
    // Initial state: Room is 22.5°C, heater and cooler are off
    let initial_state = State::builder()
        .float("temperature", 22.5)
        .bool("heater_on", false)
        .bool("cooler_on", false)
        .float("power_available", 100.0) // 100% power available
        .build();

    // Goal: Get room to 24.0°C
    let goal = Goal::builder("Adjust Temperature")
        .require_float("temperature", 24.0)
        .build();

    // Action: Turn on heater
    let turn_on_heater = Action::builder("Turn on heater")
        .cost(1.0)
        .precondition("heater_on", false)
        .precondition("power_available", 20.0) // Need at least 20% power
        .effect_set_to("heater_on", true)
        .effect_subtract_float("power_available", 20.0)
        .build();

    // Action: Heat room (can be applied multiple times)
    let heat_room = Action::builder("Heat room")
        .cost(2.0)
        .precondition("heater_on", true)
        .precondition("power_available", 5.0) // Need at least 5% power to heat
        .effect_add_float("temperature", 0.5) // Increases temperature by 0.5°C
        .effect_subtract_float("power_available", 5.0) // Consumes 5% power per heating cycle
        .build();

    // Action: Turn on cooler
    let turn_on_cooler = Action::builder("Turn on cooler")
        .cost(1.0)
        .precondition("cooler_on", false)
        .precondition("power_available", 30.0) // Need at least 30% power
        .effect_set_to("cooler_on", true)
        .effect_subtract_float("power_available", 30.0)
        .build();

    // Action: Cool room (can be applied multiple times)
    let cool_room = Action::builder("Cool room")
        .cost(2.0)
        .precondition("cooler_on", true)
        .precondition("power_available", 8.0) // Need at least 8% power to cool
        .effect_subtract_float("temperature", 1.0) // Decreases temperature by 1.0°C
        .effect_subtract_float("power_available", 8.0) // Consumes 8% power per cooling cycle
        .build();

    // Collect all actions
    let actions = vec![turn_on_heater, heat_room, turn_on_cooler, cool_room];

    // Create planner
    let planner = Planner::new();

    // Find plan
    println!("Planning to adjust room temperature...");
    println!("Initial state: {:?}", initial_state);
    println!("Goal state: {:?}", goal);

    match planner.plan(initial_state.clone(), &goal, &actions) {
        Ok(plan) => {
            println!(
                "\nFound plan with {} actions and cost {:.2}:",
                plan.actions.len(),
                plan.cost
            );
            let mut current_state = initial_state;

            for (i, action) in plan.actions.iter().enumerate() {
                println!("\nStep {}: {}", i + 1, action.name);
                current_state = action.apply_effect(&current_state);

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
        Err(e) => println!("No plan found! {}", e),
    }
}
