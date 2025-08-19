use super::{BuilderChain, CellularAutomataBuilder, XStart, YStart, AreaStartingPosition,
    CullUnreachable, VoronoiSpawning, MetaMapBuilder, BuilderMap, TileType};
use crate::map::tile_walkable;
use crate::raws::MapData;
use crate::rng;

pub fn forest_builder(map_data: &MapData) -> BuilderChain {
    let mut chain = BuilderChain::new(map_data);
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER, false));
    chain.with(VoronoiSpawning::new());
    chain.with(BrickRoad::new());
    chain
}
pub struct BrickRoad {}

impl MetaMapBuilder for BrickRoad {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl BrickRoad {
    pub fn new() -> Box<BrickRoad> {
        Box::new(BrickRoad{})
    }

    fn find_exit(&self, build_data: &mut BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if tile_walkable(tiletype) {
                available_floors.push((
                    idx,
                    rltk::DistanceAlg::PythagorasSquared.distance2d(
                        rltk::Point::new(idx as i32 % build_data.map.width, idx as i32 / build_data.map.width),
                        rltk::Point::new(seed_x, seed_y)
                    )
                ));
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
        let end_x = available_floors[0].0 as i32 % build_data.map.width;
        let end_y = available_floors[0].0 as i32 / build_data.map.width;
        (end_x, end_y)
    }

    fn paint_road(&self, build_data: &mut BuilderMap, x: i32, y: i32) {
        if x < 1 || x > build_data.map.width - 2
        || y < 1 || y > build_data.map.height - 2 {
            return;
        }
        let idx = build_data.map.xy_idx(x, y);
        if !matches!(build_data.map.tiles[idx], TileType::NextArea{..} | TileType::PreviousArea{..}) {
            build_data.map.tiles[idx] = TileType::Road;
        }
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let starting_pos = build_data.map.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y);

        // exit on the west side of the map
        let (end_x, end_y) = self.find_exit(build_data, build_data.map.width - 2, build_data.map.height / 2);
        let end_idx = build_data.map.xy_idx(end_x, end_y);

        build_data.map.populate_blocked();

        // find a path for the road through the forest
        let path = rltk::a_star_search(start_idx, end_idx, &mut build_data.map);
        if !path.success {
            panic!("No valid path for the road!");
        }
        // build the road
        for idx in path.steps.iter() {
            let x = *idx as i32 % build_data.map.width;
            let y = *idx as i32 / build_data.map.width;
            self.paint_road(build_data, x, y);
            self.paint_road(build_data, x-1, y);
            self.paint_road(build_data, x+1, y);
            self.paint_road(build_data, x, y-1);
            self.paint_road(build_data, x, y+1);
        }

        // pick between north-west and south west for the exit
        let exit_dir = rng::roll_dice(1, 2);
        let (seed_x, seed_y, stream_start_x, stream_start_y) = if exit_dir == 1 {
            (build_data.map.width - 1, 1, 0, build_data.map.height - 1)
        } else {
            (build_data.map.width - 1, build_data.map.height - 1, 1, build_data.map.height -1)
        };

        let (stairs_x, stairs_y) = self.find_exit(build_data, seed_x, seed_y);
        let stairs_idx = build_data.map.xy_idx(stairs_x, stairs_y);

        // build a stream towards the exit
        let (stream_x, stream_y) = self.find_exit(build_data, stream_start_x, stream_start_y);
        let stream_idx = build_data.map.xy_idx(stream_x, stream_y) as usize;
        let stream = rltk::a_star_search(stairs_idx, stream_idx, &mut build_data.map);
        for tile in stream.steps.iter() {
            if build_data.map.tiles[*tile as usize] == TileType::Floor {
                build_data.map.tiles[*tile as usize] = TileType::ShallowWater;
            }
        }
        build_data.add_next_exit(stairs_idx);
    }
}


