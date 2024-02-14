use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};
use std::collections::HashMap;

use super::{
    CombatStats, Player, Renderable, Name, Position, Viewshed, 
    Monster, BlocksTile, Rect, Item, ProvidesHealing, map::MAPWIDTH, 
    Consumable, Ranged, InflictsDamage, Confusion, SerializeMe,
    AreaOfEffect, RandomTable, DefenceBonus, EquipmentSlot, Equippable,
    MagicMapper, MeleePowerBonus, HungerClock, HungerState, ProvidesFood};

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

pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();

    // store where to spawn things
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        for _ in 0 .. num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // actually spawn things
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

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
            _ => {}
        }
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
            fg: RGB::named(rltk::CYAN),
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
            fg: RGB::named(rltk::CYAN),
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
            fg: RGB::named(rltk::CYAN),
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
        .add("Sword", map_depth - 3)
        .add("Tower Shield", map_depth - 3)
        .add("Iron Helmet", map_depth - 3)
        .add("Magic Mapping Scroll", 2)
        .add("Food Ration", 10)
}
