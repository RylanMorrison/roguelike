use rltk::{to_cp437, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use crate::camera;

use super::{Pools, gamelog::GameLog, Map, Name, Position, State, InBackpack,
    Viewshed, RunState, Equipped, HungerClock, HungerState, Attribute, Attributes,
    Consumable, Item};

fn white() -> RGB { RGB::named(rltk::WHITE) }
fn black() -> RGB { RGB::named(rltk::BLACK) }
fn magenta() -> RGB { RGB::named(rltk::MAGENTA) }
fn blue() -> RGB { RGB::named(rltk::BLUE) }
fn green() -> RGB { RGB::named(rltk::GREEN) }
fn yellow() -> RGB { RGB::named(rltk::YELLOW) }
fn orange() -> RGB { RGB::named(rltk::ORANGE) }
fn red() -> RGB { RGB::named(rltk::RED) }
fn gold() -> RGB { RGB::named(rltk::GOLD) }
fn box_gray() -> RGB { RGB::from_hex("#999999").unwrap() }
fn light_gray() -> RGB { RGB::from_hex("#CCCCCC").unwrap() }
fn item_colour(item: &Item) -> RGB { RGB::from_hex(item.colour.clone()).unwrap() }

pub fn draw_hollow_box(console: &mut Rltk, sx: i32, sy: i32, width: i32, height: i32, fg: RGB, bg: RGB) {
    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    draw_hollow_box(ctx, 0, 0, 79, 59, box_gray(), black()); // Overall box
    draw_hollow_box(ctx, 0, 0, 49, 45, box_gray(), black()); // Map box
    draw_hollow_box(ctx, 0, 45, 79, 14, box_gray(), black()); // Log box
    draw_hollow_box(ctx, 49, 0, 30, 8, box_gray(), black()); // Top-right panel

    ctx.set(0, 45, box_gray(), black(), to_cp437('├'));
    ctx.set(49, 8, box_gray(), black(), to_cp437('├'));
    ctx.set(49, 0, box_gray(), black(), to_cp437('┬'));
    ctx.set(49, 45, box_gray(), black(), to_cp437('┴'));
    ctx.set(79, 8, box_gray(), black(), to_cp437('┤'));
    ctx.set(79, 45, box_gray(), black(), to_cp437('┤'));

    // map name
    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 2;
    let x_pos = (22 - (name_length / 2)) as i32;
    ctx.set(x_pos, 0, box_gray(), black(), to_cp437('┤'));
    ctx.set(x_pos + name_length as i32, 0, box_gray(), black(), to_cp437('├'));
    ctx.print_color(x_pos+1, 0, white(), black(), &map.name);
    std::mem::drop(map);

    // stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health = format!("Health: {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
    let mana = format!("Mana: {}/{}", player_pools.mana.current, player_pools.mana.max);
    let level = format!("Level: {}", player_pools.level);
    let xp_level_start = (player_pools.level-1) * 1000;
    
    ctx.print_color(50, 1, white(), black(), &health);
    ctx.print_color(50, 2, white(), black(), &mana);
    ctx.print_color(50, 3, white(), black(), &level);
    ctx.draw_bar_horizontal(64, 1, 14, player_pools.hit_points.current, player_pools.hit_points.max, red(), black());
    ctx.draw_bar_horizontal(64, 2, 14, player_pools.mana.current, player_pools.mana.max, blue(), black());
    ctx.draw_bar_horizontal(64, 3, 14, player_pools.xp - xp_level_start, 1000, gold(), black());

    // attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Strength:", &attr.strength, 4, ctx);
    draw_attribute("Dexterity", &attr.dexterity, 5, ctx);
    draw_attribute("Constitution", &attr.constitution, 6, ctx);
    draw_attribute("Intelligence", &attr.intelligence, 7, ctx);

    // equipment
    let mut y = 9;
    let items = ecs.read_storage::<Item>();
    let equipped = ecs.read_storage::<Equipped>();
    let names = ecs.read_storage::<Name>();
    for (equipment, item, item_name) in (&equipped, &items, &names).join() {
        if equipment.owner == *player_entity {
            ctx.print_color(50, y, item_colour(item), black(), &item_name.name);
            y += 1;
        }
    }

    // consumables
    y += 1;
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (carried_by, _consumable, item_name) in (&backpack, &consumables, &names).join() {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(50, y, yellow(), black(), &format!("↑{}", index));
            ctx.print_color(53, y, green(), black(), &item_name.name);
            y += 1;
            index += 1;
        }
    }

    // status
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => ctx.print_color(50, 44, green(), black(), "Well Fed"),
        HungerState::Normal => {}
        HungerState::Hungry => ctx.print_color(50, 44, orange(), black(), "Hungry"),
        HungerState::Starving => ctx.print_color(50, 44, red(), black(), "Starving")
    }

    // game log
    let log = ecs.fetch::<GameLog>();
    let mut y = 46;
    for s in log.entries.iter().rev() {
        // TODO: log colours
        if y < 59 { ctx.print(2, y, s); }
        y += 1;
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    draw_tooltips(ecs, ctx);
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    ctx.print_color(50, y, light_gray(), black(), name);
    let colour: RGB = if attribute.modifiers < 0 {
        red()
    } else if attribute.modifiers == 0 {
        white()
    } else {
        green()
    };
    ctx.print_color(67, y, colour, black(), &format!("{}", attribute.base + attribute.modifiers));
    if attribute.bonus > 0 {
        ctx.set(72, y, colour, black(), rltk::to_cp437('+')); 
    }
    ctx.print_color(73, y, colour, black(), &format!("{}", attribute.bonus));
}

