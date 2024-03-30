use super::{BuilderChain, InitialMapBuilder, BuilderMap, TileType, Position};
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

enum BuildingTag {
    Pub, 
    Temple, 
    Library,
    Alchemist,
    Armoursmith,
    Weaponsmith,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<TownBuilder> {
        Box::new(TownBuilder{})
    }

    pub fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);

        let (mut available_building_tiles, mut roads, wall_gap_y) = self.town_foundations(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &mut roads, &doors);

        for y in wall_gap_y - 3..wall_gap_y + 4 {
            let exit_idx = build_data.map.xy_idx(build_data.width-2, y);
            build_data.map.tiles[exit_idx] = TileType::DownStairs;
        }

        let building_index = self.sort_buildings(&buildings);
        self.building_factory(rng, build_data, &buildings, &building_index);

        self.spawn_dockers(build_data, rng);
        self.spawn_townsfolk(build_data, rng, &mut available_building_tiles);

        for t in build_data.map.visible_tiles.iter_mut() {
            *t = true;
        }
        build_data.take_snapshot();
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        for t in build_data.map.tiles.iter_mut() {
            *t = TileType::Grass;
        }
        build_data.take_snapshot();
    }

    fn water_and_piers(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535f32; // random float between 0.0 and 1.0
        let mut water_width: Vec<i32> = Vec::new(); // # of water tiles on each row
        for y in 0..build_data.height {
            let n_water = (f32::sin(n) * 10.0) as i32 // smooth out the number
                + 14 // max 24 tiles of water
                + rng.roll_dice(1, 6); // slight randomness to make it look more natural
            water_width.push(n_water);
            n += 0.1; // move along the sine curve
            for x in 0..n_water {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::DeepWater;
            }
            for x in n_water..n_water+3 {
                // shallow water around the edges
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();

        // piers
        for _ in 0..rng.roll_dice(1, 4)+6 { // 8 to 11 piers
            let y = rng.roll_dice(1, build_data.height)-1;
            for x in 2 + rng.roll_dice(1, 6)..water_width[y as usize] + 4 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::Bridge;
            }
        }
        build_data.take_snapshot();
    }

    fn town_foundations(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) -> (HashSet<usize>, Vec<usize>, i32) {
        let mut available_building_tiles: HashSet<usize> = HashSet::new();
        let mut roads: Vec<usize> = Vec::new();
        let wall_gap_y = rng.roll_dice(1, build_data.height - 9) + 5; // random road location
        for y in 1..build_data.height-2 {
            if !(y > wall_gap_y-4 && y < wall_gap_y+4) {
                let idx = build_data.map.xy_idx(30, y);
                build_data.map.tiles[idx] = TileType::Wall;
                build_data.map.tiles[idx-1] = TileType::Grass;
                let idx_right = build_data.map.xy_idx(build_data.width - 2, y);
                build_data.map.tiles[idx_right] = TileType::Wall;
                for x in 31..build_data.width-2 {
                    let gravel_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[gravel_idx] = TileType::Gravel;
                    if y > 2 && y < build_data.height-1 {
                        available_building_tiles.insert(gravel_idx);
                    }
                }
            } else {
                for x in 30..build_data.width {
                    let road_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[road_idx] = TileType::Road;
                    roads.push(road_idx);
                }
            }
        }
        build_data.take_snapshot();

        for x in 30..build_data.width-1 {
            let idx_top = build_data.map.xy_idx(x, 1);
            build_data.map.tiles[idx_top] = TileType::Wall;
            let idx_bot = build_data.map.xy_idx(x, build_data.height-2);
            build_data.map.tiles[idx_bot] = TileType::Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, roads, wall_gap_y)
    }

    fn buildings(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap, available_building_tiles: &mut HashSet<usize>) -> Vec<(i32, i32, i32, i32)> {
        let mut buildings: Vec<(i32, i32, i32, i32)> = Vec::new();
        let mut n_buildings = 0;
        while n_buildings < 12 {
            let bx = rng.roll_dice(1, build_data.map.width - 32) + 30;
            let by = rng.roll_dice(1, build_data.map.height) - 2;
            let bw = rng.roll_dice(1, 8) + 4;
            let bh = rng.roll_dice(1, 8) + 4;
            let mut possible = true;
            for y in by..by+bh {
                for x in bx..bx+bw {
                    if x < 0 || x > build_data.width-1
                    || y < 0 || y > build_data.height-1 {
                        possible = false;
                    } else {
                        let idx = build_data.map.xy_idx(x, y);
                        if !available_building_tiles.contains(&idx) { possible = false; }
                    }
                }
            }
            if possible {
                n_buildings += 1;
                buildings.push((bx, by, bw, bh));
                for y in by..by+bh {
                    for x in bx..bx+bw {
                        let idx = build_data.map.xy_idx(x, y);
                        build_data.map.tiles[idx] = TileType::WoodFloor;
                        available_building_tiles.remove(&idx);
                        available_building_tiles.remove(&(idx+1));
                        available_building_tiles.remove(&(idx+build_data.width as usize));
                        available_building_tiles.remove(&(idx-1));
                        available_building_tiles.remove(&(idx-build_data.width as usize));
                    }
                }
                build_data.take_snapshot();
            }
        }

        // outline buildings
        let mut mapclone = build_data.map.clone();
        for y in 2..build_data.height-2 {
            for x in 32..build_data.width-2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor {
                    let mut neighbours = 0;
                    if build_data.map.tiles[idx - 1] != TileType::WoodFloor { neighbours += 1; }
                    if build_data.map.tiles[idx + 1] != TileType::WoodFloor { neighbours += 1; }
                    if build_data.map.tiles[idx - build_data.width as usize] != TileType::WoodFloor { neighbours += 1; }
                    if build_data.map.tiles[idx + build_data.width as usize] != TileType::WoodFloor { neighbours += 1; }
                    if neighbours > 0 {
                        mapclone.tiles[idx] = TileType::Wall;
                    }
                }
            }
        }
        build_data.map = mapclone;
        build_data.take_snapshot();
        buildings
    }

    fn sort_buildings(&mut self, buildings: &[(i32, i32, i32, i32)]) -> Vec<(usize, i32, BuildingTag)> {
        let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i, building) in buildings.iter().enumerate() {
            building_size.push((
                i,
                building.2 * building.3,
                BuildingTag::Unassigned
            ));
        }
        building_size.sort_by(|a,b| b.1.cmp(&a.1));
        building_size[0].2 = BuildingTag::Pub;
        building_size[1].2 = BuildingTag::Temple;
        building_size[2].2 = BuildingTag::Library;
        building_size[3].2 = BuildingTag::Alchemist;
        building_size[4].2 = BuildingTag::Weaponsmith;
        building_size[5].2 = BuildingTag::Armoursmith;
        building_size[6].2 = BuildingTag::PlayerHouse;
        for b in building_size.iter_mut().skip(7) {
            b.2 = BuildingTag::Hovel;
        }
        let last_index = building_size.len()-1;
        building_size[last_index].2 = BuildingTag::Abandoned;
        building_size
    }

    fn building_factory(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap, buildings: &[(i32, i32, i32, i32)], building_index: &[(usize, i32, BuildingTag)]) {
        for (i, building) in buildings.iter().enumerate() {
            let build_type = &building_index[i].2;
            match build_type {
                BuildingTag::Pub => self.build_pub(&building, build_data, rng),
                BuildingTag::Temple => self.build_temple(&building, build_data, rng),
                BuildingTag::Library => self.build_library(&building, build_data, rng),
                BuildingTag::Alchemist => self.build_alchemist(&building, build_data, rng),
                BuildingTag::Weaponsmith => self.build_weaponsmith(&building, build_data, rng),
                BuildingTag::Armoursmith => self.build_armoursmith(&building, build_data, rng),
                BuildingTag::Abandoned => self.build_abandoned_house(&building, build_data, rng),
                _ => {}
            }
        }
    }

    fn build_pub(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        // place the player
        build_data.starting_position = Some(Position {
            x: building.0 + (building.2 / 2),
            y: building.1 + (building.3 / 2)
        });
        let player_idx = build_data.map.xy_idx(
            building.0 + (building.2 / 2),
            building.1 + (building.3 / 2)
        );

        // place other entities
        let mut to_place: Vec<&str> = vec!["Barkeep", "Shady Salesman", "Patron", "Patron", 
            "Keg", "Table", "Chair", "Table", "Chair"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, player_idx);
    }

    fn build_temple(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        let mut to_place: Vec<&str> = vec!["Priest", "Parishioner", "Parishioner", "Chair",
            "Chair", "Candle", "Candle", "Altar"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_library(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        let mut to_place: Vec<&str> = vec!["Librarian", "Bookcase", "Bookcase", "Bookcase",
            "Table", "Table", "Table", "Chair", "Chair", "Chair", "Chair", "Patron", "Patron"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_alchemist(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        let mut to_place: Vec<&str> = vec!["Alchemist", "Cauldron", "Potion Rack", "Potion Rack", 
            "Table", "Table", "Patron", "Patron"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_abandoned_house(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        let mut to_place: Vec<&str> = Vec::new();
        let n_rats = rng.roll_dice(1, (building.2*building.3) / 2) + 1;
        // fill around half of the house with rats
        for _ in 0..n_rats {
            to_place.push("Rat");
        }
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_weaponsmith(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        let mut to_place: Vec<&str> = vec!["Weaponsmith", "Table", "Chair"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_armoursmith(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        let mut to_place: Vec<&str> = vec!["Armoursmith", "Table", "Chair"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn random_building_spawn(&mut self, building: &(i32, i32, i32, i32), build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator, to_place: &mut Vec<&str>, player_idx: usize) {
        if to_place.len() < 1 { return; }
        
        let mut available_tiles: Vec<usize> = Vec::new();
        for y in building.1 .. building.1 + building.3 {
            for x in building.0 .. building.0 + building.2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor && idx != player_idx {
                    available_tiles.push(idx);
                }
            }
        }

        // randomly shuffle 
        let count = available_tiles.len().clone() as i32;
        for i in 0..count-1 {
            let random_tile = rng.roll_dice(1, count -1) as usize;
            available_tiles.swap(i as usize, random_tile);
        }

        for tag in to_place.iter() {
            if available_tiles.len() < 1 { return; }
            
            build_data.spawn_list.push((available_tiles.remove(0), tag.to_string()));
        }
    }

    fn add_doors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap, buildings: &mut Vec<(i32, i32, i32, i32)>, wall_gap_y: i32) -> Vec<usize> {
        let mut doors = Vec::new();
        for building in buildings.iter() {
            let door_x = building.0 + 1 + rng.roll_dice(1, building.2 - 3);
            let cy = building.1 + (building.3 / 2);
            let idx = if cy > wall_gap_y {
                // north door
                build_data.map.xy_idx(door_x, building.1)
            } else {
                build_data.map.xy_idx(door_x, building.1 + building.3 - 1)
            };
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Door".to_string()));
            doors.push(idx);
        }
        build_data.take_snapshot();
        doors
    }

    fn add_paths(&mut self, build_data: &mut BuilderMap, roads: &mut Vec<usize>, doors: &[usize]) {
        build_data.map.populate_blocked();
        for door_idx in doors.iter() {
            let mut nearest_roads: Vec<(usize, f32)> = Vec::new();
            let door_pt = rltk::Point::new(*door_idx as i32 % build_data.map.width as i32, *door_idx as i32 / build_data.map.width as i32);
            for r in roads.iter() {
                nearest_roads.push((
                    *r,
                    rltk::DistanceAlg::PythagorasSquared.distance2d(
                        door_pt,
                        rltk::Point::new(*r as i32 % build_data.map.width, *r as i32 / build_data.map.width)
                    )
                ));
            }
            nearest_roads.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

            let destination = nearest_roads[0].0;
            let path = rltk::a_star_search(*door_idx, destination, &mut build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let idx = *step as usize;
                    build_data.map.tiles[idx] = TileType::Road;
                    roads.push(idx);
                }
            }
            build_data.take_snapshot();
        }
    }

    fn spawn_dockers(&mut self, build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        for (idx, tt) in build_data.map.tiles.iter().enumerate() {
            if *tt == TileType::Bridge && rng.roll_dice(1, 6) == 6 {
                let roll = rng.roll_dice(1, 3);
                match roll {
                    1 => build_data.spawn_list.push((idx, "Dock Worker".to_string())),
                    2 => build_data.spawn_list.push((idx, "Wannabe Pirate".to_string())),
                    _ => build_data.spawn_list.push((idx, "Fisher".to_string()))
                }
            }
        }
    }

    fn spawn_townsfolk(&mut self, build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator, available_building_tiles: &mut HashSet<usize>) {
        for idx in available_building_tiles.iter() {
            if rng.roll_dice(1, 30) == 1 {
                let roll = rng.roll_dice(1, 4);
                match roll {
                    1 => build_data.spawn_list.push((*idx, "Peasant".to_string())),
                    2 => build_data.spawn_list.push((*idx, "Drunk".to_string())),
                    3 => build_data.spawn_list.push((*idx, "Dock Worker".to_string())),
                    _ => build_data.spawn_list.push((*idx, "Fisher".to_string()))
                }
            }
        }
    }
}

pub fn town_builder(new_depth: i32, _rng: &mut RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new("Landfall", new_depth, width, height);
    chain.start_with(TownBuilder::new());
    chain
}
