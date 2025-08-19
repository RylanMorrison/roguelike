use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone, Debug)]
pub enum TileType {
    Wall,
    Floor,
    NextArea { map_name: String },
    PreviousArea { map_name: String },
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    TownWall,
    Placeholder
}

pub fn tile_walkable(tt: &TileType) -> bool {
    matches!(tt,
        TileType::Floor |
        TileType::NextArea{..} |
        TileType::PreviousArea{..} |
        TileType::Road |
        TileType::Grass |
        TileType::ShallowWater |
        TileType::WoodFloor |
        TileType::Bridge |
        TileType::Gravel
    )
}

pub fn tile_opaque(tt: &TileType) -> bool {
    match tt {
        TileType::Wall | TileType::TownWall => true,
        _ => false
    }
}

pub fn tile_cost(tt: &TileType) -> f32 {
    // cost checking only makes sense for walkable tiles
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::Gravel => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0
    }
}
