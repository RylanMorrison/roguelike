use super::{InitialMapBuilder, BuilderMap, Rect};
use rltk::RandomNumberGenerator;

pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder{})
    }

    pub fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;
        let mut rooms: Vec<Rect> = Vec::new();
    
        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height as i32 - h - 1) - 1;
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
