use specs::prelude::*;
use super::{Map, WantsToUseItem, WantsToUseAbility, AreaOfEffect, EquipmentChanged};
use crate::effects::*;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ( 
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, EquipmentChanged>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, map, entities, mut wants_use, aoe, mut dirty) = data;

        if wants_use.is_empty() { return; }

        for (entity, useitem) in (&entities, &wants_use).join() {
            add_effect(
                Some(entity),
                EffectType::ItemUse{ item: useitem.item },
                match useitem.target {
                    None => Targets::Single{ target: *player_entity },
                    Some(target) => {
                        if let Some(aoe) = aoe.get(useitem.item) {
                            Targets::Tiles{ tiles: aoe_tiles(&*map, target, aoe.radius) }
                        } else {
                            Targets::Tile{ tile_idx: map.xy_idx(target.x, target.y) as i32 }
                        }
                    }
                }
            );
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");
        }
        wants_use.clear();
    }
}

pub struct AbilityUseSystem {}

impl<'a> System<'a> for AbilityUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseAbility>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, EquipmentChanged>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, map, entities, mut wants_cast,
            aoe, mut dirty) = data;

        if wants_cast.is_empty() { return; }

        for (entity, use_ability) in (&entities, &wants_cast).join() {
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            add_effect(
                Some(entity),
                EffectType::AbilityUse{ ability: use_ability.ability },
                match use_ability.target {
                    None => Targets::Single{ target: *player_entity },
                    Some(target) => {
                        if let Some(aoe) = aoe.get(use_ability.ability) {
                            Targets::Tiles{ tiles: aoe_tiles(&*map, target, aoe.radius) }
                        } else {
                            Targets::Tile{ tile_idx: map.xy_idx(target.x, target.y) as i32 }
                        }
                    }
                }
            );
        }
        wants_cast.clear();
    }
}
