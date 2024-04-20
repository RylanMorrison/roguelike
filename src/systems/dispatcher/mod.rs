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
    (StatusEffectSystem, "status_effect", &[]),
    (InitiativeSystem, "initiative", &[]),
    (TurnStatusSystem, "turn_status", &[]),
    (HungerSystem, "hunger", &[]),
    (LevelUpSystem, "level_up", &[]),
    (ItemCollectionSystem, "item_collection", &[]),
    (ItemEquipSystem, "item_equip", &[]),
    (ItemUseSystem, "item_use", &[]),
    (SpellUseSystem, "spell_use", &[]),
    (ItemDropSystem, "item_drop", &[]),
    (ItemUnequipSystem, "item_unequip", &[]),
    (AdjacentAI, "adjacent_ai", &[]),
    (VisibleAI, "visible_ai", &[]),
    (ApproachAI, "approach_ai", &[]),
    (DefaultMoveAI, "default_move_ai", &[]),
    (MovementSystem, "movement", &[]),
    (TriggerSystem, "triggers", &[]),
    (MeleeCombatSystem, "melee_combat", &[]),
    (RangedCombatSystem, "ranged_combat", &[]),
    (FleeAI, "flee_ai", &[]),
    (ChaseAI, "chase_ai", &[]),
    (ParticleSpawnSystem, "particle_spawn", &[]),
    (LightingSystem, "lighting", &[]),
    (QuipSystem, "quips", &[])
);

pub fn new() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}
