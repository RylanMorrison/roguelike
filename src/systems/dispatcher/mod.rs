use specs::prelude::World;
use super::*;

#[cfg(target_arch = "wasm32")]
#[macro_use]
mod single_thread;
#[cfg(target_arch = "wasm32")]
pub use single_thread::*;

#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
mod multi_thread;
#[cfg(not(target_arch = "wasm32"))]
pub use multi_thread::*;

pub trait UnifiedDispatcher {
    fn run_now(&mut self, ecs: *mut World);
}

construct_dispatcher!(
    (MapIndexingSystem, "map_index", &[]),
    (VisibilitySystem, "visibility", &[]),
    (GearEffectSystem, "gear_effect", &[]),
    (StatusEffectSystem, "status_effect", &["gear_effect"]),
    (LevelUpCharacterSystem, "level_up", &[]),
    (InitiativeSystem, "initiative", &["status_effect", "level_up"]),
    (TurnStatusSystem, "turn_status", &["initiative"]),
    (QuipSystem, "quips", &["initiative"]),
    (HungerSystem, "hunger", &[]),
    (LearnAbilitySystem, "learn_ability", &["level_up"]),
    (LevelAbilitySystem, "level_ability", &["level_up"]),
    (AdjacentAI, "adjacent_ai", &["initiative"]),
    (VisibleAI, "visible_ai", &["adjacent_ai"]),
    (ApproachAI, "approach_ai", &["visible_ai"]),
    (ChaseAI, "chase_ai", &["visible_ai"]),
    (DefaultMoveAI, "default_move_ai", &[ "approach_ai"]),
    (MovementSystem, "movement", &[]),
    (TriggerSystem, "triggers", &[]),
    (MeleeCombatSystem, "melee_combat", &["adjacent_ai"]),
    (RangedCombatSystem, "ranged_combat", &["visible_ai"]),
    (QuestProgressSystem, "quest_progress", &[]),
    (QuestTurnInSystem, "quest_turn_in", &[]),
    (ItemCollectionSystem, "item_collection", &[]),
    (ItemEquipSystem, "item_equip", &[]),
    (ItemUseSystem, "item_use", &[]),
    (AbilityUseSystem, "ability_use", &[]),
    (RepeatAbilitySystem, "repeat_ability", &[]),
    (ItemDropSystem, "item_drop", &[]),
    (ItemUnequipSystem, "item_unequip", &[]),
    (ParticleSpawnSystem, "particle_spawn", &[]),
    (LightingSystem, "lighting", &[])
);

pub fn new() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}
