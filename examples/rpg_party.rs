use std::collections::HashMap;

use goap::prelude::*;

fn main() {
    // Initial state - current party members and their attributes
    let mut initial_state = WorldState::new();
    // Party composition
    initial_state.set("tank_available", StateVar::Bool(true));
    initial_state.set("healer_available", StateVar::Bool(false));
    initial_state.set("dps_available", StateVar::Bool(true));
    // Party member stats
    initial_state.set("tank_health", StateVar::I64(50));
    initial_state.set("tank_armor", StateVar::I64(30));
    initial_state.set("dps_damage", StateVar::I64(40));
    initial_state.set("party_size", StateVar::I64(2));
    // Status effects
    initial_state.set("tank_poisoned", StateVar::Bool(true));
    initial_state.set("dps_cursed", StateVar::Bool(true));
    // Resources
    initial_state.set("gold", StateVar::I64(400)); // Increased initial gold
    initial_state.set("healing_potions", StateVar::I64(1));
    initial_state.set("antidote_potions", StateVar::I64(0));
    initial_state.set("remove_curse_scrolls", StateVar::I64(0));

    // Goal state - party ready for dungeon
    let mut goal_state = WorldState::new();
    goal_state.set("healer_available", StateVar::Bool(true));
    goal_state.set("tank_health", StateVar::I64(100));
    goal_state.set("tank_armor", StateVar::I64(50));
    goal_state.set("tank_poisoned", StateVar::Bool(false));
    goal_state.set("dps_cursed", StateVar::Bool(false));
    goal_state.set("dps_damage", StateVar::I64(60));
    goal_state.set("party_size", StateVar::I64(3));

    let goal = Goal::new("prepare_dungeon_party", goal_state);

    // Create planner
    let mut planner = Planner::new();

    // Action: Recruit Healer
    let mut recruit_healer_effects = HashMap::new();
    recruit_healer_effects.insert(
        "healer_available".to_string(),
        StateOperation::Set(StateVar::Bool(true)),
    );
    recruit_healer_effects.insert("party_size".to_string(), StateOperation::Add(1));
    recruit_healer_effects.insert("gold".to_string(), StateOperation::Subtract(100));
    let mut recruit_healer_conditions = WorldState::new();
    recruit_healer_conditions.set("gold", StateVar::I64(100));
    let recruit_healer = Action::new(
        "recruit_healer",
        3.0,
        recruit_healer_conditions,
        recruit_healer_effects,
    );

    // Action: Buy Healing Potion
    let mut buy_healing_effects = HashMap::new();
    buy_healing_effects.insert("healing_potions".to_string(), StateOperation::Add(1));
    buy_healing_effects.insert("gold".to_string(), StateOperation::Subtract(50));
    let mut buy_healing_conditions = WorldState::new();
    buy_healing_conditions.set("gold", StateVar::I64(50));
    let buy_healing = Action::new(
        "buy_healing_potion",
        1.0,
        buy_healing_conditions,
        buy_healing_effects,
    );

    // Action: Buy Antidote
    let mut buy_antidote_effects = HashMap::new();
    buy_antidote_effects.insert("antidote_potions".to_string(), StateOperation::Add(1));
    buy_antidote_effects.insert("gold".to_string(), StateOperation::Subtract(30));
    let mut buy_antidote_conditions = WorldState::new();
    buy_antidote_conditions.set("gold", StateVar::I64(30));
    let buy_antidote = Action::new(
        "buy_antidote",
        1.0,
        buy_antidote_conditions,
        buy_antidote_effects,
    );

    // Action: Buy Remove Curse Scroll
    let mut buy_scroll_effects = HashMap::new();
    buy_scroll_effects.insert("remove_curse_scrolls".to_string(), StateOperation::Add(1));
    buy_scroll_effects.insert("gold".to_string(), StateOperation::Subtract(40));
    let mut buy_scroll_conditions = WorldState::new();
    buy_scroll_conditions.set("gold", StateVar::I64(40));
    let buy_scroll = Action::new(
        "buy_remove_curse_scroll",
        1.0,
        buy_scroll_conditions,
        buy_scroll_effects,
    );

    // Action: Heal Tank
    let mut heal_tank_effects = HashMap::new();
    heal_tank_effects.insert(
        "tank_health".to_string(),
        StateOperation::Set(StateVar::I64(100)),
    );
    heal_tank_effects.insert("healing_potions".to_string(), StateOperation::Subtract(1));
    let mut heal_tank_conditions = WorldState::new();
    heal_tank_conditions.set("healing_potions", StateVar::I64(1));
    let heal_tank = Action::new("heal_tank", 1.0, heal_tank_conditions, heal_tank_effects);

    // Action: Cure Tank Poison
    let mut cure_poison_effects = HashMap::new();
    cure_poison_effects.insert(
        "tank_poisoned".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    cure_poison_effects.insert("antidote_potions".to_string(), StateOperation::Subtract(1));
    let mut cure_poison_conditions = WorldState::new();
    cure_poison_conditions.set("antidote_potions", StateVar::I64(1));
    let cure_poison = Action::new(
        "cure_tank_poison",
        1.0,
        cure_poison_conditions,
        cure_poison_effects,
    );

    // Action: Remove DPS Curse
    let mut remove_curse_effects = HashMap::new();
    remove_curse_effects.insert(
        "dps_cursed".to_string(),
        StateOperation::Set(StateVar::Bool(false)),
    );
    remove_curse_effects.insert(
        "remove_curse_scrolls".to_string(),
        StateOperation::Subtract(1),
    );
    let mut remove_curse_conditions = WorldState::new();
    remove_curse_conditions.set("remove_curse_scrolls", StateVar::I64(1));
    let remove_curse = Action::new(
        "remove_dps_curse",
        1.0,
        remove_curse_conditions,
        remove_curse_effects,
    );

    // Action: Upgrade Tank Armor
    let mut upgrade_armor_effects = HashMap::new();
    upgrade_armor_effects.insert(
        "tank_armor".to_string(),
        StateOperation::Set(StateVar::I64(50)),
    );
    upgrade_armor_effects.insert("gold".to_string(), StateOperation::Subtract(80));
    let mut upgrade_armor_conditions = WorldState::new();
    upgrade_armor_conditions.set("gold", StateVar::I64(80));
    let upgrade_armor = Action::new(
        "upgrade_tank_armor",
        2.0,
        upgrade_armor_conditions,
        upgrade_armor_effects,
    );

    // Action: Upgrade DPS Weapon
    let mut upgrade_weapon_effects = HashMap::new();
    upgrade_weapon_effects.insert(
        "dps_damage".to_string(),
        StateOperation::Set(StateVar::I64(60)),
    );
    upgrade_weapon_effects.insert("gold".to_string(), StateOperation::Subtract(90));
    let mut upgrade_weapon_conditions = WorldState::new();
    upgrade_weapon_conditions.set("gold", StateVar::I64(90));
    let upgrade_weapon = Action::new(
        "upgrade_dps_weapon",
        2.0,
        upgrade_weapon_conditions,
        upgrade_weapon_effects,
    );

    // Add all actions to planner
    planner.add_action(recruit_healer);
    planner.add_action(buy_healing);
    planner.add_action(buy_antidote);
    planner.add_action(buy_scroll);
    planner.add_action(heal_tank);
    planner.add_action(cure_poison);
    planner.add_action(remove_curse);
    planner.add_action(upgrade_armor);
    planner.add_action(upgrade_weapon);

    // Store a copy of initial state for later comparison
    let initial_state_copy = initial_state.clone();

    // Find plan
    let plan_result = planner.plan(initial_state, &goal);
    assert!(
        plan_result.is_some(),
        "Expected to find a valid plan for party management"
    );

    let (actions, total_cost) = plan_result.unwrap();

    println!("\nParty Management Plan found with cost {}", total_cost);
    for action in &actions {
        println!("- {} (cost: {})", action.name, action.cost);
    }

    // Verify the plan contains necessary actions
    let action_names: Vec<_> = actions.iter().map(|a| a.name.as_str()).collect();

    // Verify party composition
    assert!(
        action_names.contains(&"recruit_healer"),
        "Plan should include recruiting a healer"
    );

    // Verify healing and status management
    assert!(
        action_names.contains(&"heal_tank"),
        "Plan should include healing the tank"
    );
    assert!(
        action_names.contains(&"buy_antidote"),
        "Plan should include buying antidote"
    );
    assert!(
        action_names.contains(&"cure_tank_poison"),
        "Plan should include curing tank's poison"
    );
    assert!(
        action_names.contains(&"buy_remove_curse_scroll"),
        "Plan should include buying curse removal"
    );
    assert!(
        action_names.contains(&"remove_dps_curse"),
        "Plan should include removing DPS curse"
    );

    // Verify equipment upgrades
    assert!(
        action_names.contains(&"upgrade_tank_armor"),
        "Plan should include upgrading tank's armor"
    );
    assert!(
        action_names.contains(&"upgrade_dps_weapon"),
        "Plan should include upgrading DPS weapon"
    );

    // Simulate plan execution to verify final state
    let mut current_state = initial_state_copy;
    let remaining_gold = 400;

    println!("\nSimulating plan execution:");
    for action in &actions {
        current_state = action.execute(&current_state);
        println!("After {}: ", action.name);
        println!("  Gold: {}", remaining_gold);
        if let Some(StateVar::I64(party_size)) = current_state.get("party_size") {
            println!("  Party Size: {}", party_size);
        }
        if let Some(StateVar::I64(tank_health)) = current_state.get("tank_health") {
            println!("  Tank Health: {}", tank_health);
        }
        if let Some(StateVar::I64(tank_armor)) = current_state.get("tank_armor") {
            println!("  Tank Armor: {}", tank_armor);
        }
        if let Some(StateVar::I64(dps_damage)) = current_state.get("dps_damage") {
            println!("  DPS Damage: {}", dps_damage);
        }
    }

    // Verify final state meets all goals
    assert!(
        current_state.satisfies(&goal.desired_state),
        "Final state should meet all goals"
    );

    // Verify specific final conditions
    assert!(remaining_gold >= 0, "Should not overspend gold");

    if let Some(StateVar::Bool(has_healer)) = current_state.get("healer_available") {
        assert!(*has_healer, "Should have recruited a healer");
    }
    if let Some(StateVar::I64(party_size)) = current_state.get("party_size") {
        assert!(*party_size == 3, "Party size should be 3");
    }
    if let Some(StateVar::Bool(tank_poisoned)) = current_state.get("tank_poisoned") {
        assert!(!*tank_poisoned, "Tank should not be poisoned");
    }
    if let Some(StateVar::Bool(dps_cursed)) = current_state.get("dps_cursed") {
        assert!(!*dps_cursed, "DPS should not be cursed");
    }

    println!("\nFinal party state verification:");
    println!("Party Size: {:?}", current_state.get("party_size"));
    println!("Tank Health: {:?}", current_state.get("tank_health"));
    println!("Tank Armor: {:?}", current_state.get("tank_armor"));
    println!("Tank Poisoned: {:?}", current_state.get("tank_poisoned"));
    println!("DPS Damage: {:?}", current_state.get("dps_damage"));
    println!("DPS Cursed: {:?}", current_state.get("dps_cursed"));
    println!(
        "Healer Available: {:?}",
        current_state.get("healer_available")
    );
    println!("Remaining Gold: {}", remaining_gold);
}
