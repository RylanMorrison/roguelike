use serde::Deserialize;
use super::{AttributeBonusData, SkillBonusData};

#[derive(Deserialize, Debug)]
pub struct ItemSetData {
    pub name: String,
    pub total_pieces: i32,
    pub set_bonuses: Vec<ItemSetBonusData>
}

#[derive(Deserialize, Debug)]
pub struct ItemSetBonusData {
    pub required_pieces: i32,
    pub attribute_bonuses: Option<AttributeBonusData>,
    pub skill_bonuses: Option<SkillBonusData>
}
