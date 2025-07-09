use goap::prelude::*;

fn main() {
    println!("=== Modifiable State Usage Example ===\n");

    // Create an empty state
    let mut state = State::empty();
    println!("Initial state: {}", state);

    // Basic set operations - these work great!
    state.set("health", 100i32);
    state.set("has_sword", true);
    state.set("position_x", 10.5f64);
    state.set("player_name", "Alice");
    
    println!("After setting values: {}", state);

    // Basic get operations
    println!("\n=== Getting Values (Type-safe API) ===");
    
    // Using get with type annotation - direct access to typed values
    if let Some(health_val) = state.get::<i32>("health") {
        println!("Health (as i32): {}", health_val);
    }

    // NEW UNIFIED GET API - Much cleaner!
    println!("\n=== Getting Values (Unified get API with different patterns) ===");
    
    // Direct optional access with type inference - much cleaner!
    if let Some(health) = state.get::<i32>("health") {
        println!("Health: {}", health);
    }
    
    if let Some(has_sword) = state.get::<bool>("has_sword") {
        println!("Has sword: {}", has_sword);
    }
    
    if let Some(pos_x) = state.get::<f64>("position_x") {
        println!("Position X: {}", pos_x);
    }
    
    if let Some(name) = state.get::<String>("player_name") {
        println!("Player name: {}", name);
    }
    
    // Get with default values using unwrap_or
    let health = state.get::<i32>("health").unwrap_or(0);
    let mana = state.get::<i32>("mana").unwrap_or(50); // Default to 50 if not found
    println!("Health: {}, Mana: {} (with defaults)", health, mana);
    
    // Missing keys return None (no errors to handle!)
    if let Some(missing) = state.get::<i32>("missing_key") {
        println!("Missing key: {}", missing);
    } else {
        println!("Missing key returned None (as expected)");
    }
    
    // Wrong type also returns None (no errors!)
    if let Some(wrong_type) = state.get::<String>("health") {
        println!("Health as string: {}", wrong_type);
    } else {
        println!("Wrong type returned None (as expected)");
    }

    // Modifying existing values
    println!("\n=== Modifying Values ===");
    state.set("health", 75i32);
    println!("After reducing health: {}", state);

    // Working with complex scenarios
    println!("\n=== Complex State Operations ===");
    
    // Building a game character state
    let mut character_state = State::empty();
    character_state.set("level", 1i32);
    character_state.set("experience", 0i32);
    character_state.set("gold", 100i32);
    character_state.set("stamina", 100.0f64);
    character_state.set("location", "town");
    character_state.set("is_alive", true);
    
    println!("Character state: {}", character_state);
    
    // Simulate leveling up - much cleaner with new API!
    if let Some(current_level) = character_state.get::<i32>("level") {
        character_state.set("level", current_level + 1);
        character_state.set("experience", 0i32); // Reset XP
        println!("Leveled up! New state: {}", character_state);
    }
    
    // Simulate spending gold - no more Result handling!
    if let Some(current_gold) = character_state.get::<i32>("gold") {
        if current_gold >= 50 {
            character_state.set("gold", current_gold - 50);
            character_state.set("has_magic_sword", true);
            println!("Bought magic sword! New state: {}", character_state);
        }
    }
    
    // Show even cleaner patterns with unwrap_or
    let current_stamina = character_state.get::<f64>("stamina").unwrap_or(100.0);
    let current_mana = character_state.get::<f64>("mana").unwrap_or(50.0); // Default if not set
    println!("Current stamina: {}, mana: {} (using unwrap_or)", current_stamina, current_mana);
    
    // Demonstrate state operations for more complex modifications
    println!("\n=== Using State Operations ===");
    use std::collections::HashMap;
    
    let mut operations = HashMap::new();
    operations.insert("stamina".to_string(), StateOperation::subtract_f64(25.0));
    operations.insert("experience".to_string(), StateOperation::add_i64(50));
    
    character_state.apply(&operations);
    println!("After applying operations: {}", character_state);

    // Show current API strengths and potential improvements
    println!("\n=== API Assessment ===");
    println!("✓ Current API strengths:");
    println!("  - Generic set() method works with any type");
    println!("  - get_typed() provides type-safe optional extraction");
    println!("  - No error handling required for common operations");
    println!("  - StateOperations for complex modifications");
    
    println!("\n✅ Unified get API implemented:");
    println!("  - get<T>() is the primary method - returns Option<T>");
    println!("  - Type-safe access with compile-time type checking");
    println!("  - No more error handling required for missing keys or wrong types");
    println!("  - Clean, single-level unwrapping for direct value access");
    println!("  - Perfect HashMap-like experience");
    
    println!("\n🚀 Usage patterns now possible:");
    println!("  let health = state.get::<i32>(\"health\").unwrap_or(100);");
    println!("  if let Some(gold) = state.get::<i32>(\"gold\") {{ ... }}");
    println!("  let name: Option<String> = state.get(\"name\");");
}