use specs::prelude::*;
use crate::{mana_at_level, player_hp_at_level, Attributes, CharacterClass, EquipmentChanged, WantsToLevelUp,
    Pools, RunState, Skills, WantsToLearnAbility, WantsToLevelAbility, Point, Map};
use crate::gamelog;
use crate::effects::{add_effect, EffectType, Targets};
use rltk::RGB;

pub struct LevelUpCharacterSystem {}

impl<'a> System<'a> for LevelUpCharacterSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        WriteStorage<'a, CharacterClass>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, WantsToLevelUp>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, WantsToLearnAbility>,
        WriteStorage<'a, WantsToLevelAbility>
    );
    
    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, player_pos, map, entities, mut pools,
            mut attributes, mut skills, mut character_classes,
            mut equip_dirty, mut wants_level_up, runstate,
            mut learn_abilities, mut level_abilities) = data;

        if wants_level_up.is_empty() { return; }
        if *runstate != RunState::Ticking { return; }

        for (entity, level_up, pool, char_class, char_attr, char_skills)
            in (&entities, &wants_level_up, &mut pools, &mut character_classes, &mut attributes, &mut skills).join() {
            pool.level += 1;
            pool.xp = 0; // loses overflow xp?

            let passives = &mut char_class.passives;
            for (name, passive) in level_up.passives.iter() {
                if passive.current_level < 1 { continue; }

                if passives[name].current_level != passive.current_level {
                    let current_passive = passives.get_mut(name).unwrap();
                    current_passive.current_level = passive.current_level;

                    if let Some(attribute_bonus) = &current_passive.levels[&current_passive.current_level].attribute_bonus {
                        if let Some(strength) = attribute_bonus.strength {
                            char_attr.strength.base += strength;
                        }
                        if let Some(dexterity) = attribute_bonus.dexterity {
                            char_attr.dexterity.base += dexterity;
                        }
                        if let Some(constitution) = attribute_bonus.constitution {
                            char_attr.constitution.base += constitution;
                        }
                        if let Some(intelligence) = attribute_bonus.intelligence {
                            char_attr.intelligence.base += intelligence;
                        }
                    }

                    pool.hit_points.max = player_hp_at_level(
                        char_attr.constitution.base + char_attr.constitution.total_modifiers(),
                        pool.level
                    );
                    pool.hit_points.current = pool.hit_points.max;
                    pool.mana.max = mana_at_level(
                        char_attr.intelligence.base + char_attr.intelligence.total_modifiers(),
                        pool.level
                    );
                    pool.mana.current = pool.mana.max;

                    if let Some(skill_bonus) = &current_passive.active_level().skill_bonus {
                        if let Some(melee) = skill_bonus.melee {
                            char_skills.melee.base += melee;
                        }
                        if let Some(defence) = skill_bonus.defence {
                            char_skills.defence.base += defence;
                        }
                        if let Some(ranged) = skill_bonus.ranged {
                            char_skills.ranged.base += ranged;
                        }
                        if let Some(magic) = skill_bonus.magic {
                            char_skills.magic.base += magic;
                        }
                    }

                    if let Some(learn_ability) = &current_passive.active_level().learn_ability {
                        learn_abilities.insert(*player_entity, WantsToLearnAbility{ ability_name: learn_ability.clone(), level: 1 }).expect("Unable to insert");
                    }

                    if let Some(level_ability) = &current_passive.active_level().level_ability {
                        level_abilities.insert(*player_entity, WantsToLevelAbility{ ability_name: level_ability.clone() }).expect("Unable to insert");
                    }
                }
            }

            equip_dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            if entity == *player_entity {
                gamelog::clear_log();
                gamelog::Logger::new()
                    .append("You are now level")
                    .colour(RGB::named(rltk::GOLD))
                    .append(pool.level)
                    .reset_colour()
                    .append("!")
                    .log();

                for i in 0..10 {
                    if player_pos.y - i > 1 {
                        add_effect(None,
                            EffectType::Particle{
                                glyph: rltk::to_cp437('░'),
                                fg : rltk::RGB::named(rltk::GOLD),
                                bg : rltk::RGB::named(rltk::BLACK),
                                lifespan: 400.0
                            },
                            Targets::Tile{ tile_idx : map.xy_idx(player_pos.x, player_pos.y - i) as i32 }
                        );
                    }
                }
            }
        }

        wants_level_up.clear();
    }
}
