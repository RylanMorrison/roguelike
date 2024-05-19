use rltk::RGB;
use specs::prelude::*;
use super::*;
use crate::{determine_roll, gamelog, raws, Attributes, Chest, Confusion, Consumable, Damage, DamageOverTime, Duration, Food, Healing,
    Item, KnownAbilities, KnownAbility, LootTable, MagicMapping, Map, Name, Pools, RestoresMana, RunState, SingleActivation,
    Skills, Slow, SpawnParticleBurst, SpawnParticleLine, Ability, Stun, TeachesAbility, TeleportTo, TownPortal};

pub fn item_trigger(ecs: &mut World, creator: Option<Entity>, item_entity: Entity, targets: &Targets) {
    // check charges
    if let Some(consumable) = ecs.write_storage::<Consumable>().get_mut(item_entity) {
        if consumable.charges < 1 {
            if let Some(item) = ecs.read_storage::<Item>().get(item_entity) {
                gamelog::Logger::new()
                    .item_name(&item)
                    .append("is out of charges.")
                    .log();
            }
            return;
        }
    }
    
    let did_something = event_trigger(ecs, creator, item_entity, targets);

    // delete consumables after use
    if did_something {
        if let Some(consumable) = ecs.write_storage::<Consumable>().get_mut(item_entity) {
            consumable.charges -= 1;
            if consumable.max_charges == 1 {
                ecs.entities().delete(item_entity).expect("Delete failed");
            }
        }
    }
}

pub fn environment_trigger(ecs: &mut World, creator: Option<Entity>, trigger: Entity, targets: &Targets) {
    let did_something = event_trigger(ecs, creator, trigger, targets);

    if did_something && ecs.read_storage::<SingleActivation>().get(trigger).is_some() {
        ecs.entities().delete(trigger).expect("Delete failed");
    }
}

pub fn ability_trigger(ecs: &mut World, creator: Option<Entity>, ability_entity: Entity, targets: &Targets) {
    let mut did_something = false;
    if let Some(ability) = ecs.read_storage::<Ability>().get(ability_entity) {
        let mut pools = ecs.write_storage::<Pools>();
        if let Some(caster) = creator {
            if let Some(pool) = pools.get_mut(caster) {
                if let Some(level) = ability.levels.get(&ability.current_level) {
                    let mana_cost = level.mana_cost.unwrap_or(0);
                    if mana_cost <= pool.mana.current {
                        if !pool.god_mode {
                            pool.mana.current -= mana_cost;
                        }
                        did_something = true;
                    }
                }
            }
        }
    }
    if did_something {
        event_trigger(ecs, creator, ability_entity, targets);
    }
}

