use rltk::{Point, RandomNumberGenerator, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

use crate::{spatial, InBackpack, WantsToUseItem};
use crate::raws::{faction_reaction, find_spell_entity, Reaction, RAWS};
use crate::effects::{add_effect, EffectType, Targets};

use super::{Position, Player, Viewshed, State, Map, RunState, Item, gamelog::GameLog, 
    TileType, particle_system::ParticleBuilder, Pools, WantsToMelee, WantsToPickupItem,
    HungerState, HungerClock, Door, BlocksVisibility, BlocksTile, Renderable, EntityMoved,
    Consumable, Ranged, Faction, Vendor, VendorMode, KnownSpells, WantsToCastSpell,
    Attributes, Skills, PendingLevelUp};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut playerpos = ecs.write_resource::<Point>();
    let pools = ecs.read_storage::<Pools>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let factions = ecs.read_storage::<Faction>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut swap_entities: Vec<(Entity, i32, i32)> = Vec::new();
    let vendors = ecs.read_storage::<Vendor>();
    let mut result = RunState::AwaitingInput;

    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return RunState::AwaitingInput; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = spatial::for_each_tile_content_with_gamemode(destination_idx, |potential_target| {
            if let Some(_vendor) = vendors.get(potential_target) {
                return Some(RunState::ShowVendor{ vendor: potential_target, mode: VendorMode::Sell });
            }

            let mut hostile = true;
            if pools.get(potential_target).is_some() {
                if let Some(faction) = factions.get(potential_target) {
                    let reaction = faction_reaction(
                        &faction.name,
                        "Player",
                        &RAWS.lock().unwrap()
                    );
                    if reaction != Reaction::Attack { hostile = false; }
                }
            }
            if !hostile {
                // record that entity should be swapped
                swap_entities.push((potential_target, pos.x, pos.y));

                // move the player
                pos.x = min(map.width-1, max(0, pos.x + delta_x));
                pos.y = min(map.height-1, max(0, pos.y + delta_y));
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");

                viewshed.dirty = true;
                playerpos.x = pos.x;
                playerpos.y = pos.y;
            } else {
                let target = pools.get(potential_target);
                if let Some(_target) = target {
                    wants_to_melee.insert(entity, WantsToMelee { target: potential_target }).expect("Add target failed");
                    return Some(RunState::Ticking);
                }
            }
            
            let door = doors.get_mut(potential_target);
            if let Some(door) = door {
                if !door.open {
                    door.open = true;
                    blocks_visibility.remove(potential_target);
                    blocks_movement.remove(potential_target);
                    let glyph = renderables.get_mut(potential_target).unwrap();
                    glyph.glyph = rltk::to_cp437('/');
                    viewshed.dirty = true;
                    return Some(RunState::Ticking);
                }
            }
            None
        });

        if !spatial::is_blocked(destination_idx) {
            let old_idx = map.xy_idx(pos.x, pos.y);
            pos.x = min(map.width-1 , max(0, pos.x + delta_x));
            pos.y = min(map.height-1, max(0, pos.y + delta_y));
            let new_idx = map.xy_idx(pos.x, pos.y);
            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
            spatial::move_entity(entity, old_idx, new_idx);

            viewshed.dirty = true;
            playerpos.x = pos.x;
            playerpos.y = pos.y;
            result = RunState::Ticking;
        }
    }

    for se in swap_entities.iter() {
        let their_pos = positions.get_mut(se.0);
        if let Some(their_pos) = their_pos {
            let old_idx = map.xy_idx(their_pos.x, their_pos.y);
            their_pos.x = se.1;
            their_pos.y = se.2;
            let new_idx = map.xy_idx(their_pos.x, their_pos.y);
            spatial::move_entity(se.0, old_idx, new_idx);
            result = RunState::Ticking;
        }
    }

    result
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }
    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item: item }).expect("Unable to insert want to pickup");
        }
    }
}

fn get_hotkey(key: VirtualKeyCode) -> Option<i32> {
    match key {
        VirtualKeyCode::Key1 => Some(1),
        VirtualKeyCode::Key2 => Some(2),
        VirtualKeyCode::Key3 => Some(3),
        VirtualKeyCode::Key4 => Some(4),
        VirtualKeyCode::Key5 => Some(5),
        VirtualKeyCode::Key6 => Some(6),
        VirtualKeyCode::Key7 => Some(7),
        VirtualKeyCode::Key8 => Some(8),
        VirtualKeyCode::Key9 => Some(9),
        _ => None
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // hotkeys
    if ctx.shift && ctx.key.is_some() {
        let key: Option<i32> = get_hotkey(ctx.key.unwrap());
        if let Some(key) = key {
            return use_consumable_hotkey(gs, key-1);
        }
    }

    if ctx.control && ctx.key.is_some() {
        let key: Option<i32> = get_hotkey(ctx.key.unwrap());
        if let Some(key) = key {
            return use_spell_hotkey(gs, key-1);
        }
    }

    // normal actions
    match ctx.key {
        None => { return RunState::AwaitingInput } // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::H => return try_move_player(-1, 0, &mut gs.ecs), // move east
            VirtualKeyCode::L => return try_move_player(1, 0, &mut gs.ecs), // move west
            VirtualKeyCode::K => return try_move_player(0, -1, &mut gs.ecs), // move north
            VirtualKeyCode::J => return try_move_player(0, 1, &mut gs.ecs), // move south
            VirtualKeyCode::Y => return try_move_player(-1, -1, &mut gs.ecs), // move north-east
            VirtualKeyCode::U => return try_move_player(1, -1, &mut gs.ecs), // move north-west
            VirtualKeyCode::B => return try_move_player(-1, 1, &mut gs.ecs), // move south-east
            VirtualKeyCode::N => return try_move_player(1, 1, &mut gs.ecs), // move south-west
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::Period => return try_transition_level(&mut gs.ecs),
            VirtualKeyCode::G => get_item(&mut gs.ecs), // pickup item
            VirtualKeyCode::I => return RunState::ShowInventory, // open inventory
            VirtualKeyCode::D => return RunState::ShowDropItem, // open item dropper
            VirtualKeyCode::R => return RunState::ShowUnequipItem, // open unequip menu
            VirtualKeyCode::Escape => return RunState::SaveGame, // open main menu and save the game
            VirtualKeyCode::Backslash => return RunState::ShowCheatMenu,
            _ => { return RunState::AwaitingInput }
        },
    }
    RunState::Ticking
}

