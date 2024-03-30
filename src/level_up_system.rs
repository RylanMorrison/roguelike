use specs::prelude::*;
use super::{Attributes, Skills, EquipmentChanged, PendingLevelUp, RunState, Pools};
use crate::{mana_at_level, player_hp_at_level};

pub struct LevelUpSystem {}

impl<'a> System<'a> for LevelUpSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        WriteStorage<'a, Skills>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, PendingLevelUp>,
        ReadExpect<'a, RunState>
    );
    
    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut pools, mut attributes, mut skills, mut dirty,
            mut pending_level_up, runstate) = data;

        // TODO make this more generic when other entities can level up

        if pending_level_up.count() < 1 { return; }
        if *runstate != RunState::Ticking { return; }

        let level_up = pending_level_up.get(*player_entity).unwrap();
        let player_attributes = attributes.get_mut(*player_entity).unwrap();
        let player_skills = skills.get_mut(*player_entity).unwrap();
        let player_pools = pools.get_mut(*player_entity).unwrap();

        player_pools.level += 1;

        player_attributes.strength.base = level_up.attributes.strength.base;
        player_attributes.dexterity.base = level_up.attributes.dexterity.base;
        player_attributes.constitution.base = level_up.attributes.constitution.base;
        player_attributes.intelligence.base = level_up.attributes.intelligence.base;

        player_pools.hit_points.max = player_hp_at_level(
            player_attributes.constitution.base + player_attributes.constitution.modifiers,
            player_pools.level
        );
        player_pools.hit_points.current = player_pools.hit_points.max;
        player_pools.mana.max = mana_at_level(
            player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
            player_pools.level
        );
        player_pools.mana.current = player_pools.mana.max;

        player_skills.melee.base = level_up.skills.melee.base;
        player_skills.defence.base = level_up.skills.defence.base;
        player_skills.magic.base = level_up.skills.magic.base;

        dirty.insert(*player_entity, EquipmentChanged{}).expect("Unable to insert");
        pending_level_up.clear();
    }
}
