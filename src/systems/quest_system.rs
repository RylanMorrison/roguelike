use specs::prelude::*;
use crate::{ActiveQuests, ProgressSource, QuestProgress, QuestRequirementGoal};

pub struct QuestSystem {}

impl<'a> System<'a> for QuestSystem {
    type SystemData = (
        WriteExpect<'a, ActiveQuests>,
        WriteStorage<'a, QuestProgress>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut active_quests, mut quest_progress) = data;

        if quest_progress.count() < 1 { return; }

        for progress in quest_progress.join() {
            match progress.source {
                ProgressSource::Kill => {
                    for quest in active_quests.quests.iter_mut() {
                        if quest.is_complete() { continue; }

                        for requirement in quest.requirements.iter_mut() {
                            if requirement.complete { continue; }

                            match requirement.requirement_goal {
                                QuestRequirementGoal::KillCount => {
                                    if requirement.targets.contains(&progress.target) && requirement.count < requirement.target_count {
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
