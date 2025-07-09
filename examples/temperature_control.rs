use goap::prelude::*;

fn main() {
    // Initial state: Room is 22.5°C, heater and cooler are off
    let initial_state = State::new()
        .set("temperature", 22.5)
        .set("heater_on", false)
        .set("cooler_on", false)
        .set("power_available", 100.0) // 100% power available
        .build();

    // Goal: Get room to 24.0°C
    let goal = Goal::new("Adjust Temperature")
        .requires("temperature", 24.0)
        .build();

    // Action: Turn on heater
    let turn_on_heater = Action::new("Turn on heater")
        .cost(1.0)
        .requires("heater_on", false)
        .requires("power_available", 20.0) // Need at least 20% power
        .sets("heater_on", true)
        .subtracts("power_available", 20.0)
        .build();

    // Action: Heat room (can be applied multiple times)
    let heat_room = Action::new("Heat room")
        .cost(2.0)
        .requires("heater_on", true)
        .requires("power_available", 5.0) // Need at least 5% power to heat
        .adds("temperature", 0.5) // Increases temperature by 0.5°C
        .subtracts("power_available", 5.0) // Consumes 5% power per heating cycle
        .build();

    // Action: Turn on cooler
    let turn_on_cooler = Action::new("Turn on cooler")
        .cost(1.0)
        .requires("cooler_on", false)
        .requires("power_available", 30.0) // Need at least 30% power
        .sets("cooler_on", true)
        .subtracts("power_available", 30.0)
        .build();

    // Action: Cool room (can be applied multiple times)
    let cool_room = Action::new("Cool room")
        .cost(2.0)
        .requires("cooler_on", true)
        .requires("power_available", 8.0) // Need at least 8% power to cool
        .subtracts("temperature", 1.0) // Decreases temperature by 1.0°C
        .subtracts("power_available", 8.0) // Consumes 8% power per cooling cycle
        .build();

    // Collect all actions
    let actions = vec![turn_on_heater, heat_room, turn_on_cooler, cool_room];

    // Create planner
    let planner = Planner::new();

    // Find plan
    println!("Planning to adjust room temperature...");
    println!("Initial state: {initial_state:?}");
    println!("Goal state: {goal:?}");

    match planner.plan(initial_state.clone(), &goal, &actions) {
        Ok(plan) => {
            let action_count = plan.actions.len();
            let cost = plan.cost;
            println!("\nFound plan with {action_count} actions and cost {cost:.2}:");
            let mut current_state = initial_state;

            for (i, action) in plan.actions.iter().enumerate() {
                let step = i + 1;
                let name = &action.name;
                println!("\nStep {step}: {name}");
                current_state = action.apply_effect(&current_state);

                if let Some(temp) = current_state.get::<f64>("temperature") {
                    println!("Temperature: {temp:.1}°C");
                }
                if let Some(power) = current_state.get::<f64>("power_available") {
                    println!("Power available: {power:.1}%");
                }
            }
        }
        Err(e) => println!("No plan found! {e}"),
    }
}
