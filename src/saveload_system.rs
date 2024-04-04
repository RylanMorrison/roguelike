use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator, SerializeComponents, DeserializeComponents, MarkedBuilder};
use specs::error::NoError;
use super::components::*;
use std::fs::File;
use std::path::Path;
use std::fs;
use crate::spatial;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

// can't create local save file for web based
#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs : &mut World) {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs : &mut World) {
    // Create helper

    use std::fmt::Debug;
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let dungeon_master = ecs.get_mut::<super::map::MasterDungeonMap>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper{ map : mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let dm_savehelper = ecs
        .create_entity()
        .with(DMSerializationHelper{ map: dungeon_master })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = ( ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>() );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        // TODO: specs::error::NoError used by serializer is deprecated
        serialize_individually!(ecs, serializer, data, Position, Renderable, Player, Viewshed, Name,
            BlocksTile, Pools, WantsToMelee, Item, Consumable, Ranged, Damage, AreaOfEffect, 
            Confusion, Healing, InBackpack, WantsToPickupItem, WantsToUseItem, SingleActivation,
            WantsToDropItem, SerializationHelper, Equippable, MeleeWeapon, Wearable, WantsToUnequipItem,
            ParticleLifetime, MagicMapping, HungerClock, BlocksVisibility, Door, EntityMoved, Quips, 
            Attributes, Skills, NaturalAttackDefence, LootTable, OtherLevelPosition, DMSerializationHelper, 
            LightSource, Initiative, MyTurn, Faction, WantsToApproach, WantsToFlee, MoveMode, Chasing,
            EquipmentChanged, Vendor, TownPortal, EntryTrigger, TeleportTo, ApplyMove, ApplyTeleport,
            Food, SpawnParticleLine, SpawnParticleBurst, AttributeBonus, Duration, StatusEffect,
            KnownSpells, Spell, WantsToCastSpell, RestoresMana, TeachesSpell, Slow, DamageOverTime,
            SpecialAbilities, TileSize, PendingLevelUp, SkillBonus, ItemSets, PartOfSet
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
    ecs.delete_entity(dm_savehelper).expect("Crash on cleanup");
}

pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &$data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let data = fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (&mut ecs.entities(), &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(), &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>());
        // TODO: specs::error::NoError used by deserializer is deprecated
        deserialize_individually!(ecs, de, d, Position, Renderable, Player, Viewshed, Name,
            BlocksTile, Pools, WantsToMelee, Item, Consumable, Ranged, Damage, AreaOfEffect, 
            Confusion, Healing, InBackpack, WantsToPickupItem, WantsToUseItem, SingleActivation,
            WantsToDropItem, SerializationHelper, Equippable, MeleeWeapon, Wearable, WantsToUnequipItem,
            ParticleLifetime, MagicMapping, HungerClock, BlocksVisibility, Door, EntityMoved, Quips, 
            Attributes, Skills, NaturalAttackDefence, LootTable, OtherLevelPosition, DMSerializationHelper, 
            LightSource, Initiative, MyTurn, Faction, WantsToApproach, WantsToFlee, MoveMode, Chasing,
            EquipmentChanged, Vendor, TownPortal, EntryTrigger, TeleportTo, ApplyMove, ApplyTeleport,
            Food, SpawnParticleLine, SpawnParticleBurst, AttributeBonus, Duration, StatusEffect,
            KnownSpells, Spell, WantsToCastSpell, RestoresMana, TeachesSpell, Slow, DamageOverTime,
            SpecialAbilities, TileSize, PendingLevelUp, SkillBonus, ItemSets, PartOfSet
        );
    }

    let mut deleteme: Option<Entity> = None;
    let mut dm_deleteme: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let dm_helper = ecs.read_storage::<DMSerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            spatial::set_size((worldmap.height * worldmap.width) as usize);
            deleteme = Some(e);
        }
        for (e, h) in (&entities, &dm_helper).join() {
            let mut dungeonmaster = ecs.write_resource::<super::map::MasterDungeonMap>();
            *dungeonmaster = h.map.clone();
            dm_deleteme = Some(e);
        }
        for (e,_p,pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<rltk::Point>();
            *ppos = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap()).expect("Unable to delete helper");
    ecs.delete_entity(dm_deleteme.unwrap()).expect("Unable to delete helper");
}

pub fn delete_save() {
    if does_save_exist() { std::fs::remove_file("./savegame.json").expect("Unable to delete file"); }
}
