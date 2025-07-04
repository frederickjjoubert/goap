#[cfg(test)]
mod tests {
    use goap::prelude::*;

    // Tests for basic planner functionality

    /// Test planner creation
    /// Validates: Planner can be created successfully
    /// Failure: Planner constructor is broken
    #[test]
    fn test_planner_creation() {
        let planner = Planner::new();
        // Should not panic or fail
        let _ = planner;
    }

    /// Test planner default implementation
    /// Validates: Planner::default() works the same as Planner::new()
    /// Failure: Default trait implementation is broken
    #[test]
    fn test_planner_default() {
        let planner = Planner::default();
        // Should not panic or fail
        let _ = planner;
    }

    // Tests for simple planning scenarios

    /// Test simple single-step plan with boolean goal
    /// Validates: Planner can find a single action to achieve a boolean goal
    /// Failure: Basic planning with boolean states is broken
    #[test]
    fn test_planner_simple_bool_plan() {
        let planner = Planner::new();

        let initial_state = State::new().set("has_wood", false).build();
        let goal = Goal::new("get_wood").requires("has_wood", true).build();
        let action = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let result = planner.plan(initial_state, &goal, &[action]);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "get_wood");
        assert_eq!(plan.cost, 1.0);
    }

    /// Test simple single-step plan with integer goal
    /// Validates: Planner can find a single action to achieve an integer goal
    /// Failure: Basic planning with integer states is broken
    #[test]
    fn test_planner_simple_i64_plan() {
        let planner = Planner::new();

        let initial_state = State::new().set("gold", 0).build();
        let goal = Goal::new("get_gold").requires("gold", 10).build();
        let action = Action::new("mine_gold").cost(2.0).sets("gold", 15).build();

        let result = planner.plan(initial_state, &goal, &[action]);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "mine_gold");
        assert_eq!(plan.cost, 2.0);
    }

    /// Test simple single-step plan with float goal
    /// Validates: Planner can find a single action to achieve a float goal
    /// Failure: Basic planning with float states is broken
    #[test]
    fn test_planner_simple_f64_plan() {
        let planner = Planner::new();

        let initial_state = State::new().set("health", 50.0).build();
        let goal = Goal::new("heal").requires("health", 100.0).build();
        let action = Action::new("use_potion")
            .cost(1.5)
            .sets("health", 100.0)
            .build();

        let result = planner.plan(initial_state, &goal, &[action]);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "use_potion");
        assert_eq!(plan.cost, 1.5);
    }

    /// Test simple single-step plan with string goal
    /// Validates: Planner can find a single action to achieve a string goal
    /// Failure: Basic planning with string states is broken
    #[test]
    fn test_planner_simple_string_plan() {
        let planner = Planner::new();

        let initial_state = State::new().set("location", "home").build();
        let goal = Goal::new("travel").requires("location", "town").build();
        let action = Action::new("walk_to_town")
            .cost(3.0)
            .sets("location", "town")
            .build();

        let result = planner.plan(initial_state, &goal, &[action]);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "walk_to_town");
        assert_eq!(plan.cost, 3.0);
    }

    // Tests for multi-step planning

    /// Test two-step plan with dependent actions
    /// Validates: Planner can chain actions where one action enables another
    /// Failure: Multi-step planning or action dependencies are broken
    #[test]
    fn test_planner_two_step_plan() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("has_wood", false)
            .set("has_planks", false)
            .build();

        let goal = Goal::new("get_planks").requires("has_planks", true).build();

        let get_wood = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let craft_planks = Action::new("craft_planks")
            .cost(2.0)
            .requires("has_wood", true)
            .sets("has_planks", true)
            .build();

        let actions = vec![get_wood, craft_planks];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 2);
        assert_eq!(plan.actions[0].name, "get_wood");
        assert_eq!(plan.actions[1].name, "craft_planks");
        assert_eq!(plan.cost, 3.0);
    }

    /// Test three-step plan with sequential dependencies
    /// Validates: Planner can handle longer chains of dependent actions
    /// Failure: Complex multi-step planning is broken
    #[test]
    fn test_planner_three_step_plan() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("has_wood", false)
            .set("has_planks", false)
            .set("has_table", false)
            .build();

        let goal = Goal::new("make_table").requires("has_table", true).build();

        let get_wood = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let craft_planks = Action::new("craft_planks")
            .cost(2.0)
            .requires("has_wood", true)
            .sets("has_planks", true)
            .build();

        let make_table = Action::new("make_table")
            .cost(3.0)
            .requires("has_planks", true)
            .sets("has_table", true)
            .build();

        let actions = vec![get_wood, craft_planks, make_table];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 3);
        assert_eq!(plan.actions[0].name, "get_wood");
        assert_eq!(plan.actions[1].name, "craft_planks");
        assert_eq!(plan.actions[2].name, "make_table");
        assert_eq!(plan.cost, 6.0);
    }

    // Tests for plan optimization

    /// Test planner chooses cheaper path when multiple options exist
    /// Validates: Planner optimizes for cost when multiple paths are available
    /// Failure: Cost optimization in planning is broken
    #[test]
    fn test_planner_cost_optimization() {
        let planner = Planner::new();

        let initial_state = State::new().set("has_gold", false).build();
        let goal = Goal::new("get_gold").requires("has_gold", true).build();

        // Expensive direct action
        let expensive_action = Action::new("buy_gold")
            .cost(10.0)
            .sets("has_gold", true)
            .build();

        // Cheaper action
        let cheap_action = Action::new("find_gold")
            .cost(2.0)
            .sets("has_gold", true)
            .build();

        let actions = vec![expensive_action, cheap_action];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "find_gold"); // Should choose cheaper option
        assert_eq!(plan.cost, 2.0);
    }

    /// Test planner chooses cheaper multi-step path over expensive single step
    /// Validates: Planner compares total cost across different path lengths
    /// Failure: Cross-path cost optimization is broken
    #[test]
    fn test_planner_multi_step_vs_single_step_cost() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("has_materials", false)
            .set("has_sword", false)
            .build();

        let goal = Goal::new("get_sword").requires("has_sword", true).build();

        // Expensive direct path
        let buy_sword = Action::new("buy_sword")
            .cost(10.0)
            .sets("has_sword", true)
            .build();

        // Cheaper multi-step path
        let gather_materials = Action::new("gather_materials")
            .cost(2.0)
            .sets("has_materials", true)
            .build();

        let craft_sword = Action::new("craft_sword")
            .cost(3.0)
            .requires("has_materials", true)
            .sets("has_sword", true)
            .build();

        let actions = vec![buy_sword, gather_materials, craft_sword];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 2);
        assert_eq!(plan.actions[0].name, "gather_materials");
        assert_eq!(plan.actions[1].name, "craft_sword");
        assert_eq!(plan.cost, 5.0); // Should be cheaper than 10.0
    }

    // Tests for planning with multiple goal requirements

    /// Test planning with multiple boolean requirements
    /// Validates: Planner can satisfy goals with multiple boolean conditions
    /// Failure: Multi-requirement boolean planning is broken
    #[test]
    fn test_planner_multiple_bool_requirements() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("has_key", false)
            .set("door_unlocked", false)
            .build();

        let goal = Goal::new("enter_room")
            .requires("has_key", true)
            .requires("door_unlocked", true)
            .build();

        let get_key = Action::new("get_key")
            .cost(1.0)
            .sets("has_key", true)
            .build();

        let unlock_door = Action::new("unlock_door")
            .cost(1.0)
            .requires("has_key", true)
            .sets("door_unlocked", true)
            .build();

        let actions = vec![get_key, unlock_door];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 2);
        assert_eq!(plan.cost, 2.0);
    }

    /// Test planning with mixed type requirements
    /// Validates: Planner can satisfy goals with different StateVar types
    /// Failure: Mixed-type requirement planning is broken
    #[test]
    fn test_planner_mixed_type_requirements() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("has_weapon", false)
            .set("gold", 0)
            .set("health", 50.0)
            .set("location", "home")
            .build();

        let goal = Goal::new("ready_for_adventure")
            .requires("has_weapon", true)
            .requires("gold", 100)
            .requires("health", 100.0)
            .requires("location", "town")
            .build();

        let buy_weapon = Action::new("buy_weapon")
            .cost(1.0)
            .sets("has_weapon", true)
            .build();

        let earn_gold = Action::new("work").cost(2.0).sets("gold", 150).build();

        let heal = Action::new("rest").cost(1.0).sets("health", 100.0).build();

        let travel = Action::new("travel_to_town")
            .cost(1.0)
            .sets("location", "town")
            .build();

        let actions = vec![buy_weapon, earn_gold, heal, travel];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 4);
        assert_eq!(plan.cost, 5.0);
    }

    // Tests for edge cases and error conditions

    /// Test planning when goal is already satisfied
    /// Validates: Planner returns empty plan when goal is already met
    /// Failure: Already-satisfied goal handling is broken
    #[test]
    fn test_planner_goal_already_satisfied() {
        let planner = Planner::new();

        let initial_state = State::new().set("has_item", true).build();
        let goal = Goal::new("get_item").requires("has_item", true).build();
        let action = Action::new("get_item")
            .cost(1.0)
            .sets("has_item", true)
            .build();

        let result = planner.plan(initial_state, &goal, &[action]);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 0);
        assert_eq!(plan.cost, 0.0);
    }

    /// Test planning with empty action list
    /// Validates: Planner returns error when no actions are available
    /// Failure: Empty action list handling is broken
    #[test]
    fn test_planner_empty_actions() {
        let planner = Planner::new();

        let initial_state = State::new().set("has_item", false).build();
        let goal = Goal::new("get_item").requires("has_item", true).build();
        let actions: Vec<Action> = vec![];

        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PlannerError::NoPlanFound);
    }

    /// Test planning when no solution exists
    /// Validates: Planner returns error when goal is impossible to achieve
    /// Failure: Impossible goal detection is broken
    #[test]
    fn test_planner_no_solution() {
        let planner = Planner::new();

        let initial_state = State::new().set("has_wood", false).build();
        let goal = Goal::new("get_gold").requires("has_gold", true).build();
        let action = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let result = planner.plan(initial_state, &goal, &[action]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PlannerError::NoPlanFound);
    }

    /// Test planning when actions exist but requirements can't be met
    /// Validates: Planner detects when action preconditions can't be satisfied
    /// Failure: Unreachable precondition detection is broken
    #[test]
    fn test_planner_unreachable_preconditions() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("has_wood", false)
            .set("has_tools", false)
            .build();

        let goal = Goal::new("craft_item").requires("has_item", true).build();

        // Action requires tools but no action provides tools
        let craft_item = Action::new("craft_item")
            .cost(1.0)
            .requires("has_tools", true)
            .sets("has_item", true)
            .build();

        let get_wood = Action::new("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let actions = vec![craft_item, get_wood];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PlannerError::NoPlanFound);
    }

    // Tests for complex planning scenarios

    /// Test planning with branching paths that reconverge
    /// Validates: Planner can handle complex state spaces with multiple valid paths
    /// Failure: Complex state space navigation is broken
    #[test]
    fn test_planner_branching_paths() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("has_food", false)
            .set("location", "home")
            .build();

        let goal = Goal::new("get_food_at_market")
            .requires("has_food", true)
            .requires("location", "market")
            .build();

        // Path 1: Walk to market then buy food
        let walk_to_market = Action::new("walk_to_market")
            .cost(3.0)
            .requires("location", "home")
            .sets("location", "market")
            .build();

        let buy_food_at_market = Action::new("buy_food_at_market")
            .cost(2.0)
            .requires("location", "market")
            .sets("has_food", true)
            .build();

        // Path 2: Get food at home then walk to market
        let get_food_at_home = Action::new("get_food_at_home")
            .cost(1.0)
            .requires("location", "home")
            .sets("has_food", true)
            .build();

        let walk_to_market_with_food = Action::new("walk_to_market_with_food")
            .cost(3.0)
            .requires("location", "home")
            .requires("has_food", true)
            .sets("location", "market")
            .build();

        let actions = vec![
            walk_to_market,
            buy_food_at_market,
            get_food_at_home,
            walk_to_market_with_food,
        ];

        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.actions.len(), 2);
        assert_eq!(plan.cost, 4.0); // Should find path 2 (1.0 + 3.0) vs path 1 (3.0 + 2.0)
    }

    /// Test planning with numeric goals requiring accumulation
    /// Validates: Planner can handle goals requiring accumulation of numeric values
    /// Failure: Numeric accumulation planning is broken
    #[test]
    fn test_planner_numeric_accumulation() {
        let planner = Planner::new();

        let initial_state = State::new().set("gold", 0).build();
        let goal = Goal::new("get_rich").requires("gold", 100).build();

        let small_job = Action::new("small_job").cost(1.0).adds("gold", 30).build();

        let big_job = Action::new("big_job").cost(3.0).adds("gold", 80).build();

        let actions = vec![small_job, big_job];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        // Should find optimal combination to reach 100+ gold
        assert!(plan.cost > 0.0);
        assert!(!plan.actions.is_empty());
    }

    /// Test planning with resource consumption and production
    /// Validates: Planner can handle actions that both consume and produce resources
    /// Failure: Resource transformation planning is broken
    #[test]
    fn test_planner_resource_transformation() {
        let planner = Planner::new();

        let initial_state = State::new()
            .set("wood", 0)
            .set("planks", 0)
            .set("furniture", 0)
            .build();

        let goal = Goal::new("make_furniture").requires("furniture", 1).build();

        let chop_wood = Action::new("chop_wood").cost(1.0).adds("wood", 5).build();

        let make_planks = Action::new("make_planks")
            .cost(1.0)
            .requires("wood", 2)
            .subtracts("wood", 2)
            .adds("planks", 4)
            .build();

        let make_furniture = Action::new("make_furniture")
            .cost(2.0)
            .requires("planks", 3)
            .subtracts("planks", 3)
            .adds("furniture", 1)
            .build();

        let actions = vec![chop_wood, make_planks, make_furniture];
        let result = planner.plan(initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(!plan.actions.is_empty());
        assert!(plan.cost > 0.0);
    }
}
