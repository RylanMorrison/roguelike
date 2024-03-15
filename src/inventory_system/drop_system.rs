use specs::prelude::*;
use super::{GameLog, WantsToDropItem, Name, Position, InBackpack, EquipmentChanged};

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, 
            names, mut positions, mut backpack, mut dirty) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropped_pos: Position = Position{x:0, y:0};
            {
                let dropper_pos = positions.get(entity).unwrap();
                dropped_pos.x = dropper_pos.x;
                dropped_pos.y = dropper_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropped_pos.x, y : dropped_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }
        wants_drop.clear();
    }
}
