use specs::prelude::*;
use crate::{PassiveEffectChanged, Pools, Attributes, Skills, CharacterClass, attr_bonus, player_hp_at_level,
    mana_at_level, carry_capacity_lbs};
use std::collections::HashMap;
use crate::gamelog;
use rltk::RGB;

#[derive(Debug)]
struct PassiveUpdate {
    strength: i32,
    dexterity: i32,
    constitution: i32,
    intelligence: i32,
    melee: i32,
    defence: i32,
    ranged: i32,
    magic: i32
}

pub struct PassiveEffectSystem {}

impl<'a> System<'a> for PassiveEffectSystem {
    type SystemData = (
        WriteStorage<'a, PassiveEffectChanged>,
        Entities<'a>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, CharacterClass>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut passive_dirty, entities, mut pools, mut attributes, mut skills,
            player_entity, character_classes) = data;

        if passive_dirty.is_empty() { return; }

        let mut to_update: HashMap<Entity, PassiveUpdate> = HashMap::new();
        for (entity, dirty) in (&entities, &passive_dirty).join() {
            to_update.insert(entity, PassiveUpdate {
                strength: 0,
                dexterity: 0,
                constitution: 0,
                intelligence: 0,
                melee: 0,
                defence: 0,
                ranged: 0,
                magic: 0
            });
        }
        passive_dirty.clear();

        // total up class passive effect modifiers
        for (entity, character_class) in (&entities, &character_classes).join() {
            if to_update.contains_key(&entity) {
                let totals = to_update.get_mut(&entity).unwrap();
                for (name, passive) in character_class.passives.iter() {
                    if passive.current_level < 1 { continue; }

                    let passive_level = &passive.levels[&passive.current_level];
                    if let Some(attribute_bonus) = &passive_level.attribute_bonus {
                        totals.strength += attribute_bonus.strength.unwrap_or(0);
                        totals.dexterity += attribute_bonus.dexterity.unwrap_or(0);
                        totals.constitution += attribute_bonus.constitution.unwrap_or(0);
                        totals.intelligence += attribute_bonus.intelligence.unwrap_or(0);
                    }

                    if let Some(skill_bonus) = &passive_level.skill_bonus {
                        totals.melee += skill_bonus.melee.unwrap_or(0);
                        totals.defence += skill_bonus.defence.unwrap_or(0);
                        totals.ranged += skill_bonus.ranged.unwrap_or(0);
                        totals.magic += skill_bonus.magic.unwrap_or(0);
                    }
                }
            }
        }

        for (entity, update) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                if let Some(current_attr) = attributes.get_mut(*entity) {
                    current_attr.strength.base += update.strength;
                    current_attr.dexterity.base += update.dexterity;
                    current_attr.constitution.base += update.constitution;
                    current_attr.intelligence.base += update.intelligence;

                    current_attr.strength.bonus = attr_bonus(current_attr.strength.base + current_attr.strength.modifiers);
                    current_attr.dexterity.bonus = attr_bonus(current_attr.dexterity.base + current_attr.dexterity.modifiers);
                    current_attr.constitution.bonus = attr_bonus(current_attr.constitution.base + current_attr.constitution.modifiers);
                    current_attr.intelligence.bonus = attr_bonus(current_attr.intelligence.base + current_attr.intelligence.modifiers);

                    // update health and mana
                    pool.hit_points.max = player_hp_at_level(
                        current_attr.constitution.base + current_attr.constitution.modifiers,
                        pool.level
                    );
                    pool.mana.max = mana_at_level(
                        current_attr.intelligence.base + current_attr.intelligence.modifiers,
                        pool.level
                    );
                    
                    if pool.total_weight > carry_capacity_lbs(&current_attr.strength) {
                        // overburdened
                        pool.total_initiative_penalty += 4.0;
                        if *entity == *player_entity {
                            gamelog::Logger::new().colour(RGB::named(rltk::ORANGE)).append("You are overburdened!").log();
                        }
                    }
                }
                if let Some(current_skills) = skills.get_mut(*entity) {
                    current_skills.melee.base += update.melee;
                    current_skills.defence.base += update.defence;
                    current_skills.ranged.base += update.ranged;
                    current_skills.magic.base += update.magic;
                }
            }
        }
    }
}
