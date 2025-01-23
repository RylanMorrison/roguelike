use serde::Deserialize;
use super::{RenderableData, MapMarkerData};

#[derive(Deserialize, Debug)]
pub struct ChestData {
    pub name: String,
    pub renderable: Option<RenderableData>,
    pub loot_table: Option<String>,
    pub gold: Option<String>,
    pub capacity: i32,
    pub map_marker: Option<MapMarkerData>
}
