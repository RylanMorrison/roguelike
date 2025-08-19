use super::{MetaMapBuilder, BuilderMap, Position, TileType};

pub enum XStart { LEFT, CENTER, RIGHT }

pub enum YStart { TOP, CENTER, BOTTOM }

pub struct AreaStartingPosition {
    x: XStart,
    y: YStart,
    temporary: bool
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl AreaStartingPosition {
    pub fn new(x: XStart, y: YStart, temporary: bool) -> Box<AreaStartingPosition> {
        Box::new(AreaStartingPosition{ x, y, temporary })
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let seed_x;
        let seed_y;

        match self.x {
            XStart::LEFT => seed_x = 1,
            XStart::CENTER => seed_x = build_data.map.width / 2,
            XStart::RIGHT => seed_x = build_data.map.width - 2
        }

        match self.y {
            YStart::TOP => seed_y = 1,
            YStart::CENTER => seed_y = build_data.map.height / 2,
            YStart::BOTTOM => seed_y = build_data.map.height - 2
        }

        // collect all available floor tiles and calculate the distance to the preferred starting point
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if *tiletype == TileType::Floor || *tiletype == TileType::WoodFloor || *tiletype == TileType::Grass {
                available_floors.push(
                    (
                        idx,
                        rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(idx as i32 % build_data.map.width, idx as i32 / build_data.map.width),
                            rltk::Point::new(seed_x, seed_y)
                        )
                    )
                );
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        // set the starting position to the closest
        available_floors.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
        let start_x = available_floors[0].0 as i32 % build_data.map.width;
        let start_y = available_floors[0].0 as i32 / build_data.map.width;

        build_data.map.starting_position = Some(Position{ x: start_x, y: start_y });
        if !self.temporary {
            let start_idx = build_data.map.xy_idx(start_x, start_y);
            build_data.add_next_entrance(start_idx);
        }
    }
}
