use specs::prelude::*;
use rltk::prelude::*;
use super::{ItemMenuResult, item_result_menu};
use crate::{InBackpack, Item, Name, State};

pub fn drop_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();
    let mut draw_batch = DrawBatch::new();

    let mut inventory: Vec<(Entity, Item, String)> = Vec::new();
    for (entity, item, name, backpack) in (&entities, &items, &names, &backpacks).join() {
        if backpack.owner == *player_entity {
            inventory.push((entity, item.clone(), name.name.clone()));
        }
    }

    let result = item_result_menu(
        &mut draw_batch,
        "Remove which item?",
        &inventory,
        ctx.key
    );
    draw_batch.submit(1000).expect("Draw batch submission failed");
    result
}
