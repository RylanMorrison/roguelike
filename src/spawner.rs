use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};
use std::collections::HashMap;

use super::{
    CombatStats, Player, Renderable, Name, Position, Viewshed, 
    Monster, BlocksTile, Rect, Item, ProvidesHealing, Consumable, 
    Ranged, InflictsDamage, Confusion, SerializeMe, AreaOfEffect, 
    RandomTable, DefenceBonus, EquipmentSlot, Equippable, MagicMapper, 
    MeleePowerBonus, HungerClock, HungerState, ProvidesFood, Map, 
    TileType, BlocksVisibility, Door};

const MAX_MONSTERS: i32 = 4;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player {})
        .with(Name { name: "Player".to_string() })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(CombatStats { max_hp: 30, hp: 30, defence: 2, power: 5 })
        .with(HungerClock{ state: HungerState::WellFed, duration: 20 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

// Monsters
pub fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(
        ecs, x, y, 
        rltk::to_cp437('g'), 
        "Goblin",
        CombatStats{ max_hp: 16, hp: 16, defence: 1, power: 4 }
    ); 
}

pub fn orc(ecs: &mut World, x: i32, y: i32) { 
    monster(
        ecs, x, y, 
        rltk::to_cp437('o'),
        "Orc", 
        CombatStats{ max_hp: 24, hp: 24, defence: 1, power: 6 }
    ); 
}

pub fn ogre(ecs: &mut World, x: i32, y: i32) {
    monster(
        ecs, x, y,
        rltk::to_cp437('O'),
        "Ogre",
        CombatStats{ max_hp: 40, hp: 40, defence: 3, power: 9 }
    )
}

pub fn demon(ecs: &mut World, x: i32, y: i32) {
    monster(
        ecs, x, y,
        rltk::to_cp437('D'),
        "Demon",
        CombatStats{ max_hp: 30, hp: 30, defence: 2, power: 13 }
    )
}

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S, combat_stats: CombatStats) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Monster{})
        .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name{ name: name.to_string() })
        .with(combat_stats)
        .with(BlocksTile{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_room(map: &Map, rng: &mut RandomNumberGenerator, room: &Rect, map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 + 1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }
    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

// TODO: don't spawn on player
pub fn spawn_region(_map: &Map, rng: &mut RandomNumberGenerator, area: &[usize], map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    {
        // use min to avoid spawning more entites than we have room for
        let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3);
        if num_spawns == 0 { return; }

        for _i in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 
                0usize 
            } else {
                (rng.roll_dice(1, areas.len() as i32)-1) as usize
            };
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(rng));
            areas.remove(array_index); // avoid picking the area again
        }
    }

    // actually spawn things
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

// spawn an entity using (location, name)
pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    match spawn.1.as_ref() {
        "Goblin" => goblin(ecs, x, y),
        "Orc" => orc(ecs, x, y),
        "Ogre" => ogre(ecs, x, y),
        "Demon" => demon(ecs, x, y),
        "Health Potion" => health_potion(ecs, x, y),
        "Fireball Scroll" => fireball_scroll(ecs, x, y),
        "Confusion Scroll" => confusion_scroll(ecs, x, y),
        "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
        "Dagger" => dagger(ecs, x, y),
        "Shield" => shield(ecs, x, y),
        "Leather Helmet" => leather_helmet(ecs, x, y),
        "Sword" => sword(ecs, x, y),
        "Tower Shield" => tower_shield(ecs, x, y),
        "Iron Helmet" => iron_helmet(ecs, x, y),
        "Magic Mapping Scroll" => magic_mapping_scroll(ecs, x, y),
        "Food Ration" => food_ration(ecs, x, y),
        "Door" => door(ecs, x, y),
        _ => {}
    }
}

// Consumables
fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Health Potion".to_string() })
        .with(Item{})
        .with(ProvidesHealing{ heal_amount: 8 })
        .with(Consumable{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Magic Missile Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Fireball Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 20 })
        .with(AreaOfEffect{ radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Confusion Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(Confusion{ turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_mapping_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN3),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Magic Mapping Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(MagicMapper{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn food_ration(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('%'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Food Ration".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesFood{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// equipment
fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Dagger".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::MainHand })
        .with(MeleePowerBonus{ power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('0'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Shield".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::OffHand })
        .with(DefenceBonus{ defence: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn leather_helmet(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Leather Helmet".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Head })
        .with(DefenceBonus{ defence: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn sword(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Sword".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::MainHand })
        .with(MeleePowerBonus{ power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('0'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Tower Shield".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::OffHand })
        .with(DefenceBonus{ defence: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn iron_helmet(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Iron Helmet".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Head })
        .with(DefenceBonus{ defence: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// world
fn door(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('+'),
            fg: RGB::named(rltk::CHOCOLATE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Door".to_string() })
        .with(BlocksTile{})
        .with(BlocksVisibility{})
        .with(Door{open: false})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// spawn table
fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10 - map_depth)
        .add("Orc", 1 + map_depth)
        .add("Ogre", map_depth - 4)
        .add("Demon", map_depth - 7)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
        .add("Leather Helmet", 3)
        .add("Sword", map_depth - 4)
        .add("Tower Shield", map_depth - 4)
        .add("Iron Helmet", map_depth - 4)
        .add("Magic Mapping Scroll", 2)
        .add("Food Ration", 10)
}
