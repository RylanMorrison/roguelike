use super::{MetaMapBuilder, BuilderMap};

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomBasedStairs {
    pub fn new() -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs{})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            // place the exit in the last created room
            let stairs_position = rooms[rooms.len()-1].center();
            let stairs_idx = build_data.map.xy_idx(stairs_position.0, stairs_position.1);
            build_data.add_next_exit(stairs_idx);
        } else {
            panic!("Room Based Stairs only works after rooms have been created!");
        }
    }
}
