use specs::prelude::*;
use crate::{attr_bonus, gamelog, AttributeBonus, Attributes, EquipmentChanged, Equipped, InBackpack, Item, Weapon, 
    Pools, Slow, StatusEffect, Wearable, Skills, SkillBonus, player_hp_at_level, mana_at_level, carry_capacity_lbs, ItemSets, PartOfSet};
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
    magic: i32,
    total_armour_class: f32,
    base_damage: String
}

pub struct GearEffectSystem {}

impl<'a> System<'a> for GearEffectSystem {
    type SystemData = (
        WriteStorage<'a, EquipmentChanged>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, AttributeBonus>,
        ReadStorage<'a, StatusEffect>,
        ReadStorage<'a, Slow>,
        ReadStorage<'a, Weapon>,
        ReadStorage<'a, Wearable>,
        WriteStorage<'a, Skills>,
        ReadStorage<'a, SkillBonus>,
        ReadExpect<'a, ItemSets>,
        ReadStorage<'a, PartOfSet>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut equip_dirty, entities, items, backpacks, wielded,
            mut pools, mut attributes, player, attribute_bonuses, 
            statuses, slowed, weapons, wearables, mut skills, 
            skill_bonuses, item_sets, set_pieces) = data;

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
                magic: 0,
                total_armour_class: 10.0, // TODO use armour class from entity's pools
                base_damage: "1 - 4".to_string()
            });
        }
        equip_dirty.clear();

        // total up equipped items
        for (item, equipped, entity) in (&items, &wielded, &entities).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
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
                    totals.magic += skill.magic.unwrap_or(0);
                }
            }
        }

        // calculate base damage
        for (weapon, equipped) in (&weapons, &wielded).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
                totals.base_damage = format!("{} - {}", weapon.damage_n_dice + weapon.damage_bonus, weapon.damage_n_dice * weapon.damage_die_type + weapon.damage_bonus);
            }
            // TODO display extra damage from attributes and skills
        }

        // calculate total armour class
        for (wearable, equipped) in (&wearables, &wielded).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
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

        // total up haste/slow
        for (status, slow) in (&statuses, &slowed).join() {
            if to_update.contains_key(&status.target) {
                let totals = to_update.get_mut(&status.target).unwrap();
                totals.initiative += slow.initiative_penalty;
            }
        }

        // item set bonuses
        // TODO lags a bit when equipping, find more efficient way of doing this
        // determine equipped set piece count for each item set
        let mut set_counts: HashMap<String, i32> = HashMap::new();
        for (equipped, set_piece) in (&wielded, &set_pieces).join() {
            if equipped.owner == *player && to_update.contains_key(&equipped.owner) {
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
                        totals.magic += skill_bonus.magic.unwrap_or(0);
                    }
                }
            }
        }

        // apply to pools
        for (entity, item) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                pool.total_weight = item.weight;
                pool.total_initiative_penalty = item.initiative;
                pool.total_armour_class = item.total_armour_class as i32;
                pool.base_damage = item.base_damage.clone();

                if let Some(attr) = attributes.get_mut(*entity) {
                    attr.strength.modifiers = item.strength;
                    attr.dexterity.modifiers = item.dexterity;
                    attr.constitution.modifiers = item.constitution;
                    attr.intelligence.modifiers = item.intelligence;

                    attr.strength.bonus = attr_bonus(attr.strength.base + attr.strength.modifiers);
                    attr.dexterity.bonus = attr_bonus(attr.dexterity.base + attr.dexterity.modifiers);
                    attr.constitution.bonus = attr_bonus(attr.constitution.base + attr.constitution.modifiers);
                    attr.intelligence.bonus = attr_bonus(attr.intelligence.base + attr.intelligence.modifiers);

                    // update health and mana
                    pool.hit_points.max = player_hp_at_level(
                        attr.constitution.base + attr.constitution.modifiers,
                        pool.level
                    );
                    pool.mana.max = mana_at_level(
                        attr.intelligence.base + attr.intelligence.modifiers,
                        pool.level
                    );
                    
                    if pool.total_weight > carry_capacity_lbs(&attr.strength) {
                        // overburdened
                        pool.total_initiative_penalty += 4.0;
                        if *entity == *player {
                            gamelog::Logger::new().colour(RGB::named(rltk::ORANGE)).append("You are overburdened!").log();
                        }
                    }
                }

                if let Some(skill) = skills.get_mut(*entity) {
                    skill.melee.modifiers = item.melee;
                    skill.defence.modifiers = item.defence;
                    skill.magic.modifiers = item.magic;
                }
            }
        }
    }
}
