use rltk::{to_cp437, Point, Rltk, VirtualKeyCode, RGB, TextBlock};
use specs::prelude::*;
use crate::{camera, carry_capacity_lbs, gamelog, player_xp_for_level, raws, PendingLevelUp};

use super::{Pools, Map, Name, Position, State, InBackpack,
    Viewshed, RunState, Equipped, HungerClock, HungerState, Attribute, Attributes,
    Consumable, Item, Vendor, VendorMode, StatusEffect, Duration, KnownSpells,
    Skill, Skills};

fn white() -> RGB { RGB::named(rltk::WHITE) }
fn black() -> RGB { RGB::named(rltk::BLACK) }
fn magenta() -> RGB { RGB::named(rltk::MAGENTA) }
fn cyan() -> RGB { RGB::named(rltk::CYAN) }
fn blue() -> RGB { RGB::named(rltk::BLUE) }
fn green() -> RGB { RGB::named(rltk::GREEN) }
fn yellow() -> RGB { RGB::named(rltk::YELLOW) }
fn orange() -> RGB { RGB::named(rltk::ORANGE) }
fn red() -> RGB { RGB::named(rltk::RED) }
fn gold() -> RGB { RGB::named(rltk::GOLD) }
fn box_gray() -> RGB { RGB::from_hex("#999999").unwrap() }
fn light_gray() -> RGB { RGB::from_hex("#CCCCCC").unwrap() }

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
    draw_hollow_box(ctx, 0, 0, 99, 79, box_gray(), black()); // Overall box
    draw_hollow_box(ctx, 0, 0, 69, 65, box_gray(), black()); // Map box
    draw_hollow_box(ctx, 0, 65, 99, 14, box_gray(), black()); // Log box
    draw_hollow_box(ctx, 69, 0, 30, 16, box_gray(), black()); // Top-right panel

    ctx.set(0, 65, box_gray(), black(), to_cp437('├'));
    ctx.set(69, 16, box_gray(), black(), to_cp437('├'));
    ctx.set(69, 0, box_gray(), black(), to_cp437('┬'));
    ctx.set(69, 65, box_gray(), black(), to_cp437('┴'));
    ctx.set(99, 16, box_gray(), black(), to_cp437('┤'));
    ctx.set(99, 65, box_gray(), black(), to_cp437('┤'));

    // map name
    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 2;
    let x_pos = (32 - (name_length / 2)) as i32;
    ctx.set(x_pos, 0, box_gray(), black(), to_cp437('┤'));
    ctx.set(x_pos + name_length as i32, 0, box_gray(), black(), to_cp437('├'));
    ctx.print_color(x_pos+1, 0, white(), black(), &map.name);
    // std::mem::drop(map);

    // stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health = format!("{}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
    let mana = format!("{}/{}", player_pools.mana.current, player_pools.mana.max);
    let level = format!("Level: {}", player_pools.level);
    let xp_level_start = player_xp_for_level(player_pools.level-1);

    ctx.print_color(70, 1, white(), black(), "Health: ");
    ctx.draw_bar_horizontal(80, 1, 18, player_pools.hit_points.current, player_pools.hit_points.max, red(), black());
    ctx.print_color(87, 1, white(), black(), &health);

    ctx.print_color(70, 2, white(), black(), "Mana: ");
    ctx.draw_bar_horizontal(80, 2, 18, player_pools.mana.current, player_pools.mana.max, blue(), black());
    ctx.print_color(87, 2, white(), black(), &mana);

    ctx.print_color(70, 3, white(), black(), &level);
    ctx.draw_bar_horizontal(80, 3, 18, player_pools.xp - xp_level_start, player_xp_for_level(player_pools.level), gold(), black());
    
    // attributes
    let attributes = ecs.read_storage::<Attributes>();
    let player_attributes = attributes.get(*player_entity).unwrap();
    draw_attribute("Strength:", &player_attributes.strength, 5, ctx);
    draw_attribute("Dexterity:", &player_attributes.dexterity, 6, ctx);
    draw_attribute("Constitution:", &player_attributes.constitution, 7, ctx);
    draw_attribute("Intelligence:", &player_attributes.intelligence, 8, ctx);

    // skills
    let skills = ecs.read_storage::<Skills>();
    let player_skills = &skills.get(*player_entity).unwrap();
    draw_skill("Melee:", &player_skills.melee, 10, ctx);
    draw_skill("Defence:", &player_skills.defence, 11, ctx);
    draw_skill("Magic:", &player_skills.magic, 12, ctx);

    // armour class and damage
    ctx.print_color(70, 14, light_gray(), black(), "Armour Class:");
    ctx.print_color(87, 14, white(), black(), player_pools.total_armour_class);
    ctx.print_color(70, 15, light_gray(), black(), "Base Damage:");
    ctx.print_color(87, 15, white(), black(), player_pools.base_damage.clone());

    // weight
    draw_weight(ctx, player_pools.total_weight, carry_capacity_lbs(&player_attributes.strength));

    // initiative penalty
    ctx.print_color(70, 19, white(), black(),
        &format!("Initiative Penalty: {:.0}", player_pools.total_initiative_penalty)
    );

    // gold
    ctx.print_color(70, 20, gold(), black(),
        &format!("Gold: {}", player_pools.gold)
    );

    // equipment
    let mut y = 23;
    let equipped = ecs.read_storage::<Equipped>();
    let items = ecs.read_storage::<Item>();
    let names = ecs.read_storage::<Name>();
    for (item, equipment, item_name) in (&items, &equipped, &names).join() {
        if equipment.owner == *player_entity {
            ctx.print_color(70, y, raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black(), &item_name.name);
            y += 1;
        }
    }

    // consumables
    y += 1;
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (carried_by, item_name, item, consumable) in (&backpack, &names, &items, &consumables).join() {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(70, y, yellow(), black(), &format!("↑{}", index));
            let display_name = if consumable.max_charges > 1 {
                format!("{} ({})", item_name.name.clone(), consumable.charges)
            } else {
                item_name.name.clone()
            };
            ctx.print_color(73, y, raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black(), display_name);
            y += 1;
            index += 1;
        }
    }

    // spells
    y += 1;
    let known_spells = ecs.read_storage::<KnownSpells>();
    let player_spells = &known_spells.get(*player_entity).unwrap().spells;
    let mut index = 1;
    for spell in player_spells.iter() {
        ctx.print_color(70, y, cyan(), black(), &format!("^{}", index));
        ctx.print_color(73, y, cyan(), black(), &format!("{} ({})", spell.name, spell.mana_cost));
        index += 1;
        y += 1;
    }

    // statuses
    let mut y = 64;
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => {
            ctx.print_color(70, 64, green(), black(), "Well Fed");
            y -= 1;
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            ctx.print_color(70, 64, orange(), black(), "Hungry");
            y -= 1;
        }
        HungerState::Starving => {
            ctx.print_color(70, 64, red(), black(), "Starving");
            y -= 1;
        }
    }
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();
    for (status, duration, name) in (&statuses, &durations, &names).join() {
        let fg = if status.is_debuff { red() } else { green() };
        if status.target == *player_entity {
            ctx.print_color(
                70,
                y,
                fg,
                black(),
                &format!("{} ({})", name.name, duration.turns)
            );
            y -= 1;
        }
    }

    let mut block = TextBlock::new(1, 66, 98, 13);
    block.print(&gamelog::log_display()).expect("Unable to print log");
    block.render(&mut rltk::BACKEND_INTERNAL.lock().consoles[0].console);

    // let mouse_pos = ctx.mouse_pos();
    // ctx.set_bg(mouse_pos.0, mouse_pos.1, magenta());
    // ctx.print_color(mouse_pos.0, mouse_pos.1, white(), black(), &format!("{}/{} {}", mouse_pos.0, mouse_pos.1, map.xy_idx(mouse_pos.0, mouse_pos.1) as i32));
    draw_tooltips(ecs, ctx);
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    ctx.print_color(70, y, light_gray(), black(), name);
    
    let modified_colour: RGB = if attribute.modifiers < 0 {
        red()
    } else if attribute.modifiers == 0 {
        white()
    } else {
        green()
    };
    ctx.print_color(87, y, modified_colour, black(), &format!("{}", attribute.base + attribute.modifiers));
    
    let bonus_colour: RGB = if attribute.bonus < 0 {
        red()
    } else if attribute.bonus == 0 {
        white()
    } else {
        ctx.set(92, y, green(), black(), rltk::to_cp437('+'));
        green()
    };
    ctx.print_color(93, y, bonus_colour, black(), &format!("{}", attribute.bonus));
}

