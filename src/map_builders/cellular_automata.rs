use super::{BuilderMap, InitialMapBuilder, MetaMapBuilder, TileType};
use crate::rng;

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap)  {
        self.build(build_data);
    }
}

impl MetaMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.apply_iteration(build_data);
    }
}

impl CellularAutomataBuilder {
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder{})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        // turn roughly 55% of the map into floor and the rest into wall
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let roll = rng::roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                if roll > 55 {
                    build_data.map.tiles[idx] = TileType::Floor;
                } else {
                    build_data.map.tiles[idx] = TileType::Wall;
                }
            }
        }
        build_data.take_snapshot();

        // iteratively apply cellular automata rules
        for _i in 0..15 {
            self.apply_iteration(build_data);
        }
    }

    fn apply_iteration(&mut self, build_data: &mut BuilderMap) {
        let mut newtiles = build_data.map.tiles.clone();

        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let idx = build_data.map.xy_idx(x, y);
                let mut neighbours = 0;
                if build_data.map.tiles[idx - 1] == TileType::Wall { neighbours += 1; }
                if build_data.map.tiles[idx + 1] == TileType::Wall { neighbours += 1; }
                if build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall { neighbours += 1; }
                if build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall { neighbours += 1; }
                if build_data.map.tiles[idx - (build_data.map.width as usize - 1)] == TileType::Wall { neighbours += 1; }
                if build_data.map.tiles[idx - (build_data.map.width as usize + 1)] == TileType::Wall { neighbours += 1; }
                if build_data.map.tiles[idx + (build_data.map.width as usize - 1)] == TileType::Wall { neighbours += 1; }
                if build_data.map.tiles[idx + (build_data.map.width as usize + 1)] == TileType::Wall { neighbours += 1; }

                if neighbours > 4 || neighbours == 0 {
                    newtiles[idx] = TileType::Wall;
                } else {
                    newtiles[idx] = TileType::Floor;
                }
            }
        }
        build_data.map.tiles = newtiles.clone();
        build_data.take_snapshot();
    }
}
