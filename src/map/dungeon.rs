use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use specs::prelude::*;
use super::{World, Map, Point, Entity, TileType};
use crate::components::{Position, Viewshed, OtherLevelPosition};
use crate::map_builders::level_builder;
use rltk::RandomNumberGenerator;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<i32, Map>
}

impl MasterDungeonMap {
    pub fn new() -> MasterDungeonMap {
        MasterDungeonMap{ maps: HashMap::new() }
    }

    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.depth, map.clone());
    }

    pub fn get_map(&self, depth: i32) -> Option<Map> {
        if self.maps.contains_key(&depth) {
            let mut result = self.maps[&depth].clone();
            return Some(result)
        }
        None
    }
}

pub fn level_transition(ecs: &mut World, new_depth: i32, offset: i32) -> Option<Vec<Map>> {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();

    if dungeon_master.get_map(new_depth).is_some() {
        std::mem::drop(dungeon_master); // drop to remove the borrow on ecs
        transition_to_existing_map(ecs, new_depth, offset);
        None
    } else {
        std::mem::drop(dungeon_master);
        Some(transition_to_new_map(ecs, new_depth))
    }
}

/// Transition the player down to a new depth
fn transition_to_new_map(ecs: &mut World, new_depth: i32) -> Vec<Map> {
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    let mut builder = level_builder(new_depth, &mut rng, 80, 50);
    builder.build_map(&mut rng);

    if new_depth > 1 {
        if let Some(pos) = &builder.build_data.starting_position {
            let up_idx = builder.build_data.map.xy_idx(pos.x, pos.y);
            builder.build_data.map.tiles[up_idx] = TileType::UpStairs;
        }
    }
    
    let mapgen_history = builder.build_data.history.clone();
    let player_start;
    {
        let mut worldmap_resource = ecs.write_resource::<Map>();
        if new_depth > 1 {
            builder.build_data.map.name = format!("Depth: {}", new_depth - 1);
        }
        *worldmap_resource = builder.build_data.map.clone();
        player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
    }

    std::mem::drop(rng);
    builder.spawn_entites(ecs);

    // Place the player and update resources
    let (player_x, player_y) = (player_start.x, player_start.y);
    let mut player_position = ecs.write_resource::<Point>();
    *player_position = Point::new(player_x, player_y);
    let mut position_components = ecs.write_storage::<Position>();
    let player_entity = ecs.fetch::<Entity>();
    let player_pos_comp = position_components.get_mut(*player_entity);
    if let Some(player_pos_comp) = player_pos_comp {
        player_pos_comp.x = player_x;
        player_pos_comp.y = player_y;
    }

    // reset the player's visibility
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    let player_viewshed = viewshed_components.get_mut(*player_entity);
    if let Some(player_viewshed) = player_viewshed {
        player_viewshed.dirty = true;
    }

    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    dungeon_master.store_map(&builder.build_data.map);

    mapgen_history
}

/// Transition the player back up to a previous depth
fn transition_to_existing_map(ecs: &mut World, new_depth: i32, offset: i32) {
    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    let map = dungeon_master.get_map(new_depth).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();

    // Place the player and update resources
    let w = map.width;
    let stair_type = if offset < 0 { TileType::DownStairs } else { TileType::UpStairs };
    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt == stair_type {
            // put the player on the downstairs tile
            let mut player_point = ecs.write_resource::<Point>();
            *player_point = Point::new(idx as i32 % w, idx as i32 / w);
            let mut positions = ecs.write_storage::<Position>();
            let player_pos = positions.get_mut(*player_entity);
            if let Some(player_pos) = player_pos {
                player_pos.x = idx as i32 % w;
                player_pos.y = idx as i32 / w;
            }
        }
    }
    dungeon_master.store_map(&worldmap_resource.clone());
    *worldmap_resource = map;

    // reset player visibility
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let player_viewshed = viewsheds.get_mut(*player_entity);
    if let Some(viewshed) = player_viewshed {
        viewshed.dirty = true;
    }
}

pub fn freeze_level_entities(ecs: &mut World) {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map_depth = ecs.fetch::<Map>().depth;

    // store level positions of entities as OtherLevelPositions
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &positions).join() {
        if entity != *player_entity {
            other_level_positions.insert(entity, OtherLevelPosition{
                x: pos.x,
                y: pos.y,
                depth: map_depth
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
    let map_depth = ecs.fetch::<Map>().depth;

    // restore level positions from OtherLevelPositions
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &other_level_positions).join() {
        if entity != *player_entity && pos.depth == map_depth {
            positions.insert(entity, Position{ x: pos.x, y: pos.y }).expect("Failed to insert");
            pos_to_delete.push(entity);
        }
    }

    // clean up store of OtherLevelPositions
    for p in pos_to_delete.iter() {
        other_level_positions.remove(*p);
    }
}
