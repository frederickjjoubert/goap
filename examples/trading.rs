use goap::prelude::*;

fn main() {
    // Initial state - starting resources and market conditions
    let initial_state = WorldState::builder()
        .int("gold", 1000)
        .int("total_profit", 0)
        .int("reputation", 0)
        .int("storage_space", 100)
        .int("used_storage", 0)
        .int("grain", 0)
        .int("ore", 0)
        .int("textiles", 0)
        .int("market_knowledge", 0)
        .int("trade_routes", 1)
        .bool("has_caravan", false)
        .bool("has_warehouse", false)
        .bool("has_guild_membership", false)
        .build();

    // Goal state - become a successful merchant
    let goal = Goal::builder("master_merchant")
        .require_int("total_profit", 2000)
        .require_int("reputation", 50)
        .require_int("trade_routes", 2)
        .require_bool("has_guild_membership", true)
        .build();

    let mut planner = Planner::new();

    // Action: Join Merchant's Guild
    let join_guild = Action::builder("join_guild")
        .cost(5.0)
        .precondition("gold", 1500)
        .precondition("reputation", 30)
        .precondition("has_guild_membership", false)
        .effect_subtract_int("gold", 1500)
        .effect_set_to("has_guild_membership", true)
        .effect_add_int("market_knowledge", 20)
        .effect_add_int("reputation", 20)
        .build();

    // Action: Buy Warehouse
    let buy_warehouse = Action::builder("buy_warehouse")
        .cost(10.0)
        .precondition("gold", 2000)
        .precondition("has_warehouse", false)
        .effect_subtract_int("gold", 2000)
        .effect_set_to("has_warehouse", true)
        .effect_add_int("storage_space", 200)
        .effect_add_int("reputation", 10)
        .build();

    // Action: Purchase Caravan (reduced cost)
    let buy_caravan = Action::builder("buy_caravan")
        .cost(8.0)
        .precondition("gold", 1500)
        .precondition("has_caravan", false)
        .effect_subtract_int("gold", 1500)
        .effect_set_to("has_caravan", true)
        .effect_add_int("trade_routes", 1)
        .build();

    // Action: Buy Grain (Bulk)
    let buy_grain = Action::builder("buy_grain_bulk")
        .cost(2.0)
        .precondition("gold", 500)
        .precondition("storage_space", 50)
        .precondition("used_storage", 0)
        .effect_subtract_int("gold", 500)
        .effect_add_int("grain", 50)
        .effect_add_int("used_storage", 50)
        .build();

    // Action: Buy Ore (Bulk)
    let buy_ore = Action::builder("buy_ore_bulk")
        .cost(3.0)
        .precondition("gold", 800)
        .precondition("storage_space", 40)
        .precondition("used_storage", 0)
        .effect_subtract_int("gold", 800)
        .effect_add_int("ore", 40)
        .effect_add_int("used_storage", 40)
        .build();

    // Action: Buy Textiles (Bulk)
    let buy_textiles = Action::builder("buy_textiles_bulk")
        .cost(4.0)
        .precondition("gold", 1000)
        .precondition("storage_space", 30)
        .precondition("used_storage", 0)
        .effect_subtract_int("gold", 1000)
        .effect_add_int("textiles", 30)
        .effect_add_int("used_storage", 30)
        .build();

    // Action: Sell Grain (Local Market)
    let sell_grain_local = Action::builder("sell_grain_local")
        .cost(1.0)
        .precondition("grain", 50)
        .effect_add_int("gold", 600)
        .effect_subtract_int("grain", 50)
        .effect_subtract_int("used_storage", 50)
        .effect_add_int("total_profit", 100)
        .effect_add_int("reputation", 5)
        .build();

    // Action: Sell Grain (Distant Market - requires caravan)
    let sell_grain_distant = Action::builder("sell_grain_distant")
        .cost(3.0)
        .precondition("grain", 50)
        .precondition("has_caravan", true)
        .effect_add_int("gold", 800)
        .effect_subtract_int("grain", 50)
        .effect_subtract_int("used_storage", 50)
        .effect_add_int("total_profit", 300)
        .effect_add_int("reputation", 10)
        .build();

    // Action: Sell Ore (Local Market)
    let sell_ore_local = Action::builder("sell_ore_local")
        .cost(1.0)
        .precondition("ore", 40)
        .effect_add_int("gold", 1000)
        .effect_subtract_int("ore", 40)
        .effect_subtract_int("used_storage", 40)
        .effect_add_int("total_profit", 200)
        .effect_add_int("reputation", 8)
        .build();

    // Action: Sell Ore (Distant Market - requires caravan)
    let sell_ore_distant = Action::builder("sell_ore_distant")
        .cost(3.0)
        .precondition("ore", 40)
        .precondition("has_caravan", true)
        .effect_add_int("gold", 1400)
        .effect_subtract_int("ore", 40)
        .effect_subtract_int("used_storage", 40)
        .effect_add_int("total_profit", 600)
        .effect_add_int("reputation", 15)
        .build();

    // Action: Sell Textiles (Local Market)
    let sell_textiles_local = Action::builder("sell_textiles_local")
        .cost(1.0)
        .precondition("textiles", 30)
        .effect_add_int("gold", 1300)
        .effect_subtract_int("textiles", 30)
        .effect_subtract_int("used_storage", 30)
        .effect_add_int("total_profit", 300)
        .effect_add_int("reputation", 10)
        .build();

    // Action: Sell Textiles (Distant Market - requires caravan)
    let sell_textiles_distant = Action::builder("sell_textiles_distant")
        .cost(3.0)
        .precondition("textiles", 30)
        .precondition("has_caravan", true)
        .effect_add_int("gold", 1800)
        .effect_subtract_int("textiles", 30)
        .effect_subtract_int("used_storage", 30)
        .effect_add_int("total_profit", 800)
        .effect_add_int("reputation", 20)
        .build();

    // Action: Establish New Trade Route (requires guild membership)
    let establish_route = Action::builder("establish_trade_route")
        .cost(15.0)
        .precondition("gold", 3000)
        .precondition("has_guild_membership", true)
        .precondition("reputation", 40)
        .effect_subtract_int("gold", 3000)
        .effect_add_int("trade_routes", 1)
        .effect_add_int("reputation", 30)
        .build();

    // Add all actions to planner
    planner.add_action(join_guild);
    planner.add_action(buy_warehouse);
    planner.add_action(buy_caravan);
    planner.add_action(buy_grain);
    planner.add_action(buy_ore);
    planner.add_action(buy_textiles);
    planner.add_action(sell_grain_local);
    planner.add_action(sell_grain_distant);
    planner.add_action(sell_ore_local);
    planner.add_action(sell_ore_distant);
    planner.add_action(sell_textiles_local);
    planner.add_action(sell_textiles_distant);
    planner.add_action(establish_route);

    match planner.plan(initial_state.clone(), &goal) {
        Some((actions, cost)) => {
            println!(
                "\nFound trading plan with {} actions and cost {:.2}:",
                actions.len(),
                cost
            );
            let mut current_state = initial_state;

            for (i, action) in actions.iter().enumerate() {
                println!("\nStep {}: {}", i + 1, action.name);
                current_state = action.apply_effect(&current_state);

                if let Some(StateVar::I64(gold)) = current_state.get("gold") {
                    println!("Gold: {}", gold);
                }
                if let Some(StateVar::I64(profit)) = current_state.get("total_profit") {
                    println!("Profit: {}", profit);
                }
                if let Some(StateVar::I64(reputation)) = current_state.get("reputation") {
                    println!("Reputation: {}", reputation);
                }
                if let Some(StateVar::I64(routes)) = current_state.get("trade_routes") {
                    println!("Trade Routes: {}", routes);
                }

                if let Some(StateVar::Bool(guild)) = current_state.get("has_guild_membership") {
                    if *guild {
                        println!("Guild Member: Yes");
                    }
                }
                if let Some(StateVar::Bool(caravan)) = current_state.get("has_caravan") {
                    if *caravan {
                        println!("Has Caravan: Yes");
                    }
                }
                if let Some(StateVar::Bool(warehouse)) = current_state.get("has_warehouse") {
                    if *warehouse {
                        println!("Has Warehouse: Yes");
                    }
                }
            }
        }
        None => println!("No plan found!"),
    }
}
