use goap::prelude::*;

fn main() {
    // Create initial state
    let initial_state = State::new()
        .set("has_wood", false)
        .set("has_axe", true)
        .set("at_tree", false)
        .build();

    // Create goal state
    let goal = Goal::new("gather_wood").requires("has_wood", true).build();

    // Create actions
    let move_to_tree = Action::new("move_to_tree").sets("at_tree", true).build();

    let chop_tree = Action::new("chop_tree")
        .cost(2.0)
        .requires("has_axe", true)
        .requires("at_tree", true)
        .sets("has_wood", true)
        .build();

    // Collect all actions
    let actions = vec![move_to_tree, chop_tree];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);
    assert!(plan_result.is_ok(), "Expected to find a valid plan");

    let plan = plan_result.unwrap();
    assert_eq!(plan.cost, 3.0, "Expected total cost to be 3.0");

    let expected_actions = ["move_to_tree", "chop_tree"];
    assert_eq!(
        plan.actions.len(),
        expected_actions.len(),
        "Plan length doesn't match expected length"
    );

    for (action, expected_name) in plan.actions.iter().zip(expected_actions.iter()) {
        assert_eq!(
            action.name,
            *expected_name,
            "Expected action {expected} but got {actual}",
            expected = expected_name,
            actual = action.name
        );
    }

    println!("Found plan with cost {cost}", cost = plan.cost);
    for action in plan.actions {
        let name = &action.name;
        println!("- {name}");
    }
}
