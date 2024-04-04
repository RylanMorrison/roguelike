use serde::Deserialize;
use super::{ItemAttributeBonusData, ItemSkillBonusData};

#[derive(Deserialize, Debug)]
pub struct ItemSetData {
    pub name: String,
    pub total_pieces: i32,
    pub set_bonuses: Vec<ItemSetBonusData>
}

#[derive(Deserialize, Debug)]
pub struct ItemSetBonusData {
    pub required_pieces: i32,
    pub attribute_bonuses: Option<ItemAttributeBonusData>,
    pub skill_bonuses: Option<ItemSkillBonusData>
}
