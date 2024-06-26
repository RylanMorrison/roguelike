use super::{MetaMapBuilder, BuilderMap, Rect, draw_corridor};
use crate::rng;

pub struct BspCorridors {
    corridor_size: i32
}

impl MetaMapBuilder for BspCorridors {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.corridors(build_data);
    }
}

impl BspCorridors {
    pub fn new(corridor_size: i32) -> Box<BspCorridors> {
        Box::new(BspCorridors{ corridor_size })
    }

    fn corridors(&mut self, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("BSP Corridors requires a builder with rooms!");
        }

        let mut corridors: Vec<Vec<usize>> = Vec::new();
        for i in 0..rooms.len()-1 {
            let room = rooms[i];
            let next_room = rooms[i+1];
            let start_x = room.x1 + (rng::roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng::roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng::roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng::roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            let corridor = draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y, self.corridor_size);
            corridors.push(corridor);
            build_data.take_snapshot();
        }
        build_data.corridors = Some(corridors);
    }
}
