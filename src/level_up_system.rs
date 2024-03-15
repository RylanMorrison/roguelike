use specs::prelude::*;
use super::{Attributes, Skills, EquipmentChanged, PendingLevelUp, RunState};

pub struct LevelUpSystem {}

impl<'a> System<'a> for LevelUpSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, PendingLevelUp>,
        ReadExpect<'a, RunState>
    );
    
    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut attributes, mut skills, mut dirty, mut pending_level_up, runstate) = data;

        if pending_level_up.count() < 1 { return; }
        if *runstate != RunState::Ticking { return; }

        let level_up = pending_level_up.get(*player_entity).unwrap();
        let player_attributes = attributes.get_mut(*player_entity).unwrap();
        let player_skills = skills.get_mut(*player_entity).unwrap();

        player_attributes.strength.base = level_up.attributes.strength.base;
        player_attributes.dexterity.base = level_up.attributes.dexterity.base;
        player_attributes.constitution.base = level_up.attributes.constitution.base;
        player_attributes.intelligence.base = level_up.attributes.intelligence.base;

        player_skills.melee.base = level_up.skills.melee.base;
        player_skills.defence.base = level_up.skills.defence.base;
        player_skills.magic.base = level_up.skills.magic.base;

        dirty.insert(*player_entity, EquipmentChanged{}).expect("Unable to insert");
        pending_level_up.clear();
    }
}
