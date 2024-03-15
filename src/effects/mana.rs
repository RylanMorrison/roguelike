use specs::prelude::*;
use super::*;
use crate::components::Pools;

pub fn restore_mana(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Mana{amount} = effect.effect_type {
            pool.mana.current = i32::min(pool.mana.max, pool.mana.current + amount);
            add_effect(
                None,
                EffectType::Particle{
                    glyph: rltk::to_cp437('♦'),
                    fg: rltk::RGB::named(rltk::BLUE),
                    bg: rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0
                },
                Targets::Single{target}
            );
        }
    }
}
