use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LootTableData {
    pub name: String,
    pub drops: Vec<LootDropData>
}

#[derive(Deserialize, Debug)]
pub struct LootDropData {
    pub name: String,
    pub weight: i32
}
