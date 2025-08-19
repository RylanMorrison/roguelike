use specs::prelude::*;
use crate::{spatial, Map, Position, ApplyMove, ApplyTeleport, OtherLevelPosition, EntityMoved,
    Viewshed, RunState};

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        WriteStorage<'a, Position>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
        WriteStorage<'a, ApplyTeleport>,
        WriteStorage<'a, OtherLevelPosition>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Viewshed>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, RunState>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut positions, entities, mut apply_move,
            mut apply_teleport, mut other_level, mut moved,
            mut viewsheds, player_entity, mut runstate) = data;

        // apply teleporting
        for (entity, teleport) in (&entities, &apply_teleport).join() {
            if teleport.dest_map == map.name {
                apply_move.insert(entity, ApplyMove{ dest_idx: map.xy_idx(teleport.dest_x, teleport.dest_y) }).expect("Unable to insert");
            } else if entity == *player_entity {
                *runstate = RunState::TeleportingToOtherLevel{ x: teleport.dest_x, y: teleport.dest_y, map_name: teleport.dest_map.clone() };
            } else if let Some(pos) = positions.get(entity) {
                let idx = map.xy_idx(pos.x, pos.y);
                let dest_idx = map.xy_idx(teleport.dest_x, teleport.dest_y);
                spatial::move_entity(entity, idx, dest_idx);
                other_level.insert(entity, OtherLevelPosition{
                    x: teleport.dest_x,
                    y: teleport.dest_y,
                    map_name: teleport.dest_map.clone()
                }).expect("Unable to insert");
                positions.remove(entity);
            }
        }
        apply_teleport.clear();

        // apply normal movement
        for (entity, movement, pos) in (&entities, &apply_move, &mut positions).join() {
            let start_idx = map.xy_idx(pos.x, pos.y);
            let dest_idx = movement.dest_idx as usize;
            spatial::move_entity(entity, start_idx, dest_idx);
            pos.x = movement.dest_idx as i32 % map.width;
            pos.y = movement.dest_idx as i32 / map.width;
            if let Some(vs) = viewsheds.get_mut(entity) {
                vs.dirty = true;
            }
            moved.insert(entity, EntityMoved{}).expect("Unable to insert");
        }
        apply_move.clear();
    }
}
