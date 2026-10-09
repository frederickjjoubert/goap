//! Counters and typed numeric effects.
//!
//! This example shows three things about `adds` / `subtracts`:
//!
//! 1. A variable that is missing from the initial state counts as zero, so
//!    counters like `items_crafted` and `experience` need no initial value.
//! 2. Deltas are typed. `materials` is an `i64` and is changed with integer
//!    deltas; `energy` and `experience` are `f64` and are changed with float
//!    deltas. Mixing them up is a planning error, not a silent miscalculation.
//! 3. `Display` prints float effects as decimals (`Subtract 12.500 from energy`).

use goap::prelude::*;

fn main() {
    // Only the resources the workshop starts with. No counters yet.
    let initial_state = State::builder()
        .set("has_tools", true)
        .set("materials", 6)
        .set("energy", 50.0)
        .build();

    // The goal is phrased entirely in terms of counters that do not exist yet.
    let goal = Goal::builder("productive_day")
        .requires("items_crafted", at_least(5))
        .requires("experience", at_least(7.0))
        .build();

    let craft = Action::builder("craft")
        .cost(1.0)
        .requires("has_tools", true)
        .requires("materials", at_least(2))
        .requires("energy", at_least(12.5))
        .subtracts("materials", 2)
        .subtracts("energy", 12.5)
        .adds("items_crafted", 1)
        .adds("experience", 1.5)
        .build();

    let craft_batch = Action::builder("craft_batch")
        .cost(2.5)
        .requires("has_tools", true)
        .requires("materials", at_least(6))
        .requires("energy", at_least(30.0))
        .subtracts("materials", 6)
        .subtracts("energy", 30.0)
        .adds("items_crafted", 3)
        .adds("experience", 4.0)
        .build();

    let gather = Action::builder("gather")
        .cost(1.5)
        .adds("materials", 4)
        .subtracts("energy", 5.0)
        .build();

    let rest = Action::builder("rest")
        .cost(2.0)
        .adds("energy", 40.0)
        .build();

    let actions = vec![craft, craft_batch, gather, rest];

    println!("Actions:");
    for action in &actions {
        println!("{action}\n");
    }

    let plan = Planner::new()
        .plan(&initial_state, &goal, &actions)
        .expect("the goal is reachable");

    println!("Plan (cost {:.1}):", plan.cost);
    let mut state = initial_state;
    for (step, action) in plan.actions.iter().enumerate() {
        state = action.apply_effect(&state);
        println!(
            "  {}. {:<11} materials={:<2} energy={:>5.1} items_crafted={} experience={:.1}",
            step + 1,
            action.name,
            state.get::<i64>("materials").unwrap_or(0),
            state.get::<f64>("energy").unwrap_or(0.0),
            state.get::<i64>("items_crafted").unwrap_or(0),
            state.get::<f64>("experience").unwrap_or(0.0),
        );
    }

    assert!(goal.is_satisfied(&state));
    assert!(state.get::<i64>("items_crafted").unwrap() >= 5);
    assert!(state.get::<f64>("experience").unwrap() >= 7.0);

    // A typed mistake is caught before any search runs: `materials` is an
    // integer, so a float delta on it is rejected.
    let wrong = Action::builder("gather_wrong")
        .adds("materials", 4.0)
        .build();
    match Planner::new().plan(
        &State::builder().set("materials", 0).build(),
        &goal,
        &[wrong],
    ) {
        Err(err @ PlannerError::IncompatibleStateTypes { .. }) => {
            println!("\nRejected a mistyped effect: {err}");
        }
        other => panic!("expected IncompatibleStateTypes, got {other:?}"),
    }
}
