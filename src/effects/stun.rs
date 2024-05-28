use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{Stun, StatusEffect, StatusEffectChanged, Duration, Name, SerializeMe};

pub fn apply_stun(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Stun{duration} = &effect.effect_type {
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
