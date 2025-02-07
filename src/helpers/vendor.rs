use specs::prelude::*;
use crate::{EquipmentChanged, InBackpack, Item, ItemQuality, Pools, State, Name};
use crate::raws::{self, SpawnType};
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
    let backpack = gs.ecs.read_storage::<InBackpack>();

    if backpack.count() >= 26 {
        gamelog::Logger::new().inventory_full().log();
    } else if player_pools.gold >= item_price {
        std::mem::drop(backpack);

        player_pools.gold -= item_price;
        std::mem::drop(pools);

        let player_entity = *gs.ecs.fetch::<Entity>();
        raws::spawn_named_item(
            &raws::RAWS.lock().unwrap(),
            &mut gs.ecs,
            &item_name,
            raws::SpawnType::Carried{ by: player_entity },
            ItemQuality::Standard
        );
        gs.ecs.write_storage::<EquipmentChanged>().insert(*gs.ecs.fetch::<Entity>(), EquipmentChanged{}).expect("Unable to insert");
    } else {
        gamelog::Logger::new().append("You cannot afford that.").log();
    }
}

pub fn improve_item(gs: &mut State, item_entity: Entity, improve_cost: i32) {
    let mut pools = gs.ecs.write_storage::<Pools>();
    let player_entity = *gs.ecs.fetch_mut::<Entity>();
    let player_pools = pools.get_mut(player_entity).unwrap();

    if player_pools.gold >= improve_cost {
        player_pools.gold -= improve_cost;
        std::mem::drop(pools);

        let items = gs.ecs.read_storage::<Item>();
        let item = items.get(item_entity).unwrap().clone();
        let new_item_quality = match item.quality {
            ItemQuality::Damaged => ItemQuality::Worn,
            ItemQuality::Worn => ItemQuality::Standard,
            ItemQuality::Standard => ItemQuality::Improved,
            _ => ItemQuality::Exceptional
        };
        std::mem::drop(items);

        gs.ecs.entities().delete(item_entity).expect("Unable to delete item entity");

        raws::spawn_named_item(
            &raws::RAWS.lock().unwrap(),
            &mut gs.ecs,
            &item.name,
            SpawnType::Carried { by: player_entity },
            new_item_quality
        );

        gamelog::Logger::new().append("Quality of").item_name(&item).append("improved.").log();
        gs.ecs.write_storage::<EquipmentChanged>().insert(player_entity, EquipmentChanged{}).expect("Unable to insert");
    } else {
        gamelog::Logger::new().append("You cannot afford that.").log();
    }
}