fn draw_skill(name: &str, skill: &Skill, y: i32, ctx: &mut Rltk) {
    ctx.print_color(70, y, light_gray(), black(), name);
    let colour = if skill.modifiers > 0 {
        green()
    } else if skill.modifiers == 0 {
        white()
    } else {
        red()
    };
    ctx.print_color(87, y, colour, black(), skill.bonus());
}

fn draw_weight(ctx: &mut Rltk, weight: f32, capacity: f32) {
    let colour = if weight > capacity { red() } else { white() };
    ctx.print_color(70, 18, colour, black(),
        &format!("{:0} lbs ({} lbs max)", weight, capacity)
    );
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
    mouse_map_pos.0 += min_x - 1;
    mouse_map_pos.1 += min_y - 1;
    if mouse_pos.0 < 1 || mouse_pos.0 > 69 || mouse_pos.1 < 1 || mouse_pos.1 > 60 { return; }
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

            // status effects
            let statuses = ecs.read_storage::<StatusEffect>();
            let durations = ecs.read_storage::<Duration>();
            let names = ecs.read_storage::<Name>();
            for (status, duration, name) in (&statuses, &durations, &names).join() {
                if status.target == entity {
                    tip.add(format!("{} ({})", name.name, duration.turns));
                }
            }

            // pools
            let stat = pools.get(entity);
            if let Some(stat) = stat {
                tip.add(format!("Level: {}", stat.level));
            }

            tip_boxes.push(tip);
        }
    }

    if tip_boxes.is_empty() { return; }

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    if mouse_pos.0 < 50 { // left side of the screen
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
        let x = if mouse_pos.0 < 50 {
            mouse_pos.0 - (1 + tt.width())
        } else {
            mouse_pos.0 + (1 + tt.width())
        };
        tt.render(ctx, x, y);
        y += tt.height();
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }

