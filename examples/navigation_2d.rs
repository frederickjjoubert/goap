use goap::prelude::*;
use std::collections::HashMap;

fn main() {
    // Initial state - robot starts at position (0,0)
    let mut initial_state = WorldState::new();
    initial_state.set("x", StateVar::I64(0));
    initial_state.set("y", StateVar::I64(0));
    initial_state.set("battery", StateVar::I64(300));
    initial_state.set("has_package", StateVar::Bool(false));
    initial_state.set("package_delivered", StateVar::Bool(false));

    // Goal state - deliver package to position (8,5)
    let mut goal_state = WorldState::new();
    goal_state.set("x", StateVar::I64(8));
    goal_state.set("y", StateVar::I64(5));
    goal_state.set("package_delivered", StateVar::Bool(true));

    let goal = Goal::new("deliver_package", goal_state);

    // Create planner
    let mut planner = Planner::new();

    // Helper function to create GoTo actions between points
    fn create_goto_action(from: (i64, i64), to: (i64, i64)) -> Action {
        let distance = ((to.0 - from.0).pow(2) + (to.1 - from.1).pow(2)) as f64;
        let battery_cost = (distance * 5.0) as i64; // Battery cost based on distance

        let mut conditions = WorldState::new();
        conditions.set("x", StateVar::I64(from.0));
        conditions.set("y", StateVar::I64(from.1));
        conditions.set("battery", StateVar::I64(battery_cost)); // Need enough battery

        let mut effects = HashMap::new();
        effects.insert("x".to_string(), StateOperation::Set(StateVar::I64(to.0)));
        effects.insert("y".to_string(), StateOperation::Set(StateVar::I64(to.1)));
        effects.insert(
            "battery".to_string(),
            StateOperation::Subtract(battery_cost),
        );

        let name = format!("goto_{}_{}_{}_{}", from.0, from.1, to.0, to.1);
        Action::new(&name, distance, conditions, effects)
    }

    // Add navigation waypoints
    let waypoints = vec![
        (0, 0), // Start
        (3, 2), // Waypoint 1
        (5, 2), // Package pickup
        (5, 4), // Waypoint 2
        (8, 5), // Destination
    ];

    // Create GoTo actions between waypoints
    for i in 0..waypoints.len() {
        for j in 0..waypoints.len() {
            if i != j {
                planner.add_action(create_goto_action(waypoints[i], waypoints[j]));
            }
        }
    }

    // Action: Pick up package
    let mut pickup_conditions = WorldState::new();
    pickup_conditions.set("x", StateVar::I64(5));
    pickup_conditions.set("y", StateVar::I64(2));
    pickup_conditions.set("has_package", StateVar::Bool(false));

    let mut pickup_effects = HashMap::new();
    pickup_effects.insert(
        "has_package".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );

    let pickup_action = Action::new("pickup_package", 1.0, pickup_conditions, pickup_effects);

    // Action: Deliver package
    let mut deliver_conditions = WorldState::new();
    deliver_conditions.set("x", StateVar::I64(8));
    deliver_conditions.set("y", StateVar::I64(5));
    deliver_conditions.set("has_package", StateVar::Bool(true));

    let mut deliver_effects = HashMap::new();
    deliver_effects.insert(
        "has_package".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    deliver_effects.insert(
        "package_delivered".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );

    let deliver_action = Action::new("deliver_package", 1.0, deliver_conditions, deliver_effects);

    // Add package actions
    planner.add_action(pickup_action);
    planner.add_action(deliver_action);

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal);
    assert!(plan_result.is_some(), "Expected to find a valid plan");

    let (actions, total_cost) = plan_result.unwrap();

    println!("\nNavigation Plan found with cost {}", total_cost);

    // Simulate plan execution
    let mut current_state = initial_state;
    println!("\nSimulating plan execution:");
    println!(
        "Starting at ({}, {})",
        if let Some(StateVar::I64(x)) = current_state.get("x") {
            *x
        } else {
            0_i64
        },
        if let Some(StateVar::I64(y)) = current_state.get("y") {
            *y
        } else {
            0_i64
        }
    );

    for action in &actions {
        current_state = action.execute(&current_state);

        if let (Some(StateVar::I64(x)), Some(StateVar::I64(y))) =
            (current_state.get("x"), current_state.get("y"))
        {
            println!("- {} -> Position: ({}, {})", action.name, x, y);
        }

        if action.name == "pickup_package" {
            println!("  Package picked up!");
        } else if action.name == "deliver_package" {
            println!("  Package delivered!");
        }

        if let Some(StateVar::I64(battery)) = current_state.get("battery") {
            println!("  Battery: {}", battery);
        }
    }

    // Verify final state
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );
}
