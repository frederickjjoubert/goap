use goap::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut initial_state = WorldState::new();
    initial_state.set("gold", StateVar::I64(500));
    initial_state.set("total_profit", StateVar::I64(0));
    initial_state.set("merchant_count", StateVar::I64(1));
    initial_state.set("storage_capacity", StateVar::I64(50));
    initial_state.set("current_storage", StateVar::I64(0));
    initial_state.set("grain_stock", StateVar::I64(0));
    initial_state.set("ore_stock", StateVar::I64(0));
    initial_state.set("textile_stock", StateVar::I64(0));
    initial_state.set("grain_price", StateVar::I64(50));
    initial_state.set("ore_price", StateVar::I64(80));
    initial_state.set("textile_price", StateVar::I64(120));
    initial_state.set("trade_reputation", StateVar::I64(0));
    initial_state.set("has_market_advisor", StateVar::Bool(false));
    initial_state.set("market_info_level", StateVar::I64(1));

    let mut goal = Goal::new("economic_trading", WorldState::new());
    goal.desired_state.set("total_profit", StateVar::I64(2000));
    goal.desired_state
        .set("market_info_level", StateVar::I64(3));
    goal.desired_state
        .set("storage_capacity", StateVar::I64(100));
    goal.desired_state
        .set("trade_reputation", StateVar::I64(50));

    let mut planner = Planner::new();

    // Action: Hire market advisor
    let mut conditions = WorldState::new();
    conditions.set("gold", StateVar::I64(200));
    conditions.set("has_market_advisor", StateVar::Bool(false));
    let mut effects = HashMap::new();
    effects.insert("gold".to_string(), StateOperation::Subtract(200));
    effects.insert(
        "has_market_advisor".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    effects.insert("market_info_level".to_string(), StateOperation::Add(1));
    planner.add_action(Action::new("hire_advisor", 2.0, conditions, effects));

    // Action: Research prices
    let mut conditions = WorldState::new();
    conditions.set("gold", StateVar::I64(100));
    conditions.set("has_market_advisor", StateVar::Bool(true));
    let mut effects = HashMap::new();
    effects.insert("gold".to_string(), StateOperation::Subtract(100));
    effects.insert("market_info_level".to_string(), StateOperation::Add(1));
    planner.add_action(Action::new("research_prices", 1.0, conditions, effects));

    // Action: Upgrade storage
    let mut conditions = WorldState::new();
    conditions.set("gold", StateVar::I64(200));
    let mut effects = HashMap::new();
    effects.insert("gold".to_string(), StateOperation::Subtract(200));
    effects.insert("storage_capacity".to_string(), StateOperation::Add(50));
    planner.add_action(Action::new("upgrade_storage", 3.0, conditions, effects));

    // Action: Buy grain
    let mut conditions = WorldState::new();
    conditions.set("gold", StateVar::I64(250)); // Need enough gold for 5 units at 50 each
    conditions.set("storage_capacity", StateVar::I64(5));
    conditions.set("current_storage", StateVar::I64(0));
    let mut effects = HashMap::new();
    effects.insert("gold".to_string(), StateOperation::Subtract(250));
    effects.insert("grain_stock".to_string(), StateOperation::Add(5));
    effects.insert("current_storage".to_string(), StateOperation::Add(5));
    planner.add_action(Action::new("buy_grain", 1.0, conditions, effects));

    // Action: Sell grain
    let mut conditions = WorldState::new();
    conditions.set("grain_stock", StateVar::I64(5));
    let mut effects = HashMap::new();
    effects.insert("gold".to_string(), StateOperation::Add(300)); // Sell at 60 each (20% markup)
    effects.insert("grain_stock".to_string(), StateOperation::Subtract(5));
    effects.insert("current_storage".to_string(), StateOperation::Subtract(5));
    effects.insert("total_profit".to_string(), StateOperation::Add(50));
    effects.insert("trade_reputation".to_string(), StateOperation::Add(5));
    planner.add_action(Action::new("sell_grain", 1.0, conditions, effects));

    let plan_result = planner.plan(initial_state.clone(), &goal);
    assert!(
        plan_result.is_some(),
        "Expected to find a valid plan for economic trading"
    );

    let (actions, cost) = plan_result.unwrap();
    println!("Found economic trading plan with cost {}", cost);
    for action in actions {
        println!("- {}", action.name);
    }
}
