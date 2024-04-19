use specs::prelude::*;
use crate::{EntityMoved, Position, EntryTrigger, Map, Name};
use crate::effects::*;
use crate::gamelog;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = ( 
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, Name>,
        Entities<'a>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, names, 
            entities) = data;

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
                                gamelog::Logger::new().append(format!("{} triggers!", &name.name)).log();
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
