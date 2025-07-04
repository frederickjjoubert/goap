use goap::prelude::*;

fn main() {
    // Initial state - current party members and their attributes
    let initial_state = State::new()
        // Party composition
        .set("tank_available", true)
        .set("healer_available", false)
        .set("dps_available", true)
        // Party member stats
        .set("tank_health", 50)
        .set("tank_armor", 30)
        .set("dps_damage", 40)
        .set("party_size", 2)
        // Status effects
        .set("tank_poisoned", true)
        .set("dps_cursed", true)
        // Resources
        .set("gold", 400)
        .set("healing_potions", 1)
        .set("antidote_potions", 0)
        .set("remove_curse_scrolls", 0)
        .build();

    // Goal state - party ready for dungeon
    let goal = Goal::new("prepare_dungeon_party")
        .requires("healer_available", true)
        .requires("tank_health", 100)
        .requires("tank_armor", 50)
        .requires("tank_poisoned", false)
        .requires("dps_cursed", false)
        .requires("dps_damage", 60)
        .requires("party_size", 3)
        .build();

    // Action: Recruit Healer
    let recruit_healer = Action::new("recruit_healer")
        .cost(3.0)
        .requires("gold", 100)
        .sets("healer_available", true)
        .adds("party_size", 1)
        .subtracts("gold", 100)
        .build();

    // Action: Buy Healing Potion
    let buy_healing = Action::new("buy_healing_potion")
        .cost(1.0)
        .requires("gold", 50)
        .adds("healing_potions", 1)
        .subtracts("gold", 50)
        .build();

    // Action: Buy Antidote
    let buy_antidote = Action::new("buy_antidote")
        .cost(1.0)
        .requires("gold", 30)
        .adds("antidote_potions", 1)
        .subtracts("gold", 30)
        .build();

    // Action: Buy Remove Curse Scroll
    let buy_scroll = Action::new("buy_remove_curse_scroll")
        .cost(1.0)
        .requires("gold", 40)
        .adds("remove_curse_scrolls", 1)
        .subtracts("gold", 40)
        .build();

    // Action: Heal Tank
    let heal_tank = Action::new("heal_tank")
        .cost(1.0)
        .requires("healing_potions", 1)
        .sets("tank_health", 100)
        .subtracts("healing_potions", 1)
        .build();

    // Action: Cure Tank Poison
    let cure_poison = Action::new("cure_tank_poison")
        .cost(1.0)
        .requires("antidote_potions", 1)
        .sets("tank_poisoned", false)
        .subtracts("antidote_potions", 1)
        .build();

    // Action: Remove DPS Curse
    let remove_curse = Action::new("remove_dps_curse")
        .cost(1.0)
        .requires("remove_curse_scrolls", 1)
        .sets("dps_cursed", false)
        .subtracts("remove_curse_scrolls", 1)
        .build();

    // Action: Upgrade Tank Armor
    let upgrade_armor = Action::new("upgrade_tank_armor")
        .cost(2.0)
        .requires("gold", 80)
        .sets("tank_armor", 50)
        .subtracts("gold", 80)
        .build();

    // Action: Upgrade DPS Weapon
    let upgrade_weapon = Action::new("upgrade_dps_weapon")
        .cost(2.0)
        .requires("gold", 90)
        .sets("dps_damage", 60)
        .subtracts("gold", 90)
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
        plan_result.is_ok(),
        "Expected to find a valid plan for party management"
    );

    let plan = plan_result.unwrap();

    println!(
        "\nParty Management Plan found with cost {cost}",
        cost = plan.cost
    );
    for action in &plan.actions {
        println!(
            "- {name} (cost: {cost})",
            name = action.name,
            cost = action.cost
        );
    }

    // Verify the plan contains necessary actions
    let action_names: Vec<_> = plan.actions.iter().map(|a| a.name.as_str()).collect();

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
    for action in &plan.actions {
        current_state = action.apply_effect(&current_state);
        let name = &action.name;
        println!("After {name}: ");
        println!("  Gold: {remaining_gold}");
        if let Some(StateVar::I64(party_size)) = current_state.get("party_size") {
            println!("  Party Size: {party_size}");
        }
        if let Some(StateVar::I64(tank_health)) = current_state.get("tank_health") {
            println!("  Tank Health: {tank_health}");
        }
        if let Some(StateVar::I64(tank_armor)) = current_state.get("tank_armor") {
            println!("  Tank Armor: {tank_armor}");
        }
        if let Some(StateVar::I64(dps_damage)) = current_state.get("dps_damage") {
            println!("  DPS Damage: {dps_damage}");
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
    let party_size = current_state.get("party_size");
    println!("Party Size: {party_size:?}");
    let tank_health = current_state.get("tank_health");
    println!("Tank Health: {tank_health:?}");
    let tank_armor = current_state.get("tank_armor");
    println!("Tank Armor: {tank_armor:?}");
    let tank_poisoned = current_state.get("tank_poisoned");
    println!("Tank Poisoned: {tank_poisoned:?}");
    let dps_damage = current_state.get("dps_damage");
    println!("DPS Damage: {dps_damage:?}");
    let dps_cursed = current_state.get("dps_cursed");
    println!("DPS Cursed: {dps_cursed:?}");
    let healer_available = current_state.get("healer_available");
    println!("Healer Available: {healer_available:?}");
    println!("Remaining Gold: {remaining_gold}");
}
