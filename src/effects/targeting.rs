use specs::prelude::*;
use rltk::Point;
use crate::components::{Position, InBackpack, Equipped};
use crate::map::Map;

pub fn entity_position(ecs: &World, target: Entity) -> Option<i32> {
    if let Some(pos) = ecs.read_storage::<Position>().get(target) {
        let map = ecs.fetch::<Map>();
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }
    None
}

pub fn aoe_tiles(map: &Map, target: Point, radius: i32) -> Vec<i32> {
    let mut blast_tiles = rltk::field_of_view(target, radius, &*map);
    blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
    let mut result = Vec::new();
    for t in blast_tiles.iter() {
        result.push(map.xy_idx(t.x, t.y) as i32);
    }
    result
}

pub fn aoe_points(map: &Map, target: Point, radius: i32) -> Vec<Point> {
    let mut points = rltk::field_of_view(target, radius, &*map);
    points.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
    points
}

pub fn find_item_position(ecs: &World, target: Entity, creator: Option<Entity>) -> Option<i32> {
    let positions = ecs.read_storage::<Position>();
    let map = ecs.fetch::<Map>();

    // on the map
    if let Some(pos) = positions.get(target) {
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }

    // carried by something
    if let Some(carried) = ecs.read_storage::<InBackpack>().get(target) {
        if let Some(pos) = positions.get(carried.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // equipped by something
    if let Some(equipped) = ecs.read_storage::<Equipped>().get(target) {
        if let Some(pos) = positions.get(equipped.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // the creator's position (for spells)
    if let Some(creator) = creator {
        if let Some(pos) = positions.get(creator) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // can't find it
    None
}
