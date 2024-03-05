use super::{BuilderChain, DrunkardsWalkBuilder, XStart, YStart, AreaStartingPosition, 
    CullUnreachable, VoronoiSpawning, MetaMapBuilder, BuilderMap, TileType, DistantExit};
use rltk::RandomNumberGenerator;

pub fn limestone_cavern_builder(new_depth: i32, _rng: &mut RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new("Limestone Caverns", new_depth, width, height);
    chain.start_with(DrunkardsWalkBuilder::winding_passages());
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());
    chain
}

pub struct CaveDecorator {}

impl MetaMapBuilder for CaveDecorator {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveDecorator {
    pub fn new() -> Box<CaveDecorator> {
        Box::new(CaveDecorator{})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let old_map = build_data.map.clone();
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            if *tt == TileType::Floor && rng.roll_dice(1, 6) == 1 {
                // convert some floor into gravel
                *tt = TileType::Gravel;
            } else if *tt == TileType::Floor && rng.roll_dice(1, 10) == 1 {
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
        build_data.take_snapshot();
        build_data.map.outdoors = false;
    }
}