fn event_trigger(ecs: &mut World, creator: Option<Entity>, entity: Entity, targets: &Targets) -> bool {
    let mut did_something = false;

    // single particle
    if let Some(particle) = ecs.read_storage::<SpawnParticleBurst>().get(entity) {
        add_effect(
            creator,
            EffectType::Particle{
                glyph: particle.glyph,
                fg: particle.colour,
                bg: RGB::named(rltk::BLACK),
                lifespan: particle.lifetime_ms
            },
            targets.clone()
        );
    }

    // line particles
    if let Some(particle) = ecs.read_storage::<SpawnParticleLine>().get(entity) {
        if let Some(start_pos) = targeting::find_item_position(ecs, entity, creator) {
            match targets {
                Targets::Tile{tile_idx} => spawn_line_particles(ecs, start_pos, *tile_idx, particle),
                Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| spawn_line_particles(ecs, start_pos, *tile_idx, particle)), 
                Targets::Single{target} => {
                    if let Some(end_pos) = entity_position(ecs, *target) {
                        spawn_line_particles(ecs, start_pos, end_pos, particle);
                    }
                }
                Targets::TargetList{targets} => {
                    targets.iter().for_each(|target| {
                        if let Some(end_pos) = entity_position(ecs, *target) {
                            spawn_line_particles(ecs, start_pos, end_pos, particle);
                        }
                    });
                }
            }
        }
    }

    // food
    if ecs.read_storage::<Food>().get(entity).is_some() {
        let items = ecs.read_storage::<Item>();

        add_effect(creator, EffectType::WellFed, targets.clone());
        gamelog::Logger::new()
            .append("You eat the")
            .item_name(&items.get(entity).unwrap())
            .log();
        did_something = true;
    }

    // magic mapper
    if ecs.read_storage::<MagicMapping>().get(entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        gamelog::Logger::new().append("The entire map is revealed!").log();
        *runstate = RunState::MagicMapReveal{ row: 0 };
        did_something = true;
    }

    // town portal
    if ecs.read_storage::<TownPortal>().get(entity).is_some() {
        let map = ecs.fetch::<Map>();
        if map.depth == 0 {
            gamelog::Logger::new().append("You are already in town, the scroll has no effect.").log();
        } else {
            gamelog::Logger::new().append("You are teleported back to town.").log();
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = RunState::TownPortal;
            did_something = true;
        }
    }

    // healing
    if let Some(heal) = ecs.read_storage::<Healing>().get(entity) {
        add_effect(creator, EffectType::Healing{ amount: heal.heal_amount }, targets.clone());
        did_something = true;
    }

    // mana
    if let Some(mana) = ecs.read_storage::<RestoresMana>().get(entity) {
        add_effect(creator, EffectType::Mana{ amount: mana.mana_amount }, targets.clone());
        did_something = true;
    }

    // damage
    if let Some(damage) = ecs.read_storage::<Damage>().get(entity) {
        let abilities = ecs.read_storage::<Ability>();

        let mut amount = determine_roll(&damage.damage);
        if abilities.get(entity).is_some() {
            // add attribute and skill bonuses for abilities
            // TODO put this in its own system
            if let Some(source) = creator {
                let attributes = ecs.read_storage::<Attributes>();
                if let Some(source_attributes) = attributes.get(source) {
                    amount += source_attributes.intelligence.bonus;
                }
                let skills = ecs.read_storage::<Skills>();
                if let Some(source_skills) = skills.get(source) {
                    amount += source_skills.magic.bonus();
                }
            }
        }
        add_effect(creator, EffectType::Damage{ amount }, targets.clone());
        let names = ecs.read_storage::<Name>();
        let items = ecs.read_storage::<Item>();

        // TODO clean this up
        if let Some(item) = items.get(entity) {
            gamelog::Logger::new()
                .character_name(&names.get(creator.unwrap()).unwrap().name)
                .append("deals")
                .damage(amount)
                .append("damage with")
                .item_name(item)
                .log();
        } else if abilities.get(entity).is_some() {
            gamelog::Logger::new()
                .character_name(&names.get(creator.unwrap()).unwrap().name)
                .append("deals")
                .damage(amount)
                .append("damage with")
                .ability_name(&names.get(entity).unwrap().name)
                .log();
        } else {
            gamelog::Logger::new()
                .character_name(&names.get(creator.unwrap()).unwrap().name)
                .append("deals")
                .damage(amount)
                .append("damage with")
                .append(&names.get(entity).unwrap().name)
                .log();
        }
        did_something = true;
    }

    // damage over time
    if let Some(damage) = ecs.read_storage::<DamageOverTime>().get(entity) {
        if let Some(duration) = ecs.read_storage::<Duration>().get(entity) {
            // TODO: damage over time damage should be a dice roll?
            add_effect(creator, EffectType::DamageOverTime{ damage: damage.damage, duration: duration.turns }, targets.clone());
            gamelog::Logger::new()
                .append("Damage over time deals")
                .damage(damage.damage)
                .log();
            did_something = true;
        }
    }

    // confusion
    if ecs.read_storage::<Confusion>().get(entity).is_some() {
        if let Some(duration) = ecs.read_storage::<Duration>().get(entity) {
            add_effect(creator, EffectType::Confusion{ duration: duration.turns }, targets.clone());
            did_something = true;
        }
    }

    // stun
    if ecs.read_storage::<Stun>().get(entity).is_some() {
        if let Some(duration) = ecs.read_storage::<Duration>().get(entity) {
            add_effect(creator, EffectType::Stun{ duration: duration.turns }, targets.clone());
        }
    }

    // slow
    if let Some(slow) = ecs.read_storage::<Slow>().get(entity) {
        if let Some(duration) = ecs.read_storage::<Duration>().get(entity) {
            add_effect(creator, EffectType::Slow{ initiative_penalty: slow.initiative_penalty, duration: duration.turns }, targets.clone());
            did_something = true;
        }
    }

    // teleportation
    if let Some(teleport) = ecs.read_storage::<TeleportTo>().get(entity) {
        add_effect(
            creator,
            EffectType::TeleportTo {
                x: teleport.x,
                y: teleport.y,
                depth: teleport.depth,
                player_only: teleport.player_only
            },
            targets.clone()
        );
        // effect is only evaluated later and won't actually occur if the target isn't the player and player_only is true
        // so did_something could be set to true when teleportation hasn't occurred
        // make sure the player is the one being teleported for do_something to be set to true
        // TODO refactor this when other entities can teleport!
        let player_entity = ecs.fetch::<Entity>();
        match *targets {
            Targets::Single{target} => {
                if target == *player_entity {
                    did_something = true;
                }
            }
            _ => {}
        }
    }

    // attribute modifiers
    if let Some(attribute) = ecs.read_storage::<AttributeBonus>().get(entity) {
        let duration;
        if let Some(dur) = ecs.read_storage::<Duration>().get(entity) {
            duration = dur.turns;
        } else {
            duration = 10;
        }
        add_effect(
            creator,
            EffectType::AttributeEffect {
                bonus: attribute.clone(),
                duration,
                name: ecs.read_storage::<Name>().get(entity).unwrap().name.clone()
            },
            targets.clone()
        );
        did_something = true;
    }

    // learn abilities
    if let Some(teacher) = ecs.read_storage::<TeachesAbility>().get(entity) {
        if let Some(known) = ecs.write_storage::<KnownAbilities>().get_mut(creator.unwrap()) {
            if let Some(ability_entity) = raws::find_ability_entity(ecs, &teacher.ability) {
                if let Some(ability) = ecs.read_storage::<Ability>().get(ability_entity) {
                    let mut already_known = false;
                    known.abilities.iter().for_each(|s| if s.name == teacher.ability { already_known = true });
                    if !already_known {
                        known.abilities.push(KnownAbility{ 
                            name: teacher.ability.clone(), 
                            mana_cost: ability.levels.get(&ability.current_level).unwrap().mana_cost.unwrap_or(0),
                            level: ability.current_level // need each character to keep track of their abilities?
                        });
                        gamelog::Logger::new()
                            .append("You now know how ")
                            .ability_name(&teacher.ability)
                            .log();
                        did_something = true;
                    } else {
                        gamelog::Logger::new()
                            .append("You already know how to cast")
                            .ability_name(&teacher.ability)
                            .log();
                    }
                }
            }
        }
    }

    // open chests
    let mut spawn_loot: Vec<(String, i32, i32)> = Vec::new();
    if let Some(chest) = ecs.read_storage::<Chest>().get(entity) {
        if creator.is_some() {
            if let Some(gold) = &chest.gold {
                if let Some(pools) = ecs.write_storage::<Pools>().get_mut(creator.unwrap()) {
                    pools.gold += determine_roll(gold);
                    did_something = true;
                }
            }
            if let Some(loot_table) = ecs.read_storage::<LootTable>().get(entity) {
                for _ in 0..chest.capacity {
                    let item_drop = raws::get_item_drop(
                        &raws::RAWS.lock().unwrap(),
                        &loot_table.table_name
                    );
                    if let Some(drop) = item_drop {
                        match targets {
                            Targets::Tile{tile_idx} => {
                                let map = ecs.fetch::<Map>();
                                let (x, y) = map.idx_xy(*tile_idx as usize);
                                spawn_loot.push((drop, x, y));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    for loot in spawn_loot.iter() {
        raws::spawn_named_item(
            &raws::RAWS.lock().unwrap(),
            ecs,
            &loot.0,
            raws::SpawnType::AtPosition { x: loot.1, y: loot.2 }
        );
        did_something = true;
    }

    did_something
}

fn spawn_line_particles(ecs: &World, start: i32, end: i32, particle: &SpawnParticleLine) {
    let map = ecs.fetch::<Map>();
    let start_pt = rltk::Point::new(start % map.width, start / map.width);
    let end_pt = rltk::Point::new(end % map.width, end / map.width);
    let line = rltk::line2d(rltk::LineAlg::Bresenham, start_pt, end_pt);
    for pt in line.iter() {
        add_effect(
            None,
            EffectType::Particle{
                glyph: particle.glyph,
                fg: particle.colour,
                bg: RGB::named(rltk::BLACK),
                lifespan: particle.lifetime_ms
            },
            Targets::Tile{ tile_idx: map.xy_idx(pt.x, pt.y) as i32 }
        );
    }
}
