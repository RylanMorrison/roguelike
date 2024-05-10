use specs::prelude::*;
use super::{WantsToPickupItem, Position, Name, InBackpack, EquipmentChanged, Item};
use crate::gamelog;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = ( 
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, Item>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut wants_pickup, mut positions, 
            names, mut backpack, mut dirty, items) = data;

        for pickup in wants_pickup.join() {
            // consecutive letters of the alphabet are used for inventory entries so need to limit inventory size
            if backpack.count() >= 26 {
                gamelog::Logger::new().append("You inventory is full.").log();
            } else {
                positions.remove(pickup.item);
                backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");
                dirty.insert(pickup.collected_by, EquipmentChanged{}).expect("Unable to insert");
    
                if pickup.collected_by == *player_entity {
                    if let Some(item) = items.get(pickup.item) {
                        gamelog::Logger::new()
                            .append("You pick up the")
                            .item_name(item)
                            .log();
                    }
                }
            }
        }
        wants_pickup.clear();
    }
}
