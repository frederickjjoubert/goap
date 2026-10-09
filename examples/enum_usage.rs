use goap::prelude::*;
use std::fmt;
use std::str::FromStr;

// Define custom enums
#[derive(Debug, Clone, PartialEq)]
enum Location {
    Town,
    Forest,
    Cave,
    Dungeon,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Town => write!(f, "town"),
            Location::Forest => write!(f, "forest"),
            Location::Cave => write!(f, "cave"),
            Location::Dungeon => write!(f, "dungeon"),
        }
    }
}

// `FromStr` is the inverse of `Display`; it lets `state.get::<Location>` work.
impl FromStr for Location {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "town" => Ok(Location::Town),
            "forest" => Ok(Location::Forest),
            "cave" => Ok(Location::Cave),
            "dungeon" => Ok(Location::Dungeon),
            other => Err(format!("unknown location '{other}'")),
        }
    }
}

// Implement the marker trait to enable enum support
impl EnumStateVar for Location {}

#[derive(Debug, Clone, PartialEq)]
enum CharacterClass {
    Warrior,
    Mage,
}

impl fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharacterClass::Warrior => write!(f, "warrior"),
            CharacterClass::Mage => write!(f, "mage"),
        }
    }
}

impl FromStr for CharacterClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "warrior" => Ok(CharacterClass::Warrior),
            "mage" => Ok(CharacterClass::Mage),
            other => Err(format!("unknown class '{other}'")),
        }
    }
}

impl EnumStateVar for CharacterClass {}

#[derive(Debug, Clone, PartialEq)]
enum QuestStatus {
    NotStarted,
    InProgress,
    Completed,
}

impl fmt::Display for QuestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuestStatus::NotStarted => write!(f, "not_started"),
            QuestStatus::InProgress => write!(f, "in_progress"),
            QuestStatus::Completed => write!(f, "completed"),
        }
    }
}

impl FromStr for QuestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "not_started" => Ok(QuestStatus::NotStarted),
            "in_progress" => Ok(QuestStatus::InProgress),
            "completed" => Ok(QuestStatus::Completed),
            other => Err(format!("unknown quest status '{other}'")),
        }
    }
}

impl EnumStateVar for QuestStatus {}

