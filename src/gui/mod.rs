use rltk::RGB;

mod main_menu;
mod hud;
mod tooltips;
mod ranged_target;
mod menus;
mod inventory_menu;
mod unequip_item_menu;
mod drop_item_menu;
mod vendor_menu;
mod levelup_menu;
mod game_over_menu;
mod cheat_menu;
pub use main_menu::*;
pub use hud::*;
pub use tooltips::*;
pub use ranged_target::*;
pub use menus::*;
pub use inventory_menu::*;
pub use unequip_item_menu::*;
pub use drop_item_menu::*;
pub use vendor_menu::*;
pub use levelup_menu::*;
pub use game_over_menu::*;
pub use cheat_menu::*;

pub fn white() -> RGB { RGB::named(rltk::WHITE) }
pub fn black() -> RGB { RGB::named(rltk::BLACK) }
pub fn magenta() -> RGB { RGB::named(rltk::MAGENTA) }
pub fn cyan() -> RGB { RGB::named(rltk::CYAN) }
pub fn blue() -> RGB { RGB::named(rltk::BLUE) }
pub fn green() -> RGB { RGB::named(rltk::GREEN) }
pub fn yellow() -> RGB { RGB::named(rltk::YELLOW) }
pub fn orange() -> RGB { RGB::named(rltk::ORANGE) }
pub fn red() -> RGB { RGB::named(rltk::RED) }
pub fn gold() -> RGB { RGB::named(rltk::GOLD) }
pub fn box_gray() -> RGB { RGB::from_hex("#999999").unwrap() }
pub fn light_gray() -> RGB { RGB::from_hex("#CCCCCC").unwrap() }
