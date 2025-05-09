extern crate serde;
use gui::LevelUpMenuResult;
use rltk::{GameState, Rltk, Point};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use std::collections::HashMap;

pub mod components;
pub mod map;
mod gui;
pub mod helpers;
pub mod gamelog;
pub mod map_builders;
pub mod raws;
pub mod spatial;
mod effects;
mod systems;
pub mod rng;

pub use helpers::*;
pub use components::*;
pub use map::*;
pub use systems::*;
pub use rng::*;

#[macro_use]
extern crate lazy_static;

const SHOW_MAPGEN_VISUALIZER: bool = true;
const SHOW_FPS: bool = true;

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
    ShowTargeting { min_range: f32, max_range: f32, source : Entity},
    MainMenu { menu_selection: gui::MainMenuSelection },
    InGameMenu { menu_selection: gui::InGameMenuSelection },
    CharacterClassSelectMenu { menu_selection: gui::CharacterClassSelection },
    SaveGame,
    MagicMapReveal { row: i32 },
    GameOver,
    MapGeneration,
    ShowCheatMenu,
    ShowVendor { vendor: Entity, mode: gui::VendorMode },
    TownPortal,
    TeleportingToOtherLevel { x: i32, y: i32, depth: i32 },
    LevelUp,
    ShowQuestMenu { quest_giver: Entity, index: i32}
}

pub struct State {
    pub ecs: World,
    mapgen_next_state : Option<RunState>,
    mapgen_history : Vec<Map>,
    mapgen_index : usize,
    mapgen_timer : f32,
    dispatcher: Box<dyn systems::UnifiedDispatcher + 'static>
}

impl State {
    fn run_systems(&mut self) {
        self.dispatcher.run_now(&mut self.ecs);
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
        gamelog::clear_log();
    }

