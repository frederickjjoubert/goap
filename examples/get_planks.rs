use goap::prelude::*;

fn main() {
    // Create initial state
    let initial_state = State::builder()
        .bool("has_wood", false)
        .bool("has_axe", false)
        .bool("has_money", true)
        .bool("at_store", false)
        .bool("at_tree", false)
        .build();

    // Create goal state
    let goal = Goal::builder("gather_wood")
        .require_bool("has_wood", true)
        .build();

    // Create actions
    let goto_store = Action::builder("goto_store")
        .cost(1.0)
        .effect_set_to("at_store", true)
        .build();

    let buy_axe = Action::builder("buy_axe")
        .cost(1.0)
        .precondition("at_store", true)
        .precondition("has_money", true)
        .effect_set_to("has_axe", true)
        .effect_set_to("has_money", false)
        .build();

    let goto_tree = Action::builder("goto_tree")
        .cost(1.0)
        .effect_set_to("at_store", false)
        .effect_set_to("at_tree", true)
        .build();

    let chop_tree = Action::builder("chop_tree")
        .cost(2.0)
        .precondition("has_axe", true)
        .precondition("at_tree", true)
        .effect_set_to("has_wood", true)
        .build();

    // Collect all actions
    let actions = vec![goto_store, buy_axe, goto_tree, chop_tree];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);
    assert!(plan_result.is_ok(), "Expected to find a valid plan");

    let plan = plan_result.unwrap();
    assert_eq!(plan.cost, 5.0, "Expected total cost to be 5.0");

    let expected_actions = vec!["goto_store", "buy_axe", "goto_tree", "chop_tree"];
    assert_eq!(
        plan.actions.len(),
        expected_actions.len(),
        "Plan length doesn't match expected length"
    );

    for (action, expected_name) in plan.actions.iter().zip(expected_actions.iter()) {
        assert_eq!(
            action.name, *expected_name,
            "Expected action {} but got {}",
            expected_name, action.name
        );
    }

    println!("\nComplex plan found with cost {}", plan.cost);
    for action in plan.actions {
        println!("- {}", action.name);
    }
}
