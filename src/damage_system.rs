use specs::prelude::*;
use rltk::{RandomNumberGenerator, RGB};
use super::{Pools, SufferDamage, Player, Name, gamelog::GameLog, RunState, Position, Map,
    LootTable, Attributes, particle_system::ParticleBuilder, Point};
use crate::gamesystem::{player_hp_at_level, mana_at_level};
use crate::{raws, spatial};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, ParticleBuilder>,
        ReadExpect<'a, Point>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut pools, mut damage, positions, 
            mut map, entities, player_entity, 
            attributes, mut gamelog, mut particles, 
            player_pos) = data;
        let mut xp_gain = 0;

        for(entity, pool, damage) in (&entities, &mut pools, &damage).join() {
            for dmg in damage.amount.iter() {
                pool.hit_points.current -= dmg.0;
                let pos = positions.get(entity);
                if let Some(pos) = pos {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.bloodstains.insert(idx);
                }
                if pool.hit_points.current < 1 && dmg.1 {
                    xp_gain += pool.level * 100;
                    if let Some(pos) = pos {
                        let idx = map.xy_idx(pos.x, pos.y);
                        spatial::remove_entity(entity, idx);
                    }
                }
            }
        }

        if xp_gain != 0 {
            let player_pool = pools.get_mut(*player_entity).unwrap();
            let player_attributes = attributes.get(*player_entity).unwrap();
            player_pool.xp += xp_gain;
            if player_pool.xp >= player_pool.level * 1000 {
                // level up!
                player_pool.level += 1;
                player_pool.hit_points.max = player_hp_at_level(
                    player_attributes.constitution.base + player_attributes.constitution.modifiers,
                    player_pool.level
                );
                player_pool.hit_points.current = player_pool.hit_points.max;
                player_pool.mana.max = mana_at_level(
                    player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
                    player_pool.level
                );
                player_pool.mana.current = player_pool.mana.max;
                gamelog.entries.push(format!("You are now level {}!", player_pool.level));
                for i in 0..10 {
                    if player_pos.y - i > 1 {
                        particles.add_request(
                            player_pos.x,
                            player_pos.y - i,
                            RGB::named(rltk::GOLD),
                            RGB::named(rltk::BLACK),
                            rltk::to_cp437('░'), 400.0
                        );
                    }
                }
            }
        }
        damage.clear();
    }
}

pub fn delete_the_dead(ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    {
        let pools = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();
        for (entity, pool) in (&entities, &pools).join() {
            if pool.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead", victim_name.name));
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