struct Tooltip {
    lines: Vec<String>
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.len() > max {
                max = s.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 { self.lines.len() as i32 + 2i32 }

    fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        ctx.draw_box(x, y, self.width()-1, self.height()-1, white(), box_gray());
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white() } else { light_gray() };
            ctx.print_color(x+1, y+i as i32 + 1, col, black(), &s);
        }
    }
}

fn draw_tooltips(ecs: &World, ctx : &mut Rltk) {
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    // let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    if mouse_map_pos.0 >= map.width-1 || mouse_map_pos.1 >= map.height-1 
        || mouse_map_pos.0 < 1 || mouse_map_pos.1 < 1 { 
            return; 
    }
    if !map.visible_tiles[map.xy_idx(mouse_map_pos.0, mouse_map_pos.1)] { return; }

    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    for (entity, name, position) in (&entities, &names, &positions).join() {
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 {
            let mut tip = Tooltip::new();
            tip.add(name.name.to_string());

            // attributes
            // let attr = attributes.get(entity);
            // if let Some(attr) = attr {
            //     let mut s = "".to_string();
            //     // TODO
            // }

            // pools
            let stat = pools.get(entity);
            if let Some(stat) = stat {
                // TODO: separate xp bar and level indicator
                tip.add(format!("Level: {}", stat.level));
            }

            tip_boxes.push(tip);
        }
    }

    if tip_boxes.is_empty() { return; }

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    if mouse_pos.0 < 40 { // left side of the screen
        // render to the left
        arrow = to_cp437('→');
        arrow_x = mouse_pos.0 - 1;
    } else { // right side of the screen
        // render to the right
        arrow = to_cp437('←');
        arrow_x = mouse_pos.0 + 1;
    }
    ctx.set(arrow_x, arrow_y, white(), box_gray(), arrow);

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.height();
    }

    // vertically center
    let mut y = mouse_pos.1 - (total_height / 2);
    while y + (total_height / 2) > 50 {
        y -= 1;
    }

    // actually draw
    for tt in tip_boxes.iter() {
        let x = if mouse_pos.0 < 40 {
            mouse_pos.0 - (1 + tt.width())
        } else {
            mouse_pos.0 + (1 + tt.width())
        };
        tt.render(ctx, x, y);
        y += tt.height();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }

pub fn show_inventory(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), black(), "Inventory");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), black(), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, item, name) in (&entities, &backpack, &items, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, white(), black(), to_cp437('('));
        // consecutive letters of the alphabet
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), to_cp437(')'));

        ctx.print_color(21, y, item_colour(item), black(), &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn drop_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, yellow(), black(), "Drop Which Item?");
    ctx.print_color(18, y+count as i32+1, yellow(), black(), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, item, name) in (&entities, &backpack, &items, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, white(), black(), to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), to_cp437(')'));

        ctx.print_color(21, y, item_colour(item), black(), &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn ranged_target(gs : &mut State, ctx : &mut Rltk, range : i32) -> (ItemMenuResult, Option<Point>) {
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, yellow(), black(), "Select Target:");

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1 && screen_x < (max_x - min_x)-1 
                && screen_y > 1 && screen_y < (max_y - min_y)-1 {
                    ctx.set_bg(screen_x, screen_y, blue());
                    available_cells.push(idx);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 { 
            valid_target = true; 
        } 
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, red());
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }

pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, yellow(), black(), "Taverns of Stoner Doom");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        // menu items and selection highlighting
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, magenta(), black(), "Begin New Game");
        } else {
            ctx.print_color_centered(24, white(), black(), "Begin New Game");
        }

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(25, magenta(), black(), "Load Game");
            } else {
                ctx.print_color_centered(25, white(), black(), "Load Game");
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, magenta(), black(), "Quit");
        } else {
            ctx.print_color_centered(26, white(), black(), "Quit");
        }

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

pub fn unequip_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let equipped_items = gs.ecs.read_storage::<Equipped>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let equipped = (&equipped_items, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = equipped.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, yellow(), black(), "Unequip Which Item?");
    ctx.print_color(18, y+count as i32+1, yellow(), black(), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _equipped_item, item, name) in (&entities, &equipped_items, &items, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, white(), black(), to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), to_cp437(')'));

        ctx.print_color(21, y, item_colour(item), black(), &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub enum GameOverResult {
    NoSelection,
    QuitToMenu
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.draw_box_double(14, 13, 52, 10, yellow(), black());
    ctx.print_color_centered(15, yellow(), black(), "Your journey has ended!");
    ctx.print_color_centered(17, white(), black(), "One day, we'll tell you all about how you did.");
    ctx.print_color_centered(18, white(), black(), "That day, sadly, is not in this chapter..");

    ctx.print_color_centered(20, magenta(), black(), "Press any key to return to the menu.");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult { NoResponse, Cancel, TeleportToExit }

pub fn show_cheat_mode(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let count = 2;
    let y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, yellow(), black(), "Cheating!");
    ctx.print_color(18, y+count as i32+1, yellow(), black(), "ESCAPE to cancel");

    ctx.set(17, y, white(), black(), rltk::to_cp437('('));
    ctx.set(18, y, white(), black(), rltk::to_cp437('T'));
    ctx.set(19, y, white(), black(), rltk::to_cp437(')'));

    ctx.print(21, y, "Teleport to exit");

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
                VirtualKeyCode::Escape => CheatMenuResult::Cancel,
                _ => CheatMenuResult::NoResponse
            }
        }
    }
}
