use rltk::RGB;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};
use std::collections::HashMap;
use crate::raws::*;
use crate::{MasterDungeonMap, OtherLevelPosition, StatusEffectChanged};
use crate::{Pools, Player, Renderable, Name, Position, Viewshed,
    Rect, SerializeMe, random_table::RandomTable, HungerClock, HungerState,
    Map, TileType, Attributes, Skills, Pool, LightSource, Faction,
    Initiative, EquipmentChanged, Point, EntryTrigger, TeleportTo,
    SingleActivation, mana_at_level, hp_at_level, StatusEffect,
    Duration, AttributeBonus, KnownAbilities, EntityVec, InitiativePenalty,
    MapMarker, ItemQuality
};
use crate::rng;
use crate::map::Marker;

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
        .with(HungerClock{ state: HungerState::Normal, duration: 20 })
        .with(attributes)
        .with(Skills::default())
        .with(Pools{
            hit_points: Pool{
                current: hp_at_level(constitution, 1),
                max: hp_at_level(constitution, 1)
            },
            mana: Pool{
                current: mana_at_level(intelligence, 1),
                max: mana_at_level(intelligence, 1)
            },
            xp: 0,
            level: 1,
            total_weight: 0.0,
            initiative_penalty: InitiativePenalty::initial(),
            gold: 0,
            total_armour_class: 10,
            base_damage: "1d4".to_string(),
            god_mode: false
        })
        .with(LightSource{ colour: RGB::from_f32(1.0, 1.0, 0.5), range: 8 })
        .with(Faction{ name: "Player".to_string() })
        .with(EquipmentChanged{})
        .with(StatusEffectChanged{})
        .with(KnownAbilities{ abilities: EntityVec::new() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // start with a hangover
    ecs.create_entity()
        .with(StatusEffect{ target: player, is_debuff: true })
        .with(Duration{ turns: 30 })
        .with(Name{ name: "Hangover".to_string() })
        .with(AttributeBonus{
            strength: Some(-1),
            dexterity: Some(-1),
            constitution: Some(-1),
            intelligence: Some(-1)
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    player
}

pub fn spawn_starting_gear(ecs: &mut World, raws: &RawMaster, equipment: &Vec<String>, items: &Vec<String>) {
    let player = *ecs.read_resource::<Entity>();
    for item in equipment.iter() {
        spawn_named_item(raws, ecs, item.as_str(), SpawnType::Equipped{ by: player }, ItemQuality::Worn);
    }
    for item in items.iter() {
        spawn_named_entity(raws, ecs, item.as_str(), SpawnType::Carried { by: player });
    }
}

pub fn spawn_room(map: &mut Map, room: &Rect) {
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
    spawn_region(map, &possible_targets);
}

pub fn spawn_region(map: &mut Map, area: &[usize]) {
    let spawn_table = room_table(&map.name);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    {
        // use min to avoid spawning more entites than we have room for
        let num_spawns = i32::min(areas.len() as i32 / 3, rng::roll_dice(1, 7) + (map.area_level / 2) - 3);
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
        map.spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

// spawn an entity using (location, name)
pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) -> Option<Entity> {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    let spawn_result = spawn_named_entity(
        &RAWS.lock().unwrap(), ecs,
        &spawn.1, SpawnType::AtPosition{ x, y }
    );
    if spawn_result.is_none() {
        rltk::console::log(format!("WARNING: We don't know how to spawn [{}]!", spawn.1));
        return None;
    }

    // use the entity's spawn location for any map markers the entity has
    let markers = ecs.read_storage::<MapMarker>();
    let mut map = ecs.fetch_mut::<Map>();
    if let Some(marker) = markers.get(spawn_result.unwrap()) {
        map.markers.insert(*spawn.0, Marker {
            glyph: marker.glyph,
            fg: marker.fg,
            bg: marker.bg
        });
    }

    spawn_result
}

pub fn spawn_town_portal(ecs: &mut World) -> (i32, i32) {
    let map = ecs.fetch::<Map>();
    let player_pos = ecs.fetch::<Point>();
    let player_x = player_pos.x;
    let player_y = player_pos.y;
    let current_map_name = map.name.clone();
    std::mem::drop(player_pos);
    std::mem::drop(map);

    // find a place to put the portal, close to the town exit
    let dm = ecs.fetch::<MasterDungeonMap>();
    // TODO find closest town map
    let town_map = dm.get_map("Landfall").unwrap();
    let mut stairs_idx = 0;
    for (idx, tt) in town_map.tiles.iter().enumerate() {
        if matches!(*tt, TileType::NextArea{..}) {
            stairs_idx = idx;
        }
    }
    let portal_x = (stairs_idx as i32 % town_map.width) - 2;
    let portal_y = stairs_idx as i32 / town_map.width;
    std::mem::drop(dm);

    // spawn the portal
    ecs.create_entity()
        .with(OtherLevelPosition{ x: portal_x, y: portal_y, map_name: town_map.name })
        .with(Renderable{
            glyph: rltk::to_cp437('Ω'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(EntryTrigger{})
        .with(TeleportTo{ x: player_x, y: player_y, map_name: current_map_name, depth: None, player_only: true })
        .with(Name{ name: "Town Portal".to_string() })
        .with(SingleActivation{})
        .build();

    (portal_x, portal_y)
}

// spawn table
fn room_table(map_name: &str) -> RandomTable {
    get_spawn_table_for_map(&RAWS.try_lock().unwrap(), map_name)
}
