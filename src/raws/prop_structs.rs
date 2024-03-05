use serde::Deserialize;
use super::RenderableData;

#[derive(Deserialize, Debug)]
pub struct PropData {
    pub name: String,
    pub renderable: Option<RenderableData>,
    pub blocks_tile: Option<bool>,
    pub blocks_visibility: Option<bool>,
    pub door_open: Option<bool>
}
