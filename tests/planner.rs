#[cfg(test)]
mod tests {
    use goap::prelude::*;

    #[test]
    fn test_planner_simple_plan() {
        let planner = Planner::new();

        // Initial state
        let initial_state = State::new().set("has_wood", false).build();

        // Goal state
        let goal = Goal::new("get_wood").requires("has_wood", true).build();

        // Action to get wood
        let action = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let actions = vec![action];

        // Find plan
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "get_wood");
        assert_eq!(plan.cost, 1.0);
    }

    #[test]
    fn test_planner_no_solution() {
        let planner = Planner::new();

        // Initial state
        let initial_state = State::new().set("has_wood", false).build();

        // Goal state requiring something impossible
        let goal = Goal::new("get_gold").requires("has_gold", true).build();

        // Action that doesn't help achieve goal
        let action = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let actions = vec![action];

        // Should find no solution
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PlannerError::NoPlanFound));
    }

    #[test]
    fn test_planner_multi_step_plan() {
        let planner = Planner::new();

        // Initial state
        let initial_state = State::new()
            .set("has_wood", false)
            .set("has_planks", false)
            .build();

        // Goal state
        let goal = Goal::new("get_planks").requires("has_planks", true).build();

        // Action to get wood
        let get_wood = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        // Action to craft planks
        let craft_planks = Action::new("craft_planks")
            .cost(2.0)
            .has("has_wood", true)
            .sets("has_planks", true)
            .build();

        let actions = vec![get_wood, craft_planks];

        // Find plan
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 2);
        assert_eq!(plan.actions[0].name, "get_wood");
        assert_eq!(plan.actions[1].name, "craft_planks");
        assert_eq!(plan.cost, 3.0); // Total cost should be 1.0 + 2.0
    }
}
