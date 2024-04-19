use specs::prelude::*;
use crate::{Stun, MyTurn, RunState, StatusEffect, Confusion};
use std::collections::HashSet;
use crate::effects::{EffectType, Targets, add_effect};

pub struct TurnStatusSystem {}

impl<'a> System<'a> for TurnStatusSystem {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        ReadStorage<'a, Stun>,
        ReadStorage<'a, Confusion>,
        Entities<'a>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, StatusEffect>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, stunned, confused, 
            entities, runstate,  statuses) = data;

        if *runstate != RunState::Ticking { return; }

        let mut entity_turns = HashSet::new();
        for (entity, _turn) in (&entities, &turns).join() {
            entity_turns.insert(entity);
        }

        let mut skip_turn: Vec<Entity> = Vec::new();
        for (effect_entity, status_effect) in (&entities, &statuses).join() {
            if entity_turns.contains(&status_effect.target) {
                if stunned.get(effect_entity).is_some() {
                    add_effect(
                        None,
                        EffectType::Particle{
                            glyph: rltk::to_cp437('?'),
                            fg: rltk::RGB::named(rltk::CYAN),
                            bg: rltk::RGB::named(rltk::BLACK),
                            lifespan: 200.0
                        },
                        Targets::Single{ target: status_effect.target }
                    );
                    skip_turn.push(status_effect.target);
                } else if confused.get(effect_entity).is_some() {
                    // stun should take precedence over confusion
                    add_effect(
                        None,
                        EffectType::Particle{
                            glyph: rltk::to_cp437('?'),
                            fg: rltk::RGB::named(rltk::MAGENTA),
                            bg: rltk::RGB::named(rltk::BLACK),
                            lifespan: 200.0
                        },
                        Targets::Single{ target: status_effect.target }
                    );
                }
            }
        }

        for e in skip_turn {
            turns.remove(e);
        }
    }
}
