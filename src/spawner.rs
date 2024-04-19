use rltk::RGB;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};
use std::collections::HashMap;
use super::raws::*;
use crate::{MasterDungeonMap, OtherLevelPosition};
use super::{Pools, Player, Renderable, Name, Position, Viewshed, 
    Rect, SerializeMe, random_table::RandomTable, HungerClock, HungerState, 
    Map, TileType, Attributes, Skills, Pool, LightSource, Faction,
    Initiative, EquipmentChanged, Point, EntryTrigger, TeleportTo, 
    SingleActivation, mana_at_level, player_hp_at_level, StatusEffect,
    Duration, AttributeBonus, KnownSpells
};
use crate::rng;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let attributes = Attributes::default();
    let constitution = attributes.constitution.base;
    let intelligence = attributes.intelligence.base;
    let player = ecs
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
        .with(Initiative{ current: 0 })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(HungerClock{ state: HungerState::WellFed, duration: 20 })
        .with(attributes)
        .with(Skills::default())
        .with(Pools{
            hit_points: Pool{
                current: player_hp_at_level(constitution, 1),
                max: player_hp_at_level(constitution, 1)
            },
            mana: Pool{
                current: mana_at_level(intelligence, 1),
                max: mana_at_level(intelligence, 1)
            },
            xp: 0,
            level: 1,
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
            gold: 0,
            total_armour_class: 10,
            base_damage: "1d4".to_string(),
            god_mode: false
        })
        .with(LightSource{ colour: RGB::from_f32(1.0, 1.0, 0.5), range: 8 })
        .with(Faction{ name: "Player".to_string() })
        .with(EquipmentChanged{})
        .with(KnownSpells{ spells: Vec::new() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // start with a hangover
    ecs.create_entity()
        .with(StatusEffect{ target: player, is_debuff: true })
        .with(Duration{ turns: 30 })
        .with(Name{ name: "Hangover".to_string() })
        .with(AttributeBonus{
            strength: Some(-1),
            dexterity: None,
            constitution: Some(-1),
            intelligence: Some(-1)
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // spawn starting items
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Family Longsword", SpawnType::Equipped{ by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Food Ration", SpawnType::Carried { by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Town Portal Scroll", SpawnType::Carried { by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Wooden Shield", SpawnType::Equipped { by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Leather Boots", SpawnType::Equipped { by: player });
    player
}

pub fn spawn_room(map: &Map, room: &Rect, map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {
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
    spawn_region(map, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(_map: &Map, area: &[usize], map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    {
        // use min to avoid spawning more entites than we have room for
        let num_spawns = i32::min(areas.len() as i32 / 3, rng::roll_dice(1, 7) + (map_depth / 2) - 3);
        if num_spawns == 0 { return; }

        for _ in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 
                0usize 
            } else {
                (rng::roll_dice(1, areas.len() as i32)-1) as usize
            };
            let map_idx = areas[array_index];
            if let Some(roll) = spawn_table.roll() {
                // skip rolls that don't spawn anything
                spawn_points.insert(map_idx, roll);
            }
            areas.remove(array_index); // avoid picking the area again
        }
    }

    // store where and what to spawn
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

    let spawn_result = spawn_named_entity(
        &RAWS.lock().unwrap(), ecs,
        &spawn.1, SpawnType::AtPosition{ x, y }
    );
    if spawn_result.is_some() {
        return;
    }

    rltk::console::log(format!("WARNING: We don't know how to spawn [{}]!", spawn.1));
}

pub fn spawn_town_portal(ecs: &mut World) {
    let map = ecs.fetch::<Map>();
    let player_depth = map.depth;
    let player_pos = ecs.fetch::<Point>();
    let player_x = player_pos.x;
    let player_y = player_pos.y;
    std::mem::drop(player_pos);
    std::mem::drop(map);

    // find a place to put the portal, close to the town exit
    let dm = ecs.fetch::<MasterDungeonMap>();
    let town_map = dm.get_map(0).unwrap();
    let mut stairs_idx = 0;
    for (idx, tt) in town_map.tiles.iter().enumerate() {
        if *tt == TileType::DownStairs {
            stairs_idx = idx;
        }
    }
    let portal_x = (stairs_idx as i32 % town_map.width) - 2;
    let portal_y = stairs_idx as i32 / town_map.width;
    std::mem::drop(dm);

    // spawn the portal
    ecs.create_entity()
        .with(OtherLevelPosition{ x: portal_x, y: portal_y, depth: 0 })
        .with(Renderable{
            glyph: rltk::to_cp437('Ω'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(EntryTrigger{})
        .with(TeleportTo{ x: player_x, y: player_y, depth: player_depth, player_only: true })
        .with(Name{ name: "Town Portal".to_string() })
        .with(SingleActivation{})
        .build();
}

// spawn table
fn room_table(map_depth: i32) -> RandomTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}
