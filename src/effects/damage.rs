use rltk::RandomNumberGenerator;
use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{Pools, StatusEffect, DamageOverTime, Duration, Name, MeleeWeapon};
use crate::{Map, Player, player_xp_for_level};
use crate::{spatial, Damage, SerializeMe, RunState};
use crate::raws;
use crate::player;

pub fn calculate_damage(rng: &mut RandomNumberGenerator, damage: &Damage) -> i32 {
    let (n_dice, die_type, die_bonus) = raws::parse_dice_string(&damage.damage);
    rng.roll_dice(n_dice, die_type) + die_bonus
}

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let Some(creator) = damage.creator {
                if creator == target { return; } // prevent self damage
            }
            if let EffectType::Damage{amount} = damage.effect_type {
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

                if pool.hit_points.current < 1 {
                    add_effect(
                        damage.creator,
                        EffectType::EntityDeath,
                        Targets::Single{target}
                    );
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
                    glyph: rltk::to_cp437('‼'),
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
            }

            if xp_gain != 0 || gold_gain != 0 {
                let player_pools = pools.get_mut(source).unwrap();
                
                player_pools.xp += xp_gain;
                player_pools.gold += gold_gain;
                if player_pools.xp >= player_xp_for_level(player_pools.level) {
                    player::level_up(ecs, source, player_pools);
                    let mut runstate = ecs.fetch_mut::<RunState>();
                    *runstate = RunState::LevelUp{ attribute_points: 1, skill_points: 2 };
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
    }
}
