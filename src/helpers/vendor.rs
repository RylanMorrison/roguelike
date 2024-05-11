use specs::prelude::*;
use crate::{State, Item, Pools, EquipmentChanged, ItemQuality};
use crate::raws;
use crate::gamelog;

pub fn sell_item(gs: &mut State, item_entity: Entity) {
    let price = gs.ecs.read_storage::<Item>().get(item_entity).unwrap().base_value as f32 * 0.8;

    gs.ecs.write_storage::<Pools>().get_mut(*gs.ecs.fetch::<Entity>()).unwrap().gold += price as i32;
    gs.ecs.delete_entity(item_entity).expect("Unable to delete");
    gs.ecs.write_storage::<EquipmentChanged>().insert(*gs.ecs.fetch::<Entity>(), EquipmentChanged{}).expect("Unable to insert");
}

pub fn buy_item(gs: &mut State, item_name: String, item_price: i32) {
    let mut pools = gs.ecs.write_storage::<Pools>();
    let player_pools = pools.get_mut(*gs.ecs.fetch::<Entity>()).unwrap();

    if player_pools.gold >= item_price {
        player_pools.gold -= item_price;
        std::mem::drop(pools);
        let player_entity = *gs.ecs.fetch::<Entity>();
        raws::spawn_named_item(&raws::RAWS.lock().unwrap(), &mut gs.ecs, &item_name, raws::SpawnType::Carried{ by: player_entity });
        gs.ecs.write_storage::<EquipmentChanged>().insert(*gs.ecs.fetch::<Entity>(), EquipmentChanged{}).expect("Unable to insert");
    } else {
        gamelog::Logger::new().append("You cannot afford that.").log();
    }
}

pub fn improve_item(gs: &mut State, item_entity: Entity, improve_cost: i32) {
    let mut pools = gs.ecs.write_storage::<Pools>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pools = pools.get_mut(*player_entity).unwrap();
    if player_pools.gold >= improve_cost {
        player_pools.gold -= improve_cost;
        std::mem::drop(pools);
        let mut items = gs.ecs.write_storage::<Item>();
        let item = items.get_mut(item_entity).unwrap();

        match item.quality {
            Some(ItemQuality::Damaged) => item.quality = Some(ItemQuality::Worn),
            Some(ItemQuality::Worn) => item.quality = None,
            None => item.quality = Some(ItemQuality::Improved),
            _ => item.quality = Some(ItemQuality::Exceptional)
        }
        item.base_value = raws::get_item_value(&item.quality, item.base_value);
        
        gs.ecs.write_storage::<EquipmentChanged>().insert(*player_entity, EquipmentChanged{}).expect("Unable to insert");
    }
}
