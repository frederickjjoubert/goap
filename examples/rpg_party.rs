use goap::prelude::*;

fn main() {
    // Initial state - current party members and their attributes
    let initial_state = WorldState::builder()
        // Party composition
        .bool("tank_available", true)
        .bool("healer_available", false)
        .bool("dps_available", true)
        // Party member stats
        .int("tank_health", 50)
        .int("tank_armor", 30)
        .int("dps_damage", 40)
        .int("party_size", 2)
        // Status effects
        .bool("tank_poisoned", true)
        .bool("dps_cursed", true)
        // Resources
        .int("gold", 400)
        .int("healing_potions", 1)
        .int("antidote_potions", 0)
        .int("remove_curse_scrolls", 0)
        .build();

    // Goal state - party ready for dungeon
    let goal = Goal::builder("prepare_dungeon_party")
        .require_bool("healer_available", true)
        .require_int("tank_health", 100)
        .require_int("tank_armor", 50)
        .require_bool("tank_poisoned", false)
        .require_bool("dps_cursed", false)
        .require_int("dps_damage", 60)
        .require_int("party_size", 3)
        .build();

    // Action: Recruit Healer
    let recruit_healer = Action::builder("recruit_healer")
        .cost(3.0)
        .precondition("gold", 100)
        .effect_set_to("healer_available", true)
        .effect_add_int("party_size", 1)
        .effect_subtract_int("gold", 100)
        .build();

    // Action: Buy Healing Potion
    let buy_healing = Action::builder("buy_healing_potion")
        .cost(1.0)
        .precondition("gold", 50)
        .effect_add_int("healing_potions", 1)
        .effect_subtract_int("gold", 50)
        .build();

    // Action: Buy Antidote
    let buy_antidote = Action::builder("buy_antidote")
        .cost(1.0)
        .precondition("gold", 30)
        .effect_add_int("antidote_potions", 1)
        .effect_subtract_int("gold", 30)
        .build();

    // Action: Buy Remove Curse Scroll
    let buy_scroll = Action::builder("buy_remove_curse_scroll")
        .cost(1.0)
        .precondition("gold", 40)
        .effect_add_int("remove_curse_scrolls", 1)
        .effect_subtract_int("gold", 40)
        .build();

    // Action: Heal Tank
    let heal_tank = Action::builder("heal_tank")
        .cost(1.0)
        .precondition("healing_potions", 1)
        .effect_set_to("tank_health", 100)
        .effect_subtract_int("healing_potions", 1)
        .build();

    // Action: Cure Tank Poison
    let cure_poison = Action::builder("cure_tank_poison")
        .cost(1.0)
        .precondition("antidote_potions", 1)
        .effect_set_to("tank_poisoned", false)
        .effect_subtract_int("antidote_potions", 1)
        .build();

    // Action: Remove DPS Curse
    let remove_curse = Action::builder("remove_dps_curse")
        .cost(1.0)
        .precondition("remove_curse_scrolls", 1)
        .effect_set_to("dps_cursed", false)
        .effect_subtract_int("remove_curse_scrolls", 1)
        .build();

    // Action: Upgrade Tank Armor
    let upgrade_armor = Action::builder("upgrade_tank_armor")
        .cost(2.0)
        .precondition("gold", 80)
        .effect_set_to("tank_armor", 50)
        .effect_subtract_int("gold", 80)
        .build();

    // Action: Upgrade DPS Weapon
    let upgrade_weapon = Action::builder("upgrade_dps_weapon")
        .cost(2.0)
        .precondition("gold", 90)
        .effect_set_to("dps_damage", 60)
        .effect_subtract_int("gold", 90)
        .build();

    // Collect all actions
    let actions = vec![
        recruit_healer,
        buy_healing,
        buy_antidote,
        buy_scroll,
        heal_tank,
        cure_poison,
        remove_curse,
        upgrade_armor,
        upgrade_weapon,
    ];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(initial_state.clone(), &goal, &actions);
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
    let mut current_state = initial_state.clone();
    let remaining_gold = 400;

    println!("\nSimulating plan execution:");
    for action in &actions {
        current_state = action.apply_effect(&current_state);
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
