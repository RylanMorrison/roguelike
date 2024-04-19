use rltk::RandomNumberGenerator;
use specs::prelude::*;
use crate::{raws, spatial, Chasing, Faction, Map, MyTurn, Name, Position, SpecialAbilities, Spell, Viewshed,
    WantsToApproach, WantsToCastSpell, WantsToFlee, Confusion};
use crate::raws::{Reaction, faction_reaction, RAWS};

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
        ReadStorage<'a, SpecialAbilities>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteStorage<'a, WantsToCastSpell>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Spell>,
        ReadStorage<'a, Confusion>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (turns, factions, positions, map, mut want_approach, mut want_flee,
            entities, player, viewsheds, mut chasing, special_abilities,
            mut rng, mut wants_cast, names, spells, confused) = data;

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
                        reaction.1 = Reaction::Attack;
                    }
                    match reaction.1 {
                        Reaction::Attack => {
                            if let Some(abilities) = special_abilities.get(entity) {
                                let range = rltk::DistanceAlg::Pythagoras.distance2d(
                                    rltk::Point::new(pos.x, pos.y),
                                    rltk::Point::new(reaction.0 as i32 % map.width, reaction.0 as i32 / map.width)
                                );
                                for ability in abilities.abilities.iter() {
                                    if range >= ability.min_range && range <= ability.range
                                    && rng.roll_dice(1, 100) >= (ability.chance * 100.0) as i32 {
                                        wants_cast.insert(
                                            entity,
                                            WantsToCastSpell{
                                                spell: raws::find_spell_entity_by_name(&ability.spell, &names, &spells, &entities).unwrap(),
                                                target: Some(rltk::Point::new(reaction.0 as i32 % map.width, reaction.0 as i32 / map.width))
                                            }
                                        ).expect("Unable to insert");
                                        done = true;
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
