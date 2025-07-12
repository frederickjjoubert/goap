use goap::prelude::*;

fn main() {
    println!("=== Modifiable State Usage Example ===\n");

    // Create an empty state
    let mut state = State::empty();
    println!("Initial state: {}", state);

    // Basic set operations
    state.set("health", 100i32);
    state.set("has_sword", true);
    state.set("position_x", 10.5f64);
    state.set("player_name", "Alice");

    println!("After setting values: {}", state);

    // Basic get operations
    println!("\n=== Getting Values ===");

    // Direct optional access with type inference
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

    // Missing keys return None
    if let Some(missing) = state.get::<i32>("missing_key") {
        println!("Missing key: {}", missing);
    } else {
        println!("Missing key returned None");
    }

    // Wrong type also returns None
    if let Some(wrong_type) = state.get::<String>("health") {
        println!("Health as string: {}", wrong_type);
    } else {
        println!("Wrong type returned None");
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

    // Simulate leveling up
    if let Some(current_level) = character_state.get::<i32>("level") {
        character_state.set("level", current_level + 1);
        character_state.set("experience", 0i32); // Reset XP
        println!("Leveled up! New state: {}", character_state);
    }

    // Simulate spending gold
    if let Some(current_gold) = character_state.get::<i32>("gold") {
        if current_gold >= 50 {
            character_state.set("gold", current_gold - 50);
            character_state.set("has_magic_sword", true);
            println!("Bought magic sword! New state: {}", character_state);
        }
    }

    let current_stamina = character_state.get::<f64>("stamina").unwrap_or(100.0);
    let current_mana = character_state.get::<f64>("mana").unwrap_or(50.0); // Default if not set
    println!(
        "Current stamina: {}, mana: {} (using unwrap_or)",
        current_stamina, current_mana
    );
}
