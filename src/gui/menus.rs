use rltk::prelude::*;
use specs::prelude::*;
use super::{white, black, yellow, green, red, Tooltip};
use crate::{AttributeBonus, Equippable, Item, SkillBonus, Weapon, Wearable};
use crate::raws::{self, ItemData};

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
        Rect::with_size(15, y - 2, 35, height + 1),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK))
    );
    draw_batch.print_color(
        Point::new(18, y - 2),
        &title.to_string(),
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK))
    );
    draw_batch.print_color(
        Point::new(18, y + height - 1),
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
        menu_option(draw_batch, 17, y, 97+j as FontCharType, &item.2, colour);
        item_list.push(item.0);

        if tooltip.is_none() && mouse_pos.0 >= 18 && mouse_pos.0 <= 50 && mouse_pos.1 == y {
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

pub fn item_entity_tooltip(ecs: &World, name: String, entity: Entity) -> Tooltip<String> {
    let weapons = ecs.read_storage::<Weapon>();
    let wearables = ecs.read_storage::<Wearable>();
    let equippables = ecs.read_storage::<Equippable>();
    let items = ecs.read_storage::<Item>();
    let skill_bonuses = ecs.read_storage::<SkillBonus>();
    let attribute_bonuses = ecs.read_storage::<AttributeBonus>();

    let mut tooltip = Tooltip::new();
    if let Some(item) = items.get(entity) {
        tooltip.set_color(raws::get_item_colour(item, &raws::RAWS.lock().unwrap()));
    }
    tooltip.add(name);

    if let Some(weapon) = weapons.get(entity) {
        tooltip.add(format!("Attribute: {:?}", weapon.attribute));
        tooltip.add(format!("Damage: {}", weapon.damage()));
        tooltip.add(format!("Hit bonus: {}", weapon.hit_bonus));

        let range = if let Some(range) = weapon.range { range.to_string() } else { "melee".to_string() };
        tooltip.add(format!("Range: {}", range));
    }
    if let Some(wearable) = wearables.get(entity) {
        tooltip.add(format!("Armour class: {}", wearable.armour_class));
    }
    if let Some(equippable) = equippables.get(entity) {
        tooltip.add(format!("Slot: {:?}", equippable.slot));
    }

    if let Some(attribute_bonus) = attribute_bonuses.get(entity) {
        add_bonus_line(&mut tooltip, attribute_bonus.strength, "Strength".to_string());
        add_bonus_line(&mut tooltip, attribute_bonus.dexterity, "Dexterity".to_string());
        add_bonus_line(&mut tooltip, attribute_bonus.constitution, "Constitution".to_string());
        add_bonus_line(&mut tooltip, attribute_bonus.intelligence, "Intelligence".to_string());
    }
    if let Some(skill_bonus) = skill_bonuses.get(entity) {
        add_bonus_line(&mut tooltip, skill_bonus.melee, "Melee".to_string());
        add_bonus_line(&mut tooltip, skill_bonus.defence, "Defence".to_string());
        add_bonus_line(&mut tooltip, skill_bonus.magic, "Magic".to_string());
        add_bonus_line(&mut tooltip, skill_bonus.ranged, "Ranged".to_string());
    }

    tooltip
}

pub fn item_tooltip(item: ItemData) -> Tooltip<String> {
    let mut tooltip = Tooltip::new();
    tooltip.set_color(raws::get_item_class_colour(&item.class, &raws::RAWS.lock().unwrap()));
    tooltip.add(item.name);

    if let Some(weapon) = item.weapon {
        tooltip.add(format!("Attribute: {}", weapon.attribute));
        tooltip.add(format!("Damage: {}", weapon.base_damage));
        tooltip.add(format!("Hit bonus: {}", weapon.hit_bonus));
        tooltip.add(format!("Range: {}", weapon.range));
        tooltip.add(format!("Slot: {}", weapon.slot));
    }
    if let Some(wearable) = item.wearable {
        tooltip.add(format!("Armour class: {}", wearable.armour_class));
        tooltip.add(format!("Slot: {}", wearable.slot));
    }

    if let Some(attribute_bonus) = item.attribute_bonuses {
        add_bonus_line(&mut tooltip, attribute_bonus.strength, "Strength".to_string());
        add_bonus_line(&mut tooltip, attribute_bonus.dexterity, "Dexterity".to_string());
        add_bonus_line(&mut tooltip, attribute_bonus.constitution, "Constitution".to_string());
        add_bonus_line(&mut tooltip, attribute_bonus.intelligence, "Intelligence".to_string());
    }
    if let Some(skill_bonus) = item.skill_bonuses {
        add_bonus_line(&mut tooltip, skill_bonus.melee, "Melee".to_string());
        add_bonus_line(&mut tooltip, skill_bonus.defence, "Defence".to_string());
        add_bonus_line(&mut tooltip, skill_bonus.magic, "Magic".to_string());
        add_bonus_line(&mut tooltip, skill_bonus.ranged, "Ranged".to_string());
    }

    tooltip
}

pub fn y_start(item_count: usize) -> i32 {
    (20 - (item_count / 2)) as i32
}

pub fn box_height(item_count: usize) -> i32 {
    (item_count*2+3) as i32
}

fn add_bonus_line(tooltip: &mut Tooltip<String>, bonus: Option<i32>, name: String) {
    if let Some(b) = bonus {
        match b {
            n if n > 0 => tooltip.add_colored(format!("+{} {}", b, name), green()),
            n if n < 0 => tooltip.add_colored(format!("{} {}", b, name), red()),
            _ => {}
        }
    }
}
