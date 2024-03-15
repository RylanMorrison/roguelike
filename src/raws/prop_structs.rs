use serde::Deserialize;
use super::RenderableData;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct PropData {
    pub name: String,
    pub renderable: Option<RenderableData>,
    pub blocks_tile: Option<bool>,
    pub blocks_visibility: Option<bool>,
    pub door_open: Option<bool>,
    pub entry_trigger: Option<EntryTriggerData>,
    pub light: Option<PropLightData>
}

#[derive(Deserialize, Debug)]
pub struct PropLightData {
    pub range: i32,
    pub colour: String
}

#[derive(Deserialize, Debug)]
pub struct EntryTriggerData {
    pub effects: HashMap<String, String>
}
