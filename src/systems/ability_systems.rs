use specs::prelude::*;
use crate::raws::{self, find_ability_entity_by_name, parse_particle, parse_particle_line};
use crate::{apply_effects, Ability, AreaOfEffect, Block, Confusion, Damage, DamageOverTime, Dodge, Duration, ExtraDamage, Fortress, FrostShield, KnownAbilities, 
    KnownAbility, Rage, Ranged, RunState, SelfDamage, SpawnParticleBurst, SpawnParticleLine, Stun, WantsToLearnAbility, WantsToLevelAbility, Healing, RestoresMana,
    MagicMapping, TownPortal, Food, SingleActivation, TeachesAbility, Slow};

pub struct LearnAbilitySystem {}

impl<'a> System<'a> for LearnAbilitySystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Ability>,
        WriteStorage<'a, KnownAbilities>,
        WriteStorage<'a, WantsToLearnAbility>,
        ReadExpect<'a, RunState>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, lazy, abilities, mut known_ability_lists, 
            mut wants_learn, runstate) = data;

        if wants_learn.count() < 1 { return; }
        if *runstate != RunState::Ticking { return; }

        for (entity, learn) in (&entities, &wants_learn).join() {
            let ability_entity = find_ability_entity_by_name(&learn.ability_name, &abilities, &entities).unwrap();
            let ability = abilities.get(ability_entity).unwrap();

            let mut lb = lazy.create_entity(&entities);
            apply_effects!(ability.levels[&1].effects, lb);

            let known_ability_list = &mut known_ability_lists.get_mut(entity).unwrap().abilities;
            known_ability_list.push(lb.with(KnownAbility{
                name: ability.name.clone(),
                level: 1,
                mana_cost: ability.levels[&1].mana_cost.unwrap()
            }).build());
        }

        wants_learn.clear();
    }
}


pub struct LevelAbilitySystem {}

