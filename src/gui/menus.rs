use rltk::prelude::*;
use specs::prelude::*;
use super::{white, black, yellow};
use crate::Item;
use crate::raws;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }

pub fn menu_box<T: ToString>(draw_batch: &mut DrawBatch, x: i32, y: i32, width: i32, height: i32, title: T) {
    draw_batch.draw_box(
        Rect::with_size(x, y - 2, width, height+1),
        ColorPair::new(white(), black())
    );
    draw_batch.print_color(
        Point::new(x + 3, y - 2),
        &title.to_string(),
        ColorPair::new(yellow(), black())
    );
    draw_batch.print_color(
        Point::new(x + 3, y + height - 1),
        "ESCAPE to cancel",
        ColorPair::new(yellow(), black())
    );
}

pub fn menu_option<T: ToString>(draw_batch: &mut DrawBatch, x: i32, y: i32, hotkey: FontCharType, text: T, colour: Option<RGB>) {
    draw_batch.set(
        Point::new(x, y),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        rltk::to_cp437('(')
    );
    draw_batch.set(
        Point::new(x + 1, y),
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
        hotkey
    );
    draw_batch.set(
        Point::new(x + 2, y),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        rltk::to_cp437(')')
    );
    draw_batch.print_color(
        Point::new(x + 5, y),
        &text.to_string(),
        ColorPair::new(colour.unwrap_or(RGB::named(rltk::YELLOW)), RGB::named(rltk::BLACK))
    );
}

pub fn item_result_menu<T: ToString>(ctx: &mut Rltk, draw_batch: &mut DrawBatch, title: T, items: &[(Entity, Item, String)]) -> (ItemMenuResult, Option<Entity>, Option<(Entity, String, i32, i32)>) {
    let count = items.len();
    let mut y = y_start(count);
    let height = box_height(count);
    draw_batch.draw_box(
        Rect::with_size(25, y - 2, 35, height + 1),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK))
    );
    draw_batch.print_color(
        Point::new(28, y - 2),
        &title.to_string(),
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK))
    );
    draw_batch.print_color(
        Point::new(28, y + height - 1),
        "ESCAPE to cancel",
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK))
    );

    let mut item_list: Vec<Entity> = Vec::new();
    let mut j = 0;
    let mouse_pos = ctx.mouse_pos();
    let mut tooltip: Option<(Entity, String, i32, i32)> = None;
    y += 1;
    for item in items {
        let colour = Some(raws::get_item_colour(&item.1, &raws::RAWS.lock().unwrap()));
        menu_option(draw_batch, 27, y, 97+j as FontCharType, &item.2, colour);
        item_list.push(item.0);

        if tooltip.is_none() && mouse_pos.0 >= 32 && mouse_pos.0 < 60 && mouse_pos.1 == y {
            tooltip = Some((item.0, item.2.clone(), 30, y));
        }

        y += 2;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None, tooltip),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None, tooltip) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(item_list[selection as usize]), tooltip);
                    }
                    (ItemMenuResult::NoResponse, None, tooltip)
                }
            }
        }
    }
}

pub fn y_start(item_count: usize) -> i32 {
    (20 - (item_count / 2)) as i32
}

pub fn box_height(item_count: usize) -> i32 {
    (item_count*2+3) as i32
}
