use goap::prelude::*;

fn main() {
    // Initial state - robot starts at position (0,0)
    let initial_state = State::builder()
        .int("x", 0)
        .int("y", 0)
        .int("battery", 300)
        .bool("has_package", false)
        .bool("package_delivered", false)
        .build();

    // Goal state - deliver package to position (8,5)
    let goal = Goal::builder("deliver_package")
        .require_int("x", 8)
        .require_int("y", 5)
        .require_bool("package_delivered", true)
        .build();

    // Helper function to create GoTo actions between points
    fn create_goto_action(from: (i64, i64), to: (i64, i64)) -> Action {
        let distance = ((to.0 - from.0).pow(2) + (to.1 - from.1).pow(2)) as f64;
        let battery_cost = (distance * 5.0) as i64; // Battery cost based on distance

        Action::builder(&format!(
            "goto_{}_{}_{to_x}_{to_y}",
            from.0,
            from.1,
            to_x = to.0,
            to_y = to.1
        ))
        .cost(distance)
        .precondition("x", from.0)
        .precondition("y", from.1)
        .precondition("battery", battery_cost)
        .effect_set_to("x", to.0)
        .effect_set_to("y", to.1)
        .effect_subtract_int("battery", battery_cost)
        .build()
    }

    // Add navigation waypoints
    let waypoints = [
        (0, 0), // Start
        (3, 2), // Waypoint 1
        (5, 2), // Package pickup
        (5, 4), // Waypoint 2
        (8, 5), // Destination
    ];

    // Create all navigation actions
    let mut actions = Vec::new();
    for i in 0..waypoints.len() {
        for j in 0..waypoints.len() {
            if i != j {
                actions.push(create_goto_action(waypoints[i], waypoints[j]));
            }
        }
    }

    // Action: Pick up package
    let pickup_action = Action::builder("pickup_package")
        .cost(1.0)
        .precondition("x", 5)
        .precondition("y", 2)
        .precondition("has_package", false)
        .effect_set_to("has_package", true)
        .build();

    // Action: Deliver package
    let deliver_action = Action::builder("deliver_package")
        .cost(1.0)
        .precondition("x", 8)
        .precondition("y", 5)
        .precondition("has_package", true)
        .effect_set_to("has_package", false)
        .effect_set_to("package_delivered", true)
        .build();

    // Add package actions
    actions.push(pickup_action);
    actions.push(deliver_action);

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);
    assert!(plan_result.is_ok(), "Expected to find a valid plan");

    let plan = plan_result.unwrap();

    println!("\nNavigation Plan found with cost {cost}", cost = plan.cost);

    // Simulate plan execution
    let mut current_state = initial_state;
    println!("\nSimulating plan execution:");
    let x = if let Some(StateVar::I64(x)) = current_state.get("x") {
        *x
    } else {
        0_i64
    };
    let y = if let Some(StateVar::I64(y)) = current_state.get("y") {
        *y
    } else {
        0_i64
    };
    println!("Starting at ({x}, {y})");

    for action in &plan.actions {
        current_state = action.apply_effect(&current_state);

        if let (Some(StateVar::I64(x)), Some(StateVar::I64(y))) =
            (current_state.get("x"), current_state.get("y"))
        {
            let name = &action.name;
            println!("- {name} -> Position: ({x}, {y})");
        }

        if action.name == "pickup_package" {
            println!("  Package picked up!");
        } else if action.name == "deliver_package" {
            println!("  Package delivered!");
        }

        if let Some(StateVar::I64(battery)) = current_state.get("battery") {
            println!("  Battery: {battery}");
        }
    }

    // Verify final state
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );
}
