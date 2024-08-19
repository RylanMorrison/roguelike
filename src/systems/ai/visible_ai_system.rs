use specs::prelude::*;
use crate::{spatial, AbilityType, Chasing, Confusion, Equipped, Faction, KnownAbilities, KnownAbility, Map, MyTurn, Position, Ranged, Viewshed, WantsToApproach, WantsToFlee, WantsToShoot, WantsToUseAbility, Weapon};
use crate::raws::{Reaction, faction_reaction, RAWS};
use crate::rng;

pub struct VisibleAI {}

impl<'a> System<'a> for VisibleAI {
    type SystemData = (
        ReadStorage<'a, MyTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, WantsToFlee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Viewshed>,
        WriteStorage<'a, Chasing>,
        ReadStorage<'a, KnownAbility>,
        ReadStorage<'a, KnownAbilities>,
        WriteStorage<'a, WantsToUseAbility>,
        ReadStorage<'a, Ranged>,
        ReadStorage<'a, Confusion>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, Weapon>,
        WriteStorage<'a, WantsToShoot>
        
    );

    fn run(&mut self, data: Self::SystemData) {
        let (turns, factions, positions, map, mut want_approach, mut want_flee,
            entities, player, viewsheds, mut chasing, known_abilities,
            known_ability_lists, mut wants_cast, ranged, confused,
            equipped, weapons, mut wants_shoot) = data;

        for (entity, _turn, my_faction, pos, viewshed) in (&entities, &turns, &factions, &positions, &viewsheds).join() {
            if entity != *player {
                let my_idx = map.xy_idx(pos.x, pos.y);
                let mut reactions: Vec<(usize, Reaction, Entity)> = Vec::new();
                for visible_tile in viewshed.visible_tiles.iter() {
                    let idx = map.xy_idx(visible_tile.x, visible_tile.y);
                    if my_idx != idx {
                        evaluate(idx, &factions, &my_faction.name, &mut reactions);
                    }
                }

                let mut done = false;
                let mut flee: Vec<usize> = Vec::new();
                for reaction in reactions.iter_mut() {
                    if confused.get(entity).is_some() {
                        // confused entities attack everything
                        reaction.1 = Reaction::Attack; // TODO make sure this isn't permanent
                    }
                    match reaction.1 {
                        Reaction::Attack => {
                            let range = rltk::DistanceAlg::Pythagoras.distance2d(
                                rltk::Point::new(pos.x, pos.y),
                                rltk::Point::new(reaction.0 as i32 % map.width, reaction.0 as i32 / map.width)
                            );
                            if let Some(ability_entities) = known_ability_lists.get(entity) {
                                if rng::roll_dice(1, 100) >= 50 { // TODO ability chance per ability/entity
                                    let mut potential_abilities: Vec<Entity> = Vec::new();

                                    for ability_entity in ability_entities.abilities.iter() {
                                        let known_ability = known_abilities.get(*ability_entity).unwrap();
                                        if known_ability.ability_type == AbilityType::Passive { continue; }

                                        if let Some(ranged) = ranged.get(*ability_entity) {
                                            if range > ranged.max_range || range < ranged.min_range { continue; }
                                        }

                                        potential_abilities.push(*ability_entity);
                                    }

                                    if potential_abilities.len() >= 1 {
                                        // pick a single random ability to use
                                        let random_ability: Entity = *potential_abilities.get(
                                            rng::roll_dice(0, potential_abilities.len() as i32 - 1) as usize
                                        ).unwrap();

                                        wants_cast.insert(
                                            entity,
                                            WantsToUseAbility{
                                                ability: random_ability,
                                                target: Some(rltk::Point::new(reaction.0 as i32 % map.width, reaction.0 as i32 / map.width))
                                            }
                                        ).expect("Unable to insert");

                                        done = true;
                                    }
                                }
                            }
                            if !done {
                                for (weapon, equip) in (&weapons, &equipped).join() {
                                    if let Some(weapon_range) = weapon.range {
                                        if equip.owner == entity {
                                            if weapon_range >= range as i32 {
                                                wants_shoot.insert(entity, WantsToShoot{ target: reaction.2 }).expect("Insert failed");
                                                done = true;
                                            }
                                        }
                                    }
                                }
                            }
                            if !done {
                                want_approach.insert(entity, WantsToApproach{ idx: reaction.0 as i32}).expect("Unable to insert");
                                chasing.insert(entity, Chasing{ target: reaction.2 }).expect("Unable to insert");
                                done = true;
                            }
                        }
                        Reaction::Flee => {
                            flee.push(reaction.0);
                        }
                        _ => {}
                    }

                    if !done && !flee.is_empty() {
                        want_flee.insert(entity, WantsToFlee{ indices: flee.clone() }).expect("Unable to insert");
                    }
                }
            }
        }
    }
}

fn evaluate(idx: usize, factions: &ReadStorage<Faction>, my_faction: &str, reactions: &mut Vec<(usize, Reaction, Entity)>) {
    spatial::for_each_tile_content(idx, |other_entity| {
        if let Some(faction) = factions.get(other_entity) {
            reactions.push((
                idx,
                faction_reaction(my_faction, &faction.name, &RAWS.lock().unwrap()),
                other_entity
            ));
        }
    });
}
