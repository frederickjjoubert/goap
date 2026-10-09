use goap::prelude::*;
use std::fmt;
use std::str::FromStr;

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

// `FromStr` is the inverse of `Display`; it lets `state.get::<Location>` work.
impl FromStr for Location {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "home" => Ok(Location::Home),
            "work" => Ok(Location::Work),
            "store" => Ok(Location::Store),
            other => Err(format!("unknown location '{other}'")),
        }
    }
}

// Implement the marker trait to enable enum support
impl EnumStateVar for Location {}

fn main() {
    // Initial state - character starts at home
    let initial_state = State::builder()
        .set("location", Location::Home)
        .set("has_money", false)
        .set("has_food", false)
        .build();

    // Goal state - be at home with food
    let goal = Goal::builder("get_food_and_return")
        .requires("location", Location::Home)
        .requires("has_food", true)
        .build();

    // Action: Go to work to earn money
    let go_to_work = Action::builder("go_to_work")
        .cost(1.0)
        .sets("location", Location::Work)
        .build();

    // Action: Work to earn money
    let work = Action::builder("work")
        .cost(2.0)
        .requires("location", Location::Work)
        .sets("has_money", true)
        .build();

    // Action: Go to store
    let go_to_store = Action::builder("go_to_store")
        .cost(1.0)
        .sets("location", Location::Store)
        .build();

    // Action: Buy food
    let buy_food = Action::builder("buy_food")
        .cost(1.0)
        .requires("location", Location::Store)
        .requires("has_money", true)
        .sets("has_food", true)
        .sets("has_money", false)
        .build();

    // Action: Go home
    let go_home = Action::builder("go_home")
        .cost(1.0)
        .sets("location", Location::Home)
        .build();

    // Collect all actions
    let actions = vec![go_to_work, work, go_to_store, buy_food, go_home];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(&initial_state, &goal, &actions);
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

        // Enum values are stored as strings and read back as the enum.
        if let Some(location) = current_state.get::<Location>("location") {
            println!("  Location: {location} ({location:?})");
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
        goal.is_satisfied(&current_state),
        "Final state should meet all goals"
    );
}
