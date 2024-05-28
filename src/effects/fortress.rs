use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{StatusEffect, StatusEffectChanged, Duration, Name, SerializeMe, AttributeBonus, SkillBonus, Slow};

pub fn apply_fortress(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Fortress{duration} = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: false })
            .with(AttributeBonus{
                strength: None,
                dexterity: Some(-2),
                constitution: Some(4),
                intelligence: None
            })
            .with(SkillBonus{
                melee: Some(-2),
                defence: Some(6),
                ranged: Some(-4),
                magic: Some(-4)
            })
            .with(Slow{ initiative_penalty: 3.0 })
            .with(Duration{ turns: *duration })
            .with(Name{ name: "Fortress".to_string() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}
