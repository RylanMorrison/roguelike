use specs::prelude::*;
use crate::{attr_bonus, gamelog, StatusEffectChanged, AttributeBonus, Attributes, Pools, StatusEffect,
    player_hp_at_level, mana_at_level, carry_capacity_lbs, SkillBonus, Skills, Slow, Duration, Block, Dodge};
use std::collections::HashMap;
use rltk::RGB;

#[derive(Debug)]
struct StatusUpdate {
    strength: i32,
    dexterity: i32,
    constitution: i32,
    intelligence: i32,
    melee: i32,
    defence: i32,
    ranged: i32,
    magic: i32,
    initiative_penalty: f32,
    block_chance: Option<f32>,
    dodge_chance: Option<f32>
}

pub struct StatusEffectSystem {}

impl<'a> System<'a> for StatusEffectSystem {
    type SystemData = (
        WriteStorage<'a, StatusEffectChanged>,
        Entities<'a>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, AttributeBonus>,
        ReadStorage<'a, SkillBonus>,
        ReadStorage<'a, StatusEffect>,
        ReadStorage<'a, Slow>,
        WriteStorage<'a, Block>,
        WriteStorage<'a, Dodge>,
        ReadStorage<'a, Duration>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut status_dirty, entities, mut pools, mut attributes,
            mut skills, player, attribute_bonuses, skill_bonuses,
            statuses, slows, mut blocks, mut dodges, durations) = data;

        if status_dirty.is_empty() { return; }

        let mut to_update: HashMap<Entity, StatusUpdate> = HashMap::new();
        for (entity, _dirty) in (&entities, &status_dirty).join() {
            to_update.insert(entity, StatusUpdate {
                strength: 0, 
                dexterity: 0,
                constitution: 0,
                intelligence: 0,
                melee: 0,
                defence: 0,
                ranged: 0,
                magic: 0,
                initiative_penalty: 0.0,
                block_chance: None,
                dodge_chance: None
            });
        }
        status_dirty.clear();

        // total up status effect modifiers
        for (entity, status) in (&entities, &statuses).join() {
            if to_update.contains_key(&status.target) {
                let totals = to_update.get_mut(&status.target).unwrap();

                if let Some(bonus) = attribute_bonuses.get(entity) {
                    totals.strength += bonus.strength.unwrap_or(0);
                    totals.dexterity += bonus.dexterity.unwrap_or(0);
                    totals.constitution += bonus.constitution.unwrap_or(0);
                    totals.intelligence += bonus.intelligence.unwrap_or(0);
                }

                if let Some(bonus) = skill_bonuses.get(entity) {
                    totals.melee += bonus.melee.unwrap_or(0);
                    totals.defence += bonus.defence.unwrap_or(0);
                    totals.ranged += bonus.ranged.unwrap_or(0);
                    totals.magic += bonus.magic.unwrap_or(0);
                }

                if let Some(slow) = slows.get(entity) {
                    totals.initiative_penalty += slow.initiative_penalty;
                }

                if let Some(block) = blocks.get(entity) {
                    totals.block_chance = Some(block.chance);
                }

                if let Some(dodge) = dodges.get(entity) {
                    totals.dodge_chance = Some(dodge.chance);
                }
            }
        }

        // apply to pools
        for (entity, update) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                if let Some(attr) = attributes.get_mut(*entity) {
                    attr.strength.status_effect_modifiers = update.strength;
                    attr.dexterity.status_effect_modifiers = update.dexterity;
                    attr.constitution.status_effect_modifiers = update.constitution;
                    attr.intelligence.status_effect_modifiers = update.intelligence;

                    attr.strength.bonus = attr_bonus(attr.strength.base + attr.strength.total_modifiers());
                    attr.dexterity.bonus = attr_bonus(attr.dexterity.base + attr.dexterity.total_modifiers());
                    attr.constitution.bonus = attr_bonus(attr.constitution.base + attr.constitution.total_modifiers());
                    attr.intelligence.bonus = attr_bonus(attr.intelligence.base + attr.intelligence.total_modifiers());

                    // update health and mana
                    pool.hit_points.max = player_hp_at_level(
                        attr.constitution.base + attr.constitution.total_modifiers(),
                        pool.level
                    );
                    pool.mana.max = mana_at_level(
                        attr.intelligence.base + attr.intelligence.total_modifiers(),
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
                    skill.melee.status_effect_modifiers = update.melee;
                    skill.defence.status_effect_modifiers = update.defence;
                    skill.ranged.status_effect_modifiers = update.ranged;
                    skill.magic.status_effect_modifiers = update.magic;
                }

                if let Some(dodge_chance) = update.dodge_chance {
                    if let Some(dodge) = dodges.get_mut(*entity) {
                        dodge.chance = dodge_chance;
                    } else {
                        dodges.insert(*entity, Dodge{ chance: dodge_chance }).expect("Unable to insert");
                    }
                }

                if let Some(block_chance) = update.block_chance {
                    if let Some(block) = blocks.get_mut(*entity) {
                        block.chance = block_chance;
                    } else {
                        blocks.insert(*entity, Block{ chance: block_chance }).expect("Unable to insert");
                    }
                }

                // TODO: initiative penalty
            }
        }
    }
}
