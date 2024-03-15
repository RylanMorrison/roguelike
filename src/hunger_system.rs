use specs::prelude::*;
use super::{HungerClock, HungerState, gamelog::GameLog, MyTurn, Map};
use crate::effects::{add_effect, EffectType, Targets};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, MyTurn>,
        ReadExpect<'a, Map>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity,
            mut gamelog, turns, map) = data;

        if map.depth == 0 { return; }

        for (entity, clock, _myturn) in (&entities, &mut hunger_clock, &turns).join() {
            // only processes if it is the entities turn
            clock.duration -= 1;
            if clock.duration < 1 {
                match clock.state {
                    HungerState::WellFed => {
                        clock.state = HungerState::Normal;
                        clock.duration = 200;
                        if entity == *player_entity {
                            gamelog.entries.push("You are no longer well fed.".to_string());
                        }
                    }
                    HungerState::Normal => {
                        clock.state = HungerState::Hungry;
                        clock.duration = 200;
                        if entity == *player_entity {
                            gamelog.entries.push("You are hungry.".to_string());
                        }
                    }
                    HungerState::Hungry => {
                        clock.state = HungerState::Starving;
                        clock.duration = 200;
                        if entity == *player_entity {
                            gamelog.entries.push("You are starving!".to_string());
                        }
                    }
                    HungerState::Starving => {
                        if entity == *player_entity {
                            gamelog.entries.push("Tummy hurts! You suffer 1 hp damage.".to_string());
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

