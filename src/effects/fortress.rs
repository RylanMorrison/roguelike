use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::{components::{AttributeBonus, Duration, Name, SerializeMe, SkillBonus, Slow, StatusEffect, StatusEffectChanged}, Fortress};

pub fn apply_fortress(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Fortress{duration} = &effect.effect_type {
        delete_duplicate_effect(ecs, target);

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
            .with(Fortress{})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}

fn delete_duplicate_effect(ecs: &mut World, target: Entity) {
    let entities = ecs.entities();
    let status_effects = ecs.read_storage::<StatusEffect>();
    let fortresses = ecs.read_storage::<Fortress>();

    for (entity, status_effect, _fortress) in (&entities, &status_effects, &fortresses).join() {
        if status_effect.target == target {
            entities.delete(entity).expect("Unable to delete entity");
        }
    }
}
