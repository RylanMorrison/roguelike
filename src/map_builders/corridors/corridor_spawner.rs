use super::{MetaMapBuilder, BuilderMap, spawner};

pub struct CorridorSpawner {}

impl MetaMapBuilder for CorridorSpawner {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl CorridorSpawner {
    pub fn new() -> Box<CorridorSpawner> {
        Box::new(CorridorSpawner{})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        if let Some(corridors) = &build_data.corridors {
            for c in corridors.iter() {
                // only consider long corridors for spawning
                if c.len() > 6 {
                    spawner::spawn_region(&mut build_data.map, &c);
                }
            }
        } else {
            panic!("Corridor Based Spawning only works after corridors have been created!");
        }
    }
}
