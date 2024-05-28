use specs::prelude::*;
use crate::{Attributes, Duration, StatusEffectChanged, Initiative, MyTurn, Pools, Position, RunState, StatusEffect, DamageOverTime};
use crate::effects::{add_effect, EffectType, Targets};
use crate::rng;
use rltk::Point;

pub struct InitiativeSystem {}

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = (
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MyTurn>,
        Entities<'a>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, Pools>,
        WriteStorage<'a, Duration>,
        WriteStorage<'a, StatusEffectChanged>,
        ReadStorage<'a, StatusEffect>,
        ReadStorage<'a, DamageOverTime>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut initiatives, positions, mut turns, entities,
            attributes, mut runstate, player, player_pos, pools,
            mut durations, mut dirty, statuses, dots) = data;

        if *runstate != RunState::Ticking { return; }
        turns.clear();

        for (entity, initiative, pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;
            if initiative.current < 1 {
                let mut myturn = true;

                // re-roll
                initiative.current = 6 + rng::roll_dice(1, 6);

                // give a bonus for dexterity
                if let Some(attr) = attributes.get(entity) {
                    initiative.current -= attr.dexterity.bonus;
                }

                // apply penalties from equipment
                if let Some(pools) = pools.get(entity) {
                    initiative.current += f32::floor(pools.total_initiative_penalty) as i32;
                }

                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                } else {
                    // prevent entities from acting until the player is nearby
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                        *player_pos,
                        Point::new(pos.x, pos.y)
                    );
                    if distance > 20.0 {
                        myturn = false;
                    }
                }

                if myturn {
                    turns.insert(entity, MyTurn{}).expect("Unable to insert turn");
                }
            }
        }

        // handle durations
        if *runstate == RunState::AwaitingInput {
            for (effect_entity, duration, status) in (&entities, &mut durations, &statuses).join() {
                if entities.is_alive(status.target) {
                    duration.turns -= 1;
                    if let Some(dot) = dots.get(effect_entity) {
                        add_effect(
                            None,
                            EffectType::Damage{ amount: dot.damage, hits_self: false },
                            Targets::Single{ target: status.target }
                        );
                    }
                    if duration.turns < 1 {
                        dirty.insert(status.target, StatusEffectChanged{}).expect("Unable to insert");
                        entities.delete(effect_entity).expect("Unable to delete");
                    }
                }
            }
        }
    }
}
