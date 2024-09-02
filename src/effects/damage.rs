use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{Pools, StatusEffect, StatusEffectChanged, DamageOverTime, Duration, Name, Item};
use crate::{gamelog, player_xp_for_level, KnownAbility, Map, Player};
use crate::{spatial, SerializeMe, RunState};
use crate::player;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    let names = ecs.read_storage::<Name>();
    let player_entity = ecs.fetch::<Entity>();

    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let EffectType::Damage{amount, hits_self} = damage.effect_type {
                if let Some(creator) = damage.creator {
                    if creator == target && !hits_self { return; } // prevent self damage
                    if creator == *player_entity {
                        gamelog::record_event("Damage Dealt", amount);
                    }
                }
                pool.hit_points.current -= amount;
                add_effect(
                    None,
                    EffectType::Bloodstain,
                    Targets::Single{target}
                );
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: rltk::to_cp437('‼'),
                        fg: rltk::RGB::named(rltk::ORANGE),
                        bg: rltk::RGB::named(rltk::BLACK),
                        lifespan: 200.0
                    },
                    Targets::Single{target}
                );
                if target == *player_entity {
                    gamelog::record_event("Damage Taken", amount);
                }

                if pool.hit_points.current < 1 {
                    add_effect(
                        damage.creator,
                        EffectType::EntityDeath,
                        Targets::Single{target}
                    );
                }
                if damage.creator.is_none() {
                    rltk::console::log(format!("{:?}", damage));
                    return;
                }

                let items = ecs.read_storage::<Item>();
                let known_abilities = ecs.read_storage::<KnownAbility>();

                // TODO clean this up
                if let Some(item) = items.get(damage.creator.unwrap()) {
                    gamelog::Logger::new()
                        .item_name(item)
                        .append("deals")
                        .damage(amount)
                        .append("damage to")
                        .append(&names.get(target).unwrap().name)
                        .log();
                } else if let Some(known_ability) = known_abilities.get(damage.creator.unwrap()) {
                    gamelog::Logger::new()
                        .ability_name(&known_ability.name)
                        .append("deals")
                        .damage(amount)
                        .append("damage to")
                        .append(&names.get(target).unwrap().name)
                        .log();
                } else {
                    gamelog::Logger::new()
                        .character_name(&names.get(damage.creator.unwrap()).unwrap().name)
                        .append("deals")
                        .damage(amount)
                        .append("damage to")
                        .append(&names.get(target).unwrap().name)
                        .log();
                }
            }
        }
    }
}

pub fn heal_damage(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Healing{amount} = effect.effect_type {
            pool.hit_points.current = i32::min(pool.hit_points.max, pool.hit_points.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: rltk::to_cp437('♥'),
                    fg: rltk::RGB::named(rltk::GREEN),
                    bg: rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0
                },
                Targets::Single{target}
            );
        }
    }
}

pub fn bloodstain(ecs: &mut World, tile_idx: i32) {
    let mut map = ecs.fetch_mut::<Map>();
    map.bloodstains.insert(tile_idx as usize);
}

pub fn death(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut xp_gain = 0;
    let mut gold_gain = 0;

    let mut pools = ecs.write_storage::<Pools>();

    if let Some(pos) = entity_position(ecs, target) {
        spatial::remove_entity(target, pos as usize);
    }

    if let Some(source) = effect.creator {
        if ecs.read_storage::<Player>().get(source).is_some() {
            if let Some(pools) = pools.get(target) {
                xp_gain += pools.level * 100;
                gold_gain += pools.gold;
                gamelog::record_event("Kill", 1);
            }

            if xp_gain != 0 || gold_gain != 0 {
                let player_pools = pools.get_mut(source).unwrap();
                
                player_pools.xp += xp_gain;
                player_pools.gold += gold_gain;
                if player_pools.xp >= player_xp_for_level(player_pools.level) {
                    player::level_up(ecs, source, player_pools);
                    let mut runstate = ecs.fetch_mut::<RunState>();
                    *runstate = RunState::LevelUp;
                }
            }
        }
    }
}

pub fn damage_over_time(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::DamageOverTime{damage, duration} = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: true })
            .with(DamageOverTime{ damage: *damage })
            .with(Duration{ turns: *duration  })
            .with(Name{ name: "Damage Over Time".to_string() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}
