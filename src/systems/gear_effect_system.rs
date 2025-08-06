use specs::prelude::*;
use crate::{attr_bonus, gamelog, AttributeBonus, Attributes, EquipmentChanged, Equipped, InBackpack, Item, Weapon, 
    Pools, Wearable, Skills, SkillBonus, hp_at_level, mana_at_level, carry_capacity_lbs, ItemSets, PartOfSet,
    StatusEffectChanged, RegenBonus};
use std::collections::HashMap;
use rltk::RGB;

#[derive(Debug)]
struct ItemUpdate {
    weight: f32,
    initiative: f32,
    strength: i32,
    dexterity: i32,
    constitution: i32,
    intelligence: i32,
    melee: i32,
    defence: i32,
    ranged: i32,
    magic: i32,
    health_regen: i32,
    mana_regen: i32,
    total_armour_class: f32,
    base_damage: String
}

pub struct GearEffectSystem {}

impl<'a> System<'a> for GearEffectSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, AttributeBonus>,
        ReadStorage<'a, Weapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, SkillBonus>,
        ReadStorage<'a, PartOfSet>,
        ReadExpect<'a, ItemSets>,
        WriteStorage<'a, StatusEffectChanged>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        WriteStorage<'a, RegenBonus>

    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, items, backpacks, equipped, player,
            attribute_bonuses, weapons, wearables, skill_bonuses,
            set_pieces, item_sets, mut status_dirty, mut equip_dirty,
            mut pools, mut attributes, mut skills, mut regen_bonuses) = data;

        if equip_dirty.is_empty() { return; }

        // build the map of who needs updating
        let mut to_update: HashMap<Entity, ItemUpdate> = HashMap::new();
        for (entity, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(entity, ItemUpdate{
                weight: 0.0,
                initiative: 0.0,
                strength: 0,
                dexterity: 0,
                constitution: 0,
                intelligence: 0,
                melee: 0,
                defence: 0,
                ranged: 0,
                magic: 0,
                health_regen: 0,
                mana_regen: 0,
                total_armour_class: 10.0, // TODO use armour class from entity's pools
                base_damage: "1 - 4".to_string()
            });
            status_dirty.insert(entity, StatusEffectChanged{}).expect("Failed to insert");
        }
        equip_dirty.clear();

        // total up equipped items
        for (item, equip, entity) in (&items, &equipped, &entities).join() {
            if to_update.contains_key(&equip.owner) {
                let totals = to_update.get_mut(&equip.owner).unwrap();
                totals.weight += item.weight_lbs;
                totals.initiative += item.initiative_penalty;
                if let Some(attr) = attribute_bonuses.get(entity) {
                    totals.strength += attr.strength.unwrap_or(0);
                    totals.dexterity += attr.dexterity.unwrap_or(0);
                    totals.constitution += attr.constitution.unwrap_or(0);
                    totals.intelligence += attr.intelligence.unwrap_or(0);
                }
                if let Some(skill) = skill_bonuses.get(entity) {
                    totals.melee += skill.melee.unwrap_or(0);
                    totals.defence += skill.defence.unwrap_or(0);
                    totals.ranged += skill.ranged.unwrap_or(0);
                    totals.magic += skill.magic.unwrap_or(0);
                }
                if let Some(regen_bonus) = regen_bonuses.get(entity) {
                    if let Some(health_regen) = regen_bonus.health {
                        totals.health_regen = health_regen;
                    }
                    if let Some(mana_regen) = regen_bonus.mana {
                        totals.mana_regen = mana_regen;
                    }
                }
            }
        }

        // calculate base damage
        for (weapon, equip) in (&weapons, &equipped).join() {
            if to_update.contains_key(&equip.owner) {
                let totals = to_update.get_mut(&equip.owner).unwrap();
                totals.base_damage = format!("{} - {}", weapon.damage_n_dice + weapon.damage_bonus, weapon.damage_n_dice * weapon.damage_die_type + weapon.damage_bonus);
            }
            // TODO display extra damage from attributes and skills
        }

        // calculate total armour class
        for (wearable, equip) in (&wearables, &equipped).join() {
            if to_update.contains_key(&equip.owner) {
                let totals = to_update.get_mut(&equip.owner).unwrap();
                totals.total_armour_class += wearable.armour_class;
            }
            // TODO display extra armour class from attributes and skills
        }

        // total up carried items
        for (item, carried) in (&items, &backpacks).join() {
            if to_update.contains_key(&carried.owner) {
                let totals = to_update.get_mut(&carried.owner).unwrap();
                totals.weight += item.weight_lbs;
            }
        }

        // item set bonuses
        // determine equipped set piece count for each item set
        let mut set_counts: HashMap<String, i32> = HashMap::new();
        for (equip, set_piece) in (&equipped, &set_pieces).join() {
            if equip.owner == *player && to_update.contains_key(&equip.owner) {
                // only count set pieces for the player
                if item_sets.item_sets.get(&set_piece.set_name).is_some() {
                    if set_counts.contains_key(&set_piece.set_name) {
                        *set_counts.get_mut(&set_piece.set_name).unwrap() += 1;
                    } else {
                        set_counts.insert(set_piece.set_name.clone(), 1);
                    }
                }
            }
        }

        // apply set bonuses depending on number of set pieces equipped
        for set in set_counts.keys() {
            // only the player gets set bonuses for now
            let totals = to_update.get_mut(&player).unwrap();
            if let Some(item_set) = item_sets.item_sets.get(set) {
                if let Some(set_bonus) = item_set.set_bonuses.get(&set_counts[set]) {
                    // TODO cummulative set bonuses??
                    if let Some(attr_bonus) = &set_bonus.attribute_bonus {
                        totals.strength += attr_bonus.strength.unwrap_or(0);
                        totals.dexterity += attr_bonus.dexterity.unwrap_or(0);
                        totals.constitution += attr_bonus.constitution.unwrap_or(0);
                        totals.intelligence += attr_bonus.intelligence.unwrap_or(0);
                    }
                    if let Some(skill_bonus) = &set_bonus.skill_bonus {
                        totals.melee += skill_bonus.melee.unwrap_or(0);
                        totals.defence += skill_bonus.defence.unwrap_or(0);
                        totals.ranged += skill_bonus.ranged.unwrap_or(0);
                        totals.magic += skill_bonus.magic.unwrap_or(0);
                    }
                }
            }
        }

        // apply the gear effect changes
        for (entity, item) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                pool.total_weight = item.weight;
                pool.initiative_penalty.gear_effect_penalty = item.initiative;
                pool.total_armour_class = item.total_armour_class as i32;
                pool.base_damage = item.base_damage.clone();

                if let Some(attr) = attributes.get_mut(*entity) {
                    attr.strength.item_modifiers = item.strength;
                    attr.dexterity.item_modifiers = item.dexterity;
                    attr.constitution.item_modifiers = item.constitution;
                    attr.intelligence.item_modifiers = item.intelligence;

                    attr.strength.bonus = attr_bonus(attr.strength.base + attr.strength.total_modifiers());
                    attr.dexterity.bonus = attr_bonus(attr.dexterity.base + attr.dexterity.total_modifiers());
                    attr.constitution.bonus = attr_bonus(attr.constitution.base + attr.constitution.total_modifiers());
                    attr.intelligence.bonus = attr_bonus(attr.intelligence.base + attr.intelligence.total_modifiers());

                    // update health and mana
                    pool.hit_points.max = hp_at_level(
                        attr.constitution.base + attr.constitution.total_modifiers(),
                        pool.level
                    );
                    if pool.hit_points.current > pool.hit_points.max { pool.hit_points.current = pool.hit_points.max; }

                    pool.mana.max = mana_at_level(
                        attr.intelligence.base + attr.intelligence.total_modifiers(),
                        pool.level
                    );

                    if pool.total_weight > carry_capacity_lbs(&attr.strength) {
                        // overburdened
                        pool.initiative_penalty.gear_effect_penalty += 4.0;
                        if *entity == *player {
                            gamelog::Logger::new().colour(RGB::named(rltk::ORANGE)).append("You are overburdened!").log();
                        }
                    }
                }

                if let Some(skill) = skills.get_mut(*entity) {
                    skill.melee.item_modifiers = item.melee;
                    skill.defence.item_modifiers = item.defence;
                    skill.ranged.item_modifiers = item.ranged;
                    skill.magic.item_modifiers = item.magic;
                }
            }

            if item.health_regen == 0 && item.mana_regen == 0 {
                if regen_bonuses.get(*entity).is_some() {
                    regen_bonuses.remove(*entity).expect("Unable to remove");
                }
                continue;
            }

            // apply regen bonuses from items to the entity
            if let Some(current_regen_bonus) = regen_bonuses.get_mut(*entity) {
                current_regen_bonus.health = Some(item.health_regen).filter(|&v| v != 0);
                current_regen_bonus.mana = Some(item.mana_regen).filter(|&v| v != 0);
            } else {
                regen_bonuses.insert(*entity, RegenBonus{
                    health: Some(item.health_regen).filter(|&v| v != 0),
                    mana: Some(item.mana_regen).filter(|&v| v != 0)
                }).expect("Unable to insert");
            }
        }
    }
}
