use specs::prelude::*;
use specs_derive::*;
/*
    Latest specs_derive package is using old code for ConvertSaveLoad (see bottom of lib.rs in specs_derive) 
    so can't completely switch from error::NoError to std::convert::Infallible yet
*/
use specs::{Entity, saveload::{ConvertSaveload, Marker}, error::NoError};
use serde::{Serialize, Deserialize};
use rltk::{RGB, Point, FontCharType};
use crate::gamelog::LogFragment;
use super::{attr_bonus, Map, MasterDungeonMap};
use std::{collections::{BTreeMap, HashMap}, convert::Infallible};
use crate::effects::{EffectType, Targets};

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
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
    type Error = Infallible;

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
    pub name: String,
    pub initiative_penalty: f32,
    pub weight_lbs: f32,
    pub base_value: i32,
    pub class: ItemClass,
    pub quality: ItemQuality,
    pub vendor_category: Option<String>
}

impl Item {
    pub fn full_name(&self) -> String {
        match self.quality {
            ItemQuality::Damaged => format!("Damaged {}", self.name),
            ItemQuality::Worn => format!("Worn {}", self.name),
            ItemQuality::Improved => format!("Improved {}", self.name),
            ItemQuality::Exceptional => format!("Exceptional {}", self.name),
            _ => self.name.clone()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum ItemClass { Common, Rare, Legendary, Set, Unique }

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum ItemQuality { Damaged, Worn, Standard, Improved, Exceptional, Random }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicItem {}

#[derive(Component, Serialize, Deserialize, Clone)]
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
    type Error = Infallible;

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
    type Error = Infallible;

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
    type Error = Infallible;

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
    type Error = Infallible;

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

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Ranged {
    pub min_range: f32,
    pub max_range: f32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Damage {
    pub damage: String
}

#[derive(Component, Serialize, Deserialize, Clone)]
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
    pub map: Map,
    pub quests: Quests,
    pub active_quests: ActiveQuests,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DMSerializationHelper {
    pub map: MasterDungeonMap,
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
    type Error = Infallible;

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
        Ok(Equipped{owner: entity, slot: data.1})
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum WeaponAttribute { Strength, Dexterity, Intelligence }

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

impl Weapon {
    pub fn damage(&self) -> String {
        if self.damage_bonus > 0 {
            format!("{}d{}+{}", self.damage_n_dice, self.damage_die_type, self.damage_bonus)
        } else if self.damage_bonus < 0 {
            format!("{}d{}{}", self.damage_n_dice, self.damage_die_type, self.damage_bonus)
        } else {
            format!("{}d{}", self.damage_n_dice, self.damage_die_type)
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Wearable {
    pub armour_class: f32
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
    type Error = Infallible;

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
    pub item_modifiers: i32,
    pub status_effect_modifiers: i32,
    pub bonus: i32
}

impl Attribute {
    pub fn total_modifiers(&self) -> i32 {
        self.item_modifiers + self.status_effect_modifiers
    }
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
            strength: Attribute{ base: 11, item_modifiers: 0, status_effect_modifiers: 0, bonus: attr_bonus(11) },
            dexterity: Attribute{ base: 11, item_modifiers: 0, status_effect_modifiers: 0, bonus: attr_bonus(11) },
            constitution: Attribute{ base: 11, item_modifiers: 0, status_effect_modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute{ base: 11, item_modifiers: 0, status_effect_modifiers: 0, bonus: attr_bonus(11) }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Skill {
    pub base: i32,
    pub item_modifiers: i32,
    pub status_effect_modifiers: i32
}

impl Skill {
    pub fn bonus(&self) -> i32 {
        self.base + self.total_modifiers()
    }

    pub fn total_modifiers(&self) -> i32 {
        self.item_modifiers + self.status_effect_modifiers
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub melee: Skill,
    pub defence: Skill,
    pub ranged: Skill,
    pub magic: Skill,
}

impl Skills {
    pub fn default() -> Skills {
        Skills{
            melee: Skill{ base: 1, item_modifiers: 0, status_effect_modifiers: 0 },
            defence: Skill{ base: 1, item_modifiers: 0, status_effect_modifiers: 0 },
            ranged: Skill{ base: 1, item_modifiers: 0, status_effect_modifiers: 0 },
            magic: Skill{ base: 1, item_modifiers: 0, status_effect_modifiers: 0 }
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
    pub initiative_penalty: InitiativePenalty,
    pub gold: i32,
    pub total_armour_class: i32,
    pub base_damage: String,
    pub god_mode: bool
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitiativePenalty {
    pub gear_effect_penalty: f32,
    pub status_effect_penalty: f32
}

impl InitiativePenalty {
    pub fn initial() -> InitiativePenalty {
        InitiativePenalty { gear_effect_penalty: 0.0, status_effect_penalty: 0.0 }
    }

    pub fn total(&self) -> f32 {
        self.gear_effect_penalty + self.status_effect_penalty
    }
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
    type Error = Infallible;

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
    pub category: String
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

    pub fn combine(&mut self, other: Option<&AttributeBonus>) {
        if other.is_none() { return; }

        self.strength = self.combine_attribute(self.strength, other.unwrap().strength);
        self.dexterity = self.combine_attribute(self.dexterity, other.unwrap().dexterity);
        self.constitution = self.combine_attribute(self.constitution, other.unwrap().constitution);
        self.intelligence = self.combine_attribute(self.intelligence, other.unwrap().intelligence);
    }

    fn combine_attribute(&self, my_attribute: Option<i32>, other_attribute: Option<i32>) -> Option<i32> {
        if my_attribute.is_some() {
            if other_attribute.is_some() {
                return Some(my_attribute.unwrap() + other_attribute.unwrap());
            }
        } else if other_attribute.is_some() {
            return other_attribute;
        }
        None
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SkillBonus {
    pub melee: Option<i32>,
    pub defence: Option<i32>,
    pub ranged: Option<i32>,
    pub magic: Option<i32>
}

impl SkillBonus {
    pub fn combine(&mut self, other: Option<&SkillBonus>) {
        if other.is_none() { return; }

        self.melee = self.combine_skill(self.melee, other.unwrap().melee);
        self.defence = self.combine_skill(self.defence, other.unwrap().defence);
        self.ranged = self.combine_skill(self.ranged, other.unwrap().ranged);
        self.magic = self.combine_skill(self.magic, other.unwrap().magic);
    }

    fn combine_skill(&self, my_skill: Option<i32>, other_skill: Option<i32>) -> Option<i32> {
        if my_skill.is_some() {
            if other_skill.is_some() {
                return Some(my_skill.unwrap() + other_skill.unwrap());
            }
        } else if other_skill.is_some() {
            return other_skill;
        }
        None
    }
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
    type Error = Infallible;

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

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct RestoresMana {
    pub mana_amount: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeachesAbility {
    pub ability: String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Slow {
    pub initiative_penalty: f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct DamageOverTime {
    pub damage: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct TileSize {
    pub x: i32,
    pub y: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToLevelUp {
    pub passives: BTreeMap<String, ClassPassive>
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
    type Error = Infallible;

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

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct StatusEffectChanged {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Boss {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Chest {
    pub gold: Option<String>,
    pub capacity: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct CharacterClass {
    pub name: String,
    pub passives: BTreeMap<String, ClassPassive>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassPassive {
    pub name: String,
    pub description: String,
    pub current_level: i32,
    pub levels: HashMap<i32, ClassPassiveLevel>
}

impl ClassPassive {
    pub fn is_max_level(&self) -> bool {
        self.current_level >= self.levels.len() as i32
    }

    pub fn active_level(&self) -> &ClassPassiveLevel {
        &self.levels[&self.current_level]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassPassiveLevel {
    pub attribute_bonus: Option<AttributeBonus>,
    pub skill_bonus: Option<SkillBonus>,
    pub learn_ability: Option<String>,
    pub level_ability: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum AbilityType {
    Active,
    Passive
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub ability_type: AbilityType,
    pub levels: HashMap<i32, AbilityLevel>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AbilityLevel {
    pub mana_cost: Option<i32>,
    pub effects: HashMap<String, String>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct KnownAbility {
    pub name: String,
    pub level: i32,
    pub mana_cost: i32,
    pub ability_type: AbilityType
}

// Need a wrapper to be able to (de)serialize collections of Entities. See https://github.com/amethyst/specs/issues/681
#[derive(Clone, Debug)]
pub struct EntityVec<T>(Vec<T>);

impl<T> EntityVec<T> {
    pub fn new() -> EntityVec<T> {
        EntityVec { 0: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> EntityVec<T> {
        EntityVec { 0: Vec::with_capacity(capacity) }
    }
}

impl<T> std::ops::Deref for EntityVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> std::ops::DerefMut for EntityVec<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<C, M: Serialize + Marker> ConvertSaveload<M> for EntityVec<C>
    where for<'de> M: Deserialize<'de>,
    C: ConvertSaveload<M>
{
    type Data = Vec<<C as ConvertSaveload<M>>::Data>;
    type Error = <C as ConvertSaveload<M>>::Error;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>
    {
        let mut output = Vec::with_capacity(self.len());

        for item in self.iter() {
            let converted_item = item.convert_into(|entity| ids(entity))?;

            output.push(converted_item);
        }

        Ok(output)
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>
    {
        let mut output: EntityVec<C> = EntityVec::with_capacity(data.len());

        for item in data.into_iter() {
            let converted_item = ConvertSaveload::convert_from(item, |marker| ids(marker))?;

            output.push(converted_item);
        }

        Ok(output)
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct KnownAbilities {
    pub abilities: EntityVec<Entity>
}

#[derive(Component, Clone, Debug)]
pub struct WantsToUseAbility {
    pub ability: Entity,
    pub target: Option<Point>
}

// WantsToUseAbility wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToUseAbilityData<M>(M, Option<Point>);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToUseAbility
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToUseAbilityData<M>;
    type Error = Infallible;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.ability).unwrap();
        Ok(WantsToUseAbilityData(marker, self.target))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let ability = ids(data.0).unwrap();
        let target = data.1;
        Ok(WantsToUseAbility{ability, target})
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SelfDamage {
    pub damage: String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Rage {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub chance: f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Fortress {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct FrostShield {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Dodge {
    pub chance: f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToLearnAbility {
    pub ability_name: String,
    pub level: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToLevelAbility {
    pub ability_name: String
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Quests {
    pub quests: Vec<Quest>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ActiveQuests {
    pub quests: Vec<Quest>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Quest {
    pub name: String,
    pub description: String,
    pub rewards: Vec<QuestReward>,
    pub requirements: Vec<QuestRequirement>,
    pub status: QuestStatus,
    pub next_quests: Vec<String>
}

impl Quest {
    pub fn is_complete(&self) -> bool {
        for requirement in self.requirements.iter() {
            if !requirement.complete { return false; }
        }
        true
    }

    pub fn is_available(&self) -> bool {
        self.status == QuestStatus::Available
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum QuestStatus {
    Unavailable,
    Available,
    Active,
    Complete,
    Failed
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct QuestReward {
    pub gold: Option<String>,
    pub xp: Option<i32>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum QuestRequirementGoal {
    None,
    KillCount
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct QuestRequirement {
    pub requirement_goal: QuestRequirementGoal,
    pub targets: Vec<String>,
    pub count: i32,
    pub target_count: i32,
    pub complete: bool
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum ProgressSource {
    Kill
}

#[derive(Component, Debug, Clone)]
pub struct QuestProgress {
    pub target: Entity,
    pub source: ProgressSource
}

// QuestProgress wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct QuestProgressData<M>(M, ProgressSource);

impl<M: Marker + Serialize> ConvertSaveload<M> for QuestProgress
where
    for<'de> M: Deserialize<'de>,
{
    type Data = QuestProgressData<M>;
    type Error = Infallible;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.target).unwrap();
        Ok(QuestProgressData(marker, self.source))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(QuestProgress{target: entity, source : data.1})
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct QuestGiver {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToTurnInQuest {
    pub quest: Quest
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MapMarker {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Species {
    pub name: String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Repeat {
    pub count: i32
}

#[derive(Component, Debug)]
pub struct WantsToRepeatAbility { // not (de)serialized
    pub effect_type: EffectType,
    pub targets: Targets,
    pub count: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct RegenBonus {
    pub health: Option<i32>,
    pub mana: Option<i32>
}
