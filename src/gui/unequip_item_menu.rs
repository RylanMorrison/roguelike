use specs::prelude::*;
use rltk::prelude::*;
use super::{ItemMenuResult, item_result_menu};
use crate::{Equipped, Item, Name, State};

pub fn unequip_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let equipped_items = gs.ecs.read_storage::<Equipped>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let mut inventory: Vec<(Entity, Item, String)> = Vec::new();
    for (entity, item, name, equipped) in (&entities, &items, &names, &equipped_items).join() {
        if equipped.owner == *player_entity {
            inventory.push((entity, item.clone(), name.name.clone()));
        }
    }

    let result = item_result_menu(
        &mut draw_batch,
        "Unequip which item?", 
        &inventory,
        ctx.key
    );
    draw_batch.submit(1000).expect("Draw batch submission failed");
    result
}
