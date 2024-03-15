use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{Confusion, StatusEffect, Duration, Name, SerializeMe};

pub fn add_confusion(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion{duration} = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: true })
            .with(Confusion{})
            .with(Duration{ turns: *duration })
            .with(Name{ name: "Confusion".to_string() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
    }
}
