use super::{InitialMapBuilder, MetaMapBuilder, BuilderMap, TileType, Position, paint, Symmetry};
use crate::rng;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm { WalkInwards, WalkOutwards, CentralAttractor }

pub struct DLABuilder {
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32
}

impl InitialMapBuilder for DLABuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap)  {
        self.build(build_data);
    }
}

impl MetaMapBuilder for DLABuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl DLABuilder {
    pub fn new(algorithm: DLAAlgorithm, brush_size: i32, symmetry: Symmetry, floor_percent: f32) -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm,
            brush_size,
            symmetry,
            floor_percent,
        })
    }

    pub fn walk_inwards() -> Box<DLABuilder> {
        DLABuilder::new(
            DLAAlgorithm::WalkInwards,
            1,
            Symmetry::None,
            0.25
        )
    }

    pub fn walk_outwards() -> Box<DLABuilder> {
        DLABuilder::new(
            DLAAlgorithm::WalkOutwards,
            2,
            Symmetry::None,
            0.25
        )
    }

    pub fn central_attractor() -> Box<DLABuilder> {
        DLABuilder::new(
            DLAAlgorithm::CentralAttractor,
            2,
            Symmetry::None,
            0.25
        )
    }

    pub fn insectoid() -> Box<DLABuilder> {
        DLABuilder::new(
            DLAAlgorithm::CentralAttractor,
            1,
            Symmetry::Horizontal,
            0.35
        )
    }

    pub fn heavy_erosion() -> Box<DLABuilder> {
        DLABuilder::new(
            DLAAlgorithm::WalkInwards,
            2,
            Symmetry::None,
            0.35
        )
    }

    pub fn build(&mut self, build_data: &mut BuilderMap) {
        // create a starting seed
        let starting_position = Position{ x: build_data.map.width/2, y: build_data.map.height/2 };
        let start_idx = build_data.map.xy_idx(starting_position.x, starting_position.y);
        build_data.take_snapshot();

        build_data.map.tiles[start_idx] = TileType::Floor;
        build_data.map.tiles[start_idx-1] = TileType::Floor;
        build_data.map.tiles[start_idx+1] = TileType::Floor;
        build_data.map.tiles[start_idx - build_data.map.width as usize] = TileType::Floor;
        build_data.map.tiles[start_idx + build_data.map.width as usize] = TileType::Floor;

        // set up a random walker
        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        let mut i = 0;
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng::roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng::roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    while build_data.map.tiles[digger_idx] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_direction = rng::roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } },
                            2 => { if digger_x < build_data.map.width-2 { digger_x += 1; } },
                            3 => { if digger_y > 2 { digger_y -= 1; } },
                            _ => { if digger_y < build_data.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, prev_x, prev_y);
                }
                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = starting_position.x;
                    let mut digger_y = starting_position.y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    while build_data.map.tiles[digger_idx] == TileType::Floor {
                        let stagger_direction = rng::roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < build_data.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -= 1; } }
                            _ => { if digger_y < build_data.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, digger_x, digger_y);
                }
                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng::roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng::roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);

                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham,
                        rltk::Point::new(digger_x, digger_y),
                        rltk::Point::new(starting_position.x, starting_position.y)
                    );

                    while build_data.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, prev_x, prev_y);
                }
            }
            floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
            if i % 50 == 0 {
                build_data.take_snapshot();
            }
            i += 1;
        }
    }
}
