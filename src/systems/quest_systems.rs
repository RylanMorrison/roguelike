use rltk::RGB;
use specs::prelude::*;
use crate::effects::add_effect;
use crate::{ActiveQuests, ProgressSource, QuestProgress, QuestRequirementGoal, WantsToTurnInQuest,
    Pools, Quests, Point, Map, determine_roll};
use crate::gamelog;
use crate::effects;

pub struct QuestProgressSystem {}

impl<'a> System<'a> for QuestProgressSystem {
    type SystemData = (
        WriteExpect<'a, ActiveQuests>,
        WriteStorage<'a, QuestProgress>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut active_quests, mut quest_progress) = data;

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
                                    if requirement.targets.contains(&progress.target)
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
                _ => {}
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
        WriteStorage<'a, Pools>,
        WriteExpect<'a, Quests>,
        WriteExpect<'a, ActiveQuests>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Map>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, entities, mut wants_turn_in, mut pools, mut quests,
            mut active_quests, player_pos, map) = data;

        if wants_turn_in.is_empty() { return; }

        for (entity, turn_in) in (&entities, &wants_turn_in).join() {
            let quest_name = turn_in.quest.name.clone();

            if let Some(pool) = pools.get_mut(entity) {
                if let Some(gold) = &turn_in.quest.reward.gold {
                    let gold_reward = determine_roll(gold);
                    pool.gold += gold_reward;
                    if entity == *player {
                        gamelog::Logger::new().append(format!("You receive {} gold", gold_reward)).log();
                    }
                }
            }
            
            active_quests.quests.retain(|quest| quest.name != quest_name);
            quests.quests.retain(|quest| quest.name != quest_name);

            for i in 0..10 {
                if player_pos.y - i > 1 {
                    add_effect(None,
                        effects::EffectType::Particle{
                            glyph: rltk::to_cp437('░'),
                            fg: RGB::named(rltk::BLUE),
                            bg: RGB::named(rltk::BLACK),
                            lifespan: 400.0
                        },
                        effects::Targets::Tile{ tile_idx: map.xy_idx(player_pos.x, player_pos.y -i) as i32 }
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
