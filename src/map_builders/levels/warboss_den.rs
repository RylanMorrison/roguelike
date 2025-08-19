use crate::raws::MapData;

use super::{prefabs, AreaStartingPosition, BuilderChain, CullUnreachable, PrefabBuilder, VoronoiCellBuilder, VoronoiSpawning, XStart, YStart};

pub fn warboss_den_builder(map_data: &MapData) -> BuilderChain {
    let mut chain = BuilderChain::new(map_data);
    chain.start_with(VoronoiCellBuilder::manhattan());
    chain.with(VoronoiSpawning::new());
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::BOTTOM, false));
    chain.with(PrefabBuilder::sectional(prefabs::prefab_sections::WARBOSS_DEN));
    chain
}
