use rltk::RGB;
use specs::prelude::*;
use crate::effects::add_effect;
use crate::{ActiveQuests, ProgressSource, QuestProgress, QuestRequirementGoal, WantsToTurnInQuest,
    Pools, Quests, Point, Map, RunState, WantsToLevelUp, CharacterClass, QuestStatus, Name, Species,
    determine_roll, player_xp_for_level};
use crate::gamelog;
use crate::effects;

pub struct QuestProgressSystem {}

impl<'a> System<'a> for QuestProgressSystem {
    type SystemData = (
        WriteExpect<'a, ActiveQuests>,
        WriteStorage<'a, QuestProgress>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Species>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut active_quests, mut quest_progress, names, all_species) = data;

        if quest_progress.is_empty() { return; }

        for progress in quest_progress.join() {
            match progress.source {
                ProgressSource::Kill => {
                    for quest in active_quests.quests.iter_mut() {
                        if quest.is_complete() { continue; }

                        for requirement in quest.requirements.iter_mut() {
                            if requirement.complete { continue; }

                            match requirement.requirement_goal {
                                QuestRequirementGoal::KillCount => {

                                    let target_name = if let Some(name) = names.get(progress.target) {
                                        name.name.clone()
                                    } else { "None".to_string() };

                                    let target_species = if let Some(species) = all_species.get(progress.target) {
                                        species.name.clone()
                                    } else { "None".to_string() };

                                    if (requirement.targets.contains(&target_name) || requirement.targets.contains(&target_species))
                                    && requirement.count < requirement.target_count {
                                        requirement.count += 1;
                                    }

                                    if requirement.count >= requirement.target_count {
                                        requirement.complete = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        quest_progress.clear();
    }
}

pub struct QuestTurnInSystem {}

impl<'a> System<'a> for QuestTurnInSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, WantsToTurnInQuest>,
        WriteStorage<'a, WantsToLevelUp>,
        ReadStorage<'a, CharacterClass>,
        WriteStorage<'a, Pools>,
        WriteExpect<'a, Quests>,
        WriteExpect<'a, ActiveQuests>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, RunState>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, entities, mut wants_turn_in, mut level_ups,
            character_classes, mut pools, mut quests, mut active_quests,
            player_pos, map, mut runstate) = data;

        if wants_turn_in.is_empty() { return; }

        for (entity, turn_in) in (&entities, &wants_turn_in).join() {
            let quest_name = turn_in.quest.name.clone();

            if let Some(pool) = pools.get_mut(entity) {
                for reward in &turn_in.quest.rewards {
                    if let Some(gold) = &reward.gold {
                        let gold_reward = determine_roll(gold);
                        pool.gold += gold_reward;
                        if entity == *player {
                            gamelog::Logger::new().append(format!("You receive {} gold", gold_reward)).log();
                        }
                    }
                    if let Some(xp) = &reward.xp {
                        pool.xp += xp;
                        if entity == *player {
                            gamelog::Logger::new().append(format!("You receive {} xp", xp)).log();
                            if pool.xp >= player_xp_for_level(pool.level) {
                                let player_class = character_classes.get(entity).unwrap();
                                level_ups.insert(entity, WantsToLevelUp {
                                    passives: player_class.passives.clone()
                                }).expect("Unable to insert");
                                *runstate = RunState::LevelUp;
                            }
                        }
                    }
                }
            }
            
            active_quests.quests.retain(|quest| quest.name != quest_name);
            quests.quests.retain(|quest| quest.name != quest_name);

            // unlock following quests
            let mut new_quests = false;
            turn_in.quest.next_quests.iter().for_each(|quest_name| {
                quests.quests.iter_mut().for_each(|quest| {
                    if quest.name == *quest_name {
                        quest.status = QuestStatus::Available;
                        new_quests = true;
                    }
                })
            });
            if new_quests {
                gamelog::Logger::new().append("New quests are available!").log();
            }

            for i in 0..10 {
                if player_pos.y - i > 1 {
                    add_effect(None,
                        effects::EffectType::Particle {
                            glyph: rltk::to_cp437('░'),
                            fg: RGB::named(rltk::BLUE),
                            bg: RGB::named(rltk::BLACK),
                            lifespan: 400.0
                        },
                        effects::Targets::Tile { tile_idx: map.xy_idx(player_pos.x, player_pos.y -i) as i32 }
                    );
                }
            }

            if entity == *player {
                gamelog::Logger::new()
                    .append("You turn in a quest:")
                    .colour(RGB::named(rltk::YELLOW))
                    .append(quest_name)
                    .log();
            }
        }

        wants_turn_in.clear();
    }
}
