use serde::Deserialize;
use std::collections::HashMap;
use super::{RenderableData, AttributeBonusData, SkillBonusData, RegenBonusData};

#[derive(Deserialize, Debug, Clone)]
pub struct ItemData {
    pub name: String,
    pub renderable:  Option<RenderableData>,
    pub consumable: Option<ConsumableData>,
    pub weapon: Option<WeaponData>,
    pub wearable: Option<WearableData>,
    pub initiative_penalty: Option<f32>,
    pub weight_lbs: Option<f32>,
    pub base_value: i32,
    pub vendor_category: Option<String>,
    pub class: String,
    pub attribute_bonuses: Option<AttributeBonusData>,
    pub skill_bonuses: Option<SkillBonusData>,
    pub set_name: Option<String>,
    pub regen_bonuses: Option<RegenBonusData>
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConsumableData {
    pub effects: HashMap<String, String>,
    pub charges: Option<i32>
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct WearableData {
    pub armour_class: f32,
    pub slot: String
}
