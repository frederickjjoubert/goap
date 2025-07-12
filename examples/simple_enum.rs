use goap::prelude::*;
use std::fmt;

// Define a simple location enum
#[derive(Debug, Clone, PartialEq)]
enum Location {
    Home,
    Work,
    Store,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Home => write!(f, "home"),
            Location::Work => write!(f, "work"),
            Location::Store => write!(f, "store"),
        }
    }
}

// Implement the marker trait to enable enum support
impl EnumStateVar for Location {}

fn main() {
    // Initial state - character starts at home
    let initial_state = State::new()
        .set("location", Location::Home)
        .set("has_money", false)
        .set("has_food", false)
        .build();

    // Goal state - be at home with food
    let goal = Goal::new("get_food_and_return")
        .requires("location", Location::Home)
        .requires("has_food", true)
        .build();

    // Action: Go to work to earn money
    let go_to_work = Action::new("go_to_work")
        .cost(1.0)
        .sets("location", Location::Work)
        .build();

    // Action: Work to earn money
    let work = Action::new("work")
        .cost(2.0)
        .requires("location", Location::Work)
        .sets("has_money", true)
        .build();

    // Action: Go to store
    let go_to_store = Action::new("go_to_store")
        .cost(1.0)
        .sets("location", Location::Store)
        .build();

    // Action: Buy food
    let buy_food = Action::new("buy_food")
        .cost(1.0)
        .requires("location", Location::Store)
        .requires("has_money", true)
        .sets("has_food", true)
        .sets("has_money", false)
        .build();

    // Action: Go home
    let go_home = Action::new("go_home")
        .cost(1.0)
        .sets("location", Location::Home)
        .build();

    // Collect all actions
    let actions = vec![go_to_work, work, go_to_store, buy_food, go_home];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);
    assert!(
        plan_result.is_ok(),
        "Expected to find a valid plan for simple enum example"
    );

    let plan = plan_result.unwrap();

    println!(
        "\nSimple Enum Example Plan found with cost {cost}",
        cost = plan.cost
    );
    for action in &plan.actions {
        println!(
            "- {name} (cost: {cost})",
            name = action.name,
            cost = action.cost
        );
    }

    // Simulate plan execution to show enum usage
    let mut current_state = initial_state;

    println!("\nSimulating plan execution with proper enums:");
    for action in &plan.actions {
        current_state = action.apply_effect(&current_state);
        let name = &action.name;
        println!("After {name}:");

        // Note: enum values are stored as strings but we defined them with proper enums
        if let Some(location) = current_state.get::<String>("location") {
            println!("  Location: {location}");
        }
        if let Some(has_money) = current_state.get::<bool>("has_money") {
            println!("  Has Money: {has_money}");
        }
        if let Some(has_food) = current_state.get::<bool>("has_food") {
            println!("  Has Food: {has_food}");
        }
        println!();
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );
}