pub fn show_inventory(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let count = backpacks.join().filter(|item| item.owner == *player_entity ).count();
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), black(), "Inventory");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), black(), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, name, item, _inventory_item) in (&entities, &names, &items, &backpacks).join().filter( |item| item.3.owner == *player_entity ) {
        ctx.set(17, y, white(), black(), to_cp437('('));
        // consecutive letters of the alphabet
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), to_cp437(')'));

        ctx.print_color(21, y, raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black(), &name.name.to_string());
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
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let count = backpacks.join().filter( |item| item.owner == *player_entity ).count();
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, yellow(), black(), "Drop Which Item?");
    ctx.print_color(18, y+count as i32+1, yellow(), black(), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, name, item, _inventory_item) in (&entities, &names, &items, &backpacks).join().filter( |item| item.3.owner == *player_entity ) {
        ctx.set(17, y, white(), black(), to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), to_cp437(')'));

        ctx.print_color(21, y, raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black(), &name.name.to_string());
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
    mouse_map_pos.0 += min_x - 1;
    mouse_map_pos.1 += min_y - 1;
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

    let count = equipped_items.join().filter( |item| item.owner == *player_entity ).count();
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, yellow(), black(), "Unequip Which Item?");
    ctx.print_color(18, y+count as i32+1, yellow(), black(), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, name, item, _equipped_item) in (&entities, &names, &items, &equipped_items).join().filter( |item| item.3.owner == *player_entity ) {
        ctx.set(17, y, white(), black(), to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), to_cp437(')'));

        ctx.print_color(21, y, raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black(), &name.name.to_string());
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
    ctx.draw_box_double(24, 13, 52, 12, yellow(), black());
    ctx.print_color_centered(15, yellow(), black(), "Your journey has ended!");

    ctx.print_color_centered(17, white(), black(), &format!("You lived for {} turns.", gamelog::get_event_count("Turn")));
    ctx.print_color_centered(18, red(), black(), &format!("You took {} total damage.", gamelog::get_event_count("Damage Taken")));
    ctx.print_color_centered(19, red(), black(), &format!("You dealt {} total damage.", gamelog::get_event_count("Damage Dealt")));
    ctx.print_color_centered(20, yellow(), black(), &format!("You killed {} enemies.", gamelog::get_event_count("Kill")));

    ctx.print_color_centered(22, magenta(), black(), "Press any key to return to the menu.");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult { 
    NoResponse, 
    Cancel, 
    TeleportToExit,
    FullHeal,
    RevealMap,
    GodMode,
    LevelUp,
    MakeRich
}

