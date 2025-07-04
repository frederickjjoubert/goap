use goap::prelude::*;

fn main() {
    // Initial state - starting resources and market conditions
    let initial_state = State::new()
        .set("gold", 1000)
        .set("total_profit", 0)
        .set("reputation", 0)
        .set("storage_space", 100)
        .set("used_storage", 0)
        .set("grain", 0)
        .set("ore", 0)
        .set("textiles", 0)
        .set("market_knowledge", 0)
        .set("trade_routes", 1)
        .set("has_caravan", false)
        .set("has_warehouse", false)
        .set("has_guild_membership", false)
        .build();

    // Goal state - become a successful merchant
    let goal = Goal::new("master_merchant")
        .requires("total_profit", 2000)
        .requires("reputation", 50)
        .requires("trade_routes", 2)
        .requires("has_guild_membership", true)
        .build();

    // Action: Join Merchant's Guild
    let join_guild = Action::new("join_guild")
        .cost(5.0)
        .requires("gold", 1500)
        .requires("reputation", 30)
        .requires("has_guild_membership", false)
        .subtracts("gold", 1500)
        .sets("has_guild_membership", true)
        .adds("market_knowledge", 20)
        .adds("reputation", 20)
        .build();

    // Action: Buy Warehouse
    let buy_warehouse = Action::new("buy_warehouse")
        .cost(10.0)
        .requires("gold", 2000)
        .requires("has_warehouse", false)
        .subtracts("gold", 2000)
        .sets("has_warehouse", true)
        .adds("storage_space", 200)
        .adds("reputation", 10)
        .build();

    // Action: Purchase Caravan (reduced cost)
    let buy_caravan = Action::new("buy_caravan")
        .cost(8.0)
        .requires("gold", 1500)
        .requires("has_caravan", false)
        .subtracts("gold", 1500)
        .sets("has_caravan", true)
        .adds("trade_routes", 1)
        .build();

    // Action: Buy Grain (Bulk)
    let buy_grain = Action::new("buy_grain_bulk")
        .cost(2.0)
        .requires("gold", 500)
        .requires("storage_space", 50)
        .requires("used_storage", 0)
        .subtracts("gold", 500)
        .adds("grain", 50)
        .adds("used_storage", 50)
        .build();

    // Action: Buy Ore (Bulk)
    let buy_ore = Action::new("buy_ore_bulk")
        .cost(3.0)
        .requires("gold", 800)
        .requires("storage_space", 40)
        .requires("used_storage", 0)
        .subtracts("gold", 800)
        .adds("ore", 40)
        .adds("used_storage", 40)
        .build();

    // Action: Buy Textiles (Bulk)
    let buy_textiles = Action::new("buy_textiles_bulk")
        .cost(4.0)
        .requires("gold", 1000)
        .requires("storage_space", 30)
        .requires("used_storage", 0)
        .subtracts("gold", 1000)
        .adds("textiles", 30)
        .adds("used_storage", 30)
        .build();

    // Action: Sell Grain (Local Market)
    let sell_grain_local = Action::new("sell_grain_local")
        .cost(1.0)
        .requires("grain", 50)
        .adds("gold", 600)
        .subtracts("grain", 50)
        .subtracts("used_storage", 50)
        .adds("total_profit", 100)
        .adds("reputation", 5)
        .build();

    // Action: Sell Grain (Distant Market - requires caravan)
    let sell_grain_distant = Action::new("sell_grain_distant")
        .cost(3.0)
        .requires("grain", 50)
        .requires("has_caravan", true)
        .adds("gold", 800)
        .subtracts("grain", 50)
        .subtracts("used_storage", 50)
        .adds("total_profit", 300)
        .adds("reputation", 10)
        .build();

    // Action: Sell Ore (Local Market)
    let sell_ore_local = Action::new("sell_ore_local")
        .cost(1.0)
        .requires("ore", 40)
        .adds("gold", 1000)
        .subtracts("ore", 40)
        .subtracts("used_storage", 40)
        .adds("total_profit", 200)
        .adds("reputation", 8)
        .build();

    // Action: Sell Ore (Distant Market - requires caravan)
    let sell_ore_distant = Action::new("sell_ore_distant")
        .cost(3.0)
        .requires("ore", 40)
        .requires("has_caravan", true)
        .adds("gold", 1400)
        .subtracts("ore", 40)
        .subtracts("used_storage", 40)
        .adds("total_profit", 600)
        .adds("reputation", 15)
        .build();

    // Action: Sell Textiles (Local Market)
    let sell_textiles_local = Action::new("sell_textiles_local")
        .cost(1.0)
        .requires("textiles", 30)
        .adds("gold", 1300)
        .subtracts("textiles", 30)
        .subtracts("used_storage", 30)
        .adds("total_profit", 300)
        .adds("reputation", 10)
        .build();

    // Action: Sell Textiles (Distant Market - requires caravan)
    let sell_textiles_distant = Action::new("sell_textiles_distant")
        .cost(3.0)
        .requires("textiles", 30)
        .requires("has_caravan", true)
        .adds("gold", 1800)
        .subtracts("textiles", 30)
        .subtracts("used_storage", 30)
        .adds("total_profit", 800)
        .adds("reputation", 20)
        .build();

    // Action: Establish New Trade Route (requires guild membership)
    let establish_route = Action::new("establish_trade_route")
        .cost(15.0)
        .requires("gold", 3000)
        .requires("has_guild_membership", true)
        .requires("reputation", 40)
        .subtracts("gold", 3000)
        .adds("trade_routes", 1)
        .adds("reputation", 30)
        .build();

    // Collect all actions
    let actions = vec![
        join_guild,
        buy_warehouse,
        buy_caravan,
        buy_grain,
        buy_ore,
        buy_textiles,
        sell_grain_local,
        sell_grain_distant,
        sell_ore_local,
        sell_ore_distant,
        sell_textiles_local,
        sell_textiles_distant,
        establish_route,
    ];

    // Create planner
    let planner = Planner::new();

    match planner.plan(initial_state.clone(), &goal, &actions) {
        Ok(plan) => {
            let action_count = plan.actions.len();
            let cost = plan.cost;
            println!("\nFound trading plan with {action_count} actions and cost {cost:.2}:");
            let mut current_state = initial_state;

            for (i, action) in plan.actions.iter().enumerate() {
                let step = i + 1;
                let name = &action.name;
                println!("\nStep {step}: {name}");
                current_state = action.apply_effect(&current_state);

                if let Some(StateVar::I64(gold)) = current_state.get("gold") {
                    println!("Gold: {gold}");
                }
                if let Some(StateVar::I64(profit)) = current_state.get("total_profit") {
                    println!("Profit: {profit}");
                }
                if let Some(StateVar::I64(reputation)) = current_state.get("reputation") {
                    println!("Reputation: {reputation}");
                }
                if let Some(StateVar::I64(routes)) = current_state.get("trade_routes") {
                    println!("Trade Routes: {routes}");
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
        Err(e) => println!("No plan found! {e}"),
    }
}
