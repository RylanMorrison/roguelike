use specs::prelude::*;
use rltk::prelude::*;
use super::{ItemMenuResult, yellow, black, blue, cyan, red};
use crate::{State, Viewshed};
use crate::camera;

pub fn ranged_target(gs : &mut State, ctx : &mut Rltk, min_range : f32, max_range: f32) -> (ItemMenuResult, Option<Point>) {
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();
    let mut draw_batch = DrawBatch::new();

    draw_batch.print_color(Point::new(5, 0), "Select Target:", ColorPair::new(yellow(), black()));

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx).round();
            if distance >= min_range && distance <= max_range {
                let screen_x = idx.x - min_x + 1;
                let screen_y = idx.y - min_y + 1;
                if screen_x > 1 && screen_x < (max_x - min_x)
                && screen_y > 1 && screen_y < (max_y - min_y) {
                    draw_batch.set_bg(Point::new(screen_x, screen_y), blue());
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
        draw_batch.set_bg(Point::new(mouse_pos.0, mouse_pos.1), cyan());
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)));
        }
    } else {
        draw_batch.set_bg(Point::new(mouse_pos.0, mouse_pos.1), red());
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    draw_batch.submit(1000).expect("Draw batch submission failed");

    (ItemMenuResult::NoResponse, None)
}
