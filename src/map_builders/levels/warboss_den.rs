use super::{prefab_builder, AreaStartingPosition, BuilderChain, CullUnreachable, DrunkardsWalkBuilder, PrefabBuilder, VoronoiCellBuilder, VoronoiSpawning, XStart, YStart};

pub fn warboss_den_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new("Warboss Den", new_depth, width, height);
    chain.start_with(VoronoiCellBuilder::manhattan());
    chain.with(VoronoiSpawning::new());
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::BOTTOM));
    chain.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::WARBOSS_DEN));
    chain
}