pub fn show_cheat_mode(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let count = 6;
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, white(), black());
    ctx.print_color(18, y-2, yellow(), black(), "Cheating!");
    ctx.print_color(18, y+count as i32+1, yellow(), black(), "ESCAPE to cancel");

    ctx.set(17, y, white(), black(), rltk::to_cp437('('));
    ctx.set(18, y, white(), black(), rltk::to_cp437('T'));
    ctx.set(19, y, white(), black(), rltk::to_cp437(')'));
    ctx.print(21, y, "Teleport to exit");

    y += 1;
    ctx.set(17, y, white(), black(), rltk::to_cp437('('));
    ctx.set(18, y, white(), black(), rltk::to_cp437('H'));
    ctx.set(19, y, white(), black(), rltk::to_cp437(')'));
    ctx.print(21, y, "Full heal");

    y += 1;
    ctx.set(17, y, white(), black(), rltk::to_cp437('('));
    ctx.set(18, y, white(), black(), rltk::to_cp437('R'));
    ctx.set(19, y, white(), black(), rltk::to_cp437(')'));
    ctx.print(21, y, "Reveal map");

    y += 1;
    ctx.set(17, y, white(), black(), rltk::to_cp437('('));
    ctx.set(18, y, white(), black(), rltk::to_cp437('G'));
    ctx.set(19, y, white(), black(), rltk::to_cp437(')'));
    ctx.print(21, y, "God mode");

    y += 1;
    ctx.set(17, y, white(), black(), rltk::to_cp437('('));
    ctx.set(18, y, white(), black(), rltk::to_cp437('L'));
    ctx.set(19, y, white(), black(), rltk::to_cp437(')'));
    ctx.print(21, y, "Level up");

    y += 1;
    ctx.set(17, y, white(), black(), rltk::to_cp437('('));
    ctx.set(18, y, white(), black(), rltk::to_cp437('M'));
    ctx.set(19, y, white(), black(), rltk::to_cp437(')'));
    ctx.print(21, y, "Make rich");

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
                VirtualKeyCode::H => CheatMenuResult::FullHeal,
                VirtualKeyCode::R => CheatMenuResult::RevealMap,
                VirtualKeyCode::G => CheatMenuResult::GodMode,
                VirtualKeyCode::L => CheatMenuResult::LevelUp,
                VirtualKeyCode::M => CheatMenuResult::MakeRich,
                VirtualKeyCode::Escape => CheatMenuResult::Cancel,
                _ => CheatMenuResult::NoResponse
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum VendorResult {
    NoResponse,
    Cancel,
    Buy,
    Sell,
    BuyMode,
    SellMode 
}

pub fn show_vendor_menu(gs: &mut State, ctx: &mut Rltk, vendor: Entity, mode: VendorMode) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    match mode {
        VendorMode::Buy => vendor_buy_menu(gs, ctx, vendor, mode),
        VendorMode::Sell => vendor_sell_menu(gs, ctx, vendor, mode)
    }
}

fn vendor_sell_menu(gs: &mut State, ctx: &mut Rltk, _vendor: Entity, _mode: VendorMode) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let count = backpacks.join().filter( |item| item.owner == *player_entity ).count();
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 51, (count*2+3) as i32, white(), black());
    ctx.print_color(18, y-2, yellow(), black(), "Sell which item? (SPACE to switch to buy mode)");
    ctx.print_color(18, y+(count as i32)*2+1, yellow(), black(), "ESCAPE to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, name, item, _pack) in (&entities, &names, &items, &backpacks).join().filter( |item| item.3.owner == *player_entity ) {
        ctx.set(17, y, white(), black(), rltk::to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), rltk::to_cp437(')'));

        ctx.print_color(21, y, raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black(), &name.name.to_string());
        ctx.print(50, y, &format!("{:.0} gp", item.base_value as f32 * 0.8));
        equippable.push(entity);
        y += 2;
        j += 1;
    }

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Space => { (VendorResult::BuyMode, None, None, None) }
                VirtualKeyCode::Escape => { (VendorResult::Cancel, None, None, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (VendorResult::Sell, Some(equippable[selection as usize]), None, None);
                    }
                    (VendorResult::NoResponse, None, None, None)
                }
            }
        }
    }
}

fn vendor_buy_menu(gs: &mut State, ctx: &mut Rltk, vendor: Entity, _mode: VendorMode) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    let vendors = gs.ecs.read_storage::<Vendor>();

    let inventory = raws::get_vendor_items(&vendors.get(vendor).unwrap().categories, &raws::RAWS.lock().unwrap());
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 51, (count*2+3) as i32, white(), black()); 
    ctx.print_color(18, y-2, yellow(), black(), "Buy which item? (SPACE to switch to sell mode)");
    ctx.print_color(18, y+count as i32*2+1, yellow(), black(), "ESCAPE to cancel");

    for (j, sale) in inventory.iter().enumerate() {
        ctx.set(17, y, white(), black(), rltk::to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97+j as rltk::FontCharType);
        ctx.set(19, y, white(), black(), rltk::to_cp437(')'));

        ctx.print_color(21, y, *&sale.2, black(), &sale.0);
        ctx.print(50, y, &format!("{:.0} gp", sale.1 as f32 * 1.2));
        y += 2;
    }

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Space => { (VendorResult::SellMode, None, None, None) }
                VirtualKeyCode::Escape => { (VendorResult::Cancel, None, None, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (VendorResult::Buy, None, Some(inventory[selection as usize].0.clone()), Some(inventory[selection as usize].1));
                    }
                    (VendorResult::NoResponse, None, None, None)
                }
            }
        }
    }
}

