use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{StatusEffect, StatusEffectChanged, SkillBonus, Slow, Duration, Name, SerializeMe};

pub fn apply_frost_shield(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::FrostShield{duration} = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: false })
            .with(Slow{ initiative_penalty: 2.0 })
            .with(SkillBonus{
                melee: None,
                defence: Some(8),
                ranged: None,
                magic: None
            })
            .with(Duration{ turns: *duration })
            .with(Name{ name: "Frost Shield".to_string() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}
