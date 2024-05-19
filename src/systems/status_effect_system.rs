use specs::prelude::*;
use crate::{attr_bonus, gamelog, StatusEffectChanged, AttributeBonus, Attributes, Pools, StatusEffect,
    player_hp_at_level, mana_at_level, carry_capacity_lbs};
use std::collections::HashMap;
use rltk::RGB;

#[derive(Debug)]
struct StatusUpdate {
    expired: bool,
    strength: i32,
    dexterity: i32,
    constitution: i32,
    intelligence: i32
}

pub struct StatusEffectSystem {}

impl<'a> System<'a> for StatusEffectSystem {
    type SystemData = (
        WriteStorage<'a, StatusEffectChanged>,
        Entities<'a>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, AttributeBonus>,
        ReadStorage<'a, StatusEffect>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut status_dirty, entities, mut pools, mut attributes,
            player, attribute_bonuses, statuses) = data;

        if status_dirty.is_empty() { return; }

        let mut to_update: HashMap<Entity, StatusUpdate> = HashMap::new();
        for (entity, dirty) in (&entities, &status_dirty).join() {
            to_update.insert(entity, StatusUpdate {
                expired: dirty.expired,
                strength: 0, 
                dexterity: 0,
                constitution: 0,
                intelligence: 0
            });
        }
        status_dirty.clear();

        // total up status effect modifiers
        for (status, bonus) in (&statuses, &attribute_bonuses).join() {
            if to_update.contains_key(&status.target) {
                let totals = to_update.get_mut(&status.target).unwrap();
                let modifier = if totals.expired { -1 } else { 1 };
                totals.strength += bonus.strength.unwrap_or(0) * modifier;
                totals.dexterity += bonus.dexterity.unwrap_or(0) * modifier;
                totals.constitution += bonus.constitution.unwrap_or(0) * modifier;
                totals.intelligence += bonus.intelligence.unwrap_or(0) * modifier;
            }
        }

        // apply to pools
        for (entity, update) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                if let Some(attr) = attributes.get_mut(*entity) {
                    attr.strength.modifiers += update.strength;
                    attr.dexterity.modifiers += update.dexterity;
                    attr.constitution.modifiers += update.constitution;
                    attr.intelligence.modifiers += update.intelligence;

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
            }
        }
    }
}
