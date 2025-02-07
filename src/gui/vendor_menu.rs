use specs::prelude::*;
use rltk::prelude::*;
use super::{black, box_height, item_entity_tooltip, item_tooltip, menu_box, white, y_start, yellow};
use crate::{Consumable, InBackpack, Item, ItemClass, ItemQuality, State, Vendor};
use crate::raws::{self, get_item_class_colour, ItemData};

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
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let count = backpacks.join().filter( |item| item.owner == *player_entity ).count();
    let mut y = y_start(count);
    menu_box(&mut draw_batch, 10, y, 55, box_height(count), "Sell which item? (SPACE to switch to buy mode)");

    let mut inventory: Vec<Entity> = Vec::new();
    let mouse_pos = ctx.mouse_pos();
    let mut tooltip: Option<(Entity, String, i32)> = None;
    y += 1;
    for (j, (entity, item, backpack)) in (&entities, &items, &backpacks).join().enumerate() {
        if backpack.owner == *player_entity {
            draw_batch.set(Point::new(13, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
            draw_batch.set(Point::new(14, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
            draw_batch.set(Point::new(15, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

            draw_batch.print_color(
                Point::new(18, y),
                item.full_name(),
                ColorPair::new(raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black())
            );
            draw_batch.print(Point::new(57, y), &format!("{:.0} gp", item.base_value as f32 * 0.8));

            if mouse_pos.0 >= 18 && mouse_pos.0 <= 57 && mouse_pos.1 == y {
                tooltip = Some((entity, item.full_name(), y));
            }

            inventory.push(entity);
            y += 2;
        }
    }

    draw_batch.submit(1000).expect("Draw batch submission failed");

    if let Some((entity, name, y)) = tooltip {
        let tooltip_box = item_entity_tooltip(&gs.ecs, name, entity);
        tooltip_box.render(&mut draw_batch, 30, y);
        draw_batch.submit(1100).expect("Draw batch submission failed");
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

    let mut y = y_start(count);
    menu_box(&mut draw_batch, 10, y, 55, box_height(count), "Buy which item? (SPACE to switch to improve mode)");

    let mouse_pos = ctx.mouse_pos();
    let mut tooltip: Option<(ItemData, i32, i32)> = None;
    y += 1;
    for (j, item) in inventory.iter().enumerate() {
        draw_batch.set(Point::new(13, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
        draw_batch.set(Point::new(14, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
        draw_batch.set(Point::new(15, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

        let item_class_colour = get_item_class_colour(item.class.as_str(), &raws::RAWS.lock().unwrap());

        draw_batch.print_color(Point::new(18, y), &item.name, ColorPair::new(item_class_colour, black()));
        draw_batch.print(Point::new(57, y), &format!("{:.0} gp", item.base_value as f32 * 1.2));

        if mouse_pos.0 >= 18 && mouse_pos.0 <= 57 && mouse_pos.1 == y {
            tooltip = Some((item.clone(), 30, y));
        }

        y += 2;
    }

    draw_batch.submit(1000).expect("Draw batch submission failed");

    if let Some((item, x, y)) = tooltip {
        item_tooltip(item.clone()).render(&mut draw_batch, x, y);
        draw_batch.submit(3500).expect("Draw batch submission failed");
    }

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Space => { (VendorResult::ImproveMode, None, None, None) }
                VirtualKeyCode::Escape => { (VendorResult::Cancel, None, None, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (
                            VendorResult::Buy,
                            None,
                            Some(inventory[selection as usize].name.clone()),
                            Some(inventory[selection as usize].base_value)
                        );
                    }
                    (VendorResult::NoResponse, None, None, None)
                }
            }
        }
    }
}

fn vendor_improve_menu(gs: &mut State, ctx: &mut Rltk, vendor_entity: Entity) -> (VendorResult, Option<Entity>, Option<String>, Option<i32>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let consumables = gs.ecs.read_storage::<Consumable>();
    let vendors = gs.ecs.read_storage::<Vendor>();
    let vendor = vendors.get(vendor_entity).unwrap();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let mut inventory: Vec<(Entity, &Item, String, i32)> = Vec::new();
    for (entity, item, backpack) in (&entities, &items, &backpacks).join() {
        if backpack.owner == *player_entity && consumables.get(entity).is_none() {
            if item_can_be_improved(item, &vendor.category) {
                inventory.push((entity, item, item.full_name(), item.base_value * 2));
            }
        }
    }
    inventory.sort_by(|a,b| a.3.partial_cmp(&b.3).unwrap());

    let count = inventory.len();
    let mut y = y_start(count);
    menu_box(&mut draw_batch, 10, y, 55, box_height(count), "Improve which item? (SPACE to switch to sell mode)");

    let mouse_pos = ctx.mouse_pos();
    let mut tooltip: Option<(Entity, String, i32, i32)> = None;
    y += 1;
    for (j, (entity, item, name, cost)) in inventory.iter().enumerate() {
        draw_batch.set(Point::new(13, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
        draw_batch.set(Point::new(14, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
        draw_batch.set(Point::new(15, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

        draw_batch.print_color(
            Point::new(18, y),
            name,
            ColorPair::new(raws::get_item_colour(item, &raws::RAWS.lock().unwrap()), black())
        );
        draw_batch.print(Point::new(57, y), format!("{} gp", cost));

        if tooltip.is_none() && mouse_pos.0 >= 18 && mouse_pos.0 <= 57 && mouse_pos.1 == y {
            tooltip = Some((*entity, item.full_name(), 30, y));
        }

        y += 2;
    }

    draw_batch.submit(1000).expect("Draw batch submission failed");

    if let Some((entity, name, x, y)) = tooltip {
        item_entity_tooltip(&gs.ecs, name, entity).render(&mut draw_batch, x, y);
        draw_batch.submit(3500).expect("Draw batch submission failed");
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
                        return (VendorResult::Improve, Some(inventory[selection as usize].0), None, Some(inventory[selection as usize].3));
                    }
                    (VendorResult::NoResponse, None, None, None)
                }
            }
        }
    }
}

fn item_can_be_improved(item: &Item, vendor_category: &String) -> bool {
    if item.class == ItemClass::Set || item.class == ItemClass::Unique { return false; }

    if let Some(category) = &item.vendor_category {
        if category.as_str() == vendor_category.as_str() {
            return item.quality != ItemQuality::Exceptional;
        }
    }
    
    false
}
