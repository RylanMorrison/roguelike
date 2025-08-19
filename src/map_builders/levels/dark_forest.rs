use crate::raws::MapData;

use super::{prefabs, PrefabBuilder, AreaStartingPosition, BuilderChain, CellularAutomataBuilder, CullUnreachable, DistantExit, RoomDrawer, SimpleMapBuilder, VoronoiSpawning, XStart, YStart};

pub fn dark_forest_builder(map_data: &MapData) -> BuilderChain {
    let mut chain = BuilderChain::new(map_data);
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(SimpleMapBuilder::new( 8, 12 ));
    chain.with(RoomDrawer::new());
    chain.with(CullUnreachable::new());
    chain.with(VoronoiSpawning::new());
    chain.with(AreaStartingPosition::new( XStart::LEFT, YStart::CENTER, false));
    chain.with(PrefabBuilder::sectional(prefabs::prefab_sections::WOLF_DEN));
    chain.with(DistantExit::new());
    chain
}
