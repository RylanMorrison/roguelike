use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{Confusion, StatusEffect, StatusEffectChanged, Duration, Name, SerializeMe};

pub fn apply_confusion(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion{duration} = &effect.effect_type {
        delete_duplicate_effect(ecs, target);

        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: true })
            .with(Confusion{})
            .with(Duration{ turns: *duration })
            .with(Name{ name: "Confused".to_string() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}

fn delete_duplicate_effect(ecs: &mut World, target: Entity) {
    let entities = ecs.entities();
    let status_effects = ecs.read_storage::<StatusEffect>();
    let confusions = ecs.read_storage::<Confusion>();

    for (entity, status_effect, _confusion) in (&entities, &status_effects, &confusions).join() {
        if status_effect.target == target {
            entities.delete(entity).expect("Unable to delete entity");
        }
    }
}
