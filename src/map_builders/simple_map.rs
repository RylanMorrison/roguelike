use super::{InitialMapBuilder, MetaMapBuilder, BuilderMap, Rect};
use crate::rng;

pub struct SimpleMapBuilder {
    min_room_size: i32,
    max_room_size: i32
}

impl InitialMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build_rooms(build_data);
    }
}

impl MetaMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build_rooms(build_data);
    }
}

impl SimpleMapBuilder {
    pub fn new(min_room_size: i32, max_room_size: i32) -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder{ min_room_size, max_room_size })
    }

    pub fn build_rooms(&mut self, build_data: &mut BuilderMap) {
        const MAX_ROOMS : i32 = 30;
        let mut rooms: Vec<Rect> = Vec::new();
    
        for _i in 0..MAX_ROOMS {
            let w = rng::range(self.min_room_size, self.max_room_size);
            let h = rng::range(self.min_room_size, self.max_room_size);
            let x = rng::roll_dice(1, build_data.map.width as i32 - w - 1) - 1;
            let y = rng::roll_dice(1, build_data.map.height as i32 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut no_intersect = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) { no_intersect = false }
            }
            if no_intersect {
                rooms.push(new_room);
            }
        }
        build_data.rooms = Some(rooms);
    }
}
