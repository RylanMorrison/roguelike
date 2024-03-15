use specs::prelude::*;
use super::{Name, InBackpack, GameLog, WantsToUseItem, Equippable, Equipped, EquipmentChanged, EquipmentSlot};

pub struct ItemEquipSystem {}

impl<'a> System<'a> for ItemEquipSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_use, names, equippable, 
            mut equipped, mut backpack, mut dirty) = data;

        let mut remove_use: Vec<Entity> = Vec::new();
        for (target, useitem) in (&entities, &wants_use).join() {
            if let Some(can_equip) = equippable.get(useitem.item) {
                let target_slot = can_equip.slot;

                // unequip currently equipped item
                let mut to_unequip : Vec<Entity> = Vec::new();
                for (item_entity, already_equipped) in (&entities, &equipped).join() {
                    if already_equipped.owner == target {
                        if already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                        } else if already_equipped.slot == EquipmentSlot::TwoHanded {
                            // unequip two handed if main hand or off hand is equipped
                            if target_slot == EquipmentSlot::MainHand || target_slot == EquipmentSlot::OffHand {
                                to_unequip.push(item_entity);
                            }
                        } else if target_slot == EquipmentSlot::TwoHanded {
                            // unequip both main hand and off hand if two handed is equipped
                            if already_equipped.slot == EquipmentSlot::MainHand || already_equipped.slot == EquipmentSlot::OffHand {
                                to_unequip.push(item_entity);
                            }
                        }
                    }
                }
                for item in to_unequip.iter() {
                    equipped.remove(*item);
                    backpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert backpack entry");
                    if target == *player_entity {
                        gamelog.entries.push(format!("You unequip {}.", &names.get(*item).unwrap().name));
                    }
                }

                // equip the item
                equipped.insert(useitem.item, Equipped{ owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                backpack.remove(useitem.item);
                if target == *player_entity {
                    gamelog.entries.push(format!("You equip {}.", names.get(useitem.item).unwrap().name));
                }

                remove_use.push(target);
            }
        }

        remove_use.iter().for_each(|e| {
            dirty.insert(*e, EquipmentChanged{}).expect("Unable to insert");
            wants_use.remove(*e).expect("Unable to remove");
        });
    }
}
