use serde::Deserialize;
use std::collections::HashMap;
use super::{AttributeBonusData, SkillBonusData};

#[derive(Deserialize, Debug)]
pub struct CharacterClassData {
    pub name: String,
    pub description: String,
    pub passives: Vec<CharacterClassPassiveData>
}

#[derive(Deserialize, Debug)]
pub struct CharacterClassPassiveData {
    pub name: String,
    pub description: String,
    pub levels: HashMap<String, CharacterClassAbilityLevelData>
}

#[derive(Deserialize, Debug)]
pub struct CharacterClassAbilityLevelData {
    pub attribute_bonus: Option<AttributeBonusData>,
    pub skill_bonus: Option<SkillBonusData>,
    pub teaches_ability: Option<String>,
    pub levels_ability: Option<String>
}
