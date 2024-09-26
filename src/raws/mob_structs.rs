use serde::Deserialize;
use super::{RenderableData, MapMarkerData};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct MobData {
    pub name : String,
    pub renderable : Option<RenderableData>,
    pub blocks_tile : bool,
    pub vision_range : i32,
    pub faction: Option<String>,
    pub quips: Option<Vec<String>>,
    pub attributes: MobAttributesData,
    pub skills: Option<HashMap<String, i32>>,
    pub level: Option<i32>,
    pub hp: Option<i32>,
    pub mana: Option<i32>,
    pub equipped: Option<Vec<String>>,
    pub natural: Option<MobNaturalData>,
    pub loot_table: Option<String>,
    pub light: Option<MobLight>,
    pub movement: String,
    pub gold: Option<String>,
    pub vendor: Option<String>,
    pub quest_giver: Option<bool>,
    pub abilities: Option<Vec<MobAbilityData>>,
    pub boss: Option<bool>,
    pub map_marker: Option<MapMarkerData>
}

#[derive(Deserialize, Debug)]
pub struct MobAttributesData {
    pub strength: Option<i32>,
    pub dexterity: Option<i32>,
    pub constitution: Option<i32>,
    pub intelligence: Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct MobNaturalData {
    pub armour_class: Option<i32>,
    pub attacks: Option<Vec<NaturalAttackData>>
}

#[derive(Deserialize, Debug)]
pub struct NaturalAttackData {
    pub name: String,
    pub hit_bonus: i32,
    pub damage: String
}

#[derive(Deserialize, Debug)]
pub struct MobLight {
    pub range: i32,
    pub colour: String
}

#[derive(Deserialize, Debug)]
pub struct MobAbilityData {
    pub name: String,
    pub level: Option<i32>
}
