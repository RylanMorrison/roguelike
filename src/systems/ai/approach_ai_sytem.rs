use specs::prelude::*;
use crate::{MyTurn, WantsToApproach, Position, Map, ApplyMove, RunState};

pub struct ApproachAI {}

impl<'a> System<'a> for ApproachAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
        ReadExpect<'a, RunState>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut want_approach, mut positions,
            mut map, entities, mut apply_move, runstate) = data;

        if RunState::Ticking != *runstate { return; }

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, pos, approach, _myturn) in (&entities, &mut positions, &mut want_approach, &mut turns).join() {
            // look for a path from the entity to what it wants to approach
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(approach.idx % map.width, approach.idx / map.width) as i32,
                &mut *map
            );
            if path.success && path.steps.len() > 1 {
                // make the entity approach one step
                apply_move.insert(entity, ApplyMove{ dest_idx: path.steps[1] }).expect("Unable to insert");
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
