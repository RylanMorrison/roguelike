use specs::prelude::*;
use crate::{spatial, MyTurn, MoveMode, Movement, Position, Map, ApplyMove, tile_walkable};
use rltk::RandomNumberGenerator;

pub struct DefaultMoveAI {}

impl<'a> System<'a> for DefaultMoveAI {
    type SystemData = ( 
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, MoveMode>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut move_mode, mut positions, mut map,
            mut rng, entities, mut apply_move) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, pos, mode, _myturn) in (&entities, &mut positions, &mut move_mode, &turns).join() {
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
                            apply_move.insert(entity, ApplyMove{ dest_idx }).expect("Unable to insert");
                        }
                    }
                }
                Movement::RandomWaypoint{path} => {
                    if let Some(path) = path {
                        // there is a path to follow
                        if path.len() > 1 {
                            if !spatial::is_blocked(path[1] as usize) {
                                // follow the path
                                apply_move.insert(entity, ApplyMove{ dest_idx: path[1] }).expect("Unable to insert");
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


