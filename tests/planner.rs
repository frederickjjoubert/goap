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

        let initial_state = State::builder().set("has_wood", false).build();
        let goal = Goal::builder("get_wood").requires("has_wood", true).build();
        let action = Action::builder("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let result = planner.plan(&initial_state, &goal, &[action]);
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

        let initial_state = State::builder().set("gold", 0).build();
        let goal = Goal::builder("get_gold")
            .requires("gold", at_least(10))
            .build();
        let action = Action::builder("mine_gold")
            .cost(2.0)
            .sets("gold", 15)
            .build();

        let result = planner.plan(&initial_state, &goal, &[action]);
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

        let initial_state = State::builder().set("health", 50.0).build();
        let goal = Goal::builder("heal").requires("health", 100.0).build();
        let action = Action::builder("use_potion")
            .cost(1.5)
            .sets("health", 100.0)
            .build();

        let result = planner.plan(&initial_state, &goal, &[action]);
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

        let initial_state = State::builder().set("location", "home").build();
        let goal = Goal::builder("travel").requires("location", "town").build();
        let action = Action::builder("walk_to_town")
            .cost(3.0)
            .sets("location", "town")
            .build();

        let result = planner.plan(&initial_state, &goal, &[action]);
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

        let initial_state = State::builder()
            .set("has_wood", false)
            .set("has_planks", false)
            .build();

        let goal = Goal::builder("get_planks")
            .requires("has_planks", true)
            .build();

        let get_wood = Action::builder("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let craft_planks = Action::builder("craft_planks")
            .cost(2.0)
            .requires("has_wood", true)
            .sets("has_planks", true)
            .build();

        let actions = vec![get_wood, craft_planks];
        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder()
            .set("has_wood", false)
            .set("has_planks", false)
            .set("has_table", false)
            .build();

        let goal = Goal::builder("make_table")
            .requires("has_table", true)
            .build();

        let get_wood = Action::builder("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let craft_planks = Action::builder("craft_planks")
            .cost(2.0)
            .requires("has_wood", true)
            .sets("has_planks", true)
            .build();

        let make_table = Action::builder("make_table")
            .cost(3.0)
            .requires("has_planks", true)
            .sets("has_table", true)
            .build();

        let actions = vec![get_wood, craft_planks, make_table];
        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder().set("has_gold", false).build();
        let goal = Goal::builder("get_gold").requires("has_gold", true).build();

        // Expensive direct action
        let expensive_action = Action::builder("buy_gold")
            .cost(10.0)
            .sets("has_gold", true)
            .build();

        // Cheaper action
        let cheap_action = Action::builder("find_gold")
            .cost(2.0)
            .sets("has_gold", true)
            .build();

        let actions = vec![expensive_action, cheap_action];
        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder()
            .set("has_materials", false)
            .set("has_sword", false)
            .build();

        let goal = Goal::builder("get_sword")
            .requires("has_sword", true)
            .build();

        // Expensive direct path
        let buy_sword = Action::builder("buy_sword")
            .cost(10.0)
            .sets("has_sword", true)
            .build();

        // Cheaper multi-step path
        let gather_materials = Action::builder("gather_materials")
            .cost(2.0)
            .sets("has_materials", true)
            .build();

        let craft_sword = Action::builder("craft_sword")
            .cost(3.0)
            .requires("has_materials", true)
            .sets("has_sword", true)
            .build();

        let actions = vec![buy_sword, gather_materials, craft_sword];
        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder()
            .set("has_key", false)
            .set("door_unlocked", false)
            .build();

        let goal = Goal::builder("enter_room")
            .requires("has_key", true)
            .requires("door_unlocked", true)
            .build();

        let get_key = Action::builder("get_key")
            .cost(1.0)
            .sets("has_key", true)
            .build();

        let unlock_door = Action::builder("unlock_door")
            .cost(1.0)
            .requires("has_key", true)
            .sets("door_unlocked", true)
            .build();

        let actions = vec![get_key, unlock_door];
        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder()
            .set("has_weapon", false)
            .set("gold", 0)
            .set("health", 50.0)
            .set("location", "home")
            .build();

        let goal = Goal::builder("ready_for_adventure")
            .requires("has_weapon", true)
            .requires("gold", at_least(100))
            .requires("health", 100.0)
            .requires("location", "town")
            .build();

        let buy_weapon = Action::builder("buy_weapon")
            .cost(1.0)
            .sets("has_weapon", true)
            .build();

        let earn_gold = Action::builder("work").cost(2.0).sets("gold", 150).build();

        let heal = Action::builder("rest")
            .cost(1.0)
            .sets("health", 100.0)
            .build();

        let travel = Action::builder("travel_to_town")
            .cost(1.0)
            .sets("location", "town")
            .build();

        let actions = vec![buy_weapon, earn_gold, heal, travel];
        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder().set("has_item", true).build();
        let goal = Goal::builder("get_item").requires("has_item", true).build();
        let action = Action::builder("get_item")
            .cost(1.0)
            .sets("has_item", true)
            .build();

        let result = planner.plan(&initial_state, &goal, &[action]);
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

        let initial_state = State::builder().set("has_item", false).build();
        let goal = Goal::builder("get_item").requires("has_item", true).build();
        let actions: Vec<Action> = vec![];

        let result = planner.plan(&initial_state, &goal, &actions);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PlannerError::NoPlanFound);
    }

    /// Test planning when no solution exists
    /// Validates: Planner returns error when goal is impossible to achieve
    /// Failure: Impossible goal detection is broken
    #[test]
    fn test_planner_no_solution() {
        let planner = Planner::new();

        let initial_state = State::builder().set("has_wood", false).build();
        let goal = Goal::builder("get_gold").requires("has_gold", true).build();
        let action = Action::builder("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let result = planner.plan(&initial_state, &goal, &[action]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PlannerError::NoPlanFound);
    }

    /// Test planning when actions exist but requirements can't be met
    /// Validates: Planner detects when action preconditions can't be satisfied
    /// Failure: Unreachable precondition detection is broken
    #[test]
    fn test_planner_unreachable_preconditions() {
        let planner = Planner::new();

        let initial_state = State::builder()
            .set("has_wood", false)
            .set("has_tools", false)
            .build();

        let goal = Goal::builder("craft_item")
            .requires("has_item", true)
            .build();

        // Action requires tools but no action provides tools
        let craft_item = Action::builder("craft_item")
            .cost(1.0)
            .requires("has_tools", true)
            .sets("has_item", true)
            .build();

        let get_wood = Action::builder("get_wood")
            .cost(1.0)
            .sets("has_wood", true)
            .build();

        let actions = vec![craft_item, get_wood];
        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder()
            .set("has_food", false)
            .set("location", "home")
            .build();

        let goal = Goal::builder("get_food_at_market")
            .requires("has_food", true)
            .requires("location", "market")
            .build();

        // Path 1: Walk to market then buy food
        let walk_to_market = Action::builder("walk_to_market")
            .cost(3.0)
            .requires("location", "home")
            .sets("location", "market")
            .build();

        let buy_food_at_market = Action::builder("buy_food_at_market")
            .cost(2.0)
            .requires("location", "market")
            .sets("has_food", true)
            .build();

        // Path 2: Get food at home then walk to market
        let get_food_at_home = Action::builder("get_food_at_home")
            .cost(1.0)
            .requires("location", "home")
            .sets("has_food", true)
            .build();

        let walk_to_market_with_food = Action::builder("walk_to_market_with_food")
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

        let result = planner.plan(&initial_state, &goal, &actions);
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

        let initial_state = State::builder().set("gold", 0).build();
        let goal = Goal::builder("get_rich")
            .requires("gold", at_least(100))
            .build();

        let small_job = Action::builder("small_job")
            .cost(1.0)
            .adds("gold", 30)
            .build();

        let big_job = Action::builder("big_job")
            .cost(3.0)
            .adds("gold", 80)
            .build();

        let actions = vec![small_job, big_job];
        let result = planner.plan(&initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        // Four small jobs (cost 4, 120 gold) beat big + small (cost 4, 110
        // gold) only on tie; either is optimal at cost 4.
        assert_eq!(plan.cost, 4.0);
    }

    /// Test planning with resource consumption and production
    /// Validates: Planner can handle actions that both consume and produce resources
    /// Failure: Resource transformation planning is broken
    #[test]
    fn test_planner_resource_transformation() {
        let planner = Planner::new();

        let initial_state = State::builder()
            .set("wood", 0)
            .set("planks", 0)
            .set("furniture", 0)
            .build();

        let goal = Goal::builder("make_furniture")
            .requires("furniture", at_least(1))
            .build();

        let chop_wood = Action::builder("chop_wood")
            .cost(1.0)
            .adds("wood", 5)
            .build();

        let make_planks = Action::builder("make_planks")
            .cost(1.0)
            .requires("wood", at_least(2))
            .subtracts("wood", 2)
            .adds("planks", 4)
            .build();

        let make_furniture = Action::builder("make_furniture")
            .cost(2.0)
            .requires("planks", at_least(3))
            .subtracts("planks", 3)
            .adds("furniture", 1)
            .build();

        let actions = vec![chop_wood, make_planks, make_furniture];
        let result = planner.plan(&initial_state, &goal, &actions);
        assert!(result.is_ok());

        let plan = result.unwrap();
        // chop, planks, furniture
        assert_eq!(plan.cost, 4.0);
        assert_eq!(plan.actions.len(), 3);
    }

    // Tests for optimality (BUG-001 regression)

    /// A* with the old distance-based heuristic preferred five cheap "work"
    /// steps (cost 5) over get_key + open_vault (cost 2). The heuristic must
    /// never overestimate, so the planner must find the cost-2 plan.
    #[test]
    fn test_planner_prefers_cheap_two_step_over_many_small_steps() {
        let planner = Planner::new();
        let initial_state = State::builder()
            .set("gold", 0)
            .set("has_key", false)
            .build();
        let goal = Goal::builder("rich")
            .requires("gold", at_least(100))
            .build();
        let actions = vec![
            Action::builder("work").cost(1.0).adds("gold", 20).build(),
            Action::builder("get_key")
                .cost(1.0)
                .sets("has_key", true)
                .build(),
            Action::builder("open_vault")
                .cost(1.0)
                .requires("has_key", true)
                .adds("gold", 100)
                .build(),
        ];

        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 2.0);
        let names: Vec<&str> = plan.actions.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["get_key", "open_vault"]);
    }

    /// A goal that is already over-satisfied must cost nothing to reach.
    #[test]
    fn test_planner_overshoot_is_already_satisfied() {
        let planner = Planner::new();
        let initial_state = State::builder().set("gold", 150).build();
        let goal = Goal::builder("rich")
            .requires("gold", at_least(100))
            .build();
        let actions = vec![Action::builder("work").cost(1.0).adds("gold", 20).build()];

        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert!(plan.actions.is_empty());
        assert_eq!(plan.cost, 0.0);
    }

    /// Float goals used to inflate the heuristic 1000x (fixed-point units).
    #[test]
    fn test_planner_float_goal_is_optimal() {
        let planner = Planner::new();
        let initial_state = State::builder().set("temp", 20.0).set("on", false).build();
        let goal = Goal::builder("warm")
            .requires("temp", at_least(24.0))
            .build();
        let actions = vec![
            Action::builder("nudge").cost(1.0).adds("temp", 0.5).build(),
            Action::builder("switch_on")
                .cost(1.0)
                .sets("on", true)
                .build(),
            Action::builder("blast")
                .cost(1.5)
                .requires("on", true)
                .adds("temp", 10.0)
                .build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 2.5, "{plan}");
    }

    /// Goals with "at most" and exact-zero requirements plan correctly.
    #[test]
    fn test_planner_decreasing_and_exact_goals() {
        let planner = Planner::new();
        let initial_state = State::builder().set("alert", 3).set("noise", 7).build();
        let goal = Goal::builder("quiet")
            .requires("alert", at_most(0))
            .requires("noise", 0)
            .build();
        let actions = vec![
            Action::builder("hide")
                .cost(1.0)
                .subtracts("alert", 1)
                .build(),
            Action::builder("hush")
                .cost(1.0)
                .subtracts("noise", 3)
                .build(),
            Action::builder("silence")
                .cost(2.0)
                .sets("noise", 0)
                .build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        // hide x3 (3) + silence (2) = 5; hush x3 would overshoot to -2 and
        // never equal 0, hush x2 leaves 1.
        assert_eq!(plan.cost, 5.0, "{plan}");
    }

    // Tests for early unreachable detection (BUG-004 mitigation)

    /// With an unbounded "adds" action the old planner would search forever
    /// for a goal no action can satisfy. Now it returns immediately.
    #[test]
    fn test_planner_unreachable_goal_with_unbounded_actions_returns_quickly() {
        let planner = Planner::new();
        let initial_state = State::builder().set("gold", 0).build();
        let goal = Goal::builder("g").requires("has_key", true).build();
        let actions = vec![Action::builder("work").adds("gold", 1).build()];

        let start = std::time::Instant::now();
        let result = planner.plan(&initial_state, &goal, &actions);
        assert_eq!(result.unwrap_err(), PlannerError::NoPlanFound);
        assert!(start.elapsed() < std::time::Duration::from_secs(1));
    }

    /// A goal variable that only ever moves the wrong way is unreachable.
    #[test]
    fn test_planner_unreachable_wrong_direction_returns_quickly() {
        let planner = Planner::new();
        let initial_state = State::builder().set("gold", 0).build();
        let goal = Goal::builder("g").requires("gold", at_least(10)).build();
        let actions = vec![Action::builder("spend").subtracts("gold", 1).build()];

        let start = std::time::Instant::now();
        let result = planner.plan(&initial_state, &goal, &actions);
        assert_eq!(result.unwrap_err(), PlannerError::NoPlanFound);
        assert!(start.elapsed() < std::time::Duration::from_secs(1));
    }

    // Tests for the search budget and reachability analysis (BUG-004)

    /// Unreachable only through numeric interplay: gold moves in steps of 2
    /// in both directions and unlock needs exactly 5. No cheap analysis sees
    /// the parity argument, so the budget has to stop the search. (With only
    /// the `adds`, the plan 006 heuristic proves the goal impossible as soon
    /// as gold passes 5, so `spend` keeps the space genuinely undecidable.)
    #[test]
    fn test_planner_budget_stops_unreachable_search_in_unbounded_space() {
        let planner = Planner::new();
        let initial_state = State::builder()
            .set("gold", 0)
            .set("has_key", false)
            .build();
        let goal = Goal::builder("g").requires("has_key", true).build();
        let actions = vec![
            Action::builder("work").cost(1.0).adds("gold", 2).build(),
            Action::builder("spend")
                .cost(1.0)
                .subtracts("gold", 2)
                .build(),
            Action::builder("unlock")
                .cost(1.0)
                .requires("gold", 5)
                .sets("has_key", true)
                .build(),
        ];

        let start = std::time::Instant::now();
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert_eq!(
            err,
            PlannerError::SearchLimitExceeded {
                expanded: DEFAULT_MAX_EXPANSIONS
            }
        );
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
        assert!(err.to_string().contains("max_expansions"), "{err}");
    }

    /// Unreachable because unlock's precondition can never hold. The
    /// reachability analysis proves this, so the answer is `NoPlanFound`, not
    /// a budget error, and no search runs.
    #[test]
    fn test_planner_precondition_chain_is_proved_unreachable() {
        let planner = Planner::new();
        let initial_state = State::builder()
            .set("gold", 0)
            .set("has_key", false)
            .set("has_torch", false)
            .build();
        let goal = Goal::builder("g").requires("has_key", true).build();
        let actions = vec![
            Action::builder("work").cost(1.0).adds("gold", 1).build(),
            Action::builder("unlock")
                .cost(1.0)
                .requires("has_torch", true)
                .sets("has_key", true)
                .build(),
        ];

        let start = std::time::Instant::now();
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert_eq!(err, PlannerError::NoPlanFound);
        assert!(start.elapsed() < std::time::Duration::from_millis(100));
    }

    /// A problem that needs exactly three expansions (steps 0, 1 and 2) to
    /// reach the goal, used to squeeze the budget.
    fn three_step_problem() -> (State, Goal, Vec<Action>) {
        let initial_state = State::builder().set("step", 0).build();
        let goal = Goal::builder("g").requires("step", 3).build();
        let actions = vec![Action::builder("next").cost(1.0).adds("step", 1).build()];
        (initial_state, goal, actions)
    }

    #[test]
    fn test_planner_max_expansions_is_honoured() {
        let (initial_state, goal, actions) = three_step_problem();

        // Needs 3 expansions (steps 0, 1, 2). A budget of 2 is too small.
        let err = Planner::new()
            .max_expansions(2)
            .plan(&initial_state, &goal, &actions)
            .unwrap_err();
        assert_eq!(err, PlannerError::SearchLimitExceeded { expanded: 2 });

        // Exactly enough: the goal node is popped after 3 expansions and the
        // check runs after the goal test, so the plan is still returned.
        let plan = Planner::new()
            .max_expansions(3)
            .plan(&initial_state, &goal, &actions)
            .unwrap();
        assert_eq!(plan.actions.len(), 3);

        let plan = Planner::new()
            .plan(&initial_state, &goal, &actions)
            .unwrap();
        assert_eq!(plan.cost, 3.0);
    }

    #[test]
    fn test_planner_without_expansion_limit_still_finds_no_plan_in_finite_space() {
        let planner = Planner::new().without_expansion_limit();
        assert_eq!(planner.expansion_limit(), None);
        assert_eq!(
            Planner::new().expansion_limit(),
            Some(DEFAULT_MAX_EXPANSIONS)
        );
        assert_eq!(Planner::default(), Planner::new());

        // A finite, unreachable problem: `a` can be set only when `b` holds,
        // and `b` only when `a` already is 1. Whether the analysis or the
        // search refutes it, the call must return NoPlanFound.
        let initial_state = State::builder().set("a", 0).set("b", false).build();
        let goal = Goal::builder("g").requires("a", 1).build();
        let actions = vec![
            Action::builder("set_a")
                .cost(1.0)
                .requires("b", true)
                .sets("a", 1)
                .build(),
            Action::builder("set_b")
                .cost(1.0)
                .requires("a", 1)
                .sets("b", true)
                .build(),
        ];
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert_eq!(err, PlannerError::NoPlanFound);
    }

    // Reachability must never reject a solvable problem.

    #[test]
    fn test_planner_reachability_allows_enabling_chains() {
        let planner = Planner::new();

        // Three actions each enabling the next.
        let initial_state = State::builder()
            .set("a", false)
            .set("b", false)
            .set("c", false)
            .build();
        let goal = Goal::builder("g").requires("c", true).build();
        let actions = vec![
            Action::builder("c")
                .cost(1.0)
                .requires("b", true)
                .sets("c", true)
                .build(),
            Action::builder("b")
                .cost(1.0)
                .requires("a", true)
                .sets("b", true)
                .build(),
            Action::builder("a").cost(1.0).sets("a", true).build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.actions.len(), 3);

        // Enabled only once a numeric variable crosses a threshold.
        let initial_state = State::builder()
            .set("gold", 0)
            .set("has_sword", false)
            .build();
        let goal = Goal::builder("g").requires("has_sword", true).build();
        let actions = vec![
            Action::builder("buy")
                .cost(1.0)
                .requires("gold", at_least(30))
                .sets("has_sword", true)
                .build(),
            Action::builder("work").cost(1.0).adds("gold", 10).build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 4.0);

        // Enabled only by a string Set from another action.
        let initial_state = State::builder()
            .set("location", "home")
            .set("has_bread", false)
            .build();
        let goal = Goal::builder("g").requires("has_bread", true).build();
        let actions = vec![
            Action::builder("buy_bread")
                .cost(1.0)
                .requires("location", "bakery")
                .sets("has_bread", true)
                .build(),
            Action::builder("walk")
                .cost(1.0)
                .sets("location", "bakery")
                .build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.actions.len(), 2);
    }

    #[test]
    fn test_planner_reachability_handles_variables_missing_from_initial_state() {
        let planner = Planner::new();
        let initial_state = State::builder().set("ready", false).build();
        let goal = Goal::builder("g").requires("count", at_least(2)).build();

        // Absent and only ever added to: a missing variable counts as zero,
        // so two increments reach the goal.
        let actions = vec![Action::builder("inc").cost(1.0).adds("count", 1).build()];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 2.0);
        assert_eq!(plan.actions.len(), 2);

        // Absent and only ever decremented: the goal needs an increase, so
        // the reachability analysis proves it impossible.
        let actions = vec![
            Action::builder("dec")
                .cost(1.0)
                .subtracts("count", 1)
                .build(),
        ];
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert_eq!(err, PlannerError::NoPlanFound);

        // Set to 5 is cheaper than two increments from zero.
        let actions = vec![
            Action::builder("init").cost(1.5).sets("count", 5).build(),
            Action::builder("inc").cost(1.0).adds("count", 1).build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 1.5);
    }

    /// Dead actions are dropped before the search. The plan must still name
    /// the right actions when the dead one comes first in the slice.
    #[test]
    fn test_planner_drops_dead_actions_and_keeps_indices_straight() {
        let planner = Planner::new();
        let initial_state = State::builder().set("x", 0).set("has_wings", false).build();
        let goal = Goal::builder("g").requires("x", 2).build();
        let actions = vec![
            Action::builder("fly")
                .cost(0.5)
                .requires("has_wings", true)
                .sets("x", 2)
                .build(),
            Action::builder("walk").cost(1.0).adds("x", 1).build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        let names: Vec<&str> = plan.actions.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["walk", "walk"]);
        assert_eq!(plan.cost, 2.0);
    }

    // Reachability should reject what it can see.

    #[test]
    fn test_planner_reachability_rejects_wrong_direction_precondition() {
        let planner = Planner::new();
        let initial_state = State::builder().set("gold", 5).set("done", false).build();
        let goal = Goal::builder("g").requires("done", true).build();
        let actions = vec![
            Action::builder("spend")
                .cost(1.0)
                .subtracts("gold", 1)
                .build(),
            Action::builder("finish")
                .cost(1.0)
                .requires("gold", at_least(10))
                .sets("done", true)
                .build(),
        ];
        let start = std::time::Instant::now();
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert_eq!(err, PlannerError::NoPlanFound);
        assert!(start.elapsed() < std::time::Duration::from_millis(100));
    }

    // Tests for typed effects (BUG-005, BUG-009)

    /// BUG-005: an integer delta on a float variable used to add 0.005 per
    /// step and return a 1000-action plan. It is now rejected up front.
    #[test]
    fn test_planner_rejects_delta_of_wrong_type() {
        let planner = Planner::new();
        let initial_state = State::builder().set("temp", 20.0).build();
        let goal = Goal::builder("warm")
            .requires("temp", at_least(25.0))
            .build();
        let actions = vec![Action::builder("heat").cost(1.0).adds("temp", 5).build()];

        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert_eq!(
            err,
            PlannerError::IncompatibleStateTypes {
                variable: "temp".to_string(),
                expected: StateType::F64,
                expected_origin: VariableOrigin::InitialState,
                found: StateType::I64,
                found_origin: VariableOrigin::Effects("heat".to_string()),
            }
        );
        let msg = err.to_string();
        assert!(msg.contains("temp"), "{msg}");
        assert!(msg.contains("f64"), "{msg}");
        assert!(msg.contains("i64"), "{msg}");
        assert!(msg.contains("heat"), "{msg}");

        // The other direction, and Subtract, are rejected the same way.
        let initial_state = State::builder().set("count", 1).build();
        let goal = Goal::builder("g").requires("count", at_least(5)).build();
        let actions = vec![Action::builder("x").subtracts("count", -1.5).build()];
        assert!(matches!(
            planner.plan(&initial_state, &goal, &actions),
            Err(PlannerError::IncompatibleStateTypes { .. })
        ));

        // Correctly typed, the same problem plans in one step.
        let initial_state = State::builder().set("temp", 20.0).build();
        let goal = Goal::builder("warm")
            .requires("temp", at_least(25.0))
            .build();
        let actions = vec![Action::builder("heat").cost(1.0).adds("temp", 5.0).build()];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.cost, 1.0);
    }

    /// A mismatch between two effects (Set f64, Add i64) with the variable
    /// absent from the initial state is caught as well.
    #[test]
    fn test_planner_rejects_delta_that_disagrees_with_a_set_effect() {
        let planner = Planner::new();
        let initial_state = State::new();
        let goal = Goal::builder("g").requires("x", at_least(1.0)).build();
        let actions = vec![
            Action::builder("init").sets("x", 0.0).build(),
            Action::builder("inc").adds("x", 1).build(),
        ];
        assert!(matches!(
            planner.plan(&initial_state, &goal, &actions),
            Err(PlannerError::IncompatibleStateTypes { .. })
        ));
    }

    /// A hand-built `Add` with a non-numeric payload cannot come from the
    /// builder, but the planner still refuses it rather than searching with
    /// an action that does nothing.
    #[test]
    fn test_planner_rejects_non_numeric_delta() {
        let planner = Planner::new();
        let initial_state = State::builder().set("flag", false).build();
        let goal = Goal::builder("g").requires("flag", true).build();
        let mut effects = std::collections::HashMap::new();
        effects.insert(
            "flag".to_string(),
            StateOperation::Add(StateVar::Bool(true)),
        );
        let actions = vec![Action::from_parts("odd", 1.0, Conditions::new(), effects)];

        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert_eq!(
            err,
            PlannerError::InvalidEffect {
                variable: "flag".to_string(),
                action: "odd".to_string(),
                operation: StateOperation::Add(StateVar::Bool(true)),
            }
        );
        let msg = err.to_string();
        assert!(msg.contains("odd"), "{msg}");
        assert!(msg.contains("added or subtracted"), "{msg}");
    }

    /// BUG-009: counters that start absent are driven by `adds` alone, and
    /// the plan is still optimal (the heuristic prices steps from zero).
    #[test]
    fn test_planner_counter_missing_from_initial_state() {
        let planner = Planner::new();
        let initial_state = State::builder().set("has_tools", true).build();
        let goal = Goal::builder("g")
            .requires("items_crafted", at_least(3))
            .requires("experience", at_least(2.5))
            .build();
        let actions = vec![
            Action::builder("craft")
                .cost(1.0)
                .requires("has_tools", true)
                .adds("items_crafted", 1)
                .adds("experience", 1.0)
                .build(),
            Action::builder("craft_batch")
                .cost(2.5)
                .requires("has_tools", true)
                .adds("items_crafted", 3)
                .adds("experience", 2.5)
                .build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 2.5);
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "craft_batch");
    }

    /// Regression: a variable that starts absent can be created either by a
    /// `Set` or by an `Add` from zero. The reachability analysis must keep
    /// zero as a possible value even when a `Set` is listed first.
    #[test]
    fn test_planner_missing_variable_keeps_zero_reachable_after_a_set() {
        let planner = Planner::new();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![
            Action::builder("init").cost(1.0).sets("x", 10).build(),
            Action::builder("add").cost(1.0).adds("x", 1).build(),
        ];
        let plan = planner.plan(&State::new(), &goal, &actions).unwrap();
        assert_eq!(plan.cost, 1.0);
        assert_eq!(plan.actions[0].name, "add");

        // Same with the variable present initially: zero is not special.
        let initial = State::builder().set("x", 10).build();
        let err = planner.plan(&initial, &goal, &actions).unwrap_err();
        assert_eq!(err, PlannerError::NoPlanFound);
    }

    /// Saturation inside the search does not panic and the search still
    /// terminates with a correct answer.
    #[test]
    fn test_planner_saturating_effects_do_not_panic() {
        let planner = Planner::new();
        let initial_state = State::builder().set("x", i64::MAX - 1).build();
        let goal = Goal::builder("g").requires("x", at_least(i64::MAX)).build();
        let actions = vec![Action::builder("inc").cost(1.0).adds("x", 5).build()];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 1.0);
    }

    // Tests for validation

    #[test]
    fn test_planner_rejects_negative_cost() {
        let planner = Planner::new();
        let initial_state = State::builder().set("x", 0).build();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![
            Action::builder("free_lunch").cost(-1.0).build(),
            Action::builder("do_it").cost(10.0).sets("x", 1).build(),
        ];
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert!(
            matches!(err, PlannerError::InvalidActionCost { .. }),
            "{err}"
        );
    }

    #[test]
    fn test_planner_rejects_nan_cost() {
        let planner = Planner::new();
        let initial_state = State::builder().set("x", 0).build();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![Action::builder("nan").cost(f64::NAN).sets("x", 1).build()];
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert!(
            matches!(err, PlannerError::InvalidActionCost { .. }),
            "{err}"
        );
    }

    #[test]
    fn test_planner_zero_cost_is_allowed() {
        let planner = Planner::new();
        let initial_state = State::builder().set("x", 0).build();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![
            Action::builder("noop").cost(0.0).build(),
            Action::builder("do_it").cost(1.0).sets("x", 1).build(),
        ];
        let plan = planner.plan(&initial_state, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 1.0);
    }

    // Tests for exact cost arithmetic (BUG-013)

    /// Costs are summed in thousandths, so float rounding never leaks into
    /// `plan.cost`.
    #[test]
    fn test_planner_cost_is_exact_over_many_small_steps() {
        let planner = Planner::new();

        // Ten steps of 0.1 are exactly 1.0, not 0.9999999999999999.
        let goal = Goal::builder("g").requires("x", at_least(10)).build();
        let actions = vec![Action::builder("step").cost(0.1).adds("x", 1).build()];
        let plan = planner
            .plan(&State::builder().set("x", 0).build(), &goal, &actions)
            .unwrap();
        assert_eq!(plan.actions.len(), 10);
        assert_eq!(plan.cost, 1.0);

        // 0.1 + 0.2 + 0.3 in any order is exactly 0.6.
        let goal = Goal::builder("g")
            .requires("a", true)
            .requires("b", true)
            .requires("c", true)
            .build();
        let actions = vec![
            Action::builder("x").cost(0.1).sets("a", true).build(),
            Action::builder("y").cost(0.2).sets("b", true).build(),
            Action::builder("z").cost(0.3).sets("c", true).build(),
        ];
        let initial = State::builder()
            .set("a", false)
            .set("b", false)
            .set("c", false)
            .build();
        let plan = planner.plan(&initial, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 0.6);
    }

    /// Costs beyond the fixed-point range are rejected like NaN and infinity;
    /// the largest representable cost is accepted.
    #[test]
    fn test_planner_rejects_cost_above_representable_range() {
        let planner = Planner::new();
        let goal = Goal::builder("g").requires("x", 1).build();

        let actions = vec![Action::builder("huge").cost(1e16).sets("x", 1).build()];
        let err = planner
            .plan(&State::builder().set("x", 0).build(), &goal, &actions)
            .unwrap_err();
        assert!(
            matches!(err, PlannerError::InvalidActionCost { .. }),
            "{err}"
        );

        let actions = vec![
            Action::builder("max")
                .cost(StateVar::MAX_F64)
                .sets("x", 1)
                .build(),
        ];
        let plan = planner
            .plan(&State::builder().set("x", 0).build(), &goal, &actions)
            .unwrap();
        assert_eq!(plan.cost, StateVar::MAX_F64);
    }

    /// A NaN or infinite goal or precondition value cannot be built: the
    /// float conversion panics at construction time (BUG-014).
    #[test]
    #[should_panic(expected = "try_from_f64")]
    fn test_planner_nan_condition_value_panics_at_construction() {
        let _ = Goal::builder("g").requires("x", at_least(f64::NAN)).build();
    }

    /// Summing two maximal costs saturates instead of overflowing, and the
    /// plan is still found.
    #[test]
    fn test_planner_saturating_cost_sum_does_not_panic() {
        let planner = Planner::new();
        let goal = Goal::builder("g").requires("x", 2).build();
        let actions = vec![
            Action::builder("one")
                .cost(StateVar::MAX_F64)
                .requires("x", 0)
                .sets("x", 1)
                .build(),
            Action::builder("two")
                .cost(StateVar::MAX_F64)
                .requires("x", 1)
                .sets("x", 2)
                .build(),
        ];
        let plan = planner
            .plan(&State::builder().set("x", 0).build(), &goal, &actions)
            .unwrap();
        assert_eq!(plan.actions.len(), 2);
        assert!(plan.cost >= StateVar::MAX_F64);
    }

    /// The planner uses the builder's rounded cost: 0.0004 rounds to zero, so
    /// the "cheaper" action is free and the cost-1 action is never needed.
    #[test]
    fn test_planner_compares_costs_at_thousandth_resolution() {
        let planner = Planner::new();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![
            Action::builder("almost_free")
                .cost(0.0004)
                .sets("x", 1)
                .build(),
            Action::builder("cheap").cost(0.001).sets("x", 1).build(),
        ];
        let plan = planner
            .plan(&State::builder().set("x", 0).build(), &goal, &actions)
            .unwrap();
        assert_eq!(plan.actions[0].name, "almost_free");
        assert_eq!(plan.cost, 0.0);
    }

    #[test]
    fn test_planner_reports_type_mismatch_between_goal_and_state() {
        let planner = Planner::new();
        let initial_state = State::builder().set("gold", 10).build();
        let goal = Goal::builder("g").requires("gold", at_least(5.0)).build();
        let err = planner.plan(&initial_state, &goal, &[]).unwrap_err();
        assert_eq!(
            err,
            PlannerError::IncompatibleStateTypes {
                variable: "gold".to_string(),
                expected: StateType::I64,
                expected_origin: VariableOrigin::InitialState,
                found: StateType::F64,
                found_origin: VariableOrigin::Goal("g".to_string()),
            }
        );
        let msg = err.to_string();
        assert!(msg.contains("'gold'"), "{msg}");
        assert!(msg.contains("i64"), "{msg}");
        assert!(msg.contains("f64"), "{msg}");
    }

    #[test]
    fn test_planner_reports_type_mismatch_in_effects() {
        let planner = Planner::new();
        let initial_state = State::builder().set("gold", 10).build();
        let goal = Goal::builder("g").requires("gold", at_least(50)).build();
        let actions = vec![Action::builder("weird").sets("gold", "lots").build()];
        let err = planner.plan(&initial_state, &goal, &actions).unwrap_err();
        assert!(
            matches!(err, PlannerError::IncompatibleStateTypes { .. }),
            "{err}"
        );
    }

    // Property test: A* must agree with uniform-cost search on random
    // finite problems. Any disagreement means the heuristic overestimated.

    /// Minimal xorshift generator so the test needs no dependencies.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }

        fn chance(&mut self, num: u64, den: u64) -> bool {
            self.below(den) < num
        }
    }

    /// Uniform-cost search using only the public API. Optimal by
    /// construction for non-negative costs. Costs are summed in thousandths,
    /// the planner's own resolution, so the comparison can be exact.
    fn reference_cost(initial: &State, goal: &Goal, actions: &[Action]) -> Option<f64> {
        use std::cmp::{Ordering, Reverse};
        use std::collections::{BinaryHeap, HashMap};

        fn thousandths(cost: f64) -> u64 {
            (cost * 1000.0).round() as u64
        }

        struct Entry(u64, State);
        impl PartialEq for Entry {
            fn eq(&self, o: &Self) -> bool {
                self.0 == o.0
            }
        }
        impl Eq for Entry {}
        impl PartialOrd for Entry {
            fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
                Some(self.cmp(o))
            }
        }
        impl Ord for Entry {
            fn cmp(&self, o: &Self) -> Ordering {
                Reverse(self.0).cmp(&Reverse(o.0))
            }
        }

        let mut best: HashMap<State, u64> = HashMap::new();
        let mut heap = BinaryHeap::new();
        best.insert(initial.clone(), 0);
        heap.push(Entry(0, initial.clone()));
        while let Some(Entry(g, state)) = heap.pop() {
            if g > best[&state] {
                continue;
            }
            if goal.is_satisfied(&state) {
                return Some(g as f64 / 1000.0);
            }
            for action in actions {
                if !action.can_execute(&state) {
                    continue;
                }
                let next = action.apply_effect(&state);
                let ng = g + thousandths(action.cost);
                if ng < *best.get(&next).unwrap_or(&u64::MAX) {
                    best.insert(next.clone(), ng);
                    heap.push(Entry(ng, next));
                }
            }
        }
        None
    }

    const INT_VARS: [&str; 3] = ["a", "b", "c"];
    const BOOL_VARS: [&str; 3] = ["p", "q", "r"];
    const INT_MIN: i64 = 0;
    const INT_MAX: i64 = 12;

    fn random_problem(rng: &mut Rng) -> (State, Goal, Vec<Action>) {
        // Each integer variable is absent from the initial state one time in
        // four, so missing-variable semantics (conditions fail, `Set`
        // creates, `Add` counts from zero) are exercised too.
        let mut state = State::builder();
        for v in INT_VARS {
            if rng.chance(1, 4) {
                continue;
            }
            state = state.set(
                v,
                rng.below((INT_MAX - INT_MIN + 1) as u64) as i64 + INT_MIN,
            );
        }
        for v in BOOL_VARS {
            state = state.set(v, rng.chance(1, 2));
        }
        let initial = state.build();

        // Goal: 1-3 conditions.
        let mut goal = Goal::builder("g");
        let n_conds = 1 + rng.below(3);
        for _ in 0..n_conds {
            if rng.chance(1, 2) {
                let v = INT_VARS[rng.below(3) as usize];
                let t = rng.below((INT_MAX - INT_MIN + 1) as u64) as i64 + INT_MIN;
                goal = match rng.below(6) {
                    0 => goal.requires(v, t),
                    1 => goal.requires(v, at_least(t)),
                    2 => goal.requires(v, at_most(t)),
                    3 => goal.requires(v, more_than(t)),
                    4 => goal.requires(v, less_than(t)),
                    _ => goal.requires(v, between(t, (t + 3).min(INT_MAX))),
                };
            } else {
                let v = BOOL_VARS[rng.below(3) as usize];
                goal = goal.requires(v, rng.chance(1, 2));
            }
        }
        let goal = goal.build();

        // Actions: 3-7, each with 1-3 effects and 0-2 preconditions. Integer
        // effects are clamped by preconditions so the state space is finite.
        let mut actions = Vec::new();
        let n_actions = 3 + rng.below(5);
        for i in 0..n_actions {
            // Costs in tenths (0.1 to 5.0) so that fixed-point summation is exercised.
            let mut a = Action::builder(&format!("a{i}")).cost((1 + rng.below(50)) as f64 / 10.0);
            let n_pre = rng.below(3);
            for _ in 0..n_pre {
                if rng.chance(1, 2) {
                    let v = INT_VARS[rng.below(3) as usize];
                    let t = rng.below((INT_MAX - INT_MIN + 1) as u64) as i64 + INT_MIN;
                    a = if rng.chance(1, 2) {
                        a.requires(v, at_least(t))
                    } else {
                        a.requires(v, at_most(t))
                    };
                } else {
                    let v = BOOL_VARS[rng.below(3) as usize];
                    a = a.requires(v, rng.chance(1, 2));
                }
            }
            let n_eff = 1 + rng.below(3);
            for _ in 0..n_eff {
                if rng.chance(1, 2) {
                    let v = INT_VARS[rng.below(3) as usize];
                    let delta = 1 + rng.below(4) as i64;
                    a = match rng.below(3) {
                        0 => a.requires(v, at_most(INT_MAX - delta)).adds(v, delta),
                        1 => a.requires(v, at_least(INT_MIN + delta)).subtracts(v, delta),
                        _ => a.sets(
                            v,
                            rng.below((INT_MAX - INT_MIN + 1) as u64) as i64 + INT_MIN,
                        ),
                    };
                } else {
                    let v = BOOL_VARS[rng.below(3) as usize];
                    a = a.sets(v, rng.chance(1, 2));
                }
            }
            actions.push(a.build());
        }
        (initial, goal, actions)
    }

    #[test]
    fn test_planner_matches_uniform_cost_search_on_random_problems() {
        let planner = Planner::new();
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let mut solved = 0;
        for case in 0..600 {
            let (initial, goal, actions) = random_problem(&mut rng);
            let expected = reference_cost(&initial, &goal, &actions);
            let actual = planner.plan(&initial, &goal, &actions);
            match (expected, actual) {
                (None, Err(PlannerError::NoPlanFound)) => {}
                (Some(e), Ok(plan)) => {
                    assert!(
                        plan.cost == e,
                        "case {case}: planner cost {} but optimal is {e}\ninitial: {initial}\n{goal}\nactions:\n{}",
                        plan.cost,
                        actions
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                    // The returned plan must actually work.
                    let mut s = initial.clone();
                    for a in &plan.actions {
                        assert!(a.can_execute(&s), "case {case}: plan step not executable");
                        s = a.apply_effect(&s);
                    }
                    assert!(
                        goal.is_satisfied(&s),
                        "case {case}: plan does not reach goal"
                    );
                    solved += 1;
                }
                (e, a) => panic!("case {case}: reference {e:?} vs planner {a:?}"),
            }
        }
        assert!(
            solved > 50,
            "only {solved} solvable cases; generator too strict"
        );
    }

    // Plan 005 (BUG-016, BUG-017, BUG-019, BUG-023, BUG-025)

    /// BUG-019: `plan` borrows the initial state, so one state serves many
    /// calls without cloning.
    #[test]
    fn test_planner_plan_borrows_initial_state() {
        let initial = State::builder().set("x", 0).build();
        let goal = Goal::builder("g").requires("x", 2).build();
        let actions = vec![Action::builder("inc").adds("x", 1).build()];
        let planner = Planner::new();
        let first = planner.plan(&initial, &goal, &actions).unwrap();
        let second = planner.plan(&initial, &goal, &actions).unwrap();
        assert_eq!(first, second);
        assert_eq!(initial.get::<i64>("x"), Some(0));
    }

    /// BUG-023: `Plan` is `Clone + PartialEq`.
    #[test]
    fn test_plan_equality_and_clone() {
        let initial = State::builder().set("x", 0).build();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![Action::builder("inc").adds("x", 1).build()];
        let plan = Planner::new().plan(&initial, &goal, &actions).unwrap();
        let copy = plan.clone();
        assert_eq!(plan, copy);
        assert_eq!(
            plan,
            Plan {
                actions: actions.clone(),
                cost: 1.0
            }
        );
        assert_ne!(
            plan,
            Plan {
                actions: actions.clone(),
                cost: 2.0
            }
        );
    }

    fn priority_problem() -> (State, Vec<Action>) {
        let initial = State::builder().set("x", 0).set("locked", true).build();
        let actions = vec![Action::builder("inc").adds("x", 1).build()];
        (initial, actions)
    }

    /// BUG-016: the highest-priority goal with a plan wins, whatever its
    /// position in the slice.
    #[test]
    fn test_plan_by_priority_picks_highest_priority_achievable_goal() {
        let (initial, actions) = priority_problem();
        let goals = vec![
            Goal::builder("low").priority(1).requires("x", 1).build(),
            Goal::builder("impossible")
                .priority(50)
                .requires("locked", false)
                .build(),
            Goal::builder("high").priority(10).requires("x", 3).build(),
            Goal::builder("mid").priority(5).requires("x", 2).build(),
        ];
        let (index, plan) = Planner::new()
            .plan_by_priority(&initial, &goals, &actions)
            .unwrap();
        assert_eq!(index, 2);
        assert_eq!(goals[index].name, "high");
        assert_eq!(plan.cost, 3.0);
        assert_eq!(plan.actions.len(), 3);
    }

    /// BUG-016: equal priorities keep slice order.
    #[test]
    fn test_plan_by_priority_is_stable_for_equal_priorities() {
        let (initial, actions) = priority_problem();
        let goals = vec![
            Goal::builder("a").priority(3).requires("x", 2).build(),
            Goal::builder("b").priority(3).requires("x", 1).build(),
        ];
        let (index, plan) = Planner::new()
            .plan_by_priority(&initial, &goals, &actions)
            .unwrap();
        assert_eq!(index, 0);
        assert_eq!(plan.cost, 2.0);

        let reversed: Vec<Goal> = goals.into_iter().rev().collect();
        let (index, plan) = Planner::new()
            .plan_by_priority(&initial, &reversed, &actions)
            .unwrap();
        assert_eq!(index, 0);
        assert_eq!(plan.cost, 1.0);
    }

    /// BUG-016: with no achievable goal, a budget failure outranks a proof
    /// of impossibility, because that goal may still be achievable.
    #[test]
    fn test_plan_by_priority_reports_budget_before_impossible() {
        let (initial, actions) = priority_problem();
        let impossible = Goal::builder("impossible")
            .priority(10)
            .requires("locked", false)
            .build();
        // Reachable in the relaxed analysis (x can grow) but not with 3 expansions.
        let too_big = Goal::builder("far")
            .priority(1)
            .requires("x", at_least(100))
            .build();

        let planner = Planner::new().max_expansions(3);
        let err = planner
            .plan_by_priority(&initial, &[impossible.clone(), too_big.clone()], &actions)
            .unwrap_err();
        assert_eq!(err, PlannerError::SearchLimitExceeded { expanded: 3 });

        let err = planner
            .plan_by_priority(&initial, std::slice::from_ref(&impossible), &actions)
            .unwrap_err();
        assert_eq!(err, PlannerError::NoPlanFound);

        // Two budget failures: the higher-priority goal's error is reported.
        let huge = Goal::builder("huge")
            .priority(5)
            .requires("x", at_least(1_000))
            .build();
        let err = planner
            .max_expansions(2)
            .plan_by_priority(&initial, &[too_big, huge], &actions)
            .unwrap_err();
        assert_eq!(err, PlannerError::SearchLimitExceeded { expanded: 2 });
    }

    /// BUG-016: a malformed problem is reported, not skipped.
    #[test]
    fn test_plan_by_priority_aborts_on_validation_error() {
        let (initial, actions) = priority_problem();
        let goals = vec![
            Goal::builder("fine").priority(1).requires("x", 1).build(),
            Goal::builder("mistyped")
                .priority(10)
                .requires("x", at_least(1.0))
                .build(),
        ];
        let err = Planner::new()
            .plan_by_priority(&initial, &goals, &actions)
            .unwrap_err();
        assert!(
            matches!(err, PlannerError::IncompatibleStateTypes { ref variable, .. } if variable == "x"),
            "{err:?}"
        );

        let bad_cost = vec![Action::builder("free").cost(-1.0).build()];
        let err = Planner::new()
            .plan_by_priority(&initial, &goals, &bad_cost)
            .unwrap_err();
        assert_eq!(
            err,
            PlannerError::InvalidActionCost {
                action: "free".to_string(),
                cost: -1.0
            }
        );
    }

    /// BUG-016: no goals, no plan.
    #[test]
    fn test_plan_by_priority_with_no_goals_is_no_plan_found() {
        let (initial, actions) = priority_problem();
        assert_eq!(
            Planner::new().plan_by_priority(&initial, &[], &actions),
            Err(PlannerError::NoPlanFound)
        );
    }

    /// BUG-017: `InvalidActionCost` carries the action and its cost.
    #[test]
    fn test_planner_invalid_cost_reports_action_and_cost() {
        let initial = State::builder().set("x", 0).build();
        let goal = Goal::builder("g").requires("x", 1).build();
        let actions = vec![
            Action::builder("fine").build(),
            Action::builder("broken").cost(f64::INFINITY).build(),
        ];
        let err = Planner::new().plan(&initial, &goal, &actions).unwrap_err();
        assert_eq!(
            err,
            PlannerError::InvalidActionCost {
                action: "broken".to_string(),
                cost: f64::INFINITY,
            }
        );
        let msg = err.to_string();
        assert!(msg.contains("'broken'"), "{msg}");
        assert!(msg.contains("inf"), "{msg}");
    }

    /// BUG-017: a type conflict between two actions reports both origins.
    #[test]
    fn test_planner_type_conflict_reports_variable_types_and_origins() {
        let initial = State::new();
        let goal = Goal::builder("g").requires("done", true).build();
        let actions = vec![
            Action::builder("count")
                .adds("n", 1)
                .sets("done", true)
                .build(),
            Action::builder("measure")
                .requires("n", at_least(0.5))
                .build(),
        ];
        let err = Planner::new().plan(&initial, &goal, &actions).unwrap_err();
        assert_eq!(
            err,
            PlannerError::IncompatibleStateTypes {
                variable: "n".to_string(),
                expected: StateType::I64,
                expected_origin: VariableOrigin::Effects("count".to_string()),
                found: StateType::F64,
                found_origin: VariableOrigin::Preconditions("measure".to_string()),
            }
        );
        assert_eq!(
            err.to_string(),
            "Incompatible state types: variable 'n' is i64 in the effects of action 'count' \
             but f64 in the preconditions of action 'measure'"
        );
    }

    /// BUG-025: an ordered comparison on a non-numeric value is rejected up
    /// front, whether it came through a helper with a `StateVar` or was
    /// built by hand, in a goal or in a precondition.
    #[test]
    fn test_planner_rejects_ordered_condition_on_non_numeric_value() {
        let initial = State::builder().set("flag", false).build();
        let actions = vec![Action::builder("set").sets("flag", true).build()];

        let goal = Goal::builder("g")
            .requires("flag", at_least(StateVar::Bool(true)))
            .build();
        let err = Planner::new().plan(&initial, &goal, &actions).unwrap_err();
        assert_eq!(
            err,
            PlannerError::InvalidCondition {
                variable: "flag".to_string(),
                origin: VariableOrigin::Goal("g".to_string()),
                condition: Condition::AtLeast(StateVar::Bool(true)),
            }
        );
        assert!(err.to_string().contains("flag >= true"), "{err}");

        let mut conditions = Conditions::new();
        conditions.set("flag", Condition::LessThan(StateVar::from("x")));
        let goal = Goal::from_conditions("hand_built", conditions, 1);
        assert!(matches!(
            Planner::new().plan(&initial, &goal, &actions),
            Err(PlannerError::InvalidCondition { ref variable, .. }) if variable == "flag"
        ));

        let goal = Goal::builder("g").requires("flag", true).build();
        let bad_action = Action::builder("odd")
            .requires("flag", at_most(StateVar::Bool(false)))
            .build();
        let err = Planner::new()
            .plan(&initial, &goal, &[bad_action])
            .unwrap_err();
        assert_eq!(
            err,
            PlannerError::InvalidCondition {
                variable: "flag".to_string(),
                origin: VariableOrigin::Preconditions("odd".to_string()),
                condition: Condition::AtMost(StateVar::Bool(false)),
            }
        );
    }

    /// BUG-025: a `Between` with bounds of different types is rejected.
    #[test]
    fn test_planner_rejects_between_with_mixed_types() {
        let initial = State::builder().set("x", 3).build();
        let actions = vec![Action::builder("bump").adds("x", 1).build()];
        let mut conditions = Conditions::new();
        conditions.set(
            "x",
            Condition::Between(StateVar::I64(1), StateVar::F64(5000)),
        );
        let goal = Goal::from_conditions("g", conditions, 1);
        let err = Planner::new().plan(&initial, &goal, &actions).unwrap_err();
        assert_eq!(
            err,
            PlannerError::InvalidCondition {
                variable: "x".to_string(),
                origin: VariableOrigin::Goal("g".to_string()),
                condition: Condition::Between(StateVar::I64(1), StateVar::F64(5000)),
            }
        );
        // Well-typed, the same shape plans normally.
        let goal = Goal::builder("g")
            .requires("x", between(StateVar::I64(4), StateVar::I64(5)))
            .build();
        assert_eq!(
            Planner::new().plan(&initial, &goal, &actions).unwrap().cost,
            1.0
        );
    }

    // Plan 006 (BUG-022): the heuristic prices precondition chains.

    /// A 12-step `Set` chain with 8 free boolean toggles as distractors
    /// (2^8 * 13 reachable states). The effect-only heuristic was a constant
    /// 1 here and needed 1990 expansions; chain-aware guidance walks the 12
    /// states on the solution path.
    #[test]
    fn test_planner_heuristic_sees_through_precondition_chains() {
        const CHAIN: usize = 12;
        const TOGGLES: usize = 8;
        let mut builder = State::builder().set("s0", true);
        for i in 1..=CHAIN {
            builder = builder.set(&format!("s{i}"), false);
        }
        for t in 0..TOGGLES {
            builder = builder.set(&format!("d{t}"), false);
        }
        let initial = builder.build();
        let mut actions = Vec::new();
        for i in 1..=CHAIN {
            actions.push(
                Action::builder(&format!("step{i}"))
                    .requires(&format!("s{}", i - 1), true)
                    .sets(&format!("s{i}"), true)
                    .build(),
            );
        }
        for t in 0..TOGGLES {
            actions.push(
                Action::builder(&format!("on{t}"))
                    .sets(&format!("d{t}"), true)
                    .build(),
            );
            actions.push(
                Action::builder(&format!("off{t}"))
                    .sets(&format!("d{t}"), false)
                    .build(),
            );
        }
        let goal = Goal::builder("end")
            .requires(&format!("s{CHAIN}"), true)
            .build();
        let plan = Planner::new()
            .max_expansions(40)
            .plan(&initial, &goal, &actions)
            .expect("the chain-aware heuristic needs far fewer than 40 expansions");
        assert_eq!(plan.cost, CHAIN as f64);
        let names: Vec<&str> = plan.actions.iter().map(|a| a.name.as_str()).collect();
        let expected: Vec<String> = (1..=CHAIN).map(|i| format!("step{i}")).collect();
        assert_eq!(names, expected);
    }

    /// A numeric production chain (mine -> smelt -> forge) with 6 toggles.
    /// Needed 389 expansions with the effect-only heuristic.
    #[test]
    fn test_planner_heuristic_prices_resource_chains() {
        const TOGGLES: usize = 6;
        let mut builder = State::builder()
            .set("ore", 0)
            .set("ingots", 0)
            .set("has_sword", false);
        for t in 0..TOGGLES {
            builder = builder.set(&format!("d{t}"), false);
        }
        let initial = builder.build();
        let mut actions = vec![
            Action::builder("mine").cost(1.0).adds("ore", 5).build(),
            Action::builder("smelt")
                .cost(1.0)
                .requires("ore", at_least(5))
                .subtracts("ore", 5)
                .adds("ingots", 1)
                .build(),
            Action::builder("forge")
                .cost(2.0)
                .requires("ingots", at_least(3))
                .subtracts("ingots", 3)
                .sets("has_sword", true)
                .build(),
        ];
        for t in 0..TOGGLES {
            actions.push(
                Action::builder(&format!("on{t}"))
                    .sets(&format!("d{t}"), true)
                    .build(),
            );
            actions.push(
                Action::builder(&format!("off{t}"))
                    .sets(&format!("d{t}"), false)
                    .build(),
            );
        }
        let goal = Goal::builder("armed").requires("has_sword", true).build();
        let plan = Planner::new()
            .max_expansions(60)
            .plan(&initial, &goal, &actions)
            .expect("the chain-aware heuristic needs far fewer than 60 expansions");
        assert_eq!(plan.cost, 8.0);
        assert_eq!(plan.actions.len(), 7);
    }

    /// The chain bound must not double count an enabler shared by two goal
    /// conditions: the plan is still optimal and found.
    #[test]
    fn test_planner_shared_enabler_stays_optimal() {
        let initial = State::builder()
            .set("tool", false)
            .set("a", false)
            .set("b", false)
            .build();
        let goal = Goal::builder("both")
            .requires("a", true)
            .requires("b", true)
            .build();
        let actions = vec![
            Action::builder("get_tool")
                .cost(10.0)
                .sets("tool", true)
                .build(),
            Action::builder("set_a")
                .requires("tool", true)
                .sets("a", true)
                .build(),
            Action::builder("set_b")
                .requires("tool", true)
                .sets("b", true)
                .build(),
            // A tempting detour that would look cheaper to an overestimating bound.
            Action::builder("set_a_slowly")
                .cost(11.5)
                .sets("a", true)
                .build(),
            Action::builder("set_b_slowly")
                .cost(11.5)
                .sets("b", true)
                .build(),
        ];
        let plan = Planner::new().plan(&initial, &goal, &actions).unwrap();
        assert_eq!(plan.cost, 12.0);
    }
}
