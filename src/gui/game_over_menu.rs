use rltk::prelude::*;
use super::{yellow, black, white, orange, magenta};
use crate::gamelog;

pub enum GameOverResult {
    NoSelection,
    QuitToMenu
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    let mut draw_batch = DrawBatch::new();
    draw_batch.print_color_centered(15, "Your journey has ended!", ColorPair:: new(yellow(), black()));

    draw_batch.print_color_centered(17, &format!("You lived for {} turns.", gamelog::get_event_count("Turn")), ColorPair::new(white(), black()));
    draw_batch.print_color_centered(18, &format!("You took {} total damage.", gamelog::get_event_count("Damage Taken")), ColorPair::new(orange(), black()));
    draw_batch.print_color_centered(19, &format!("You dealt {} total damage.", gamelog::get_event_count("Damage Dealt")), ColorPair::new(orange(), black()));
    draw_batch.print_color_centered(20, &format!("You killed {} enemies.", gamelog::get_event_count("Kill")), ColorPair::new(yellow(), black()));

    draw_batch.print_color_centered(22, "Press any key to return to the main menu.", ColorPair::new(magenta(), black()));

    draw_batch.submit(2000).expect("Draw batch submission failed");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu
    }
}
