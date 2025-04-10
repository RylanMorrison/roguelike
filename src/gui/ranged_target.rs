use specs::prelude::*;
use rltk::prelude::*;
use super::{ItemMenuResult, yellow, black, blue, cyan, red, light_gray};
use crate::{AreaOfEffect, State, Viewshed, Map};
use crate::camera;
use crate::effects::aoe_points;

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, min_range: f32, max_range: f32, source: Entity) -> (ItemMenuResult, Option<Point>) {
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
    let mouse_pos = ctx.mouse_pos(); // position on the screen
    let mut mouse_map_pos = mouse_pos; // position on the map
    mouse_map_pos.0 += min_x - 1;
    mouse_map_pos.1 += min_y - 1;
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 { 
            valid_target = true; 
        } 
    }

    if valid_target {
        let aoe = gs.ecs.read_storage::<AreaOfEffect>();
        if let Some(ability_aoe) = aoe.get(source) {
            // display projected area of effect
            let map = gs.ecs.fetch::<Map>();
            // use the position of the mouse on the map for calculation
            let points = aoe_points(&*map, Point::new(mouse_map_pos.0, mouse_map_pos.1), ability_aoe.radius);
            for point in points.iter() {
                // use the position of the mouse on the screen for display
                draw_batch.set_bg(Point::new(point.x - min_x + 1, point.y - min_y + 1), cyan());
            }
        }
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
