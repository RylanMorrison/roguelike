use std::convert::Infallible;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator, SerializeComponents, DeserializeComponents, MarkedBuilder};
use crate::components::*;
use std::fs::File;
use std::path::Path;
use std::fs;
use crate::{gamelog, spatial};

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<Infallible, SimpleMarker<SerializeMe>>::serialize(
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

    let mapcopy = ecs.get_mut::<crate::map::Map>().unwrap().clone();
    let dungeon_master = ecs.get_mut::<crate::map::MasterDungeonMap>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper{ map : mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let dm_savehelper = ecs
        .create_entity()
        .with(DMSerializationHelper{
            map: dungeon_master,
            log: gamelog::clone_log(),
            events: gamelog::clone_events()
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = ( ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>() );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(ecs, serializer, data, Position, Renderable, Player, Viewshed, Name,
            BlocksTile, Pools, WantsToMelee, Item, Consumable, Ranged, Damage, AreaOfEffect,
            Confusion, Healing, InBackpack, WantsToPickupItem, WantsToUseItem, SingleActivation,
            WantsToDropItem, SerializationHelper, Equippable, Weapon, Wearable, WantsToUnequipItem,
            ParticleLifetime, MagicMapping, HungerClock, BlocksVisibility, Door, EntityMoved, Quips,
            Attributes, Skills, NaturalAttackDefence, LootTable, OtherLevelPosition, DMSerializationHelper,
            LightSource, Initiative, MyTurn, Faction, WantsToApproach, WantsToFlee, MoveMode, Chasing,
            EquipmentChanged, Vendor, TownPortal, EntryTrigger, TeleportTo, ApplyMove, ApplyTeleport,
            Food, SpawnParticleLine, SpawnParticleBurst, AttributeBonus, Duration, StatusEffect,
            KnownAbilities, KnownAbility, AttributeBonus, WantsToUseAbility, RestoresMana, TeachesAbility,
            Slow, DamageOverTime, TileSize, PendingCharacterLevelUp, SkillBonus, ItemSets, PartOfSet, Target,
            WantsToShoot, Stun, StatusEffectChanged, Boss, Chest, CharacterClass, SelfDamage, Rage, Block, Fortress,
            FrostShield, Dodge, WantsToLearnAbility, WantsToLevelAbility
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
        DeserializeComponents::<Infallible, _>::deserialize(
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
        deserialize_individually!(ecs, de, d, Position, Renderable, Player, Viewshed, Name,
            BlocksTile, Pools, WantsToMelee, Item, Consumable, Ranged, Damage, AreaOfEffect,
            Confusion, Healing, InBackpack, WantsToPickupItem, WantsToUseItem, SingleActivation,
            WantsToDropItem, SerializationHelper, Equippable, Weapon, Wearable, WantsToUnequipItem,
            ParticleLifetime, MagicMapping, HungerClock, BlocksVisibility, Door, EntityMoved, Quips,
            Attributes, Skills, NaturalAttackDefence, LootTable, OtherLevelPosition, DMSerializationHelper,
            LightSource, Initiative, MyTurn, Faction, WantsToApproach, WantsToFlee, MoveMode, Chasing,
            EquipmentChanged, Vendor, TownPortal, EntryTrigger, TeleportTo, ApplyMove, ApplyTeleport,
            Food, SpawnParticleLine, SpawnParticleBurst, AttributeBonus, Duration, StatusEffect,
            KnownAbilities, KnownAbility, Ability, WantsToUseAbility, RestoresMana, TeachesAbility,
            Slow, DamageOverTime, TileSize, PendingCharacterLevelUp, SkillBonus, ItemSets, PartOfSet, Target,
            WantsToShoot, Stun, StatusEffectChanged, Boss, Chest, CharacterClass, SelfDamage, Rage, Block, Fortress,
            FrostShield, Dodge, WantsToLearnAbility, WantsToLevelAbility
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
            let mut worldmap = ecs.write_resource::<crate::map::Map>();
            *worldmap = h.map.clone();
            spatial::set_size((worldmap.height * worldmap.width) as usize);
            deleteme = Some(e);
        }
        for (e, h) in (&entities, &dm_helper).join() {
            let mut dungeonmaster = ecs.write_resource::<crate::map::MasterDungeonMap>();
            *dungeonmaster = h.map.clone();
            dm_deleteme = Some(e);
            gamelog::restore_log(&mut h.log.clone());
            gamelog::load_events(h.events.clone());
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
