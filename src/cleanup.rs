use specs::prelude::*;
use rltk::RandomNumberGenerator;
use super::{Pools, Player, Name, RunState, Position, LootTable};
use crate::raws;
use crate::gamelog;

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
    let mut to_spawn: Vec<(String, Position)> = Vec::new();
    {
        let positions = ecs.write_storage::<Position>();
        let loot_tables = ecs.read_storage::<LootTable>();
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        for victim in dead.iter() {
            let pos = positions.get(*victim);
            if let Some(table) = loot_tables.get(*victim) {
                let drop_finder = raws::get_item_drop(
                    &raws::RAWS.lock().unwrap(),
                    &mut rng,
                    &table.table_name
                );
                // store what loot to spawn
                if let Some(tag) = drop_finder {
                    if let Some(pos) = pos {
                        to_spawn.push((tag, pos.clone()));
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
