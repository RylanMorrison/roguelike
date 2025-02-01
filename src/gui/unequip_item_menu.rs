use specs::prelude::*;
use rltk::prelude::*;
use super::{ItemMenuResult, item_result_menu, item_entity_tooltip};
use crate::{Equipped, Item, State};

pub fn unequip_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped_items = gs.ecs.read_storage::<Equipped>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let mut inventory: Vec<(Entity, Item, String)> = Vec::new();
    for (entity, item, equipped) in (&entities, &items, &equipped_items).join() {
        if equipped.owner == *player_entity {
            inventory.push((entity, item.clone(), item.full_name()));
        }
    }

    let (menu_result, selected_entity, tooltip) = item_result_menu(
        ctx,
        &mut draw_batch,
        "Unequip which item?", 
        &inventory
    );
    draw_batch.submit(1000).expect("Draw batch submission failed");

    if let Some((entity, name, x, y)) = tooltip {
        item_entity_tooltip(&gs.ecs, name, entity).render(&mut draw_batch, x, y);
        draw_batch.submit(1100).expect("Draw batch submission failed");
    }

    (menu_result, selected_entity)
}
