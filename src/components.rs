use specs::prelude::*;
use specs_derive::*;
use specs::{Entity, saveload::{ConvertSaveload, Marker}, error::NoError};
use serde::{Serialize, Deserialize};
use rltk::RGB;
use super::attr_bonus;
use std::collections::HashMap;

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Name {
    pub name: String
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct WantsToMelee {
    pub target: Entity
}

// WantsToMelee wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToMeleeData<M>(M);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToMelee
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToMeleeData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.target).unwrap();
        Ok(WantsToMeleeData(marker))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(WantsToMelee{target: entity})
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: Vec<(i32, bool)>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32, from_player: bool) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push((amount, from_player));
        } else {
            let damage = SufferDamage { amount: vec![(amount, from_player)] };
            store.insert(victim, damage).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Item {
    pub colour: String
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32
}

#[derive(Component, Debug)]
pub struct InBackpack {
    pub owner: Entity
}

// InBackpack wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct InBackpackData<M>(M);

impl<M: Marker + Serialize> ConvertSaveload<M> for InBackpack
where
    for<'de> M: Deserialize<'de>,
{
    type Data = InBackpackData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.owner).unwrap();
        Ok(InBackpackData(marker))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(InBackpack{owner: entity})
    }
}

#[derive(Component, Debug)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}

// WantsToPickupItem wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToPickupItemData<M>(M, M);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToPickupItem
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToPickupItemData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.collected_by).unwrap();
        let marker2 = ids(self.item).unwrap();
        Ok(WantsToPickupItemData(marker, marker2))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let collected_by = ids(data.0).unwrap();
        let item = ids(data.1).unwrap();
        Ok(WantsToPickupItem{collected_by, item})
    }
}

#[derive(Component, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>
}

// WantsToUseItem wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToUseItemData<M>(M, Option<rltk::Point>);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToUseItem
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToUseItemData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.item).unwrap();
        Ok(WantsToUseItemData(marker, self.target))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let item = ids(data.0).unwrap();
        let target = data.1;
        Ok(WantsToUseItem{item, target})
    }
}

#[derive(Component, Debug)]
pub struct WantsToDropItem {
    pub item: Entity
}

// WantsToDropItem wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToDropItemData<M>(M);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToDropItem
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToDropItemData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.item).unwrap();
        Ok(WantsToDropItemData(marker))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(WantsToDropItem{item: entity})
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32
}

#[derive(Component, Deserialize, Serialize, Clone)]
pub struct InflictsDamage {
    pub damage: String
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32
}

// Serialization helper. We need to implement ConvertSaveload for each type that contains an Entity
pub struct SerializeMe {}

// Used to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DMSerializationHelper {
    pub map: super::map::MasterDungeonMap
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    TwoHanded,
    Head,
    Body,
    Hands,
    Feet
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

#[derive(Component, Debug)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}

// Equipped wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct EquippedData<M>(M, EquipmentSlot);

impl<M: Marker + Serialize> ConvertSaveload<M> for Equipped
where
    for<'de> M: Deserialize<'de>,
{
    type Data = EquippedData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.owner).unwrap();
        Ok(EquippedData(marker, self.slot))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(Equipped{owner: entity, slot : data.1})
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WeaponAttribute { Strength, Dexterity }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MeleeWeapon {
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Wearable {
    pub armour_class: f32,
    pub slot: EquipmentSlot
}

#[derive(Component, Debug)]
pub struct WantsToUnequipItem {
    pub item: Entity
}

// WantsToUnequipItem wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToUnequipItemData<M>(M);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToUnequipItem
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToUnequipItemData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.item).unwrap();
        Ok(WantsToUnequipItemData(marker))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(WantsToUnequipItem{item: entity})
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState { WellFed, Normal, Hungry, Starving }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state : HungerState,
    pub duration : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesFood {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Door {
    pub open: bool
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Quips {
    pub available : Vec<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32
}

// See: https://roll20.net/compendium/dnd5e/Ability%20Scores#content
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub strength: Attribute,
    pub dexterity: Attribute,
    pub constitution: Attribute,
    pub intelligence: Attribute
}

impl Attributes {
    pub fn default() -> Attributes {
        Attributes { 
            strength: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            dexterity: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            constitution: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Skill {
    Melee,
    Defence,
    Magic
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub skills: HashMap<Skill, i32>
}

impl Skills {
    pub fn default() -> Skills {
        let mut skills = Skills{ skills: HashMap::new() };
        skills.skills.insert(Skill::Melee, 1);
        skills.skills.insert(Skill::Defence, 1);
        skills.skills.insert(Skill::Magic, 1);
        skills
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pool {
    pub max: i32,
    pub current: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub xp: i32,
    pub level: i32
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NaturalAttack {
    pub name: String,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct NaturalAttackDefence {
    pub armour_class: Option<i32>,
    pub attacks: Vec<NaturalAttack>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LootTable {
    pub table_name: String
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct OtherLevelPosition {
    pub x: i32,
    pub y: i32,
    pub depth: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct LightSource {
    pub colour: RGB,
    pub range: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Initiative {
    pub current: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MyTurn {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Faction {
    pub name: String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToApproach {
    pub idx: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToFlee {
    pub indices: Vec<usize>
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Movement { 
    Static, 
    Random,
    RandomWaypoint{ path: Option<Vec<usize>> }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MoveMode {
    pub mode : Movement
}

#[derive(Component, Debug)]
pub struct Chasing {
    pub target: Entity
}

// Chasing wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct ChasingData<M>(M);

impl<M: Marker + Serialize> ConvertSaveload<M> for Chasing
where
    for<'de> M: Deserialize<'de>,
{
    type Data = ChasingData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.target).unwrap();
        Ok(ChasingData(marker))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(Chasing{target: entity})
    }
}
