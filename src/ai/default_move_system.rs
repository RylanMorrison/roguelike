use specs::prelude::*;
use crate::{spatial, MyTurn, MoveMode, Movement, Position, Map, Viewshed, EntityMoved, tile_walkable};
use rltk::RandomNumberGenerator;

pub struct DefaultMoveAI {}

impl<'a> System<'a> for DefaultMoveAI {
    type SystemData = ( 
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, MoveMode>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, RandomNumberGenerator>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut move_mode, mut positions, mut map,
            mut viewsheds, mut entity_moved, mut rng, entities) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, pos, mode, viewshed, _myturn) in (&entities, &mut positions, &mut move_mode, &mut viewsheds, &turns).join() {
            match &mut mode.mode {
                Movement::Static => {},
                Movement::Random => {
                    // move in a random direction
                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = rng.roll_dice(1, 5);
                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        _ => {}
                    }

                    if x > 0 && x < map.width - 1
                    && y > 0 && y < map.height - 1 {
                        let dest_idx = map.xy_idx(x, y);
                        if !spatial::is_blocked(dest_idx) {
                            let idx = map.xy_idx(pos.x, pos.y);
                            pos.x = x;
                            pos.y = y;
                            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                            spatial::move_entity(entity, idx, dest_idx);
                            viewshed.dirty = true;
                        }
                    }
                }
                Movement::RandomWaypoint{path} => {
                    if let Some(path) = path {
                        // there is a path to follow
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        if path.len() > 1 {
                            if !spatial::is_blocked(path[1] as usize) {
                                // follow the path
                                pos.x = path[1] as i32 % map.width;
                                pos.y = path[1] as i32 / map.width;
                                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                                let new_idx = map.xy_idx(pos.x, pos.y);
                                spatial::move_entity(entity, idx, new_idx);
                                viewshed.dirty = true;
                                path.remove(0); // remove the first step in the path
                            }
                            // wait a turn to see if the path clears up
                        } else {
                            mode.mode = Movement::RandomWaypoint { path: None };
                        }
                    } else {
                        // pick a random location
                        let target_x = rng.roll_dice(1, map.width - 2);
                        let target_y = rng.roll_dice(1, map.height - 2);
                        let idx = map.xy_idx(target_x, target_y);
                        if tile_walkable(map.tiles[idx]) {
                            // store the path to the location as the new path if possible to walk to the location
                            let path = rltk::a_star_search(
                                map.xy_idx(pos.x, pos.y) as i32,
                                map.xy_idx(target_x, target_y) as i32,
                                &mut *map
                            );
                            if path.success && path.steps.len() > 1 {
                                mode.mode = Movement::RandomWaypoint { path: Some(path.steps) }
                            }
                        }
                    }
                }
            }
            turn_done.push(entity);
        }

        // remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}


