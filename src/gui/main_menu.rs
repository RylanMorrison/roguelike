use rltk::prelude::*;
use super::{yellow, black, magenta, white};
use crate::{saveload_system, State, RunState};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }

pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let save_exists = saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    let mut draw_batch = DrawBatch::new();

    draw_batch.print_color_centered(15, "Taverns of Stoner Doom", ColorPair::new(yellow(), black()));

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        // menu items and selection highlighting
        if selection == MainMenuSelection::NewGame {
            draw_batch.print_color_centered(24, "Begin New Game", ColorPair::new(magenta(), black()));
        } else {
            draw_batch.print_color_centered(24, "Begin New Game", ColorPair::new(white(), black()));
        }

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                draw_batch.print_color_centered(25, "Load Game", ColorPair::new(magenta(), black()));
            } else {
                draw_batch.print_color_centered(25, "Load Game", ColorPair::new(white(), black()));
            }
        }

        if selection == MainMenuSelection::Quit {
            draw_batch.print_color_centered(26, "Quit", ColorPair::new(magenta(), black()));
        } else {
            draw_batch.print_color_centered(26, "Quit", ColorPair::new(white(), black()));
        }

        draw_batch.submit(2000).expect("Draw batch submission failed");

        // menu interaction
        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit } }
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}
