use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct SpellData {
    pub name: String,
    pub mana_cost: i32,
    pub effects: HashMap<String, String>
}
