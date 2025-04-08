use specs::prelude::*;
use crate::raws;
use crate::raws::{find_ability_entity_by_name, parse_particle, parse_particle_line, parse_ranged_string};
use crate::{apply_effects, Ability, AbilityType, AreaOfEffect, Block, Confusion, Damage, DamageOverTime, Dodge, Duration, Food, Fortress, 
    FrostShield, Healing, KnownAbilities, KnownAbility, MagicMapping, Rage, Ranged, RestoresMana, RunState, SelfDamage, SingleActivation, 
    Slow, SpawnParticleBurst, SpawnParticleLine, Stun, TeachesAbility, TownPortal, WantsToLearnAbility, WantsToLevelAbility};

pub struct LearnAbilitySystem {}

impl<'a> System<'a> for LearnAbilitySystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Ability>,
        WriteStorage<'a, KnownAbilities>,
        WriteStorage<'a, WantsToLearnAbility>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, Dodge>,
        WriteStorage<'a, Block>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, lazy, abilities, mut known_ability_lists, 
            mut wants_learn, runstate, mut dodges, mut blocks) = data;

        if wants_learn.count() < 1 { return; }
        if *runstate != RunState::Ticking {
            // entities can start with abilities
            if *runstate != RunState::PreRun {
                return;
            }
        }

        for (entity, learn) in (&entities, &wants_learn).join() {
            let ability_entity = find_ability_entity_by_name(&learn.ability_name, &abilities, &entities).unwrap();
            let ability = abilities.get(ability_entity).unwrap();
            let effects = &ability.levels[&learn.level].effects;

            let mut lb = lazy.create_entity(&entities);
            apply_effects!(raws, effects, lb);

            let known_ability_list = &mut known_ability_lists.get_mut(entity).unwrap().abilities;
            let known_ability_entity = lb.with(KnownAbility{
                name: ability.name.clone(),
                level: learn.level,
                mana_cost: ability.levels[&learn.level].mana_cost.unwrap_or(0),
                ability_type: ability.ability_type.clone()
            }).build();
            known_ability_list.push(known_ability_entity);

            if ability.ability_type == AbilityType::Passive {
                // apply passive effects of abilities to the user
                if let Some(dodge_chance) = effects.get("dodge") {
                    dodges.insert(entity, Dodge{ chance: dodge_chance.parse::<f32>().unwrap() }).expect("Unable to insert");
                }
                if let Some(block_chance) = effects.get("block") {
                    blocks.insert(entity, Block{ chance: block_chance.parse::<f32>().unwrap() }).expect("Unable to insert");
                }
            }
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
            mut ranged, mut damage, mut self_damage, mut aoe, mut confusion, mut duration,
            mut stun, mut dot, mut rage, mut blocks, mut fortress, mut frost_shield, mut dodges,
            mut healing, mut slow, mut particle_line, mut particle_burst, runstate) = data;

        if wants_level.count() < 1 { return; }
        if *runstate != RunState::Ticking { return; }

        for (entity, want_level) in (&entities, &wants_level).join() {
            let ability_entity = find_ability_entity_by_name(&want_level.ability_name, &abilities, &entities).unwrap();
            let ability = abilities.get(ability_entity).unwrap();

            let entity_known_ability_list = &known_ability_lists.get_mut(entity).unwrap().abilities;
            for ability_entity in entity_known_ability_list.iter() {
                let known_ability = known_abilities.get_mut(*ability_entity).unwrap();
                if known_ability.name != ability.name { continue; }

                known_ability.level += 1;
                known_ability.mana_cost = ability.levels[&known_ability.level].mana_cost.unwrap_or(0);

                // update current known ability with effects from the next ability level
                let new_effects = &ability.levels.get(&known_ability.level).unwrap().effects;

                // Ranged
                if let Some(new_range_string) = new_effects.get("ranged") {
                    let (new_min, new_max) = parse_ranged_string(new_range_string.clone());
                    if let Some(current_ranged) = ranged.get_mut(*ability_entity) {
                        current_ranged.min_range = new_min;
                        current_ranged.max_range = new_max;
                    } else {
                        ranged.insert(*ability_entity, Ranged{ min_range: new_min, max_range: new_max }).expect("Unable to insert");
                    }
                }

                // Damage
                if let Some(new_damage_string) = new_effects.get("damage") {
                    if let Some(current_damage) = damage.get_mut(*ability_entity) {
                        current_damage.damage = new_damage_string.clone();
                    } else {
                        damage.insert(*ability_entity, Damage{ damage: new_damage_string.clone() }).expect("Unable to insert");
                    }
                }

                // Self Damage
                if let Some(new_self_damage) = new_effects.get("self_damage") {
                    if let Some(current_self_damage) = self_damage.get_mut(*ability_entity) {
                        current_self_damage.damage = new_self_damage.clone();
                    } else {
                        self_damage.insert(*ability_entity, SelfDamage{ damage: new_self_damage.clone() }).expect("Unable to insert");
                    }
                }

                // Area of Effect
                if let Some(new_aoe_string) = new_effects.get("area_of_effect") {
                    let new_radius = new_aoe_string.parse::<i32>().unwrap();
                    if let Some(current_aoe) = aoe.get_mut(*ability_entity) {
                        current_aoe.radius = new_radius;
                    } else {
                        aoe.insert(*ability_entity, AreaOfEffect{ radius: new_radius }).expect("Unable to insert");
                    }
                }

                // Confusion
                if let Some(new_confusion_string) = new_effects.get("confusion") {
                    let new_duration = new_confusion_string.parse::<i32>().unwrap();
                    if confusion.get(*ability_entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*ability_entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        confusion.insert(*ability_entity, Confusion{}).expect("Unable to insert");
                        duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Stun
                if let Some(new_stun_string) = new_effects.get("stun") {
                    let new_duration = new_stun_string.parse::<i32>().unwrap();
                    if stun.get(*ability_entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*ability_entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        stun.insert(*ability_entity, Stun{}).expect("Unable to insert");
                        duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Duration
                if let Some(new_duration_string) = new_effects.get("duration") {
                    let new_duration = new_duration_string.parse::<i32>().unwrap();
                    if let Some(current_duration) = duration.get_mut(*ability_entity) {
                        current_duration.turns = new_duration;
                    } else {
                        duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Damage Over Time
                if let Some(new_dot_string) = new_effects.get("damage_over_time") {
                    let new_dot = new_dot_string.parse::<i32>().unwrap();
                    if let Some(current_dot) = dot.get_mut(*ability_entity) {
                        current_dot.damage = new_dot;
                    } else {
                        dot.insert(*ability_entity, DamageOverTime{ damage: new_dot }).expect("Unable to insert");
                    }
                }

                // Rage
                if let Some(new_rage_string) = new_effects.get("rage") {
                    let new_duration = new_rage_string.parse::<i32>().unwrap();
                    if rage.get(*ability_entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*ability_entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        rage.insert(*ability_entity, Rage{}).expect("Unable to insert");
                        duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Block
                if let Some(new_block_string) = new_effects.get("block") {
                    let new_chance = new_block_string.parse::<f32>().unwrap();
                    // ability
                    if let Some(ability_block) = blocks.get_mut(*ability_entity) {
                        ability_block.chance = new_chance;
                    } else {
                        blocks.insert(*ability_entity, Block{ chance: new_chance }).expect("Unable to insert");
                    }
                    // user
                    if let Some(user_block) = blocks.get_mut(entity) {
                        user_block.chance = new_chance
                    } else {
                        blocks.insert(entity, Block{ chance: new_chance }).expect("Unable to insert");
                    }
                }

                // Fortress
                if let Some(new_fortress_string) = new_effects.get("fortress") {
                    let new_duration = new_fortress_string.parse::<i32>().unwrap();
                    if fortress.get(*ability_entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*ability_entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        fortress.insert(*ability_entity, Fortress{}).expect("Unable to insert");
                        duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Frost Shield
                if let Some(new_frost_shield_string) = new_effects.get("frost_shield") {
                    let new_duration = new_frost_shield_string.parse::<i32>().unwrap();
                    if frost_shield.get(*ability_entity).is_some() {
                        if let Some(current_duration) = duration.get_mut(*ability_entity) {
                            current_duration.turns = new_duration;
                        } else {
                            duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                        }
                    } else {
                        frost_shield.insert(*ability_entity, FrostShield{}).expect("Unable to insert");
                        duration.insert(*ability_entity, Duration{ turns: new_duration }).expect("Unable to insert");
                    }
                }

                // Dodge
                if let Some(new_dodge_string) = new_effects.get("dodge") {
                    let new_chance = new_dodge_string.parse::<f32>().unwrap();
                    // ability
                    if let Some(ability_dodge) = dodges.get_mut(*ability_entity) {
                        ability_dodge.chance = new_chance;
                    } else {
                        dodges.insert(*ability_entity, Dodge{ chance: new_chance }).expect("Unable to insert");
                    }
                    // user
                    if let Some(user_dodge) = dodges.get_mut(entity) {
                        user_dodge.chance = new_chance
                    } else {
                        dodges.insert(entity, Dodge{ chance: new_chance }).expect("Unable to insert");
                    }
                }

                // Healing
                if let Some(new_healing_string) = new_effects.get("healing") {
                    let new_healing = new_healing_string.parse::<i32>().unwrap();
                    if let Some(current_healing) = healing.get_mut(*ability_entity) {
                        current_healing.heal_amount = new_healing;
                    } else {
                        healing.insert(*ability_entity, Healing{ heal_amount: new_healing }).expect("Unable to insert");
                    }
                }

                // Slow
                if let Some(new_slow_string) = new_effects.get("slow") {
                    let new_initiative_penalty = new_slow_string.parse::<f32>().unwrap();
                    if let Some(current_slow) = slow.get_mut(*ability_entity) {
                        current_slow.initiative_penalty = new_initiative_penalty;
                    } else {
                        slow.insert(*ability_entity, Slow{ initiative_penalty: new_initiative_penalty }).expect("Unable to insert");
                    }
                }

                // Particle Line
                if let Some(new_particle_string) = new_effects.get("particle_line") {
                    let new_particle = raws::parse_particle_line(new_particle_string);
                    particle_line.insert(*ability_entity, new_particle).expect("Unable to insert");
                }

                // Particle Burst
                if let Some(new_particle_string) = new_effects.get("particle") {
                    let new_particle = raws::parse_particle(new_particle_string);
                    particle_burst.insert(*ability_entity, new_particle).expect("Unable to insert");
                }
            }
        }

        wants_level.clear();
    }
}
