use specs::prelude::*;
use super::{WantsToUnequipItem, Equipped, InBackpack, EquipmentChanged, Item};
use crate::gamelog;

pub struct ItemUnequipSystem {}

impl<'a> System<'a> for ItemUnequipSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, WantsToUnequipItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, Item>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, entities, mut wants_unequip,
            mut equipped, mut backpack, mut dirty, items) = data;

        for (entity, to_unequip) in (&entities, &wants_unequip).join() {
            equipped.remove(to_unequip.item);
            backpack.insert(to_unequip.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
            if entity == *player_entity {
                if let Some(item) = items.get(to_unequip.item) {
                    gamelog::Logger::new()
                    .append("You unequip")
                    .item_name(item)
                    .log();
                }
            }
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");
        }
        wants_unequip.clear();
    }
}
