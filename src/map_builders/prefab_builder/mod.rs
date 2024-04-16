use super::{InitialMapBuilder, MetaMapBuilder, BuilderMap, TileType, Position};
use rltk::RandomNumberGenerator;
pub mod prefab_levels;
pub mod prefab_sections;
pub mod prefab_rooms;
use prefab_levels::*;
use prefab_sections::*;
use prefab_rooms::*;
use std::collections::HashSet;

#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    Constant{ level: prefab_levels::PrefabLevel },
    Sectional{ section: prefab_sections::PrefabSection },
    RoomVaults
}

pub struct PrefabBuilder {
    mode: PrefabMode
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    pub fn new(mode: PrefabMode) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode,
        })
    }

    pub fn constant(level: PrefabLevel) -> Box<PrefabBuilder> {
        PrefabBuilder::new(PrefabMode::Constant{level})
    }

    pub fn sectional(section: prefab_sections::PrefabSection) -> Box<PrefabBuilder> {
        PrefabBuilder::new(PrefabMode::Sectional{section})
    }

    pub fn vaults() -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RoomVaults,
        })
    }
 
    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::Constant{level} => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional{section} => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data)
        }
        build_data.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, idx: usize, build_data: &mut BuilderMap) {
        let rng = RandomNumberGenerator::new();
        match ch {
            ' ' => build_data.map.tiles[idx] = TileType::Floor,
            '#' => build_data.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % build_data.map.width;
                let y = idx as i32 / build_data.map.width;
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.starting_position = Some(Position{ x: x as i32, y: y as i32 });
            }
            '>' => build_data.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, random_goblin(rng)));
            }
            'o' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, random_orc(rng)));
            }
            'O' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Ogre".to_string()));
            }
            'D' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Demon".to_string()));
            }
            '%' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Food Ration".to_string()));
            }
            '!' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Health Potion".to_string()));
            }
            '/' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, random_melee_weapon(rng)));
            }
            '0' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, random_shield(rng)));
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

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data: &mut BuilderMap) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);
        
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i +=1 ;
            }
        }
    }

    fn apply_sectional(&mut self, section: &PrefabSection, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);

        // place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (build_data.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (build_data.map.width - 1) - section.width as i32
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (build_data.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (build_data.map.height - 1) - section.height as i32
        }

        // build the map
        self.apply_previous_iteration(|x,y| {
            x < chunk_x || x > (chunk_x + section.width as i32) ||
            y < chunk_y || y > (chunk_y + section.height as i32)
        }, rng, build_data);
        build_data.take_snapshot();

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_previous_iteration<F>(&mut self, mut filter: F, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap)
        where F: FnMut(i32, i32) -> bool
    {
        // build the map
        let width = build_data.map.width;
        build_data.spawn_list.retain(|(idx, _name)| {
            let x = *idx as i32 % width;
            let y = *idx as i32 / width;
            filter(x, y)
        });
        build_data.take_snapshot();
    }

    fn apply_room_vaults(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // apply the previous builder and keep all entities it spawns
        self.apply_previous_iteration(|_x,_y| true, rng, build_data);

        // chance of encountering room vault dependent on depth
        let vault_roll = rng.roll_dice(1, 6) + build_data.map.depth;
        if vault_roll < 4 { return; }

        let master_vault_list = vec![GUARDED_WEAPON, GUARDED_SHIELD, OGRE_TRIO];

        // filter the vault list down to ones that are applicable to the current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| { build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth })
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
                let x = (idx % build_data.map.width as usize) as i32;
                let y = (idx / build_data.map.width as usize) as i32;

                // check for map overflow
                if x > 1
                && (x + vault.width as i32) < build_data.map.width - 2
                && y > 1
                && (y + vault.height as i32) < build_data.map.height - 2 {
                    let mut possible = true;
                    for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {
                            idx = build_data.map.xy_idx(tx + x, ty + y);
                            if build_data.map.tiles[idx] != TileType::Floor {
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
                if idx >= build_data.map.tiles.len() - 1 { break; }
            }

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
                let width = build_data.map.width;
                let height = build_data.map.height;
                build_data.spawn_list.retain(|e| {
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
                        let idx = build_data.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                        self.char_to_map(string_vec[i], idx, build_data);
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                build_data.take_snapshot();
                possible_vaults.remove(vault_index);
            }
        }
    }

    
}

fn random_melee_weapon(mut rng: RandomNumberGenerator) -> String {
    let roll = rng.roll_dice(1, 3);
    match roll {
        1 => "Mithril Longsword".to_string(),
        _ => "Steel Longsword".to_string()
    }
}

fn random_shield(mut rng: RandomNumberGenerator) -> String {
    let roll = rng.roll_dice(1, 3);
    match roll {
        1 => "Tower Shield".to_string(),
        _ => "Bulwark Shield".to_string()
    }
}

fn random_goblin(mut rng: RandomNumberGenerator) -> String {
    let roll = rng.roll_dice(1, 2);
    match roll {
        1 => "Goblin Warrior".to_string(),
        _ => "Goblin Archer".to_string()
    }
}

fn random_orc(mut rng: RandomNumberGenerator) -> String {
    let roll = rng.roll_dice(1, 2);
    match roll {
        1 => "Orc Warrior".to_string(),
        _ => "Orc Archer".to_string()
    }
}
