use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct QuestData {
    pub name: String,
    pub description: String,
    pub reward: QuestRewardData,
    pub requirements: Vec<QuestRewardRequirement>
}

#[derive(Deserialize, Debug)]
pub struct QuestRewardData {
    pub gold: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct QuestRewardRequirement {
    pub goal: String,
    pub targets: Vec<String>,
    pub count: Option<i32>
}
