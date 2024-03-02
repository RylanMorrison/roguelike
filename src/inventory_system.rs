use specs::prelude::*;
use crate::{particle_system::ParticleBuilder, Confusion, Equippable};

use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog, WantsToUseItem,
    Consumable, ProvidesHealing, CombatStats, WantsToDropItem, InflictsDamage, Map, SufferDamage,
    AreaOfEffect, Equipped, WantsToUnequipItem, MagicMapper, RunState, HungerState, ProvidesFood, 
    HungerClock};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            // consecutive letters of the alphabet are used for inventory entries so need to limit inventory size
            if backpack.count() >= 26 {
                gamelog.entries.push("Your inventory is full.".to_string());
            } else {
                positions.remove(pickup.item);
                backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");
    
                if pickup.collected_by == *player_entity {
                    gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
                }
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ( 
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, ProvidesFood>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, map, 
            entities, mut wants_use, names,
            consumables, healing, inflict_damage,
            mut combat_stats, mut suffer_damage, aoe,
            mut confused, equippable, mut equipped, 
            mut backpack, mut particle_builder,
            positions, magic_mapper, mut runstate,
            mut hunger_clocks, provides_food) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            let mut used_item = true;

            // targeting
            let mut targets: Vec<Entity> = Vec::new();
            match useitem.target {
                None => { targets.push(*player_entity) }
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            // single target
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            // AoE
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                                particle_builder.add_request(tile_idx.x, tile_idx.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('░'), 200.0);
                            }
                        }
                    }
                }
            }

            // equipping
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // unequip any currently equipped item
                    let mut to_unequip : Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequip {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert backpack entry");
                    }

                    // equip the item
                    equipped.insert(useitem.item, Equipped{ owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equip {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }
            
            // healing
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            if stats.hp == stats.max_hp {
                                // prevent wastage
                                gamelog.entries.push("You are already full health.".to_string());
                                used_item = false;
                                continue;
                            } 
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("You drink the {}, healing {} hp.", names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::GREEN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('♥'), 200.0);
                            }
                            used_item = true;
                        }
                    }
                }
            }

            // food
            let item_provides_food = provides_food.get(useitem.item);
            match item_provides_food {
                None => {}
                Some(_) => {
                    let target = targets[0];
                    let hunger = hunger_clocks.get_mut(target);
                    if let Some(hunger) = hunger {
                        hunger.state = HungerState::WellFed;
                        hunger.duration = 20;
                        gamelog.entries.push(format!("You eat the {}.", names.get(useitem.item).unwrap().name));
                    }
                    used_item = true;
                }
            }


            // damaging
            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("You use {} on {}, inflicting {} damage.", item_name.name, mob_name.name, damage.damage));

                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::RED), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                            }
                        }
                        used_item = true;
                    }
                }
            }

            // confusion
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(useitem.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(useitem.item).unwrap();
                                gamelog.entries.push(format!("You use {} on {}, confusing them.", item_name.name, mob_name.name));
                                
                                let pos = positions.get(*mob);
                                if let Some(pos) = pos {
                                    particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::MAGENTA), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
                                }
                            }
                            used_item = true;
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused.insert(mob.0, Confusion{ turns: mob.1 }).expect("Unable to insert status");
            }

            // magic mapping
            let is_mapper = magic_mapper.get(useitem.item);
            match is_mapper {
                None => {}
                Some(_) => {
                    gamelog.entries.push("The map is revealed to you!".to_string());
                    used_item = true;
                    *runstate = RunState::MagicMapReveal{ row : 0 };
                }
            }
            
            if used_item {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }
        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropped_pos: Position = Position{x:0, y:0};
            {
                let dropper_pos = positions.get(entity).unwrap();
                dropped_pos.x = dropper_pos.x;
                dropped_pos.y = dropper_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropped_pos.x, y : dropped_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }
        wants_drop.clear();
    }
}

pub struct ItemUnequipSystem {}

impl<'a> System<'a> for ItemUnequipSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToUnequipItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, names, mut wants_unequip, mut equipped, mut backpack) = data;
        for (entity, to_unequip) in (&entities, &wants_unequip).join() {
            equipped.remove(to_unequip.item);
            backpack.insert(to_unequip.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
            if entity == *player_entity {
                gamelog.entries.push(format!("You unequip {}.", names.get(to_unequip.item).unwrap().name));
            }
        }
        wants_unequip.clear();
    }
}