fn main() {
    // Initial state - character starts in town as a warrior with no quests
    let initial_state = State::builder()
        .set("location", Location::Town)
        .set("character_class", CharacterClass::Warrior)
        .set("level", 1)
        .set("has_sword", false)
        .set("has_magic_scroll", false)
        .set("dragon_quest", QuestStatus::NotStarted)
        .set("gold", 100)
        .build();

    // Goal state - complete the dragon quest as a mage in the dungeon
    let goal = Goal::builder("complete_dragon_quest")
        .requires("location", Location::Dungeon)
        .requires("character_class", CharacterClass::Mage)
        .requires("dragon_quest", QuestStatus::Completed)
        .requires("level", at_least(5))
        .build();

    // Action: Travel to different locations
    let travel_to_forest = Action::builder("travel_to_forest")
        .cost(2.0)
        .sets("location", Location::Forest)
        .build();

    let travel_to_cave = Action::builder("travel_to_cave")
        .cost(3.0)
        .sets("location", Location::Cave)
        .build();

    let travel_to_dungeon = Action::builder("travel_to_dungeon")
        .cost(4.0)
        .requires("level", at_least(3))
        .sets("location", Location::Dungeon)
        .build();

    // Action: Change character class
    let become_mage = Action::builder("become_mage")
        .cost(5.0)
        .requires("character_class", CharacterClass::Warrior)
        .requires("has_magic_scroll", true)
        .requires("gold", at_least(20))
        .sets("character_class", CharacterClass::Mage)
        .subtracts("gold", 20)
        .build();

    // Action: Acquire items
    let buy_magic_scroll = Action::builder("buy_magic_scroll")
        .cost(1.0)
        .requires("location", Location::Town)
        .requires("gold", at_least(10))
        .sets("has_magic_scroll", true)
        .subtracts("gold", 10)
        .build();

    let find_sword = Action::builder("find_sword")
        .cost(2.0)
        .requires("location", Location::Forest)
        .sets("has_sword", true)
        .build();

    // Action: Level up through training
    let train_level_2 = Action::builder("train_to_level_2")
        .cost(3.0)
        .requires("level", 1)
        .requires("gold", at_least(10))
        .sets("level", 2)
        .subtracts("gold", 10)
        .build();

    let train_level_3 = Action::builder("train_to_level_3")
        .cost(4.0)
        .requires("level", 2)
        .requires("gold", at_least(10))
        .sets("level", 3)
        .subtracts("gold", 10)
        .build();

    let train_level_4 = Action::builder("train_to_level_4")
        .cost(5.0)
        .requires("level", 3)
        .requires("gold", at_least(10))
        .sets("level", 4)
        .subtracts("gold", 10)
        .build();

    let train_level_5 = Action::builder("train_to_level_5")
        .cost(6.0)
        .requires("level", 4)
        .requires("gold", at_least(10))
        .sets("level", 5)
        .subtracts("gold", 10)
        .build();

    // Action: Start and complete quest
    let start_dragon_quest = Action::builder("start_dragon_quest")
        .cost(1.0)
        .requires("location", Location::Town)
        .requires("dragon_quest", QuestStatus::NotStarted)
        .sets("dragon_quest", QuestStatus::InProgress)
        .build();

    let complete_dragon_quest = Action::builder("complete_dragon_quest")
        .cost(10.0)
        .requires("location", Location::Dungeon)
        .requires("character_class", CharacterClass::Mage)
        .requires("dragon_quest", QuestStatus::InProgress)
        .requires("level", at_least(5))
        .sets("dragon_quest", QuestStatus::Completed)
        .adds("gold", 200)
        .build();

    // Collect all actions
    let actions = vec![
        travel_to_forest,
        travel_to_cave,
        travel_to_dungeon,
        become_mage,
        buy_magic_scroll,
        find_sword,
        train_level_2,
        train_level_3,
        train_level_4,
        train_level_5,
        start_dragon_quest,
        complete_dragon_quest,
    ];

    // Create planner
    let planner = Planner::new();

    // Find plan
    let plan_result = planner.plan(&initial_state, &goal, &actions);
    assert!(
        plan_result.is_ok(),
        "Expected to find a valid plan for enum-based RPG quest"
    );

    let plan = plan_result.unwrap();

    println!(
        "\nEnum-based RPG Quest Plan found with cost {cost}",
        cost = plan.cost
    );
    for action in &plan.actions {
        println!(
            "- {name} (cost: {cost})",
            name = action.name,
            cost = action.cost
        );
    }

    // Simulate plan execution to show enum usage
    let mut current_state = initial_state;

    println!("\nSimulating plan execution with proper enums:");
    for action in &plan.actions {
        current_state = action.apply_effect(&current_state);
        let name = &action.name;
        println!("After {name}:");

        // Enum values are stored as strings and read back as the enum.
        if let Some(location) = current_state.get::<Location>("location") {
            println!("  Location: {location} ({location:?})");
        }
        if let Some(class) = current_state.get::<CharacterClass>("character_class") {
            println!("  Class: {class} ({class:?})");
        }
        if let Some(quest_status) = current_state.get::<QuestStatus>("dragon_quest") {
            println!("  Dragon Quest: {quest_status} ({quest_status:?})");
        }
        if let Some(level) = current_state.get::<i64>("level") {
            println!("  Level: {level}");
        }
        if let Some(gold) = current_state.get::<i64>("gold") {
            println!("  Gold: {gold}");
        }
        println!();
    }

    // Verify final state meets all goals
    assert!(
        goal.is_satisfied(&current_state),
        "Final state should meet all goals"
    );

    println!("✅ Quest completed successfully using proper enum types!");
    println!("The character is now a level 5 mage in the dungeon with the dragon quest completed.");
}