pub enum LevelUpMenuResult {
    NoResponse,
    AssignedAttribute,
    AssignedSkill,
    Done
}

fn draw_level_choice(ctx: &mut Rltk, y: i32, name: &str, value: &i32, selection: &str, selected: bool) {
    let colour = if selected { green() } else { white() };
    let mod_value = if selected { value + 1 } else { *value };

    ctx.print_color(22, y, colour, black(), name);
    ctx.print_color(39, y, colour, black(), mod_value);
    ctx.print_color(47, y, yellow(), black(), selection);
}

pub fn show_levelup_menu(gs: &mut State, ctx: &mut Rltk, attribute_points: i32, skill_points: i32) -> LevelUpMenuResult {
    let player = gs.ecs.fetch::<Entity>();
    let attributes = gs.ecs.read_storage::<Attributes>();
    let player_attributes = attributes.get(*player).unwrap();
    let skills = gs.ecs.read_storage::<Skills>();
    let player_skills = skills.get(*player).unwrap();
    let mut pending_level_ups = gs.ecs.write_storage::<PendingLevelUp>();
    let level_up = pending_level_ups.get_mut(*player).unwrap();
    
    ctx.draw_box(12, 25, 51, 20, white(), black());
    ctx.print_color(15, 25, yellow(), black(), "Level Up");
    ctx.print_color(15, 27, yellow(), black(), "Pick one attribute and two skills to improve");

    draw_level_choice(ctx, 29, "Strength", &player_attributes.strength.base, "(a)", level_up.attributes.strength.base > player_attributes.strength.base);
    draw_level_choice(ctx, 31, "Dexterity", &player_attributes.dexterity.base, "(b)", level_up.attributes.dexterity.base > player_attributes.dexterity.base);
    draw_level_choice(ctx, 33, "Constitution", &player_attributes.constitution.base, "(c)", level_up.attributes.constitution.base > player_attributes.constitution.base);
    draw_level_choice(ctx, 35, "Intelligence", &player_attributes.intelligence.base, "(d)", level_up.attributes.intelligence.base > player_attributes.intelligence.base);

    draw_level_choice(ctx, 38, "Melee", &player_skills.melee.base, "(e)", level_up.skills.melee.base > player_skills.melee.base);
    draw_level_choice(ctx, 40, "Defence", &player_skills.defence.base, "(f)", level_up.skills.defence.base > player_skills.defence.base);
    draw_level_choice(ctx, 42, "Magic", &player_skills.magic.base, "(g)", level_up.skills.magic.base > player_skills.magic.base);
    
    ctx.print_color(15, 45, yellow(), black(), "ENTER when done");

    match ctx.key {
        None => {},
        Some(key) => {
            match key {
                VirtualKeyCode::A => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.strength.base = player_attributes.strength.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                },
                VirtualKeyCode::B => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.dexterity.base = player_attributes.dexterity.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                },
                VirtualKeyCode::C => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.constitution.base = player_attributes.constitution.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                }
                VirtualKeyCode::D => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.intelligence.base = player_attributes.intelligence.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                },
                VirtualKeyCode::E => {
                    if level_up.skills.melee.base > player_skills.melee.base || skill_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.skills.melee.base = player_skills.melee.base + 1;
                        return LevelUpMenuResult::AssignedSkill;
                    }
                },
                VirtualKeyCode::F => {
                    if level_up.skills.defence.base > player_skills.defence.base || skill_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.skills.defence.base = player_skills.defence.base + 1;
                        return LevelUpMenuResult::AssignedSkill;
                    }
                },
                VirtualKeyCode::G => {
                    if level_up.skills.magic.base > player_skills.magic.base || skill_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.skills.magic.base = player_skills.magic.base + 1;
                        return LevelUpMenuResult::AssignedSkill;
                    }
                },
                VirtualKeyCode::Return => {
                    if attribute_points == 0 && skill_points == 0 {
                        return LevelUpMenuResult::Done;
                    } else {
                        return LevelUpMenuResult::NoResponse;
                    }
                }
                _ => {}
            }
        }
    }

    LevelUpMenuResult::NoResponse
}
