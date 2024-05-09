use specs::prelude::*;
use rltk::prelude::*;
use super::{black, menu_box, white, yellow};
use crate::{InBackpack, Item, ItemQuality, Name, State, Vendor};
use crate::raws;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum VendorMode { Buy, Sell, Improve }

#[derive(PartialEq, Copy, Clone)]
pub enum VendorResult {
    NoResponse,
    Cancel,
    Buy,
    Sell,
    Improve,
    BuyMode,
    SellMode,
    ImproveMode
}

pub fn show_vendor_menu(gs: &mut State, ctx: &mut Rltk, vendor: Entity, mode: VendorMode) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    match mode {
        VendorMode::Buy => vendor_buy_menu(gs, ctx, vendor),
        VendorMode::Sell => vendor_sell_menu(gs, ctx),
        VendorMode::Improve => vendor_improve_menu(gs, ctx, vendor)
    }
}

fn vendor_sell_menu(gs: &mut State, ctx: &mut Rltk) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let count = backpacks.join().filter( |item| item.owner == *player_entity ).count();
    let mut y = (25 - (count / 2)) as i32;
    menu_box(&mut draw_batch, 15, y, (count*2+3) as i32, "Sell which item? (SPACE to switch to buy mode)");

    let mut inventory: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, item, name, backpack) in (&entities, &items, &names, &backpacks).join() {
        if backpack.owner == *player_entity {
            draw_batch.set(Point::new(17, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
            draw_batch.set(Point::new(18, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
            draw_batch.set(Point::new(19, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

            draw_batch.print_color(Point::new(21, y), &name.name.to_string(), ColorPair::new(raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black()));
            draw_batch.print(Point::new(50, y), &format!("{:.0} gp", item.base_value as f32 * 0.8));

            inventory.push(entity);
            y += 2;
            j += 1;
        }
    }

    draw_batch.submit(1000).expect("Draw batch submission failed");

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Space => { (VendorResult::BuyMode, None, None, None) }
                VirtualKeyCode::Escape => { (VendorResult::Cancel, None, None, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (VendorResult::Sell, Some(inventory[selection as usize]), None, None);
                    }
                    (VendorResult::NoResponse, None, None, None)
                }
            }
        }
    }
}

fn vendor_buy_menu(gs: &mut State, ctx: &mut Rltk, vendor: Entity) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    let vendors = gs.ecs.read_storage::<Vendor>();
    let mut draw_batch = DrawBatch::new();

    let inventory = raws::get_vendor_items(&vendors.get(vendor).unwrap().category, &raws::RAWS.lock().unwrap());
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;

    menu_box(&mut draw_batch, 15, y, (count*2+3) as i32, "Buy which item? (SPACE to switch to improve mode)");

    for (j, sale) in inventory.iter().enumerate() {
        draw_batch.set(Point::new(17, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
        draw_batch.set(Point::new(18, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
        draw_batch.set(Point::new(19, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

        draw_batch.print_color(Point::new(21, y), &sale.0, ColorPair::new(*&sale.2, black()));
        draw_batch.print(Point::new(50, y), &format!("{:.0} gp", sale.1 as f32 * 1.2));
        y += 2;
    }

    draw_batch.submit(1000).expect("Draw batch submission failed");

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Space => { (VendorResult::ImproveMode, None, None, None) }
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

fn vendor_improve_menu(gs: &mut State, ctx: &mut Rltk, vendor_entity: Entity) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let vendors = gs.ecs.read_storage::<Vendor>();
    let vendor = vendors.get(vendor_entity).unwrap();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let mut inventory: Vec<(Entity, &Item, String, i32)> = Vec::new();
    for (entity, item, name, backpack) in (&entities, &items, &names, &backpacks).join() {
        if backpack.owner == *player_entity {
            if item_can_be_improved(item, &vendor.category) {
                inventory.push((entity, item, name.name.clone(), item.base_value * 2));
            }
        }
    }
    inventory.sort_by(|a,b| a.3.partial_cmp(&b.3).unwrap());

    let count = inventory.len();
    rltk::console::log(count);
    let mut y = (25 - (count / 2)) as i32;
    menu_box(&mut draw_batch, 15, y, (count*2+3) as i32, "Improve which item? (SPACE to switch to sell mode)");

    let mut j = 0;
    for (_entity, item, name, cost) in inventory.iter() {
        draw_batch.set(Point::new(17, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
        draw_batch.set(Point::new(18, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
        draw_batch.set(Point::new(19, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

        draw_batch.print_color(Point::new(21, y), name.to_string(), ColorPair::new(raws::get_item_colour(item, &raws::RAWS.lock().unwrap()), black()));
        draw_batch.print(Point::new(50, y), format!("{} gp", cost));

        y += 2;
        j += 1;
    }

    draw_batch.submit(1000).expect("Draw batch submission failed");

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Space => { (VendorResult::SellMode, None, None, None) }
                VirtualKeyCode::Escape => { (VendorResult::Cancel, None, None, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (VendorResult::Improve, Some(inventory[selection as usize].0), None, Some(inventory[selection as usize].3));
                    }
                    (VendorResult::NoResponse, None, None, None)
                }
            }
        }
    }
}

fn item_can_be_improved(item: &Item, vendor_category: &String) -> bool {
    if let Some(category) = &item.vendor_category {
        if category.as_str() == vendor_category.as_str() {
            if item.quality.as_ref() != Some(&ItemQuality::Exceptional) {
                return true;
            }
        }
    }
    
    false
}
