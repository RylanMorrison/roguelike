use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{StatusEffect, Duration, Name, EquipmentChanged, SerializeMe};

pub fn apply_effect(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::AttributeEffect{bonus, name, duration} = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: bonus.is_debuff() })
            .with(bonus.clone())
            .with(Duration{ turns: *duration })
            .with(Name{ name: name.clone() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<EquipmentChanged>().insert(target, EquipmentChanged{}).expect("Insert failed");
    }
}
