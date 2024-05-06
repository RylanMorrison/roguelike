use super::{AreaStartingPosition, BuilderChain, CellularAutomataBuilder, CullUnreachable, DistantExit, RoomBasedSpawner, RoomDrawer, SimpleMapBuilder, VoronoiSpawning, XStart, YStart};

pub fn dark_forest_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new("The Dark Forest", new_depth, width, height);
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(SimpleMapBuilder::new( 8, 12 ));
    chain.with(RoomDrawer::new());
    chain.with(CullUnreachable::new());
    chain.with(VoronoiSpawning::new());
    chain.with(AreaStartingPosition::new( XStart::LEFT, YStart::CENTER ));
    chain.with(DistantExit::new());
    chain
}
