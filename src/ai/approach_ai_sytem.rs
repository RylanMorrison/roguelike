use specs::prelude::*;
use crate::{spatial, MyTurn, WantsToApproach, Position, Map, Viewshed, EntityMoved};

pub struct ApproachAI {}

impl<'a> System<'a> for ApproachAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut want_approach, mut positions, mut map,
            mut viewsheds, mut entity_moved, entities) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, pos, approach, viewshed, _myturn) in (&entities, &mut positions, &mut want_approach, &mut viewsheds, &mut turns).join() {
            // look for a path from the entity to what it wants to approach
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(approach.idx % map.width, approach.idx / map.width) as i32,
                &mut *map
            );
            if path.success && path.steps.len() > 1 {
                // make the entity approach one step
                let mut idx = map.xy_idx(pos.x, pos.y);
                pos.x = path.steps[1] as i32 % map.width;
                pos.y = path.steps[1] as i32 / map.width;
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                let new_idx = map.xy_idx(pos.x, pos.y);
                spatial::move_entity(entity, idx, new_idx);
                viewshed.dirty = true;
            }
            turn_done.push(entity);
        }
        want_approach.clear();

        // remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