fn use_consumable_hotkey(gs: &mut State, key: i32) -> RunState {
    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();
    let mut carried_consumables = Vec::new();

    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(carried_consumables[key as usize]) {
            return RunState::ShowTargeting { range: ranged.range, source: carried_consumables[key as usize] };
        }
        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent.insert(
            *player_entity,
            WantsToUseItem{ item: carried_consumables[key as usize], target: None }
        ).expect("Unable to insert intent");
        return RunState::Ticking;
    }
    RunState::Ticking
}

fn use_spell_hotkey(gs: &mut State, key: i32) -> RunState {
    let player_entity = gs.ecs.fetch::<Entity>();
    let known_spells = gs.ecs.read_storage::<KnownSpells>();
    let player_spells = &known_spells.get(*player_entity).unwrap().spells;

    if (key as usize) < player_spells.len() {
        let pools = gs.ecs.read_storage::<Pools>();
        let player_pools = pools.get(*player_entity).unwrap();
        if player_pools.mana.current >= player_spells[key as usize].mana_cost {
            if let Some(spell_entity) = find_spell_entity(&gs.ecs, &player_spells[key as usize].name) {
                if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(spell_entity) {
                    return RunState::ShowTargeting { range: ranged.range, source: spell_entity };
                }
                let mut intent = gs.ecs.write_storage::<WantsToCastSpell>();
                intent.insert(
                    *player_entity,
                    WantsToCastSpell{ spell: spell_entity, target: None }
                ).expect("Unable to insert intent");
                return RunState::Ticking;
            }
        } else {
            let mut gamelog = gs.ecs.fetch_mut::<GameLog>();
            gamelog.entries.push(format!("You don't have enough mana to cast {}!", player_spells[key as usize].name));
        }
    }
    RunState::Ticking
}

pub fn try_transition_level(ecs: &mut World) -> RunState {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);

    match map.tiles[player_idx] {
        TileType::DownStairs => RunState::NextLevel,
        TileType::UpStairs => RunState::PreviousLevel,
        _ => {
            let mut gamelog = ecs.fetch_mut::<GameLog>();
            gamelog.entries.push("There is nowhere to go from here.".to_string());
            RunState::Ticking
        }
    }
}

pub fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let factions = ecs.read_storage::<Faction>();
    let worldmap = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();

    // prevent skip turn healing if monsters are nearby
    let mut can_heal = true;
    let viewshed = viewsheds.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap.xy_idx(tile.x, tile.y);
        spatial::for_each_tile_content(idx, |entity_id| {
            let faction = factions.get(entity_id);
            match faction {
                None => {},
                Some(faction) => {
                    let reaction = faction_reaction(
                        &faction.name,
                        "Player",
                        &RAWS.lock().unwrap()
                    );
                    if reaction == Reaction::Attack {
                        can_heal = false;
                    }
                }
            }
        });
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let hunger = hunger_clocks.get(*player_entity);
    if let Some(hunger) = hunger {
        match hunger.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    // heal player when turn is skipped
    if can_heal {
        let mut pools = ecs.write_storage::<Pools>();
        let player_pool = pools.get_mut(*player_entity).unwrap();
        if player_pool.hit_points.current < player_pool.hit_points.max {
            player_pool.hit_points.current = i32::min(player_pool.hit_points.current + 1, player_pool.hit_points.max);
            let pos = positions.get(*player_entity);
            if let Some(pos) = pos {
                particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::GREEN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('♥'), 200.0);
            }
        }
        // sometimes restore mana
        if player_pool.mana.current < player_pool.mana.max {
            let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
            if rng.roll_dice(1, 6) == 1 {
                player_pool.mana.current = i32::min(player_pool.mana.current + 1, player_pool.mana.max);
            }
        }
    }

    RunState::Ticking
}

pub fn level_up(ecs: &World, source: Entity, pools: &mut Pools) {
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    gamelog.entries.push(format!("You are now level {}", pools.level + 1));

    let player_pos = ecs.fetch::<rltk::Point>();
    let map = ecs.fetch::<Map>();
    for i in 0..10 {
        if player_pos.y - i > 1 {
            add_effect(None, 
                EffectType::Particle{ 
                    glyph: rltk::to_cp437('░'),
                    fg : rltk::RGB::named(rltk::GOLD),
                    bg : rltk::RGB::named(rltk::BLACK),
                    lifespan: 400.0
                }, 
                Targets::Tile{ tile_idx : map.xy_idx(player_pos.x, player_pos.y - i) as i32 }
            );
        }
    }

    let attributes = ecs.read_storage::<Attributes>();
    let player_attributes = attributes.get(source).unwrap();
    let skills = ecs.read_storage::<Skills>();
    let player_skills = skills.get(source).unwrap();
    let mut pending_level_ups = ecs.write_storage::<PendingLevelUp>();
    pending_level_ups.insert(source, PendingLevelUp{ attributes: player_attributes.clone(), skills: player_skills.clone() }).expect("Unable to insert");
}
