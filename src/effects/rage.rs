use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{StatusEffect, StatusEffectChanged, Duration, Name, SerializeMe, AttributeBonus, SkillBonus};

pub fn apply_rage(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Rage{duration} = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: false })
            .with(AttributeBonus{
                strength: Some(4),
                dexterity: Some(4),
                constitution: None,
                intelligence: None
            })
            .with(SkillBonus{
                melee: Some(6),
                defence: Some(-4),
                ranged: None,
                magic: None
            })
            .with(Duration{ turns: *duration })
            .with(Name{ name: "Rage".to_string() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}
