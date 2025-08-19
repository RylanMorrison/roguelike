use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SpawnTableEntry {
    pub name: String,
    pub weights: HashMap<String, i32>
}
