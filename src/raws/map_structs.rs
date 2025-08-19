use std::collections::VecDeque;

use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct MapData {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub area_level: i32,
    #[serde(default)]
    pub start: bool,
    #[serde(default)]
    pub town: bool,
    #[serde(default)]
    pub indoors: bool,
    pub prev_maps: Option<VecDeque<String>>,
    pub next_maps: Option<VecDeque<String>>
}
