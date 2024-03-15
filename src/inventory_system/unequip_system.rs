use specs::prelude::*;
use super::{GameLog, Name, WantsToUnequipItem, Equipped, InBackpack, EquipmentChanged};

pub struct ItemUnequipSystem {}

impl<'a> System<'a> for ItemUnequipSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToUnequipItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, names, mut wants_unequip, 
            mut equipped, mut backpack, mut dirty) = data;
        for (entity, to_unequip) in (&entities, &wants_unequip).join() {
            equipped.remove(to_unequip.item);
            backpack.insert(to_unequip.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
            if entity == *player_entity {
                gamelog.entries.push(format!("You unequip {}.", names.get(to_unequip.item).unwrap().name));
            }
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");
        }
        wants_unequip.clear();
    }
}
