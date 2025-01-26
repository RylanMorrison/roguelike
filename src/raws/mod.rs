mod item_structs;
mod item_set_structs;
mod mob_structs;
mod prop_structs;
mod spawn_table_structs;
mod loot_structs;
mod faction_structs;
mod ability_structs;
mod chest_structs;
mod character_class_structs;
mod quest_structs;
mod species_structs;
mod rawmaster;

pub use item_structs::*;
use item_set_structs::*;
use mob_structs::*;
use prop_structs::*;
use spawn_table_structs::*;
use loot_structs::*;
pub use ability_structs::*;
pub use faction_structs::*;
pub use chest_structs::*;
pub use character_class_structs::*;
pub use quest_structs::*;
pub use species_structs::*;
pub use rawmaster::*;

use serde::Deserialize;
use std::sync::Mutex;
use std::collections::HashMap;

rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

#[derive(Deserialize, Debug, Clone)]
pub struct RenderableData {
    pub glyph: String,
    pub fg: Option<String>,
    pub bg: String,
    pub order: i32,
    pub x_size: Option<i32>,
    pub y_size: Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct MapMarkerData {
    pub glyph: String,
    pub fg: Option<String>,
    pub bg: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
pub struct AttributeBonusData {
    pub strength: Option<i32>,
    pub dexterity: Option<i32>,
    pub constitution: Option<i32>,
    pub intelligence: Option<i32>
}

#[derive(Deserialize, Debug, Clone)]
pub struct SkillBonusData {
    pub melee: Option<i32>,
    pub defence: Option<i32>,
    pub ranged: Option<i32>,
    pub magic: Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items: Vec<ItemData>,
    pub item_sets: Vec<ItemSetData>,
    pub item_class_colours: HashMap<String, String>,
    pub mobs: Vec<MobData>,
    pub props: Vec<PropData>,
    pub abilities: Vec<AbilityData>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_tables: Vec<LootTableData>,
    pub faction_table: Vec<FactionData>,
    pub chests: Vec<ChestData>,
    pub character_classes: Vec<CharacterClassData>,
    pub quests: Vec<QuestData>,
    pub species: Vec<SpeciesData>
}

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");

    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string");
    let decoder: Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");

    RAWS.lock().unwrap().load(decoder);
}
