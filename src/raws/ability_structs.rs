use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct AbilityData {
    pub name: String,
    pub description: String,
    pub levels: HashMap<String, AbilityLevelData>
}

#[derive(Deserialize, Debug)]
pub struct AbilityLevelData {
    pub mana_cost: Option<i32>,
    pub effects: HashMap<String, String>
}
