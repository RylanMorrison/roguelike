use specs::prelude::*;
use super::{GameLog, WantsToPickupItem, Position, Name, InBackpack, EquipmentChanged};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = ( 
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup,
            mut positions, names, mut backpack, mut dirty) = data;

        for pickup in wants_pickup.join() {
            // consecutive letters of the alphabet are used for inventory entries so need to limit inventory size
            if backpack.count() >= 26 {
                gamelog.entries.push("Your inventory is full.".to_string());
            } else {
                positions.remove(pickup.item);
                backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");
                dirty.insert(pickup.collected_by, EquipmentChanged{}).expect("Unable to insert");
    
                if pickup.collected_by == *player_entity {
                    gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
                }
            }
        }
        wants_pickup.clear();
    }
}
