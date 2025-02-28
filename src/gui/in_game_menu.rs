use rltk::prelude::*;
use super::{yellow, black, magenta, white};
use crate::{State, RunState};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum InGameMenuSelection { Continue, NewGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum InGameMenuResult { NoSelection{ selected: InGameMenuSelection }, Selected{ selected: InGameMenuSelection } }

pub fn in_game_menu(gs: &mut State, ctx: &mut Rltk) -> InGameMenuResult {
  let runstate = gs.ecs.fetch::<RunState>();
  let mut draw_batch = DrawBatch::new();

  draw_batch.print_color_centered(15, "Roguelike", ColorPair::new(yellow(), black()));

  if let RunState::InGameMenu{menu_selection: selection} = *runstate {
      // menu items and selection highlighting
      if selection == InGameMenuSelection::Continue {
          draw_batch.print_color_centered(24, "Continue", ColorPair::new(magenta(), black()));
      } else {
          draw_batch.print_color_centered(24, "Continue", ColorPair::new(white(), black()));
      }

      if selection == InGameMenuSelection::NewGame {
          draw_batch.print_color_centered(26, "New Game", ColorPair::new(magenta(), black()));
      } else {
          draw_batch.print_color_centered(26, "New Game", ColorPair::new(white(), black()));
      }

      if selection == InGameMenuSelection::Quit {
          draw_batch.print_color_centered(28, "Quit", ColorPair::new(magenta(), black()));
      } else {
          draw_batch.print_color_centered(28, "Quit", ColorPair::new(white(), black()));
      }

      draw_batch.submit(2000).expect("Draw batch submission failed");

      // menu interaction
      match ctx.key {
          None => return InGameMenuResult::NoSelection{ selected: selection },
          Some(key) => {
              match key {
                  VirtualKeyCode::Escape => { return InGameMenuResult::Selected{ selected: InGameMenuSelection::Continue } }
                  VirtualKeyCode::Up => {
                      let newselection;
                      match selection {
                        InGameMenuSelection::Continue => newselection = InGameMenuSelection::Quit,
                        InGameMenuSelection::NewGame => newselection = InGameMenuSelection::Continue,
                        InGameMenuSelection::Quit => newselection = InGameMenuSelection::NewGame
                      }
                      return InGameMenuResult::NoSelection{ selected: newselection }
                  }
                  VirtualKeyCode::Down => {
                      let newselection;
                      match selection {
                        InGameMenuSelection::Continue => newselection = InGameMenuSelection::NewGame,
                        InGameMenuSelection::NewGame => newselection = InGameMenuSelection::Quit,
                        InGameMenuSelection::Quit => newselection = InGameMenuSelection::Continue
                      }
                      return InGameMenuResult::NoSelection{ selected: newselection }
                  }
                  VirtualKeyCode::Return => return InGameMenuResult::Selected{ selected : selection },
                  _ => return InGameMenuResult::NoSelection{ selected: selection }
              }
          }
      }
  }

  InGameMenuResult::NoSelection { selected: InGameMenuSelection::Continue }
}
