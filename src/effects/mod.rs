use std::sync::Mutex;
use specs::prelude::*;
use std::collections::{HashSet, VecDeque};
mod damage;
mod mana;
mod targeting;
mod particles;
mod triggers;
mod hunger;
mod stun;
mod confusion;
mod movement;
mod attributes;
mod slow;
mod rage;
mod fortress;
mod frost_shield;
pub use targeting::*;
use rltk::{FontCharType, RGB, Point};
use crate::spatial;
use super::AttributeBonus;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

#[derive(Debug, Clone)]
pub enum EffectType {
    Damage { amount: i32, hits_self: bool },
    Healing { amount: i32 },
    Mana { amount: i32 },
    Bloodstain,
    Particle { glyph: FontCharType, fg: RGB, bg: RGB, lifespan: f32 },
    ParticleProjectile { glyph: FontCharType, fg: RGB, bg: RGB, speed: f32, path: Vec<Point> },
    EntityDeath,
    ItemUse { item: Entity },
    WellFed,
    Confusion { duration: i32 },
    TeleportTo { x: i32, y: i32, depth: i32, player_only: bool },
    TriggerFire { trigger: Entity },
    AttributeEffect { bonus: AttributeBonus, name: String, duration: i32 },
    AbilityUse { ability: Entity, is_repeat: bool },
    Slow { initiative_penalty: f32, duration: i32 },
    DamageOverTime { damage: i32, duration: i32 },
    Stun { duration: i32 },
    Rage { duration: i32 },
    Fortress { duration: i32 },
    FrostShield { duration: i32 }
}

#[derive(Clone, Debug)]
pub enum Targets {
    Single { target: Entity },
    TargetList { targets: Vec<Entity> },
    Tile { tile_idx: i32 },
    Tiles { tiles: Vec<i32> }
}

#[derive(Debug)]
pub struct EffectSpawner {
    pub creator: Option<Entity>,
    pub effect_type: EffectType,
    pub targets: Targets,
    dedupe: HashSet<Entity>
}

pub fn add_effect(creator: Option<Entity>, effect_type: EffectType, targets: Targets) {
    EFFECT_QUEUE
        .lock()
        .unwrap()
        .push_back(EffectSpawner{
            creator,
            effect_type,
            targets,
            dedupe: HashSet::new()
        });
}

pub fn run_effects_queue(ecs: &mut World) {
    loop {
        let effect: Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(mut effect) = effect {
            target_applicator(ecs, &mut effect);
        } else {
            break;
        }
    }
}

fn target_applicator(ecs: &mut World, effect: &mut EffectSpawner) {
    if let EffectType::ItemUse{item} = effect.effect_type {
        triggers::item_trigger(ecs, effect.creator, item, &effect.targets);
    } else if let EffectType::TriggerFire{trigger} = effect.effect_type {
        triggers::environment_trigger(ecs, effect.creator, trigger, &effect.targets);
    } else if let EffectType::AbilityUse{ability, is_repeat} = effect.effect_type {
        triggers::ability_trigger(ecs, effect.creator, ability, &effect.targets, is_repeat);
    } else {
        match &effect.targets.clone() {
            Targets::Tile{tile_idx} => affect_tile(ecs, effect, *tile_idx),
            Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
            Targets::Single{target} => affect_entity(ecs, effect, *target),
            Targets::TargetList{targets} => targets.iter().for_each(|entity| affect_entity(ecs, effect, *entity))
        }
    }
}

fn affect_tile(ecs: &mut World, effect: &mut EffectSpawner, tile_idx: i32) {
    match &effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
        EffectType::Particle{..} => particles::particle_to_tile(ecs, tile_idx, &effect),
        EffectType::ParticleProjectile {..} => particles::projectile(ecs, tile_idx, &effect),
        _ => {
            let content = spatial::get_tile_content_clone(tile_idx as usize);
            content.iter().for_each(|entity| affect_entity(ecs, effect, *entity));
        }
    }
}

fn affect_entity(ecs: &mut World, effect: &mut EffectSpawner, target: Entity) {
    if effect.dedupe.contains(&target) { return; }
    effect.dedupe.insert(target);

    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath{..} => damage::death(ecs, effect, target),
        EffectType::Healing{..} => damage::heal_damage(ecs, effect, target),
        EffectType::Mana{..} => mana::restore_mana(ecs, effect, target),
        EffectType::Bloodstain{..} => {
            if let Some(pos) = entity_position(ecs, target) {
                damage::bloodstain(ecs, pos)
            }
        }
        EffectType::Particle{..} => {
            if let Some(pos) = entity_position(ecs, target) {
                particles::particle_to_tile(ecs, pos, &effect)
            }
        }
        EffectType::WellFed => hunger::well_fed(ecs, target),
        EffectType::Confusion{..} => confusion::apply_confusion(ecs, effect, target),
        EffectType::Stun{..} => stun::apply_stun(ecs, effect, target),
        EffectType::TeleportTo{..} => movement::apply_teleport(ecs, effect, target),
        EffectType::AttributeEffect{..} => attributes::apply_effect(ecs, effect, target),
        EffectType::Slow{..} => slow::apply_slow(ecs, effect, target),
        EffectType::DamageOverTime{..} => damage::damage_over_time(ecs, effect, target),
        EffectType::Rage{..} => rage::apply_rage(ecs, effect, target),
        EffectType::Fortress{..} => fortress::apply_fortress(ecs, effect, target),
        EffectType::FrostShield{..} => frost_shield::apply_frost_shield(ecs, effect, target),
        _ => {}
    }
}
