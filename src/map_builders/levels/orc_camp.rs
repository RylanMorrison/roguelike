use crate::raws::MapData;

use super::{area_starting_points::AreaStartingPosition, prefabs, BuilderChain, CellularAutomataBuilder,
    CullUnreachable, PrefabBuilder, VoronoiSpawning, XStart, YStart};

pub fn orc_camp_builder(map_data: &MapData) -> BuilderChain {
    let mut chain = BuilderChain::new(map_data);
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(VoronoiSpawning::new());
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER, false));
    chain.with(PrefabBuilder::sectional(prefabs::prefab_sections::ORC_CAMP));
    chain
}
