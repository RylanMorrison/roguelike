use serde::Deserialize;
use std::collections::HashMap;
use super::RenderableData;

#[derive(Deserialize, Debug)]
pub struct ItemData {
    pub name: String,
    pub renderable:  Option<RenderableData>,
    pub consumable: Option<ConsumableData>,
    pub weapon: Option<WeaponData>,
    pub wearable: Option<WearableData>
}

#[derive(Deserialize, Debug)]
pub struct ConsumableData {
    pub effects: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct WeaponData {
    pub range: String,
    pub attribute: String,
    pub base_damage: String,
    pub hit_bonus: i32,
    pub slot: String
}

#[derive(Deserialize, Debug)]
pub struct WearableData {
    pub armour_class: f32,
    pub slot: String
}
