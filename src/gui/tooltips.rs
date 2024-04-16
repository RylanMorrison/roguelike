use specs::prelude::*;
use rltk::prelude::*;
use super::{box_gray, light_gray, white, black};
use crate::{Map, Name, Position, Pools, StatusEffect, Duration};
use crate::camera;

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

    fn render(&self, draw_batch: &mut DrawBatch, x: i32, y: i32) {
        draw_batch.draw_box(Rect::with_size(x, y, self.width()-1, self.height()-1), ColorPair::new(white(), box_gray()));
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white() } else { light_gray() };
            draw_batch.print_color(Point::new(x+1, y+i as i32 + 1), &s, ColorPair::new(col, black()));
        }
    }
}

pub fn draw_tooltips(ecs: &World, ctx : &mut Rltk) {
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    // let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();
    let mut draw_batch = DrawBatch::new();

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
    draw_batch.set(Point::new(arrow_x, arrow_y), ColorPair::new(white(), box_gray()), arrow);

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
        tt.render(&mut draw_batch, x, y);
        y += tt.height();
    }

    draw_batch.submit(500).expect("Draw batch submission failed");
}
