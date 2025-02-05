use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::{Duration, StatusEffectChanged, Name, SerializeMe, StatusEffect};

pub fn apply_effect(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::AttributeEffect{bonus, name, duration} = &effect.effect_type {
        delete_duplicate_effect(ecs, name, target);

        ecs.create_entity()
            .with(StatusEffect{ target, is_debuff: bonus.is_debuff() })
            .with(bonus.clone())
            .with(Duration{ turns: *duration })
            .with(Name{ name: name.clone() })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}

fn delete_duplicate_effect(ecs: &mut World, name: &String, target: Entity) {
    let entities = ecs.entities();
    let names = ecs.read_storage::<Name>();
    let status_effects = ecs.read_storage::<StatusEffect>();
    let bonuses = ecs.read_storage::<AttributeBonus>();

    for (effect_name, entity, status_effect, _bonus) in (&names, &entities, &status_effects, &bonuses).join() {
        if effect_name.name == *name && status_effect.target == target {
            entities.delete(entity).expect("Unable to delete entity");
        }
    }
}
