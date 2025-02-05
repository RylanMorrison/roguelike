use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{Stun, StatusEffect, StatusEffectChanged, Duration, Name, SerializeMe};

pub fn apply_stun(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Stun{duration} = &effect.effect_type {
        delete_duplicate_effect(ecs, target);

        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: true })
            .with(Stun{})
            .with(Duration{ turns: *duration })
            .with(Name{ name: "Stunned".to_string() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}

fn delete_duplicate_effect(ecs: &mut World, target: Entity) {
    let entities = ecs.entities();
    let status_effects = ecs.read_storage::<StatusEffect>();
    let stuns = ecs.read_storage::<Stun>();

    for (entity, status_effect, _stun) in (&entities, &status_effects, &stuns).join() {
        if status_effect.target == target {
            entities.delete(entity).expect("Unable to delete entity");
        }
    }
}
