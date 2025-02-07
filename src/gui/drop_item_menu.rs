use specs::prelude::*;
use rltk::prelude::*;
use super::{ItemMenuResult, item_result_menu, item_entity_tooltip};
use crate::{InBackpack, Item, State};

pub fn drop_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let mut inventory: Vec<(Entity, Item, String)> = Vec::new();
    for (entity, item, backpack) in (&entities, &items, &backpacks).join() {
        if backpack.owner == *player_entity {
            inventory.push((entity, item.clone(), item.full_name()));
        }
    }

    let (menu_result, selected_entity, tooltip) = item_result_menu(
        ctx,
        &mut draw_batch,
        "Drop which item?",
        &inventory
    );
    draw_batch.submit(1000).expect("Draw batch submission failed");

    if let Some((entity, name, x, y)) = tooltip {
        item_entity_tooltip(&gs.ecs, name, entity).render(&mut draw_batch, x, y);
        draw_batch.submit(3500).expect("Draw batch submission failed");
    }

    (menu_result, selected_entity)
}
