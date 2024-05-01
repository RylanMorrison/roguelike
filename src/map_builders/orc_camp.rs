use super::{area_starting_points::AreaStartingPosition, prefab_builder, BuilderChain, CellularAutomataBuilder, 
    CullUnreachable, PrefabBuilder, VoronoiSpawning, XStart, YStart};

pub fn orc_camp_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new("Orc Camp", new_depth, width, height);
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(VoronoiSpawning::new());
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));
    chain.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::ORC_CAMP));
    chain
}
