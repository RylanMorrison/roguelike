extern crate serde;
use gui::LevelUpMenuResult;
use rltk::{GameState, Rltk, Point};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use std::collections::HashMap;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod ai;
pub use ai::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod ranged_combat_system;
use ranged_combat_system::RangedCombatSystem;
mod cleanup;
mod gui;
mod gamelog;
mod spawner;
mod inventory_system;
use inventory_system::{ItemCollectionSystem, ItemEquipSystem, ItemUnequipSystem, ItemUseSystem, ItemDropSystem, SpellUseSystem};
pub mod saveload_system;
pub mod random_table;
pub mod particle_system;
use particle_system::ParticleSpawnSystem;
pub mod hunger_system;
use hunger_system::HungerSystem;
pub mod map_builders;
pub mod camera;
pub mod raws;
mod gamesystem;
pub use gamesystem::*;
pub mod lighting_system;
use lighting_system::LightingSystem;
pub mod spatial;
mod movement_system;
use movement_system::MovementSystem;
mod trigger_system;
use trigger_system::TriggerSystem;
mod effects;
mod level_up_system;
use level_up_system::LevelUpSystem;
#[macro_use]
extern crate lazy_static;

const SHOW_MAPGEN_VISUALIZER : bool = true;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum VendorMode { Buy, Sell }

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState { 
    PreRun,
    AwaitingInput,
    Ticking,
    NextLevel,
    PreviousLevel,
    ShowInventory,
    ShowUnequipItem,
    ShowDropItem,
    ShowTargeting { range : i32, source : Entity},
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame,
    MagicMapReveal { row: i32 },
    GameOver,
    MapGeneration,
    ShowCheatMenu,
    ShowVendor { vendor: Entity, mode: VendorMode },
    TownPortal,
    TeleportingToOtherLevel { x: i32, y: i32, depth: i32 },
    LevelUp{ attribute_points: i32, skill_points: i32 }
}

pub struct State {
    pub ecs: World,
    mapgen_next_state : Option<RunState>,
    mapgen_history : Vec<Map>,
    mapgen_index : usize,
    mapgen_timer : f32
}

impl State {
    fn run_systems(&mut self) {
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut level_ups = LevelUpSystem{};
        level_ups.run_now(&self.ecs);
        let mut gear_effects = GearEffectSystem{};
        gear_effects.run_now(&self.ecs);
        let mut initiative = InitiativeSystem{};
        initiative.run_now(&self.ecs);
        let mut turnstatus = TurnStatusSystem{};
        turnstatus.run_now(&self.ecs);
        let mut quipping = QuipSystem{};
        quipping.run_now(&self.ecs);
        let mut adjacent = AdjacentAI{};
        adjacent.run_now(&self.ecs);
        let mut visible = VisibleAI{};
        visible.run_now(&self.ecs);
        let mut approach = ApproachAI{};
        approach.run_now(&self.ecs);
        let mut flee = FleeAI{};
        flee.run_now(&self.ecs);
        let mut chasing = ChaseAI{};
        chasing.run_now(&self.ecs);
        let mut defaultmove = DefaultMoveAI{};
        defaultmove.run_now(&self.ecs);
        let mut movement = MovementSystem{};
        movement.run_now(&self.ecs);
        let mut triggers = TriggerSystem{};
        triggers.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut ranged = RangedCombatSystem{};
        ranged.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut itemequip = ItemEquipSystem{};
        itemequip.run_now(&self.ecs);
        let mut itemuse = ItemUseSystem{};
        itemuse.run_now(&self.ecs);
        let mut spelluse = SpellUseSystem{};
        spelluse.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);
        let mut unequip_items = ItemUnequipSystem{};
        unequip_items.run_now(&self.ecs);
        let mut hunger = HungerSystem{};
        hunger.run_now(&self.ecs);
        effects::run_effects_queue(&mut self.ecs);
        let mut particles = ParticleSpawnSystem{};
        particles.run_now(&self.ecs);
        let mut lighting = LightingSystem{};
        lighting.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn generate_world_map(&mut self, new_depth: i32, offset: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = map::level_transition(&mut self.ecs, new_depth, offset);
        if let Some(history) = map_building_info {
            self.mapgen_history = history;
        } else {
            map::thaw_level_entities(&mut self.ecs);
        }
    }

