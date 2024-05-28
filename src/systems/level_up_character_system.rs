use specs::prelude::*;
use crate::{carry_capacity_lbs, mana_at_level, player_hp_at_level, Attributes, CharacterClass, EquipmentChanged, PendingCharacterLevelUp,
    Pools, RunState, Skills, WantsToLearnAbility, WantsToLevelAbility};
use crate::gamelog;
use rltk::RGB;

pub struct LevelUpCharacterSystem {}

impl<'a> System<'a> for LevelUpCharacterSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        WriteStorage<'a, CharacterClass>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, PendingCharacterLevelUp>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, WantsToLearnAbility>,
        WriteStorage<'a, WantsToLevelAbility>
    );
    
    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut pools, mut attributes, 
            mut skills, mut character_classes, mut equip_dirty,
            mut pending_character_level_ups, runstate, 
            mut learn_abilities, mut level_abilities) = data;

        // TODO make this more generic when other entities can level up
        if pending_character_level_ups.count() < 1 { return; }
        if *runstate != RunState::Ticking { return; }

        let level_up = pending_character_level_ups.get(*player_entity).unwrap();
        let player_pools = pools.get_mut(*player_entity).unwrap();

        player_pools.level += 1;
        player_pools.xp = 0; // loses overflow xp?

        let player_class = character_classes.get_mut(*player_entity).unwrap();
        let player_passives = &mut player_class.passives;
        let player_attributes = attributes.get_mut(*player_entity).unwrap();
        let player_skills = skills.get_mut(*player_entity).unwrap();
        for (name, passive) in level_up.passives.iter() {
            if passive.current_level < 1 { continue; }

            if player_passives[name].current_level != passive.current_level {
                let current_passive = player_passives.get_mut(name).unwrap();
                current_passive.current_level = passive.current_level;

                if let Some(attribute_bonus) = &current_passive.levels[&current_passive.current_level].attribute_bonus {
                    if let Some(strength) = attribute_bonus.strength {
                        player_attributes.strength.base += strength;
                    }
                    if let Some(dexterity) = attribute_bonus.dexterity {
                        player_attributes.dexterity.base += dexterity;
                    }
                    if let Some(constitution) = attribute_bonus.constitution {
                        player_attributes.constitution.base += constitution;
                    }
                    if let Some(intelligence) = attribute_bonus.intelligence {
                        player_attributes.intelligence.base += intelligence;
                    }
                }

                player_pools.hit_points.max = player_hp_at_level(
                    player_attributes.constitution.base + player_attributes.constitution.total_modifiers(),
                    player_pools.level
                );
                player_pools.hit_points.current = player_pools.hit_points.max;
                player_pools.mana.max = mana_at_level(
                    player_attributes.intelligence.base + player_attributes.intelligence.total_modifiers(),
                    player_pools.level
                );
                player_pools.mana.current = player_pools.mana.max;

                if player_pools.total_weight > carry_capacity_lbs(&player_attributes.strength) {
                    // overburdened
                    player_pools.total_initiative_penalty += 4.0;
                    gamelog::Logger::new().colour(RGB::named(rltk::ORANGE)).append("You are overburdened!").log();
                }

                if let Some(skill_bonus) = &current_passive.active_level().skill_bonus {
                    if let Some(melee) = skill_bonus.melee {
                        player_skills.melee.base += melee;
                    }
                    if let Some(defence) = skill_bonus.defence {
                        player_skills.defence.base += defence;
                    }
                    if let Some(ranged) = skill_bonus.ranged {
                        player_skills.ranged.base += ranged;
                    }
                    if let Some(magic) = skill_bonus.magic {
                        player_skills.magic.base += magic;
                    }
                }

                if let Some(learn_ability) = &current_passive.active_level().learn_ability {
                    learn_abilities.insert(*player_entity, WantsToLearnAbility{ ability_name: learn_ability.clone() }).expect("Unable to insert");
                }

                if let Some(level_ability) = &current_passive.active_level().level_ability {
                    level_abilities.insert(*player_entity, WantsToLevelAbility{ ability_name: level_ability.clone() }).expect("Unable to insert");
                }
            }
        }

        equip_dirty.insert(*player_entity, EquipmentChanged{}).expect("Unable to insert");
        pending_character_level_ups.clear();
    }
}