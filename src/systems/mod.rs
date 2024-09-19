mod dispatcher;
mod ai;
mod inventory;
mod hunger_system;
mod level_up_character_system;
mod ability_systems;
mod lighting_system;
mod map_indexing_system;
mod melee_combat_system;
mod movement_system;
pub mod particle_system;
mod ranged_combat_system;
mod trigger_system;
mod visibility_system;
mod turn_status_system;
mod gear_effect_system;
mod status_effect_system;
pub mod saveload_system;
mod quest_systems;

pub use dispatcher::UnifiedDispatcher;
use ai::*;
use inventory::*;
use hunger_system::HungerSystem;
use level_up_character_system::LevelUpCharacterSystem;
use ability_systems::*;
use lighting_system::LightingSystem;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use movement_system::MovementSystem;
use particle_system::ParticleSpawnSystem;
use ranged_combat_system::RangedCombatSystem;
use trigger_system::TriggerSystem;
use visibility_system::VisibilitySystem;
use turn_status_system::TurnStatusSystem;
use gear_effect_system::GearEffectSystem;
use status_effect_system::StatusEffectSystem;
pub use saveload_system::*;
use quest_systems::*;

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    dispatcher::new()
}
