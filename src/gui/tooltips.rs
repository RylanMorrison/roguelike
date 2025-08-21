use std::fmt::Display;
use specs::prelude::*;
use rltk::prelude::*;
use super::{black, box_gray, light_gray, white, green, red};
use crate::{Map, Name, Position, Pools, StatusEffect, Duration, Item, AttributeBonus, Equippable, SkillBonus, Weapon, Wearable, TileType};
use crate::camera;
use crate::raws::{self, ItemData};


struct Line<S> {
    text: S,
    color: RGB
}

pub struct Tooltip<S> {
    lines: Vec<Line<S>>,
    color: Option<RGB>
}

impl<S> Tooltip<S> where S: ToString + Display {
    pub fn new() -> Tooltip<S> {
        Tooltip {
            lines: Vec::new(),
            color: None
        }
    }

    pub fn add(&mut self, text: S) {
        self.lines.push(Line{ text, color: light_gray() });
    }

    pub fn add_colored(&mut self, text: S, color: RGB) {
        self.lines.push(Line{ text, color });
    }

    pub fn set_color(&mut self, color: RGB) {
        self.color = Some(color);
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            let text = s.text.to_string();
            if text.len() > max {
                max = text.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 { self.lines.len() as i32 * 2 + 2i32 }

    pub fn render(&self, draw_batch: &mut DrawBatch, x: i32, y: i32) {
        if self.lines.len() < 1 { return; }

        let mut t_y = y.clone();
        let color = self.color.unwrap_or(white());
        draw_batch.draw_box(Rect::with_size(x, y, self.width()+1, self.height()), ColorPair::new(color, box_gray()));
        t_y += 1;

        // heading
        draw_batch.print_color(
            Point::new(x+2, t_y as i32 + 1),
            &self.lines.first().unwrap().text,
            ColorPair::new(color, black()));

        t_y += 2;
        for (i, line) in self.lines.iter().skip(1).enumerate() {
            draw_batch.print_color(Point::new(x+2, t_y+i as i32 + 1), &line.text, ColorPair::new(line.color, black()));
            t_y += 1;
        }
    }
}

pub fn draw_map_tooltips(ecs: &World, ctx : &mut Rltk) {
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x - 1;
    mouse_map_pos.1 += min_y - 1;
    if mouse_pos.0 < 1 || mouse_pos.0 > 99 || mouse_pos.1 < 1 || mouse_pos.1 > 80 { return; }
    if mouse_map_pos.0 >= map.width-1 || mouse_map_pos.1 >= map.height-1
        || mouse_map_pos.0 < 1 || mouse_map_pos.1 < 1 {
            return;
    }

    let idx = map.xy_idx(mouse_map_pos.0, mouse_map_pos.1);
    if !map.visible_tiles[idx] { return; }

    let mut tip_boxes: Vec<Tooltip<String>> = Vec::new();

    // tiles
    match &map.tiles[idx] {
        TileType::NextArea { map_name } | TileType::PreviousArea { map_name } => {
            let mut tooltip = Tooltip::new();
            tooltip.add(format!("To {}", map_name.to_string()));
            tip_boxes.push(tooltip);
        }
        _ => {}
    }

    // entities
    for (entity, name, position) in (&entities, &names, &positions).join() {
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 {
            if let Some(item) = items.get(entity) {
                tip_boxes.push(ground_item_tooltip(ecs, item.full_name(), entity));
                continue;
            }
            let mut tip = Tooltip::new();
            tip.add(name.name.to_string());

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
            if let Some(stat) = pools.get(entity) {
                tip.add(format!("Level: {}", stat.level));
                tip.add(format!("HP: {}/{}", stat.hit_points.current, stat.hit_points.max));
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
    draw_batch.set(
        Point::new(arrow_x, arrow_y),
        ColorPair::new(white(), box_gray()),
        arrow
    );

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
            mouse_pos.0 - (3 + tt.width())
        } else {
            mouse_pos.0 + 2
        };
        tt.render(&mut draw_batch, x, y);
        y += tt.height();
    }

    draw_batch.submit(3500).expect("Draw batch submission failed");
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

fn ground_item_tooltip(ecs: &World, name: String, entity: Entity) -> Tooltip<String> {
    let items = ecs.read_storage::<Item>();
    let mut tooltip = Tooltip::new();

    if let Some(item) = items.get(entity) {
        tooltip.set_color(raws::get_item_colour(item, &raws::RAWS.lock().unwrap()));
    }
    tooltip.add(name);
    tooltip
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