    fn change_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        // build a new map and place the player
        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth + offset, offset);

        gamelog::Logger::new().append("You change floor.").log();
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

        let mut dungeon_master = self.ecs.write_resource::<MasterDungeonMap>();
        dungeon_master.reset();
        std::mem::drop(dungeon_master);
        gamelog::clear_events();
        saveload_system::delete_save();
        initialise_resources(&mut self.ecs);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();
        systems::particle_system::update_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu{..} => {}
            RunState::InGameMenu{..} => {}
            RunState::CharacterClassSelectMenu{..} => {}
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
                if newrunstate != RunState::AwaitingInput {
                    gamelog::record_event("Turn", 1)
                }
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
                        RunState::LevelUp => newrunstate = RunState::LevelUp,
                        _ => newrunstate = RunState::Ticking
                    }
                }
                if should_change_target {
                    player::change_target(&mut self.ecs);
                }
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
                            newrunstate = RunState::ShowTargeting{ min_range: is_item_ranged.min_range, max_range: is_item_ranged.max_range, source: item_entity };
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
            RunState::ShowTargeting{min_range, max_range, source} => {
                let result = gui::ranged_target(self, ctx, min_range, max_range, source);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        if self.ecs.read_storage::<KnownAbility>().get(source).is_some() {
                            let mut intent = self.ecs.write_storage::<WantsToUseAbility>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseAbility{ ability: source, target: result.1 }).expect("Unable to insert intent");
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
                            gui::MainMenuSelection::NewGame => { newrunstate = RunState::CharacterClassSelectMenu { menu_selection: gui::CharacterClassSelection::Warrior } }
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                // delete save file after loading from it
                                saveload_system::delete_save();
                            }
                            gui::MainMenuSelection::Quit => ::std::process::exit(0)
                        }
                    }
                }
            }
            RunState::InGameMenu { .. } => {
                let result = gui::in_game_menu(self, ctx);
                match result {
                    gui::InGameMenuResult::NoSelection{ selected } => newrunstate = RunState::InGameMenu{ menu_selection: selected },
                    gui::InGameMenuResult::Selected{ selected } => {
                        match selected {
                            gui::InGameMenuSelection::Continue => { newrunstate = RunState::AwaitingInput }
                            gui::InGameMenuSelection::NewGame => { newrunstate = RunState::GameOver }
                            gui::InGameMenuSelection::Quit => ::std::process::exit(0)
                        }
                    }
                }
            }
            RunState::CharacterClassSelectMenu { .. } => {
                let result = gui::character_class_select_menu(self, ctx);
                match result {
                    gui::CharacterClassResult::NoSelection { selected } => newrunstate = RunState::CharacterClassSelectMenu { menu_selection: selected },
                    gui::CharacterClassResult::Selected { selected } => {
                        match selected {
                            gui::CharacterClassSelection::Warrior => {
                                raws::spawn_named_character_class(&raws::RAWS.lock().unwrap(), &mut self.ecs, "Warrior");
                            }
                            gui::CharacterClassSelection::Sorceror => {
                                raws::spawn_named_character_class(&raws::RAWS.lock().unwrap(), &mut self.ecs, "Sorceror");
                            }
                            gui::CharacterClassSelection::Ranger => {
                                raws::spawn_named_character_class(&raws::RAWS.lock().unwrap(), &mut self.ecs, "Ranger");
                            }
                        }
                        self.mapgen_next_state = Some(RunState::PreRun);
                        self.generate_world_map(0, 0);
                        newrunstate = RunState::MapGeneration;
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::InGameMenu{ menu_selection : gui::InGameMenuSelection::Continue };
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
                        newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_menu(ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::TeleportToExit => {
                        self.change_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        gamelog::Logger::new().append("You teleport to the next level").log();
                        newrunstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::FullHeal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        gamelog::Logger::new().append("Your wounds are fully healed").log();
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::RevealMap => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for tile in map.revealed_tiles.iter_mut() {
                            *tile = true;
                        }
                        gamelog::Logger::new().append("The layout of the current map is revealed to you").log();
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        if player_pools.god_mode {
                            player_pools.god_mode = false;
                            gamelog::Logger::new().append("God mode deactivated").log();
                        } else {
                            player_pools.god_mode = true;
                            gamelog::Logger::new().append("God mode activated").log();
                        };
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::LevelUp => {
                        let player = self.ecs.fetch::<Entity>();
                        let character_classes = self.ecs.read_storage::<CharacterClass>();
                        let player_class = character_classes.get(*player).unwrap();
                        let mut level_ups = self.ecs.write_storage::<WantsToLevelUp>();
                        level_ups.insert(*player, WantsToLevelUp{ passives: player_class.passives.clone() }).expect("Unable to insert");
                        gamelog::Logger::new().append("You level up").log();
                        newrunstate = RunState::LevelUp;
                    }
                    gui::CheatMenuResult::MakeRich => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.gold = 999999;
                        gamelog::Logger::new().append("You are now filthy rich").log();
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::QuestComplete => {
                        let active_quests = &mut self.ecs.fetch_mut::<ActiveQuests>().quests;
                        for quest in active_quests.iter_mut() {
                            for requirement in quest.requirements.iter_mut() {
                                requirement.complete = true
                            }
                        }
                        gamelog::Logger::new().append("All active quests are complete").log();
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::IncreaseAttributes => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut attributes = self.ecs.write_storage::<Attributes>();
                        let player_attributes = attributes.get_mut(*player).unwrap();
                        player_attributes.strength.base += 1;
                        player_attributes.dexterity.base += 1;
                        player_attributes.constitution.base += 1;
                        player_attributes.intelligence.base += 1;
                        self.ecs.write_storage::<EquipmentChanged>().insert(*player, EquipmentChanged {}).expect("Unable to insert");
                        gamelog::Logger::new().append("Your attributes increase by 1").log();
                        newrunstate = RunState::Ticking;
                    }
                    gui::CheatMenuResult::DamageSelf => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        let damage = player_pools.hit_points.max / 10;
                        player_pools.hit_points.current -= damage;
                        gamelog::Logger::new().append(format!("You take {} damage", damage)).log();
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowVendor{vendor, mode} => {
                let result = gui::show_vendor_menu(self, ctx, vendor, mode);
                match result.0 {
                    gui::VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::VendorResult::NoResponse => {}
                    gui::VendorResult::Sell => {
                        vendor::sell_item(self, result.1.unwrap());
                    }
                    gui::VendorResult::Buy => {
                        vendor::buy_item(self, 
                            result.2.unwrap(), result.3.unwrap()
                        );
                    }
                    gui::VendorResult::Improve => {
                        vendor::improve_item(self, 
                            result.1.unwrap(), result.3.unwrap()
                        );
                    }
                    gui::VendorResult::BuyMode => newrunstate = RunState::ShowVendor { vendor, mode: gui::VendorMode::Buy },
                    gui::VendorResult::SellMode => newrunstate = RunState::ShowVendor { vendor, mode: gui::VendorMode::Sell },
                    gui::VendorResult::ImproveMode => newrunstate = RunState::ShowVendor { vendor, mode: gui::VendorMode::Improve }
                }
                self.run_systems(); // TODO set runstate to AwaitingInput instead?
            }
            RunState::ShowQuestMenu{quest_giver, index} => {
                let result = gui::show_quest_giver_menu(self, ctx, quest_giver, index);
                match result {
                    gui::QuestGiverResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::QuestGiverResult::NoResponse => {}
                    gui::QuestGiverResult::TakeOnQuest => {
                        let quests = &mut self.ecs.fetch_mut::<Quests>().quests;
                        let active_quests = &mut self.ecs.fetch_mut::<ActiveQuests>().quests;
                        let current_quest = quests.get(index as usize).unwrap();

                        active_quests.push(current_quest.clone());
                        newrunstate = RunState::ShowQuestMenu { quest_giver, index };
                    }
                    gui::QuestGiverResult::TurnInQuest => {
                        let wants_turn_in = &mut self.ecs.write_storage::<WantsToTurnInQuest>();
                        let player = self.ecs.fetch::<Entity>();
                        let quests = &self.ecs.fetch::<Quests>().quests;
                        let quest = quests.get(index as usize).unwrap();
                        wants_turn_in.insert(*player, WantsToTurnInQuest{ quest: quest.clone() }).expect("Unable to insert");

                        newrunstate = RunState::Ticking;
                    }
                    gui::QuestGiverResult::ShowPreviousQuest => {
                        let mut new_index = index - 1;
                        if new_index < 0 { new_index = 0; }

                        newrunstate = RunState::ShowQuestMenu { quest_giver, index: new_index };
                    }
                    gui::QuestGiverResult::ShowNextQuest => {
                        let quests = &mut self.ecs.fetch_mut::<Quests>().quests;
                        let mut new_index = index + 1;
                        if new_index >= quests.len() as i32 { new_index = (quests.len() - 1) as i32; }

                        newrunstate = RunState::ShowQuestMenu { quest_giver, index: new_index };
                    }
                }
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
            RunState::LevelUp => {
                let result = gui::show_levelup_menu(self, ctx);
                match result {
                    LevelUpMenuResult::Done => newrunstate = RunState::Ticking,
                    _ => newrunstate = RunState::LevelUp
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        cleanup::delete_the_dead(&mut self.ecs);

        rltk::render_draw_buffer(ctx).expect("Render error");
        if SHOW_FPS {
            ctx.print(1, 79, &format!("FPS: {}", ctx.fps));
        }
    }
}

macro_rules! register_components {
    ($ecs:expr, [$($t:ty),*]) => {
        $(
            $ecs.register::<$t>();
        )*
    };
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple(100, 80)
        .unwrap()
        .with_title("Roguelike")
        .with_fps_cap(30.0)
        .with_font("vga8x16.png", 8, 16)
        .with_sparse_console(100, 40, "vga8x16.png")
        .build()?;
    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame }),
        mapgen_index : 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
        dispatcher: systems::build()
    };
    register_components!(&mut gs.ecs, [
        SimpleMarker<SerializeMe>, SerializationHelper, DMSerializationHelper, Position, Renderable, Player,
        Viewshed, Name, BlocksTile, WantsToMelee, Item, Healing, Damage, AreaOfEffect, Consumable, Ranged,
        InBackpack, WantsToPickupItem, WantsToUseItem, WantsToDropItem, Confusion, Equipped, Equippable,
        Weapon, Wearable, WantsToUnequipItem, ParticleLifetime, MagicMapping, HungerClock, Food,
        BlocksVisibility, Door, EntityMoved, Quips, Attributes, Skills, Pools, NaturalAttackDefence,
        LootTable, OtherLevelPosition, LightSource, Initiative, Faction, WantsToApproach, MoveMode, Chasing,
        EquipmentChanged, Vendor, TownPortal, EntryTrigger, TeleportTo, ApplyMove, ApplyTeleport,
        SingleActivation, SpawnParticleLine, SpawnParticleBurst, AttributeBonus, SkillBonus, Duration,
        StatusEffect, RestoresMana, TeachesAbility, Slow, DamageOverTime, TileSize, WantsToLevelUp, ItemSets,
        PartOfSet, Target, WantsToShoot, Stun, StatusEffectChanged, Boss, Chest, CharacterClass, Ability,
        KnownAbility, MyTurn, KnownAbilities, WantsToUseAbility, SelfDamage, Rage, Block, Fortress, FrostShield,
        Dodge, WantsToLearnAbility, WantsToLevelAbility, Quests, ActiveQuests, QuestProgress, QuestGiver,
        WantsToTurnInQuest, MapMarker, Species, Repeat, WantsToRepeatAbility, RegenBonus
    ]);
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();
    initialise_resources(&mut gs.ecs);
    rltk::main_loop(context, gs)
}

fn initialise_resources(ecs: &mut World) {
    // store global resources
    ecs.insert(map::MasterDungeonMap::new());
    ecs.insert(Map::new("New Map", 0, 64, 64)); // w & h don't matter here
    ecs.insert(Point::new(0, 0));
    ecs.insert(particle_system::ParticleBuilder::new());
    ecs.insert(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });

    let player_entity = spawner::player(ecs, 0, 0);
    ecs.insert(player_entity);
    raws::spawn_all_abilities(ecs);

    ecs.insert(ItemSets{ item_sets: HashMap::new() });
    raws::store_all_item_sets(ecs);

    ecs.insert(Quests{ quests: Vec::new() });
    ecs.insert(ActiveQuests{ quests: Vec::new() });
    raws::store_all_quests(ecs);
}