    fn change_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        // build a new map and place the player
        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth + offset, offset);

        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog.entries.push("You change floor.".to_string());
    }

    pub fn game_over_cleanup(&mut self) {
        // delete all entities
        let mut to_delete : Vec<Entity> = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // replace the world maps
        // self.ecs.insert(map::MasterDungeonMap::new());

        // build a new map
        self.generate_world_map(0, 0);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();
        particle_system::update_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu{..} => {}
            RunState::GameOver{..} => {}
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        match newrunstate {
            RunState::MapGeneration => {
                if SHOW_MAPGEN_VISUALIZER {
                    ctx.cls();
                    if self.mapgen_index < self.mapgen_history.len() {
                        camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                    }

                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 200.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index >= self.mapgen_history.len() {
                            newrunstate = self.mapgen_next_state.unwrap();
                        }
                    }
                } else {
                    newrunstate = self.mapgen_next_state.unwrap();
                }
            }
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::Ticking => {
                let mut should_change_target = false;
                while newrunstate == RunState::Ticking {
                    self.run_systems();
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => {
                            newrunstate = RunState::AwaitingInput;
                            should_change_target = true;
                        }
                        RunState::MagicMapReveal { .. } => newrunstate = RunState::MagicMapReveal { row: 0 },
                        RunState::TownPortal => newrunstate = RunState::TownPortal,
                        RunState::TeleportingToOtherLevel{ x, y, depth } => newrunstate = RunState::TeleportingToOtherLevel { x, y, depth },
                        RunState::LevelUp { attribute_points, skill_points } => newrunstate = RunState::LevelUp{ attribute_points, skill_points },
                        _ => newrunstate = RunState::Ticking
                    }
                }
                if should_change_target {
                    player::change_target(&mut self.ecs);
                }
                /*
                    The run order of systems causes an issue where data is updated by the systems but only utilised on
                    the next iteration. For example:
                    - GearEffectSystem runs to ensure all gear and pools are up to date before initiative checks are run
                    - InitiativeSystem runs to determine turn order and expire status effects
                    - Gear and pools without the expired status effect are only updated the next time GearEffectSystem is run
                    Therefore, run all systems again before proceeding from Ticking to ensure everything is up to date
                 */
                self.run_systems();
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting{ range: is_item_ranged.range, source: item_entity };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item: item_entity, target: None }).expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowUnequipItem => {
                let result = gui::unequip_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToUnequipItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUnequipItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                } 
            }
            RunState::ShowTargeting{range, source} => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        if self.ecs.read_storage::<Spell>().get(source).is_some() {
                            let mut intent = self.ecs.write_storage::<WantsToCastSpell>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToCastSpell{ spell: source, target: result.1 }).expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item: source, target: result.1 }).expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::MagicMapReveal{row} => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == map.height-1 {
                    newrunstate = RunState::Ticking;
                } else {
                    newrunstate = RunState::MagicMapReveal{ row: row+1 };
                }
            }
            RunState::MainMenu{ .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection{ selected } => newrunstate = RunState::MainMenu{ menu_selection: selected },
                    gui::MainMenuResult::Selected{ selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                // delete save file after loading from it
                                saveload_system::delete_save();
                            }
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu{ menu_selection : gui::MainMenuSelection::Quit };
            }
            RunState::NextLevel => {
                self.change_level(1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::PreviousLevel => {
                self.change_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MapGeneration;
                        self.mapgen_next_state = Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame });
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_mode(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::TeleportToExit => {
                        self.change_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::FullHeal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::RevealMap => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for tile in map.revealed_tiles.iter_mut() {
                            *tile = true;
                        }
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.god_mode = if player_pools.god_mode { false } else { true };
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::LevelUp => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player::level_up(&self.ecs, *player, player_pools);
                        newrunstate = RunState::LevelUp{ attribute_points: 1, skill_points: 2 };
                    }
                    gui::CheatMenuResult::MakeRich => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.gold = 999999;
                        newrunstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowVendor{vendor, mode} => {
                let result = gui::show_vendor_menu(self, ctx, vendor, mode);
                match result.0 {
                    gui::VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::VendorResult::NoResponse => {}
                    gui::VendorResult::Sell => {
                        let price = self.ecs.read_storage::<Item>().get(result.1.unwrap()).unwrap().base_value as f32 * 0.8;
                        self.ecs.write_storage::<Pools>().get_mut(*self.ecs.fetch::<Entity>()).unwrap().gold += price as i32;
                        self.ecs.delete_entity(result.1.unwrap()).expect("Unable to delete");
                        self.ecs.write_storage::<EquipmentChanged>().insert(*self.ecs.fetch::<Entity>(), EquipmentChanged{}).expect("Unable to insert");
                    }
                    gui::VendorResult::Buy => {
                        let tag = result.2.unwrap();
                        let price = result.3.unwrap();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*self.ecs.fetch::<Entity>()).unwrap();
                        if player_pools.gold >= price {
                            player_pools.gold -= price;
                            std::mem::drop(pools);
                            let player_entity = *self.ecs.fetch::<Entity>();
                            raws::spawn_named_item(&raws::RAWS.lock().unwrap(), &mut self.ecs, &tag, raws::SpawnType::Carried{ by: player_entity });
                            self.ecs.write_storage::<EquipmentChanged>().insert(*self.ecs.fetch::<Entity>(), EquipmentChanged{}).expect("Unable to insert");
                        }
                    }
                    gui::VendorResult::BuyMode => newrunstate = RunState::ShowVendor { vendor, mode: VendorMode::Buy },
                    gui::VendorResult::SellMode => newrunstate = RunState::ShowVendor { vendor, mode: VendorMode::Sell }
                }
                self.run_systems();
            }
            RunState::TownPortal => {
                spawner::spawn_town_portal(&mut self.ecs);

                // transition
                let map_depth = self.ecs.fetch::<Map>().depth;
                let destination_offset = 0 - map_depth; // town is depth 0
                self.change_level(destination_offset);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TeleportingToOtherLevel { x, y, depth } => {
                self.change_level(depth);
                let player_entity = self.ecs.fetch::<Entity>();
                if let Some(pos) = self.ecs.write_storage::<Position>().get_mut(*player_entity) {
                    pos.x = x;
                    pos.y = y;
                }
                let mut ppos = self.ecs.fetch_mut::<Point>();
                ppos.x = x;
                ppos.y = y;
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::LevelUp{attribute_points, skill_points} => {
                let result = gui::show_levelup_menu(self, ctx, attribute_points, skill_points);
                match result {
                    LevelUpMenuResult::NoResponse => {},
                    LevelUpMenuResult::AssignedAttribute => newrunstate = RunState::LevelUp { attribute_points: attribute_points - 1, skill_points },
                    LevelUpMenuResult::AssignedSkill => newrunstate = RunState::LevelUp { attribute_points, skill_points: skill_points - 1 },
                    LevelUpMenuResult::Done => newrunstate = RunState::Ticking
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        cleanup::delete_the_dead(&mut self.ecs);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple(100, 80)
        .unwrap()
        .with_title("Taverns of Stoner Doom")
        .with_fps_cap(30.0)
        .build()?;
    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame }),
        mapgen_index : 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Healing>();
    gs.ecs.register::<Damage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Weapon>();
    gs.ecs.register::<Wearable>();
    gs.ecs.register::<WantsToUnequipItem>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<MagicMapping>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<Food>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<NaturalAttackDefence>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<EquipmentChanged>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<TownPortal>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<TeleportTo>();
    gs.ecs.register::<ApplyMove>();
    gs.ecs.register::<ApplyTeleport>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<SpawnParticleLine>();
    gs.ecs.register::<SpawnParticleBurst>();
    gs.ecs.register::<AttributeBonus>();
    gs.ecs.register::<SkillBonus>();
    gs.ecs.register::<Duration>();
    gs.ecs.register::<StatusEffect>();
    gs.ecs.register::<KnownSpells>();
    gs.ecs.register::<Spell>();
    gs.ecs.register::<WantsToCastSpell>();
    gs.ecs.register::<RestoresMana>();
    gs.ecs.register::<TeachesSpell>();
    gs.ecs.register::<Slow>();
    gs.ecs.register::<DamageOverTime>();
    gs.ecs.register::<SpecialAbilities>();
    gs.ecs.register::<TileSize>();
    gs.ecs.register::<PendingLevelUp>();
    gs.ecs.register::<ItemSets>();
    gs.ecs.register::<PartOfSet>();
    gs.ecs.register::<Target>();
    gs.ecs.register::<WantsToShoot>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    // store global resources
    gs.ecs.insert(map::MasterDungeonMap::new());
    gs.ecs.insert(Map::new("New Map", 0, 64, 64)); // w & h don't matter here
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MapGeneration{});

    raws::spawn_all_spells(&mut gs.ecs);
    gs.ecs.insert(ItemSets{ item_sets: HashMap::new() });
    raws::store_all_item_sets(&mut gs.ecs);
    gs.ecs.insert(gamelog::GameLog{ entries : vec!["Welcome to Taverns of Stoner Doom".to_string()] });

    gs.generate_world_map(0, 0);
    rltk::main_loop(context, gs)
}
