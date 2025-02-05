use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{StatusEffect, StatusEffectChanged, SkillBonus, Slow, Duration, Name, SerializeMe, FrostShield};

pub fn apply_frost_shield(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::FrostShield{duration} = &effect.effect_type {
        delete_duplicate_effect(ecs, target);

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
            .with(FrostShield{})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}

fn delete_duplicate_effect(ecs: &mut World, target: Entity) {
    let entities = ecs.entities();
    let status_effects = ecs.read_storage::<StatusEffect>();
    let frost_shields = ecs.read_storage::<FrostShield>();

    for (entity, status_effect, _frost_shield) in (&entities, &status_effects, &frost_shields).join() {
        if status_effect.target == target {
            entities.delete(entity).expect("Unable to delete entity");
        }
    }
}
