use specs::{prelude::*, saveload::SimpleMarker, saveload::MarkedBuilder};
use super::*;
use crate::components::{StatusEffect, StatusEffectChanged, Duration, Name, SerializeMe, AttributeBonus, SkillBonus, Faction, Rage};
use crate::raws::{RAWS, faction_reaction, Reaction};

pub fn apply_rage(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Rage{duration} = &effect.effect_type {
        if let Some(creator) = effect.creator {
            let factions = ecs.read_storage::<Faction>();
            if let Some(creator_faction) = factions.get(creator) {
                if let Some(target_faction) = factions.get(target) {
                    let reaction = faction_reaction(
                        &creator_faction.name,
                        &target_faction.name,
                        &RAWS.lock().unwrap()
                    );
                    // don't apply rage to enemies
                    if reaction == Reaction::Attack { return; }
                }
            }
        }

        delete_duplicate_effect(ecs, target);

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
            .with(Rage{})
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.write_storage::<StatusEffectChanged>().insert(target, StatusEffectChanged{}).expect("Insert failed");
    }
}

fn delete_duplicate_effect(ecs: &mut World, target: Entity) {
    let entities = ecs.entities();
    let status_effects = ecs.read_storage::<StatusEffect>();
    let rages = ecs.read_storage::<Rage>();

    for (entity, status_effect, _rage) in (&entities, &status_effects, &rages).join() {
        if status_effect.target == target {
            entities.delete(entity).expect("Unable to delete entity");
        }
    }
}
