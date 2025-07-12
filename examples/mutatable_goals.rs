use goap::prelude::*;

fn main() {
    println!("=== Mutatable Goals API Design Example ===\n");

    // Current API - Using builder pattern
    println!("=== Current Builder API ===");
    let mut goal = Goal::new("survive_battle")
        .priority(100)
        .requires("health", 50)
        .requires("has_weapon", true)
        .requires("armor_durability", 25)
        .requires("mana", 30)
        .build();

    println!("Original goal:\n{}\n", goal);

    // PROPOSED: Direct mutation API similar to State
    println!("=== PROPOSED: Direct Mutation API ===");

    // Problem: Goals don't currently support direct mutation
    // We need methods like:
    // - goal.set_priority(priority)
    // - goal.get_priority()
    // - goal.add_requirement(key, value)
    // - goal.remove_requirement(key)
    // - goal.get_requirement<T>(key) -> Option<T>
    // - goal.update_requirement(key, value)
    // - goal.is_requirement_met(key, current_state) -> bool

    println!("🚀 DESIRED API (not yet implemented):");
    println!("  // Create goal with basic info");
    println!("  let mut goal = Goal::empty(\"survive_battle\");");
    println!("  goal.set_priority(100);");
    println!("  ");
    println!("  // Add requirements with unified API");
    println!("  goal.add_requirement(\"health\", 50);");
    println!("  goal.add_requirement(\"has_weapon\", true);");
    println!("  goal.add_requirement(\"armor_durability\", 25);");
    println!("  goal.add_requirement(\"mana\", 30);");
    println!("  ");
    println!("  // Get requirements");
    println!("  let min_health = goal.get_requirement::<i32>(\"health\").unwrap_or(100);");
    println!("  let needs_weapon = goal.get_requirement::<bool>(\"has_weapon\").unwrap_or(false);");
    println!("  ");
    println!("  // Modify existing requirements");
    println!("  goal.update_requirement(\"health\", min_health + 25);");
    println!("  ");
    println!("  // Remove requirements");
    println!("  goal.remove_requirement(\"armor_durability\");");
    println!("  ");
    println!("  // Check individual requirements");
    println!("  if goal.is_requirement_met(\"health\", &current_state) {{");
    println!("      println!(\"Health requirement satisfied!\");");
    println!("  }}");

    println!("\n=== Real-world Use Cases ===");

    // Use case 1: Dynamic goal adaptation
    println!("🎯 Use Case 1: Dynamic Goal Adaptation");
    println!("  // Adjust goal based on player performance");
    println!("  if player_deaths > 3 {{");
    println!("      // Make survival goal easier");
    println!(
        "      let current_health_req = goal.get_requirement::<i32>(\"health\").unwrap_or(100);"
    );
    println!("      goal.update_requirement(\"health\", current_health_req - 20);");
    println!("      goal.set_priority(goal.get_priority() + 50); // Higher priority");
    println!("  }}");
    println!("  ");
    println!("  // Adjust based on available resources");
    println!("  if player_gold < 100 {{");
    println!("      goal.remove_requirement(\"expensive_armor\");");
    println!("      goal.add_requirement(\"basic_armor\", true);");
    println!("  }}");

    // Use case 2: Contextual goal modification
    println!("\n🌙 Use Case 2: Contextual Goal Modification");
    println!("  // Modify goals based on game context");
    println!("  if is_night_time {{");
    println!("      goal.add_requirement(\"has_torch\", true);");
    println!("      goal.add_requirement(\"night_vision_potion\", 1);");
    println!("  }}");
    println!("  ");
    println!("  if is_underwater_level {{");
    println!("      goal.add_requirement(\"oxygen_tank\", 1);");
    println!("      goal.remove_requirement(\"fire_resistance\");");
    println!("  }}");

    // Use case 3: Progressive goal refinement
    println!("\n📈 Use Case 3: Progressive Goal Refinement");
    println!("  // Start with basic goal, add complexity over time");
    println!("  let mut goal = Goal::empty(\"complete_quest\");");
    println!("  goal.set_priority(50);");
    println!("  ");
    println!("  // Phase 1: Basic requirements");
    println!("  goal.add_requirement(\"quest_started\", true);");
    println!("  goal.add_requirement(\"player_level\", 5);");
    println!("  ");
    println!("  // Phase 2: Add complexity based on player actions");
    println!("  if player_chose_hard_mode {{");
    println!("      goal.add_requirement(\"special_item\", true);");
    println!("      goal.update_requirement(\"player_level\", 10);");
    println!("      goal.set_priority(goal.get_priority() + 25);");
    println!("  }}");
    println!("  ");
    println!("  // Phase 3: Final requirements");
    println!("  if near_quest_end {{");
    println!("      goal.add_requirement(\"boss_defeated\", true);");
    println!("      goal.add_requirement(\"artifacts_collected\", 3);");
    println!("  }}");

    // Use case 4: Goal debugging and analysis
    println!("\n🔍 Use Case 4: Goal Debugging and Analysis");
    println!("  // Debug goal requirements");
    println!("  println!(\"Goal '{{}}' requires:\", goal.get_name());");
    println!("  for (key, value) in goal.get_all_requirements() {{");
    println!("      println!(\"  - {{}}: {{}}\", key, value);");
    println!("  }}");
    println!("  ");
    println!("  // Check progress toward goal");
    println!("  let unmet_requirements = goal.get_unmet_requirements(&current_state);");
    println!("  if unmet_requirements.is_empty() {{");
    println!("      println!(\"Goal can be achieved!\");");
    println!("  }} else {{");
    println!("      println!(\"Still need: {{:?}}\", unmet_requirements);");
    println!("  }}");
    println!("  ");
    println!("  // Calculate goal completion percentage");
    println!("  let completion = goal.calculate_completion_percentage(&current_state);");
    println!("  println!(\"Goal {{}}% complete\", completion);");

    // Use case 5: Goal composition and merging
    println!("\n🔗 Use Case 5: Goal Composition and Merging");
    println!("  // Combine multiple goals into one");
    println!("  let mut main_goal = Goal::empty(\"complete_chapter\");");
    println!("  main_goal.set_priority(200);");
    println!("  ");
    println!("  // Merge requirements from sub-goals");
    println!("  main_goal.merge_requirements(&combat_goal);");
    println!("  main_goal.merge_requirements(&exploration_goal);");
    println!("  main_goal.merge_requirements(&puzzle_goal);");
    println!("  ");
    println!("  // Override specific requirements");
    println!("  main_goal.update_requirement(\"health\", 100); // Ensure full health");
    println!("  main_goal.set_priority(main_goal.get_priority() + 50);");

    println!("\n=== API Design Principles ===");
    println!("✅ Key design principles for mutatable goals:");
    println!("  1. Direct mutation methods (no need to rebuild)");
    println!("  2. Type-safe get/set operations like State API");
    println!("  3. Optional values with unwrap_or patterns");
    println!("  4. Unified interface for all value types");
    println!("  5. Easy requirement management and introspection");
    println!("  6. Progress tracking and completion analysis");
    println!("  7. No error handling for missing keys (return None)");
    println!("  8. Goal composition and merging capabilities");
    println!("  9. Priority management and adjustment");
    println!("  10. Contextual requirement checking");

    println!("\n=== Current Limitations ===");
    println!("❌ Current API limitations:");
    println!("  - Must use builder pattern for all modifications");
    println!("  - Cannot modify existing goals without rebuilding");
    println!("  - No easy way to inspect individual requirements");
    println!("  - Difficult to do conditional modifications");
    println!("  - No direct access to individual requirements");
    println!("  - Complex to modify based on runtime conditions");
    println!("  - No progress tracking or partial completion analysis");
    println!("  - No goal composition or merging capabilities");

    println!("\n=== Implementation Strategy ===");
    println!("🛠️ How to implement:");
    println!("  1. Add empty() constructor for creating blank goals");
    println!("  2. Add priority management methods:");
    println!("     - set_priority(priority)");
    println!("     - get_priority() -> u16");
    println!("     - adjust_priority(delta)");
    println!("  3. Add requirement management methods:");
    println!("     - add_requirement<T>(key, value)");
    println!("     - get_requirement<T>(key) -> Option<T>");
    println!("     - update_requirement<T>(key, value)");
    println!("     - remove_requirement(key)");
    println!("     - has_requirement(key) -> bool");
    println!("  4. Add requirement checking methods:");
    println!("     - is_requirement_met(key, state) -> bool");
    println!("     - get_unmet_requirements(state) -> Vec<String>");
    println!("     - calculate_completion_percentage(state) -> f64");
    println!("  5. Add introspection methods:");
    println!("     - get_all_requirements() -> iterator");
    println!("     - get_requirement_count() -> usize");
    println!("     - get_name() -> &str");
    println!("     - set_name(name)");
    println!("  6. Add composition methods:");
    println!("     - merge_requirements(other_goal)");
    println!("     - copy_requirements_from(other_goal)");
    println!("     - combine_with(other_goal) -> Goal");

    println!("\n=== Example Usage Patterns ===");
    println!("📚 Common patterns this API would enable:");
    println!("  ");
    println!("  // Pattern 1: Adaptive goal difficulty");
    println!("  let mut goal = Goal::empty(\"defeat_boss\");");
    println!("  goal.set_priority(100);");
    println!("  if player_skill_level < 50 {{");
    println!("      goal.add_requirement(\"health_potion\", 5);");
    println!("      goal.add_requirement(\"magic_weapon\", true);");
    println!("  }}");
    println!("  ");
    println!("  // Pattern 2: Progress-based goal modification");
    println!("  let completion = goal.calculate_completion_percentage(&current_state);");
    println!("  if completion > 80.0 {{");
    println!("      goal.set_priority(goal.get_priority() + 20); // Almost done, prioritize!");
    println!("  }}");
    println!("  ");
    println!("  // Pattern 3: Contextual requirement adjustment");
    println!("  if current_state.get::<String>(\"location\") == Some(\"desert\".to_string()) {{");
    println!("      goal.add_requirement(\"water_bottles\", 3);");
    println!("      goal.remove_requirement(\"cold_weather_gear\");");
    println!("  }}");
    println!("  ");
    println!("  // Pattern 4: Goal debugging and analysis");
    println!("  let unmet = goal.get_unmet_requirements(&current_state);");
    println!("  if !unmet.is_empty() {{");
    println!("      println!(\"Cannot achieve goal yet. Need: {{:?}}\", unmet);");
    println!("  }}");
    println!("  ");
    println!("  // Pattern 5: Dynamic goal creation");
    println!("  let mut dynamic_goal = Goal::empty(\"custom_objective\");");
    println!("  for (req, value) in player_selected_requirements {{");
    println!("      dynamic_goal.add_requirement(req, value);");
    println!("  }}");
    println!("  dynamic_goal.set_priority(player_selected_priority);");

    println!("\n=== Integration with Planning ===");
    println!("🔄 How this integrates with the planner:");
    println!("  ");
    println!("  // Goals can be modified during planning");
    println!("  let mut goals = vec![survival_goal, combat_goal, exploration_goal];");
    println!("  ");
    println!("  // Adjust goals based on current state");
    println!("  for goal in &mut goals {{");
    println!("      if goal.calculate_completion_percentage(&current_state) > 90.0 {{");
    println!(
        "          goal.set_priority(goal.get_priority() / 2); // Deprioritize almost-complete goals"
    );
    println!("      }}");
    println!("  }}");
    println!("  ");
    println!("  // Create dynamic goals based on opportunities");
    println!("  if current_state.get::<bool>(\"rare_item_nearby\").unwrap_or(false) {{");
    println!("      let mut opportunistic_goal = Goal::empty(\"grab_rare_item\");");
    println!("      opportunistic_goal.set_priority(150);");
    println!("      opportunistic_goal.add_requirement(\"rare_item_collected\", true);");
    println!("      goals.push(opportunistic_goal);");
    println!("  }}");
    println!("  ");
    println!("  // Plan with modified goals");
    println!("  let best_plan = planner.plan_multi_goal(current_state, &goals, &actions);");
}
