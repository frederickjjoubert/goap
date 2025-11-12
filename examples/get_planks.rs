use goap::prelude::*;

fn main() {
    // Create initial state
    let initial_state = State::new()
        .set("has_wood", false)
        .set("has_planks", false)
        .set("has_axe", false)
        .set("has_money", true)
        .set("at_store", false)
        .set("at_tree", false)
        .set("at_sawmill", false)
        .build();

    // Create goal state
    let goal = Goal::new("craft_planks")
        .requires("has_planks", true)
        .build();

    // Create actions
    let goto_store = Action::new("goto_store")
        .cost(1.0)
        .sets("at_store", true)
        .build();

    let buy_axe = Action::new("buy_axe")
        .cost(1.0)
        .requires("at_store", true)
        .requires("has_money", true)
        .sets("has_axe", true)
        .sets("has_money", false)
        .build();

    let goto_tree = Action::new("goto_tree")
        .cost(1.0)
        .sets("at_store", false)
        .sets("at_tree", true)
        .build();

    let chop_tree = Action::new("chop_tree")
        .cost(2.0)
        .requires("has_axe", true)
        .requires("at_tree", true)
        .sets("has_wood", true)
        .build();

    let goto_sawmill = Action::new("goto_sawmill")
        .cost(1.0)
        .requires("has_wood", true)
        .sets("at_tree", false)
        .sets("at_sawmill", true)
        .build();

    let saw_planks = Action::new("saw_planks")
        .cost(1.0)
        .requires("has_wood", true)
        .requires("at_sawmill", true)
        .sets("has_planks", true)
        .build();

    // Collect all actions
    let actions = vec![
        goto_store,
        buy_axe,
        goto_tree,
        chop_tree,
        goto_sawmill,
        saw_planks,
    ];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);
    assert!(plan_result.is_ok(), "Expected to find a valid plan");

    let plan = plan_result.unwrap();
    assert_eq!(plan.cost, 7.0, "Expected total cost to be 7.0");

    let expected_actions = [
        "goto_store",
        "buy_axe",
        "goto_tree",
        "chop_tree",
        "goto_sawmill",
        "saw_planks",
    ];
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

    println!("\nComplex plan found with cost {cost}", cost = plan.cost);
    for action in plan.actions {
        let name = &action.name;
        println!("- {name}");
    }
}
