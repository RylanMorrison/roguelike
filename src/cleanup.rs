use specs::prelude::*;
use std::collections::HashMap;
use super::{Pools, Player, Name, RunState, Position, LootTable};
use crate::raws;
use crate::gamelog;
use crate::rng;

pub fn delete_the_dead(ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    {
        let pools = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        for (entity, pool) in (&entities, &pools).join() {
            if pool.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            gamelog::Logger::new()
                                .character_name(&victim_name.name)
                                .append("is dead.")
                                .log();
                        }
                        dead.push(entity);
                    },
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    // loot
    let mut to_spawn: HashMap<String, Position> = HashMap::new();
    {
        let positions = ecs.write_storage::<Position>();
        let loot_tables = ecs.read_storage::<LootTable>();
        for victim in dead.iter() {
            let position = positions.get(*victim);
            if let Some(table) = loot_tables.get(*victim) {
                for _ in 1..4 {
                    let roll = rng::roll_dice(1, 4);
                    if roll == 4 {
                        let item_drop = raws::get_item_drop(
                            &raws::RAWS.lock().unwrap(),
                            &table.table_name
                        );
                        // store what loot to spawn
                        if let Some(drop) = item_drop {
                            if let Some(pos) = position {
                                if !to_spawn.contains_key(&drop) {
                                    to_spawn.insert(drop, pos.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    {
        for drop in to_spawn.iter() {
            raws::spawn_named_item(
                &raws::RAWS.lock().unwrap(),
                ecs,
                &drop.0,
                raws::SpawnType::AtPosition{ x: drop.1.x, y: drop.1.y }
            );
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }    
}
