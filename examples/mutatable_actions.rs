use goap::prelude::*;

fn main() {
    println!("=== Mutatable Actions API Design Example ===\n");

    // Current API - Using builder pattern
    println!("=== Current Builder API ===");
    let mut action = Action::new("attack_enemy")
        .cost(3.0)
        .requires("has_weapon", true)
        .requires("stamina", 10)
        .requires("target_in_range", true)
        .sets("enemy_health", 0)
        .subtracts("stamina", 10)
        .adds("experience", 50)
        .build();

    println!("Original action:\n{}\n", action);

    // PROPOSED: Direct mutation API similar to State
    println!("=== PROPOSED: Direct Mutation API ===");

    // Problem: Actions don't currently support direct mutation
    // We need methods like:
    // - action.set_cost(cost)
    // - action.add_precondition(key, value)
    // - action.remove_precondition(key)
    // - action.get_precondition<T>(key) -> Option<T>
    // - action.add_effect_set(key, value)
    // - action.add_effect_add(key, value)
    // - action.add_effect_subtract(key, value)
    // - action.remove_effect(key)
    // - action.get_effect(key) -> Option<&StateOperation>

    println!("🚀 DESIRED API (not yet implemented):");
    println!("  // Create action with basic info");
    println!("  let mut action = Action::empty(\"attack_enemy\");");
    println!("  action.set_cost(3.0);");
    println!("  ");
    println!("  // Add preconditions with unified API");
    println!("  action.add_precondition(\"has_weapon\", true);");
    println!("  action.add_precondition(\"stamina\", 10);");
    println!("  action.add_precondition(\"target_in_range\", true);");
    println!("  ");
    println!("  // Add effects with different operations");
    println!("  action.add_effect_set(\"enemy_health\", 0);");
    println!("  action.add_effect_subtract(\"stamina\", 10);");
    println!("  action.add_effect_add(\"experience\", 50);");
    println!("  ");
    println!("  // Get preconditions and effects");
    println!("  let required_stamina = action.get_precondition::<i32>(\"stamina\").unwrap_or(0);");
    println!(
        "  let has_weapon_req = action.get_precondition::<bool>(\"has_weapon\").unwrap_or(false);"
    );
    println!("  ");
    println!("  // Modify existing preconditions");
    println!("  action.update_precondition(\"stamina\", required_stamina + 5);");
    println!("  ");
    println!("  // Remove preconditions/effects");
    println!("  action.remove_precondition(\"target_in_range\");");
    println!("  action.remove_effect(\"enemy_health\");");

    println!("\n=== Real-world Use Cases ===");

    // Use case 1: Dynamic difficulty adjustment
    println!("🎯 Use Case 1: Dynamic Difficulty Adjustment");
    println!("  // Adjust action costs based on player skill");
    println!("  if player_skill > 80 {{");
    println!("      action.set_cost(action.get_cost() * 1.5); // Make harder");
    println!("  }}");
    println!("  ");
    println!("  // Adjust resource requirements");
    println!(
        "  let current_stamina_req = action.get_precondition::<i32>(\"stamina\").unwrap_or(10);"
    );
    println!(
        "  action.update_precondition(\"stamina\", current_stamina_req * difficulty_multiplier);"
    );

    // Use case 2: Equipment-based modifications
    println!("\n🛡️ Use Case 2: Equipment-Based Modifications");
    println!("  // Modify action based on equipped items");
    println!("  if player_has_magic_sword {{");
    println!("      action.add_effect_add(\"magic_damage\", 25);");
    println!("      action.add_precondition(\"mana\", 5);");
    println!("  }}");
    println!("  ");
    println!("  if player_has_stamina_ring {{");
    println!("      // Reduce stamina cost");
    println!("      let current_stamina_cost = action.get_effect_value(\"stamina\");");
    println!("      action.update_effect_subtract(\"stamina\", current_stamina_cost / 2);");
    println!("  }}");

    // Use case 3: Conditional action modification
    println!("\n⚡ Use Case 3: Conditional Action Modification");
    println!("  // Modify actions based on game state");
    println!("  if is_night_time {{");
    println!("      action.add_precondition(\"has_torch\", true);");
    println!("      action.set_cost(action.get_cost() * 0.8); // Stealth bonus");
    println!("  }}");
    println!("  ");
    println!("  if player_is_poisoned {{");
    println!("      action.add_effect_subtract(\"health\", 2);");
    println!("  }}");

    // Use case 4: Action debugging and introspection
    println!("\n🔍 Use Case 4: Action Debugging and Introspection");
    println!("  // Debug action requirements");
    println!("  println!(\"Action requires:\");");
    println!("  for (key, value) in action.get_all_preconditions() {{");
    println!("      println!(\"  - {{}}: {{}}\", key, value);");
    println!("  }}");
    println!("  ");
    println!("  // Check if action would be valid");
    println!("  if let Some(missing_req) = action.get_missing_preconditions(&current_state) {{");
    println!("      println!(\"Cannot execute: missing {{}}\", missing_req);");
    println!("  }}");

    println!("\n=== API Design Principles ===");
    println!("✅ Key design principles for mutatable actions:");
    println!("  1. Direct mutation methods (no need to rebuild)");
    println!("  2. Type-safe get/set operations like State API");
    println!("  3. Optional values with unwrap_or patterns");
    println!("  4. Unified interface for all value types");
    println!("  5. Separate methods for different effect types (set/add/subtract)");
    println!("  6. Easy introspection and debugging");
    println!("  7. No error handling for missing keys (return None)");
    println!("  8. Chainable operations where appropriate");

    println!("\n=== Current Limitations ===");
    println!("❌ Current API limitations:");
    println!("  - Must use builder pattern for all modifications");
    println!("  - Cannot modify existing actions without rebuilding");
    println!("  - No easy way to inspect preconditions/effects");
    println!("  - Difficult to do conditional modifications");
    println!("  - No direct access to individual requirements");
    println!("  - Complex to modify based on runtime conditions");

    println!("\n=== Implementation Strategy ===");
    println!("🛠️ How to implement:");
    println!("  1. Add empty() constructor for creating blank actions");
    println!("  2. Add set_cost() and get_cost() methods");
    println!("  3. Add precondition management methods:");
    println!("     - add_precondition<T>(key, value)");
    println!("     - get_precondition<T>(key) -> Option<T>");
    println!("     - update_precondition<T>(key, value)");
    println!("     - remove_precondition(key)");
    println!("  4. Add effect management methods:");
    println!("     - add_effect_set<T>(key, value)");
    println!("     - add_effect_add(key, value)");
    println!("     - add_effect_subtract(key, value)");
    println!("     - get_effect(key) -> Option<&StateOperation>");
    println!("     - remove_effect(key)");
    println!("  5. Add introspection methods:");
    println!("     - get_all_preconditions() -> iterator");
    println!("     - get_all_effects() -> iterator");
    println!("     - get_missing_preconditions(&state) -> Option<Vec<String>>");

    println!("\n=== Example Usage Patterns ===");
    println!("📚 Common patterns this API would enable:");
    println!("  ");
    println!("  // Pattern 1: Runtime action customization");
    println!("  let mut action = Action::empty(\"custom_spell\");");
    println!("  action.set_cost(base_cost * player_level);");
    println!("  for (resource, amount) in spell_requirements {{");
    println!("      action.add_precondition(resource, amount);");
    println!("  }}");
    println!("  ");
    println!("  // Pattern 2: Conditional action modification");
    println!("  if player_has_talent(\"reduced_costs\") {{");
    println!("      action.set_cost(action.get_cost() * 0.8);");
    println!("  }}");
    println!("  ");
    println!("  // Pattern 3: Easy action debugging");
    println!("  let stamina_req = action.get_precondition::<i32>(\"stamina\").unwrap_or(0);");
    println!("  println!(\"This action requires {{}} stamina\", stamina_req);");
    println!("  ");
    println!("  // Pattern 4: Action composition");
    println!("  let mut combo_action = base_action.clone();");
    println!("  combo_action.add_effect_add(\"combo_points\", 1);");
    println!("  combo_action.set_cost(combo_action.get_cost() + 1.0);");
}
