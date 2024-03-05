use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};
use std::collections::HashMap;

use super::{
    Pools, Player, Renderable, Name, Position, Viewshed, 
    Rect, SerializeMe, random_table::RandomTable, HungerClock, HungerState, 
    Map, TileType, raws::*, Attributes, Skills, Pool, LightSource, Faction,
    Initiative, mana_at_level, player_hp_at_level};

const MAX_MONSTERS: i32 = 4;

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
            level: 1
        })
        .with(LightSource{ colour: RGB::from_f32(1.0, 1.0, 0.5), range: 8 })
        .with(Faction{ name: "Player".to_string() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // spawn starting items
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Family Longsword", SpawnType::Equipped{ by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Food Ration", SpawnType::Carried { by: player });
    player
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
            if let Some(roll) = spawn_table.roll(rng) {
                // skip rolls that don't spawn anything
                spawn_points.insert(map_idx, roll);
            }
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

    let spawn_result = spawn_named_entity(
        &RAWS.lock().unwrap(), ecs,
        &spawn.1, SpawnType::AtPosition{ x, y }
    );
    if spawn_result.is_some() {
        return;
    }

    rltk::console::log(format!("WARNING: We don't know how to spawn [{}]!", spawn.1));
}

// spawn table
fn room_table(map_depth: i32) -> RandomTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}
