use serde::{Serialize, Deserialize};

const WALKABLE_TILES: [TileType; 9]  = [
    TileType::Floor,
    TileType::DownStairs,
    TileType::UpStairs,
    TileType::Road,
    TileType::Grass,
    TileType::ShallowWater,
    TileType::WoodFloor,
    TileType::Bridge,
    TileType::Gravel
];

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    UpStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    TownWall
}

pub fn tile_walkable(tt: TileType) -> bool {
    if WALKABLE_TILES.contains(&tt) { return true; }
    false
}

pub fn tile_opaque(tt: TileType) -> bool {
    match tt {
        TileType::Wall => true,
        _ => false
    }
}

pub fn tile_cost(tt: TileType) -> f32 {
    // cost checking only makes sense for walkable tiles
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::Gravel => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0
    }
}

