use super::{MetaMapBuilder, BuilderMap, spawner};

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomBasedSpawner {
    pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner{})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(&mut build_data.map, room);
            }
        } else {
            panic!("Room Based Spawning only works after rooms have been created!");
        }
    }
}
