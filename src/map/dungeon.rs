use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use specs::prelude::*;
use super::{World, Map, Point, Entity};
use crate::{components::{OtherLevelPosition, Position, Viewshed}, spawner};

#[derive(Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<String, Map>,
    visited: Vec<String>
}

impl MasterDungeonMap {
    pub fn new() -> MasterDungeonMap {
        MasterDungeonMap {
            maps: HashMap::new(),
            visited: Vec::new()
        }
    }

    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.name.clone(), map.clone());
    }

    pub fn get_map(&self, name: &str) -> Option<Map> {
        if self.maps.contains_key(name) {
            let result = self.maps[name].clone();
            return Some(result)
        }
        None
    }

    pub fn store_visited(&mut self, map_name: &str) {
        self.visited.push(map_name.to_string());
    }

    pub fn has_visited(&self, map_name: &str) -> bool {
        self.visited.contains(&map_name.to_string())
    }

    pub fn reset(&mut self) {
        self.visited = Vec::new();
    }
}

fn change_player_position(ecs: &mut World, new_position: (i32, i32)) {
    let (new_x, new_y) = new_position;

    let mut player_point = ecs.write_resource::<Point>();
    player_point.x = new_x;
    player_point.y = new_y;

    let player_entity = ecs.read_resource::<Entity>();
    let mut positions = ecs.write_storage::<Position>();
    if let Some(player_position) = positions.get_mut(*player_entity) {
        player_position.x = new_x;
        player_position.y = new_y;
    }
}

fn reset_player_visibility(ecs: &mut World) {
    let player_entity = ecs.fetch::<Entity>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let player_viewshed = viewsheds.get_mut(*player_entity);
    if let Some(viewshed) = player_viewshed {
        viewshed.dirty = true;
    }
}

pub fn transition_map(ecs: &mut World, map_name: &str, player_position: Option<(i32, i32)>) {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    if let Some(new_map) = dungeon_master.get_map(&map_name) {
        let visited = dungeon_master.has_visited(&new_map.name);
        std::mem::drop(dungeon_master);

        // Place the player and update resources
        let current_map_name = ecs.read_resource::<Map>().name.clone();
        if let Some(position) = player_position {
            change_player_position(ecs, position);
        } else if let Some(entrance_point) = new_map.transitions.get(&current_map_name) {
            change_player_position(ecs, (entrance_point.x, entrance_point.y));
        } else if let Some(start_position) = &new_map.starting_position {
            change_player_position(ecs, (start_position.x, start_position.y));
        }
        reset_player_visibility(ecs);

        // update stored map and current map
        let mut current_map = ecs.write_resource::<Map>();
        let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
        dungeon_master.store_map(&current_map);
        *current_map = new_map.clone();
        std::mem::drop(current_map);
        std::mem::drop(dungeon_master);

        if visited {
            thaw_level_entities(ecs);
        } else {
            spawn_entities(ecs);
            let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
            dungeon_master.visited.push(new_map.name);
        }
    }
}

pub fn spawn_entities(ecs: &mut World) {
    let spawn_list = ecs.fetch::<Map>().spawn_list.clone();
    for (location, name) in spawn_list.iter() {
        spawner::spawn_entity(ecs, &(location, name));
    }

    // update the stored map
    let map = ecs.fetch::<Map>();
    let mut dungeon_master = ecs.fetch_mut::<MasterDungeonMap>();
    dungeon_master.store_map(&map);
}

pub fn freeze_level_entities(ecs: &mut World) {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map = ecs.fetch::<Map>();

    // store level positions of entities as OtherLevelPositions
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &positions).join() {
        if entity != *player_entity {
            other_level_positions.insert(entity, OtherLevelPosition{
                x: pos.x,
                y: pos.y,
                map_name: map.name.clone()
            }).expect("Failed to insert");
            pos_to_delete.push(entity);
        }
    }

    // remove level positions of entities
    for p in pos_to_delete.iter() {
        positions.remove(*p);
    }
}

pub fn thaw_level_entities(ecs: &mut World) {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map = ecs.fetch::<Map>();

    // restore level positions from OtherLevelPositions
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &other_level_positions).join() {
        if entity != *player_entity && pos.map_name == map.name {
            positions.insert(entity, Position{ x: pos.x, y: pos.y }).expect("Failed to insert");
            pos_to_delete.push(entity);
        }
    }

    // clean up store of OtherLevelPositions
    for p in pos_to_delete.iter() {
        other_level_positions.remove(*p);
    }
}
