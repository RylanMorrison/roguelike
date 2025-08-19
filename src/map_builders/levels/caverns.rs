use super::{BuilderChain, DrunkardsWalkBuilder, XStart, YStart, AreaStartingPosition,
    CullUnreachable, VoronoiSpawning, MetaMapBuilder, BuilderMap, TileType, DistantExit};
use crate::{raws::MapData, rng};

pub fn caverns_builder(map_data: &MapData) -> BuilderChain {
    let mut chain = BuilderChain::new(map_data);
    chain.start_with(DrunkardsWalkBuilder::winding_passages());
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER, false));
    chain.with(VoronoiSpawning::new());
    // chain.with(DistantExit::new());
    chain.with(CavernDecorator::new());
    chain
}

pub struct CavernDecorator {}

impl MetaMapBuilder for CavernDecorator {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl CavernDecorator {
    pub fn new() -> Box<CavernDecorator> {
        Box::new(CavernDecorator{})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let old_map = build_data.map.clone();
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            if *tt == TileType::Floor && rng::roll_dice(1, 6) == 1 {
                // convert some floor into gravel
                *tt = TileType::Gravel;
            } else if *tt == TileType::Floor && rng::roll_dice(1, 10) == 1 {
                // convert some floors into shallow pools
                *tt = TileType::ShallowWater;
            } else if *tt == TileType::Wall {
                // convert some walls into deep pools
                let mut neighbours = 0;
                let x = idx as i32 % old_map.width;
                let y = idx as i32 / old_map.width;
                if x > 0 && old_map.tiles[idx-1] == TileType::Wall { neighbours += 1; }
                if x < old_map.width - 2 && old_map.tiles[idx+1] == TileType::Wall { neighbours += 1; }
                if y > 0 && old_map.tiles[idx-old_map.width as usize] == TileType::Wall { neighbours += 1; }
                if y < old_map.height - 2 && old_map.tiles[idx+old_map.width as usize] == TileType::Wall { neighbours += 1; }
                if neighbours == 2 {
                    *tt = TileType::DeepWater;
                }
            }
        }
    }
}


