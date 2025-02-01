use rltk::prelude::*;
use super::{yellow, black, red, cyan, green, white};
use crate::{State, RunState};
use crate::raws;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum CharacterClassSelection { Warrior, Sorceror, Ranger }

#[derive(PartialEq, Copy, Clone)]
pub enum CharacterClassResult { NoSelection{ selected: CharacterClassSelection}, Selected{ selected: CharacterClassSelection } }

pub fn character_class_select_menu(gs: &mut State, ctx: &mut Rltk) -> CharacterClassResult {
    let runstate = gs.ecs.fetch::<RunState>();
    let mut draw_batch = DrawBatch::new();

    draw_batch.print_color_centered(15, "Roguelike", ColorPair::new(yellow(), black()));
    draw_batch.print_color_centered(18, "Choose your character class", ColorPair::new(yellow(), black()));

    if let RunState::CharacterClassSelectMenu{ menu_selection: selection } = *runstate {
        if selection == CharacterClassSelection::Warrior {
            draw_batch.print_color_centered(25, "Warrior", ColorPair::new(red(), black()));
            draw_batch.print_color_centered(
                26,
                raws::get_character_class_description(&raws::RAWS.lock().unwrap(), "Warrior").unwrap(),
                ColorPair::new(red(), black())
            );
        } else {
            draw_batch.print_color_centered(25, "Warrior", ColorPair::new(white(), black()));
        }

        if selection == CharacterClassSelection::Sorceror {
            draw_batch.print_color_centered(28, "Sorceror", ColorPair::new(cyan(), black()));
            draw_batch.print_color_centered(
                29,
                raws::get_character_class_description(&raws::RAWS.lock().unwrap(),
                "Sorceror").unwrap(),
                ColorPair::new(cyan(), black())
            );
        } else {
            draw_batch.print_color_centered(28, "Sorceror", ColorPair::new(white(), black()));
        }

        if selection == CharacterClassSelection::Ranger {
            draw_batch.print_color_centered(31, "Ranger", ColorPair::new(green(), black()));
            draw_batch.print_color_centered(
                32,
                raws::get_character_class_description(&raws::RAWS.lock().unwrap(),
                "Ranger").unwrap(),
                ColorPair::new(green(), black())
            );
        } else {
            draw_batch.print_color_centered(31, "Ranger", ColorPair::new(white(), black()));
        }

        draw_batch.submit(2000).expect("Draw batch submission failed");

        match ctx.key {
            None => return CharacterClassResult::NoSelection { selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Up => {
                        let new_selection;
                        match selection {
                            CharacterClassSelection::Warrior => new_selection = CharacterClassSelection::Ranger,
                            CharacterClassSelection::Sorceror => new_selection = CharacterClassSelection::Warrior,
                            CharacterClassSelection::Ranger => new_selection = CharacterClassSelection::Sorceror
                        }
                        return CharacterClassResult::NoSelection { selected: new_selection };
                    },
                    VirtualKeyCode::Down => {
                        let new_selection;
                        match selection {
                            CharacterClassSelection::Warrior => new_selection = CharacterClassSelection::Sorceror,
                            CharacterClassSelection::Sorceror => new_selection = CharacterClassSelection::Ranger,
                            CharacterClassSelection::Ranger => new_selection = CharacterClassSelection::Warrior
                        }
                        return CharacterClassResult::NoSelection { selected: new_selection };
                    },
                    VirtualKeyCode::Return => return CharacterClassResult::Selected { selected: selection },
                    _ => return CharacterClassResult::NoSelection { selected: selection }
                }
            }
        }
    }

    CharacterClassResult::NoSelection { selected: CharacterClassSelection::Warrior }
}
