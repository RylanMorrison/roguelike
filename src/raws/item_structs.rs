use serde::Deserialize;
use std::collections::HashMap;
use super::RenderableData;

#[derive(Deserialize, Debug)]
pub struct ItemData {
    pub name: String,
    pub renderable:  Option<RenderableData>,
    pub consumable: Option<ConsumableData>,
    pub weapon: Option<WeaponData>,
    pub wearable: Option<WearableData>,
    pub initiative_penalty: Option<f32>,
    pub weight_lbs: Option<f32>,
    pub base_value: Option<i32>,
    pub vendor_category: Option<String>,
    pub class: String,
    pub attribute_bonuses: Option<ItemAttributeBonusData>,
    pub skill_bonuses: Option<ItemSkillBonusData>
}

#[derive(Deserialize, Debug)]
pub struct ConsumableData {
    pub effects: HashMap<String, String>,
    pub charges: Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct WeaponData {
    pub range: String,
    pub attribute: String,
    pub base_damage: String,
    pub hit_bonus: i32,
    pub slot: String,
    pub proc_chance: Option<f32>,
    pub proc_target: Option<String>,
    pub proc_effects: Option<HashMap<String, String>>
}

#[derive(Deserialize, Debug)]
pub struct WearableData {
    pub armour_class: f32,
    pub slot: String
}

#[derive(Deserialize, Debug)]
pub struct ItemAttributeBonusData {
    pub strength: Option<i32>,
    pub dexterity: Option<i32>,
    pub constitution: Option<i32>,
    pub intelligence: Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct ItemSkillBonusData {
    pub melee: Option<i32>,
    pub defence: Option<i32>,
    pub magic: Option<i32>
}
