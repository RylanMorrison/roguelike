use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{StatusEffect, StatusEffectChanged, Slow, Duration, Name, SerializeMe};

pub fn apply_slow(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Slow{initiative_penalty, duration} = &effect.effect_type {
        delete_duplicate_effect(ecs, target);

        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: *initiative_penalty > 0.0 })
            .with(Slow{ initiative_penalty: *initiative_penalty })
            .with(Duration{ turns: *duration })
            .with( // TODO: separate slow and haste
                if *initiative_penalty > 0.0 {
                    Name{ name: "Slowed".to_string() }
                } else {
                    Name{ name: "Hasted".to_string() }
                }
            )
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}

fn delete_duplicate_effect(ecs: &mut World, target: Entity) {
    let entities = ecs.entities();
    let status_effects = ecs.read_storage::<StatusEffect>();
    let slows = ecs.read_storage::<Slow>();

    for (entity, status_effect, _slow) in (&entities, &status_effects, &slows).join() {
        if status_effect.target == target {
            entities.delete(entity).expect("Unable to delete entity");
        }
    }
}
