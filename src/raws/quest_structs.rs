use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct QuestData {
    pub name: String,
    pub description: String,
    pub rewards: Vec<QuestRewardData>,
    pub requirements: Vec<QuestRewardRequirementData>,
    pub initial: Option<bool>,
    pub next_quests: Option<Vec<String>>
}

#[derive(Deserialize, Debug)]
pub struct QuestRewardData {
    pub gold: Option<String>,
    pub xp: Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct QuestRewardRequirementData {
    pub goal: String,
    pub targets: Vec<String>,
    pub count: Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct QuestPrerequisiteData {
    pub quests: Vec<String>,
    pub status: String
}
