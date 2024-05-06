use specs::prelude::*;
use rltk::RGB;
use crate::{HungerClock, HungerState, MyTurn, RunState};
use crate::effects::{add_effect, EffectType, Targets};
use crate::gamelog;

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, MyTurn>,
        ReadExpect<'a, RunState>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity,
            turns, runstate) = data;

        if *runstate != RunState::AwaitingInput { return; }

        for (entity, clock, _myturn) in (&entities, &mut hunger_clock, &turns).join() {
            // only processes if it is the entities turn
            clock.duration -= 1;
            if clock.duration < 1 {
                match clock.state {
                    HungerState::WellFed => {
                        clock.state = HungerState::Normal;
                        clock.duration = 200;
                        if entity == *player_entity {
                            gamelog::Logger::new().append("You are no longer well fed").log();
                        }
                    }
                    HungerState::Normal => {
                        clock.state = HungerState::Hungry;
                        clock.duration = 200;
                        if entity == *player_entity {
                            gamelog::Logger::new().colour(RGB::named(rltk::ORANGE)).append("You are hungry.").log();
                        }
                    }
                    HungerState::Hungry => {
                        clock.state = HungerState::Starving;
                        clock.duration = 200;
                        if entity == *player_entity {
                            gamelog::Logger::new().colour(RGB::named(rltk::RED)).append("You are starving!").log();
                        }
                    }
                    HungerState::Starving => {
                        if entity == *player_entity {
                            gamelog::Logger::new().colour(RGB::named(rltk::CRIMSON)).append("Tummy hurts! You suffer 1 hp damage.").log();
                        }
                        add_effect(
                            None,
                            EffectType::Damage{ amount: 1 },
                            Targets::Single{ target: entity }
                        )
                    }
                }
            }
        }
    }
}
