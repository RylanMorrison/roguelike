use specs::prelude::*;
use specs_derive::*;
use specs::{Entity, saveload::{ConvertSaveload, Marker}, error::NoError};
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point, FontCharType};
use crate::gamelog::LogFragment;

use super::attr_bonus;
use std::collections::HashMap;

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
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

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub initiative_penalty: f32,
    pub weight_lbs: f32,
    pub base_value: i32,
    pub class: ItemClass
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum ItemClass { Common, Rare, Legendary, Set, Unique }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicItem {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Healing {
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
    pub target: Option<Point>
}

// WantsToUseItem wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToUseItemData<M>(M, Option<Point>);

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
pub struct Consumable {
    pub max_charges: i32,
    pub charges: i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32
}

#[derive(Component, Deserialize, Serialize, Clone)]
pub struct Damage {
    pub damage: String
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Confusion {}

// Serialization helper. We need to implement ConvertSaveload for each type that contains an Entity
pub struct SerializeMe {}

// Used to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DMSerializationHelper {
    pub map: super::map::MasterDungeonMap,
    pub log: Vec<Vec<LogFragment>>,
    pub events: HashMap<String, i32>
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
pub struct Weapon {
    pub range: Option<i32>,
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
    pub proc_chance: Option<f32>,
    pub proc_target: Option<String>
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleAnimation {
    pub step_time: f32,
    pub path: Vec<Point>,
    pub current_step: usize,
    pub timer: f32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
    pub animation: Option<ParticleAnimation>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapping {}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState { WellFed, Normal, Hungry, Starving }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state : HungerState,
    pub duration : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Food {}

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Skill {
    pub base: i32,
    pub modifiers: i32
}

impl Skill {
    pub fn bonus(&self) -> i32 {
        self.base + self.modifiers
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub melee: Skill,
    pub defence: Skill,
    pub magic: Skill
}

impl Skills {
    pub fn default() -> Skills {
        Skills{
            melee: Skill{ base: 1, modifiers: 0 },
            defence: Skill{ base: 1, modifiers: 0 },
            magic: Skill{ base: 1, modifiers: 0 }
        }
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
    pub level: i32,
    pub total_weight: f32,
    pub total_initiative_penalty: f32,
    pub gold: i32,
    pub total_armour_class: i32,
    pub base_damage: String,
    pub god_mode: bool
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

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EquipmentChanged {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Vendor {
    pub categories: Vec<String>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TownPortal {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeleportTo {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
    pub player_only: bool
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ApplyMove {
    pub dest_idx: usize
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ApplyTeleport {
    pub dest_x: i32,
    pub dest_y: i32,
    pub dest_depth: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleLine {
    pub glyph: FontCharType,
    pub colour: RGB,
    pub lifetime_ms: f32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleBurst {
    pub glyph: FontCharType,
    pub colour: RGB,
    pub lifetime_ms: f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct AttributeBonus {
    pub strength: Option<i32>,
    pub dexterity: Option<i32>,
    pub constitution: Option<i32>,
    pub intelligence: Option<i32>
}

impl AttributeBonus {
    pub fn is_debuff(&self) -> bool {
        let total_bonus = self.strength.unwrap_or(0) + self.dexterity.unwrap_or(0) 
            + self.constitution.unwrap_or(0) + self.intelligence.unwrap_or(0);
        total_bonus < 0
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SkillBonus {
    pub melee: Option<i32>,
    pub defence: Option<i32>,
    pub magic: Option<i32>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Duration {
    pub turns: i32
}

#[derive(Component, Debug, Clone)]
pub struct StatusEffect {
    pub target: Entity,
    pub is_debuff: bool
}

// StatusEffect wrapper
// TODO serialization causes saving to crash
#[derive(Serialize, Deserialize, Clone)]
pub struct StatusEffectData<M>(M, bool);

impl<M: Marker + Serialize> ConvertSaveload<M> for StatusEffect
where
    for<'de> M: Deserialize<'de>,
{
    type Data = StatusEffectData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.target).unwrap();
        Ok(StatusEffectData(marker, self.is_debuff))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(StatusEffect{target: entity, is_debuff: data.1})
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpell {
    pub name: String,
    pub mana_cost: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpells {
    pub spells: Vec<KnownSpell>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Spell {
    pub mana_cost: i32
}

#[derive(Component, Debug, Clone)]
pub struct WantsToCastSpell {
    pub spell: Entity,
    pub target: Option<Point>
}

// WantsToCastSpell wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToCastSpellData<M>(M, Option<Point>);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToCastSpell
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToCastSpellData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.spell).unwrap();
        Ok(WantsToCastSpellData(marker, self.target))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let spell = ids(data.0).unwrap();
        let target = data.1;
        Ok(WantsToCastSpell{spell, target})
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct RestoresMana {
    pub mana_amount: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeachesSpell {
    pub spell: String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Slow {
    pub initiative_penalty: f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct DamageOverTime {
    pub damage: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecialAbility {
    pub spell: String,
    pub chance: f32,
    pub range: f32,
    pub min_range: f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SpecialAbilities {
    pub abilities: Vec<SpecialAbility>
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct TileSize {
    pub x: i32,
    pub y: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PendingLevelUp {
    pub attributes: Attributes,
    pub skills: Skills
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemSetBonus {
    pub attribute_bonus: Option<AttributeBonus>,
    pub skill_bonus: Option<SkillBonus>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemSet {
    pub total_pieces: i32,
    pub set_bonuses: HashMap<i32, ItemSetBonus>
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct ItemSets {
    pub item_sets: HashMap<String, ItemSet>
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PartOfSet {
    pub set_name: String
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Target {}

#[derive(Component, Clone, Debug)]
pub struct WantsToShoot {
    pub target: Entity
}

// WantsToShoot wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToShootData<M>(M);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToShoot
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToShootData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.target).unwrap();
        Ok(WantsToShootData(marker))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(WantsToShoot{target: entity})
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Stun {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct StatusEffectChanged {
    pub expired: bool
}
