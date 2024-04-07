use specs::prelude::*;
use super::{EntityMoved, Position, EntryTrigger, Map, Name, gamelog::GameLog};
use super::effects::*;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = ( 
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, names, 
            entities, mut gamelog) = data;

        // Iterate the entities that moved and their final position
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            crate::spatial::for_each_tile_content(idx, |e| {
                if entity != e { // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(e);
                    match maybe_trigger {
                        None => {},
                        Some(_trigger) => {
                            // We triggered it
                            let name = names.get(e);
                            if let Some(name) = name {
                                gamelog.entries.push(format!("{} triggers!", &name.name));
                            }

                            add_effect(
                                Some(entity),
                                EffectType::TriggerFire{ trigger: e },
                                Targets::Tile{ tile_idx: idx as i32 }
                            );
                        }
                    }
                }
            });
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}