impl<'a> System<'a> for LevelAbilitySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Ability>,
        WriteStorage<'a, KnownAbilities>,
        WriteStorage<'a, KnownAbility>,
        WriteStorage<'a, WantsToLevelAbility>,
        WriteStorage<'a, Ranged>,
        WriteStorage<'a, Damage>,
        WriteStorage<'a, ExtraDamage>,
        WriteStorage<'a, SelfDamage>,
        WriteStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, Duration>,
        WriteStorage<'a, Stun>,
        WriteStorage<'a, DamageOverTime>,
        WriteStorage<'a, Rage>,
        WriteStorage<'a, Block>,
        WriteStorage<'a, Fortress>,
        WriteStorage<'a, FrostShield>,
        WriteStorage<'a, Dodge>,
        WriteStorage<'a, Healing>,
        WriteStorage<'a, Slow>,
        WriteStorage<'a, SpawnParticleLine>,
        WriteStorage<'a, SpawnParticleBurst>,
        ReadExpect<'a, RunState>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, abilities, mut known_ability_lists, mut known_abilities, mut wants_level,
            mut ranged, mut damage, mut extra_damage, mut self_damage, mut aoe, mut confusion,
            mut duration, mut stun, mut dot, mut rage, mut block, mut fortress, mut frost_shield,
            mut dodge, mut healing, mut slow, mut particle_line, mut particle_burst, runstate) = data;

        if wants_level.count() < 1 { return; }
        if *runstate != RunState::Ticking { return; }

        for (entity, want_level) in (&entities, &wants_level).join() {
            let ability_entity = find_ability_entity_by_name(&want_level.ability_name, &abilities, &entities).unwrap();
            let ability = abilities.get(ability_entity).unwrap();

            let entity_known_ability_list = &known_ability_lists.get_mut(entity).unwrap().abilities;
            for entity in entity_known_ability_list.iter() {
                let known_ability = known_abilities.get_mut(*entity).unwrap();
                if known_ability.name != ability.name { continue; }

                known_ability.level += 1;
                known_ability.mana_cost = ability.levels[&known_ability.level].mana_cost.unwrap();

                // update current known ability with effects from the next ability level
                let new_effects = &ability.levels.get(&known_ability.level).unwrap().effects;

                // Ranged
                if let Some(new_range_string) = new_effects.get("ranged") {
                    let new_range = new_range_string.parse::<i32>().unwrap();
                    if let Some(current_ranged) = ranged.get_mut(*entity) {
                        current_ranged.range = new_range;
                    } else {
                        ranged.insert(*entity, Ranged{ range: new_range }).expect("Unable to insert");
                    }
                }

                // Damage
                if let Some(new_damage_string) = new_effects.get("damage") {
                    if let Some(current_damage) = damage.get_mut(*entity) {
                        current_damage.damage = new_damage_string.clone();
                    } else {
                        damage.insert(*entity, Damage{ damage: new_damage_string.clone() }).expect("Unable to insert");
                    }
                }

                // Extra Damage
                if let Some(new_extra_damage_string) = new_effects.get("extra_damage") {
                    if let Some(current_extra_damage) = extra_damage.get_mut(*entity) {
                        current_extra_damage.damage = new_extra_damage_string.clone();
                    } else {
                        extra_damage.insert(*entity, ExtraDamage{ damage: new_extra_damage_string.clone() }).expect("Unable to insert");
                    }
                }

                // Self Damage
                if let Some(new_self_damage) = new_effects.get("self_damage") {
                    if let Some(current_self_damage) = self_damage.get_mut(*entity) {
                        current_self_damage.damage = new_self_damage.clone();
                    } else {
                        self_damage.insert(*entity, SelfDamage{ damage: new_self_damage.clone() }).expect("Unable to insert");
                    }
                }

                // Area of Effect
                if let Some(new_aoe_string) = new_effects.get("area_of_effect") {
                    let new_radius = new_aoe_string.parse::<i32>().unwrap();
                    if let Some(current_aoe) = aoe.get_mut(*entity) {
                        current_aoe.radius = new_radius;
                    } else {
                        aoe.insert(*entity, AreaOfEffect{ radius: new_radius }).expect("Unable to insert");
                    }
                }

                // Confusion
                if let Some(new_confusion_string) = new_effects.get("confusion") {
                    let new_duration = new_confusion_string.parse::<i32>().unwrap();
                    if confusion.get(*entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        confusion.insert(*entity, Confusion{}).expect("Unable to insert");
                        duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Stun
                if let Some(new_stun_string) = new_effects.get("stun") {
                    let new_duration = new_stun_string.parse::<i32>().unwrap();
                    if stun.get(*entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        stun.insert(*entity, Stun{}).expect("Unable to insert");
                        duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Duration
                if let Some(new_duration_string) = new_effects.get("duration") {
                    let new_duration = new_duration_string.parse::<i32>().unwrap();
                    if let Some(current_duration) = duration.get_mut(*entity) {
                        current_duration.turns = new_duration;
                    } else {
                        duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Damage Over Time
                if let Some(new_dot_string) = new_effects.get("damage_over_time") {
                    let new_dot = new_dot_string.parse::<i32>().unwrap();
                    if let Some(current_dot) = dot.get_mut(*entity) {
                        current_dot.damage = new_dot;
                    } else {
                        dot.insert(*entity, DamageOverTime{ damage: new_dot }).expect("Unable to insert");
                    }
                }

                // Rage
                if let Some(new_rage_string) = new_effects.get("rage") {
                    let new_duration = new_rage_string.parse::<i32>().unwrap();
                    if rage.get(*entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        rage.insert(*entity, Rage{}).expect("Unable to insert");
                        duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Block
                if let Some(new_block_string) = new_effects.get("block") {
                    let new_chance = new_block_string.parse::<f32>().unwrap();
                    if let Some(current_block) = block.get_mut(*entity) {
                        current_block.chance = new_chance;
                    } else {
                        block.insert(*entity, Block{ chance: new_chance }).expect("Unable to insert");
                    }
                }

                // Fortress
                if let Some(new_fortress_string) = new_effects.get("fortress") {
                    let new_duration = new_fortress_string.parse::<i32>().unwrap();
                    if fortress.get(*entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        fortress.insert(*entity, Fortress{}).expect("Unable to insert");
                        duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Frost Shield
                if let Some(new_frost_shield_string) = new_effects.get("frost_shield") {
                    let new_duration = new_frost_shield_string.parse::<i32>().unwrap();
                    if frost_shield.get(*entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        frost_shield.insert(*entity, FrostShield{}).expect("Unable to insert");
                        duration.insert(*entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Dodge
                if let Some(new_dodge_string) = new_effects.get("dodge") {
                    let new_chance = new_dodge_string.parse::<f32>().unwrap();
                    if let Some(current_dodge) = dodge.get_mut(*entity) {
                        current_dodge.chance = new_chance;
                    } else {
                        dodge.insert(*entity, Dodge{ chance: new_chance }).expect("Unable to insert");
                    }
                }

                // Healing
                if let Some(new_healing_string) = new_effects.get("healing") {
                    let new_healing = new_healing_string.parse::<i32>().unwrap();
                    if let Some(current_healing) = healing.get_mut(*entity) {
                        current_healing.heal_amount = new_healing;
                    } else {
                        healing.insert(*entity, Healing{ heal_amount: new_healing }).expect("Unable to insert");
                    }
                }

                // Slow
                if let Some(new_slow_string) = new_effects.get("slow") {
                    let new_initiative_penalty = new_slow_string.parse::<f32>().unwrap();
                    if let Some(current_slow) = slow.get_mut(*entity) {
                        current_slow.initiative_penalty = new_initiative_penalty;
                    } else {
                        slow.insert(*entity, Slow{ initiative_penalty: new_initiative_penalty }).expect("Unable to insert");
                    }
                }

                // Particle Line
                if let Some(new_particle_string) = new_effects.get("particle_line") {
                    let new_particle = raws::parse_particle_line(new_particle_string);
                    particle_line.insert(*entity, new_particle).expect("Unable to insert");
                }

                // Particle Burst
                if let Some(new_particle_string) = new_effects.get("particle") {
                    let new_particle = raws::parse_particle(new_particle_string);
                    particle_burst.insert(*entity, new_particle).expect("Unable to insert");
                }
            }
        }

        wants_level.clear();
    }
}
