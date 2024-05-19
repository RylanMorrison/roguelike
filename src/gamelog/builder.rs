use rltk::prelude::*;
use super::{LogFragment, append_entry};
use crate::raws;
use crate::Item;

#[derive(Clone)]
pub struct Logger {
    current_colour: RGB,
    fragments: Vec<LogFragment>
}

impl Logger {
    pub fn new() -> Self {
        Logger{
            current_colour: RGB::named(rltk::WHITE),
            fragments: Vec::new()
        }
    }

    pub fn colour(mut self, colour: RGB) -> Self {
        self.current_colour = colour;
        self
    }

    pub fn reset_colour(mut self) -> Self {
        self.current_colour = RGB::named(rltk::WHITE);
        self
    }

    pub fn append<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(
            LogFragment{
                colour: self.current_colour,
                text: text.to_string()
            }
        );
        self
    }

    pub fn character_name<T: ToString>(mut self, text: T) -> Self {
        let colour = if text.to_string() == "Player" {
            RGB::named(rltk::YELLOW)
        } else {
            RGB::named(rltk::WHITE)
        };
        
        self.fragments.push(
            LogFragment{
                colour,
                text: text.to_string()
            }
        );
        self
    }

    pub fn item_name(mut self, item: &Item) -> Self {
        let colour = raws::get_item_colour(item, &raws::RAWS.lock().unwrap());

        self.fragments.push(
            LogFragment{
                colour,
                text: item.full_name()
            }
        );
        self
    }

    pub fn ability_name<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(
            LogFragment{
                colour: RGB::named(rltk::CYAN),
                text: text.to_string()
            }
        );
        self
    }

    pub fn damage(mut self, damage: i32) -> Self {
        self.fragments.push(
            LogFragment{
                colour: RGB::named(rltk::RED),
                text: damage.to_string()
            }
        );
        self
    }

    pub fn speech<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(
            LogFragment{
                colour: RGB::named(rltk::LIGHTGREEN),
                text: format!("\"{}\"", text.to_string())
            }
        );
        self
    }

    pub fn log(self) {
        append_entry(self.fragments);
    }
}
