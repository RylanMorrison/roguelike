use specs::prelude::*;
use crate::{gamelog, Quips, Name, MyTurn, Viewshed, RunState};
use crate::rng;
use rltk::Point;

pub struct QuipSystem {}

impl<'a> System<'a> for QuipSystem {
    type SystemData = (
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, MyTurn>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadExpect<'a, RunState>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut quips, names, turns, player_pos, viewsheds, runstate) = data;

        if *runstate != RunState::Ticking { return; }

        for (quip, name, viewshed, _turn) in (&mut quips, &names, &viewsheds, &turns).join() {
            if !quip.available.is_empty() && viewshed.visible_tiles.contains(&player_pos) && rng::roll_dice(1, 6) == 1 {
                let quip_index = if quip.available.len() == 1 {
                    0
                } else {
                    (rng::roll_dice(1, quip.available.len() as i32)-1) as usize
                };
                gamelog::Logger::new()
                    .character_name(&name.name)
                    .append("says")
                    .speech(&quip.available[quip_index])
                    .log();
                quip.available.remove(quip_index);
            }
        }
    }
}
