use specs::prelude::*;
use crate::{Initiative, Position, MyTurn, Attributes, RunState};
use rltk::{RandomNumberGenerator, Point};

pub struct InitiativeSystem {}

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = (
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MyTurn>,
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Point>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut initiatives, positions, mut turns, entities, mut rng,
            attributes, mut runstate, player, player_pos) = data;

        if *runstate != RunState::Ticking { return; }
        turns.clear();

        for (entity, initiative, pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;
            if initiative.current < 1 {
                let mut myturn = true;

                // re-roll
                initiative.current = 6 + rng.roll_dice(1, 6);

                // give a bonus for dexterity
                if let Some(attr) = attributes.get(entity) {
                    initiative.current -= attr.dexterity.bonus;
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
    }
}
