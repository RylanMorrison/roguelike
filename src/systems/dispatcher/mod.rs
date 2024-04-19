use specs::prelude::World;
use super::*;

#[macro_use]
mod single_thread;
pub use single_thread::*;

pub trait UnifiedDispatcher {
    fn run_now(&mut self, ecs: *mut World);
}

construct_dispatcher!(
    (MapIndexingSystem, "map_index", &[]),
    (VisibilitySystem, "visibility", &[]),
    (GearEffectSystem, "gear_effect", &[]),
    (InitiativeSystem, "initiative", &[]),
    (TurnStatusSystem, "turn_status", &[]),
    (QuipSystem, "quips", &[]),
    (AdjacentAI, "adjacent_ai", &[]),
    (VisibleAI, "visible_ai", &[]),
    (ApproachAI, "approach_ai", &[]),
    (FleeAI, "flee_ai", &[]),
    (ChaseAI, "chase_ai", &[]),
    (DefaultMoveAI, "default_move_ai", &[]),
    (MovementSystem, "movement", &[]),
    (TriggerSystem, "triggers", &[]),
    (MeleeCombatSystem, "melee_combat", &[]),
    (RangedCombatSystem, "ranged_combat", &[]),
    (ItemCollectionSystem, "item_collection", &[]),
    (ItemEquipSystem, "item_equip", &[]),
    (ItemUseSystem, "item_use", &[]),
    (SpellUseSystem, "spell_use", &[]),
    (ItemDropSystem, "item_drop", &[]),
    (ItemUnequipSystem, "item_unequip", &[]),
    (HungerSystem, "hunger", &[]),
    (LevelUpSystem, "level_up", &[]),
    (ParticleSpawnSystem, "particle_spawn", &[]),
    (LightingSystem, "lighting", &[])
);

pub fn new() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}
