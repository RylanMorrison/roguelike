use super::{MapBuilder, Map, TileType, Position, SHOW_MAPGEN_VISUALIZER,
    remove_unreachable_areas_returning_most_distant, RandomNumberGenerator,
    Rect};
pub mod prefab_levels;
pub mod prefab_sections;
pub mod prefab_rooms;
use prefab_sections::*;
use prefab_rooms::*;
use std::collections::HashSet;
use rltk::console;

#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    Constant{ level: prefab_levels::PrefabLevel },
    Sectional{ section: prefab_sections::PrefabSection },
    RoomVaults
}

pub struct PrefabBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawn_list: Vec<(usize, String)>,
    previous_builder: Option<Box<dyn MapBuilder>>
}

impl MapBuilder for PrefabBuilder {
    fn build_map(&mut self)  {
        self.build();
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl PrefabBuilder {
    pub fn new(new_depth: i32) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new(),
            mode: PrefabMode::Constant{ level: prefab_levels::WFC_POPULATED },
            spawn_list: Vec::new(),
            previous_builder: None
        }
    }

    pub fn new_section(new_depth: i32, section: prefab_sections::PrefabSection, previous_builder: Box<dyn MapBuilder>) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new(),
            mode: PrefabMode::Sectional{section},
            spawn_list: Vec::new(),
            previous_builder: Some(previous_builder)
        }
    }

    pub fn new_room(new_depth: i32, previous_builder: Box<dyn MapBuilder>) -> PrefabBuilder {
        PrefabBuilder {
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new(),
            mode: PrefabMode::RoomVaults,
            spawn_list: Vec::new(),
            previous_builder: Some(previous_builder)
        }
    }
 
    fn build(&mut self) {
        match self.mode {
            PrefabMode::Constant{level} => self.load_ascii_map(&level),
            PrefabMode::Sectional{section} => self.apply_sectional(&section),
            PrefabMode::RoomVaults => self.apply_room_vaults()
        }
        self.take_snapshot();

        // Find a starting point; start at the middle and walk left until we find an open tile
        let mut start_idx;
        if self.starting_position.x == 0 {
            self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();
        }

        let mut has_exit = false;
        for t in self.map.tiles.iter() {
            if *t == TileType::DownStairs { has_exit = true; }
        }

        if !has_exit {
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);

            // Find all tiles we can reach from the starting point
            let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }
    }

    fn char_to_map(&mut self, ch: char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                self.starting_position = Position{ x: x as i32, y: y as i32 };
            }
            '>' => self.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Orc".to_string()));
            }
            'O' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Ogre".to_string()));
            }
            'D' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Demon".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Food Ration".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Health Potion".to_string()));
            }
            '/' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, self.random_melee_weapon()));
            }
            '0' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, self.random_shield()));
            }
            _ => {
                rltk::console::log(format!("Unknown glyph loading map: {}", ch));
            }
        }
    }

    // convert to a vector with newlines removed
    fn read_ascii_to_vec(template: &str) -> Vec<char> {
        let mut string_vec: Vec<char> = template.chars().filter(|a| *a != '\r' && *a != '\n').collect();
        // spaces are being read as character 160 instead of 32 so convert them
        for c in string_vec.iter_mut() { if *c as u8 == 160u8 { *c = ' '; } }
        string_vec
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);
        
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i +=1 ;
            }
        }
    }

    fn apply_sectional(&mut self, section: &PrefabSection) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);

        // place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (self.map.width - 1) - section.width as i32
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (self.map.height - 1) - section.height as i32
        }

        // build the map
        self.apply_previous_iteration(|x,y,e| {
            x < chunk_x || x > (chunk_x + section.width as i32) ||
            y < chunk_y || y > (chunk_y + section.height as i32)
        });
        self.take_snapshot();

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }

    fn apply_previous_iteration<F>(&mut self, mut filter: F)
        where F: FnMut(i32, i32, &(usize, String)) -> bool
    {
        // build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();
        for e in prev_builder.get_spawn_list().iter() {
            let idx = e.0;
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            if filter(x, y, e) {
                self.spawn_list.push((idx, e.1.to_string()));
            }
        }
        self.take_snapshot();
    }

    fn apply_room_vaults(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // apply the previous builder and keep all entities it spawns
        self.apply_previous_iteration(|_x,_y,_e| true);

        // chance of encountering room vault dependent on depth
        let vault_roll = rng.roll_dice(1, 6) + self.depth;
        if vault_roll < 4 { return; }

        let master_vault_list = vec![GUARDED_WEAPON, GUARDED_SHIELD, OGRE_TRIO];

        // filter the vault list down to ones that are applicable to the current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| { self.depth >= v.first_depth && self.depth <= v.last_depth })
            .collect();

        if possible_vaults.is_empty() { return; }

        let n_vaults = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles: HashSet<usize> = HashSet::new();

        for _i in 0..n_vaults {
            let vault_index = if possible_vaults.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, possible_vaults.len() as i32)-1) as usize
            };
            let vault = possible_vaults[vault_index];

            // find all possible places where the vault could fit
            let mut vault_positions: Vec<Position> = Vec::new();
            let mut idx = 0usize;
            loop {
                let x = (idx % self.map.width as usize) as i32;
                let y = (idx / self.map.width as usize) as i32;

                // check for map overflow
                if x > 1
                    && (x + vault.width as i32) < self.map.width - 2
                    && y > 1
                    && (y + vault.height as i32) < self.map.height - 2
                {
                    let mut possible = true;
                    for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {
                            idx = self.map.xy_idx(tx + x, ty + y);
                            if self.map.tiles[idx] != TileType::Floor {
                                possible = false;
                            }
                            if used_tiles.contains(&idx) {
                                possible = false;
                            }
                        }
                    }

                    if possible {
                        vault_positions.push(Position{x, y});
                        break;
                    }
                }
                idx += 1;
                if idx >= self.map.tiles.len() - 1 { break; }
            }

            console::log(vault_positions.is_empty());

            if !vault_positions.is_empty() {
                let pos_idx = if vault_positions.len() == 1 {
                    0
                } else {
                    // pick a random vault
                    (rng.roll_dice(1, vault_positions.len() as i32)-1) as usize
                };
                let pos = &vault_positions[pos_idx];

                let chunk_x = pos.x;
                let chunk_y = pos.y;

                // can't use `self` in `retain`
                let width = self.map.width;
                let height = self.map.height;
                self.spawn_list.retain(|e| {
                    let idx = e.0 as i32;
                    let x = idx % width;
                    let y = idx / height;
                    x < chunk_x || x > chunk_x + vault.width as i32 || y < chunk_y || y > chunk_y + vault.height as i32
                });

                // load the ascii and add it to the map
                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                        self.char_to_map(string_vec[i], idx);
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                self.take_snapshot();
                possible_vaults.remove(vault_index);
            }
        }
    }

    pub fn random_melee_weapon(&self) -> String {
        let mut rng = RandomNumberGenerator::new();
        let roll = rng.roll_dice(1, 3);
        match roll {
            1 => "Sword".to_string(),
            _ => "Dagger".to_string()
        }
    }

    pub fn random_shield(&self) -> String {
        let mut rng = RandomNumberGenerator::new();
        let roll = rng.roll_dice(1, 3);
        match roll {
            1 => "Tower Shield".to_string(),
            _ => "Shield".to_string()
        }
    }
}
