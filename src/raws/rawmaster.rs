use std::collections::{HashMap, HashSet};
use specs::prelude::*;
use rltk::{RandomNumberGenerator, RGB};
use crate::components::*;
use super::{Raws, RenderableData, SpawnTableEntry, Reaction};
use crate::random_table::RandomTable;
use crate::{attr_bonus, npc_hp, mana_at_level};
use regex::Regex;
use specs::saveload::{MarkedBuilder, SimpleMarker};

/// Parse a dice string into its values. 
/// eg. 1d10+4 => (1, 10, 4)
pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity }
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
    loot_index: HashMap<String, usize>,
    faction_index: HashMap<String, HashMap<String, Reaction>>

}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster{
            raws: Raws { 
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
                faction_table: Vec::new()
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
            loot_index: HashMap::new(),
            faction_index: HashMap::new()
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        let mut used_names: HashSet<String> = HashSet::new();
        for (i,item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!("WARNING - duplicate item name in raws [{}]", item.name));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i,mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!("WARNING - duplicate item name in raws [{}]", mob.name));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        for (i,prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!("WARNING - duplicate item name in raws [{}]", prop.name));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }
        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!("WANRING - Spawn table references unspecified entity {}", spawn.name));
            }
        }
        for (i, loot) in self.raws.loot_tables.iter().enumerate() {
            self.loot_index.insert(loot.name.clone(), i);
        }
        for faction in self.raws.faction_table.iter() {
            let mut reactions: HashMap<String, Reaction> = HashMap::new();
            for response in faction.responses.iter() {
                reactions.insert(
                    response.0.clone(),
                    match response.1.as_str() {
                        "ignore" => Reaction::Ignore,
                        "flee" => Reaction::Flee,
                        _ => Reaction::Attack
                    }
                );
            }
            self.faction_index.insert(faction.name.clone(), reactions);
        }
    }
}

fn find_slot_for_equippable_item(tag: &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }
    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if let Some(weapon) = &item.weapon {
        return string_to_weapon_slot(&weapon.slot);
    }
    if let Some(wearable) = &item.wearable {
        return string_to_wearable_slot(&wearable.slot);
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}

fn string_to_weapon_slot(slot: &str) -> EquipmentSlot {
    match slot {
        "Main Hand" => EquipmentSlot::MainHand,
        "Off Hand" => EquipmentSlot::OffHand,
        "Two Handed" => EquipmentSlot::TwoHanded,
        _ => {
            rltk::console::log(format!("Warning: Unknown weapon slot type [{}]", slot));
            EquipmentSlot::MainHand
        }
    }
}

fn string_to_wearable_slot(slot: &str) -> EquipmentSlot {
    match slot {
        "Off Hand" => EquipmentSlot::OffHand,
        "Head" => EquipmentSlot::Head,
        "Body" => EquipmentSlot::Body,
        "Hands" => EquipmentSlot::Hands,
        "Feet" => EquipmentSlot::Feet,
        _ => {
            rltk::console::log(format!("Warning: Unknown wearable slot type [{}]", slot));
            EquipmentSlot::Head
        }
    }
}

fn spawn_position<'a>(pos: SpawnType, new_entity: EntityBuilder<'a>, tag: &str, raws: &RawMaster) -> EntityBuilder<'a> {
    let eb = new_entity;

    match pos {
        SpawnType::AtPosition{x,y} => eb.with(Position{ x, y }),
        SpawnType::Carried{by} => eb.with(InBackpack{ owner: by }),
        SpawnType::Equipped{by} => {
            let slot = find_slot_for_equippable_item(tag, raws);
            eb.with(Equipped{ owner: by, slot })
        }
    }
}

fn get_renderable_component(renderable: &RenderableData) -> Renderable {
    Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order
    }
}

pub fn spawn_named_entity(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, ecs, key, pos);
    }
    if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, ecs, key, pos);
    }
    if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs, key, pos);
    }
    None
}

pub fn spawn_named_item(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    let item_template = &raws.raws.items[raws.item_index[key]];
    let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

    // spawn in the specified location
    eb = spawn_position(pos, eb, key, raws);

    // renderable
    let mut colour = "#000000".to_string();
    if let Some(renderable) = &item_template.renderable {
        eb = eb.with(get_renderable_component(renderable));
        colour = renderable.fg.clone();
    }
    eb = eb.with(Name{ name: item_template.name.clone() });
    eb = eb.with(Item{ colour });

    // consumables
    if let Some(consumable) = &item_template.consumable {
        eb = eb.with(Consumable{});
        for effect in consumable.effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "healing" => {
                    eb = eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() });
                }
                "ranged" => {
                    eb = eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() });
                }
                "damage" => {
                    eb = eb.with(InflictsDamage{ damage: effect.1.to_string() });
                }
                "aoe" => {
                    eb = eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() });
                }
                "confusion" => {
                    eb = eb.with(Confusion{ turns: effect.1.parse::<i32>().unwrap() });
                }
                "magic_mapping" => {
                    eb = eb.with(MagicMapper{});
                }
                "provides_food" => { // TODO: remove provides
                    eb = eb.with(ProvidesFood{});
                }
                _ => {
                    rltk::console::log(format!("Warning: consumable effect {} not implemented", effect_name));
                }
            }
        }
    }

    // equipment
    if let Some(weapon) = &item_template.weapon {
        eb = eb.with(Equippable{ slot: EquipmentSlot::MainHand });
        let (n_dice, die_type, bonus) = parse_dice_string(&weapon.base_damage);
        let wpn = MeleeWeapon {
            attribute: if weapon.attribute.as_str() == "Strength" {
                WeaponAttribute::Strength
            } else {
                WeaponAttribute::Dexterity
            },
            damage_n_dice: n_dice,
            damage_die_type: die_type,
            damage_bonus: bonus,
            hit_bonus: weapon.hit_bonus
        };
        eb = eb.with(wpn);
    }
    if let Some(wearable) = &item_template.wearable {
        let slot = string_to_wearable_slot(&wearable.slot);
        eb = eb.with(Equippable{ slot });
        eb = eb.with(Wearable{ slot, armour_class: wearable.armour_class });
    }

    Some(eb.build())
}

pub fn spawn_named_mob(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    let mob_template = &raws.raws.mobs[raws.mob_index[key]];
    let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

    // spawn in the specified location
    eb = spawn_position(pos, eb, key, raws);

    // name
    eb = eb.with(Name{ name: mob_template.name.clone() });

    // renderable
    if let Some(renderable) = &mob_template.renderable {
        eb = eb.with(get_renderable_component(renderable));
    }

    // initiative
    eb = eb.with(Initiative{current: 2});

    // movement
    match mob_template.movement.as_ref() {
        "random" => eb = eb.with(MoveMode{ mode: Movement::Random }),
        "random_waypoint" => eb = eb.with(MoveMode{ mode: Movement::RandomWaypoint { path: None } }),
        _ => eb = eb.with(MoveMode{ mode: Movement::Static })
    }

    if mob_template.blocks_tile {
        eb = eb.with(BlocksTile{});
    }

    // quips
    if let Some(quips) = &mob_template.quips {
        eb = eb.with(Quips{
            available: quips.clone()
        });
    }

    // attributes
    let mut attr = Attributes::default();
    let mut mob_constitution = attr.constitution.base;
    let mut mob_intelligence = attr.intelligence.base;
    if let Some(strength) = mob_template.attributes.strength {
        attr.strength = Attribute{ base: strength, modifiers: 0, bonus: attr_bonus(strength) };
    }
    if let Some(dexterity) = mob_template.attributes.dexterity {
        attr.dexterity = Attribute{ base: dexterity, modifiers: 0, bonus: attr_bonus(dexterity) };
    }
    if let Some(constitution) = mob_template.attributes.constitution {
        attr.constitution = Attribute{ base: constitution, modifiers: 0, bonus: attr_bonus(constitution) };
        mob_constitution = constitution;
    }
    if let Some(intelligence) = mob_template.attributes.intelligence {
        attr.intelligence = Attribute{ base: intelligence, modifiers: 0, bonus: attr_bonus(intelligence) };
        mob_intelligence = intelligence;
    }
    eb = eb.with(attr);

    // pools
    let mob_level = if mob_template.level.is_some() {
        mob_template.level.unwrap()
    } else {
        1
    };
    let mob_hp = npc_hp(mob_constitution, mob_level);
    let mob_mana = mana_at_level(mob_intelligence, mob_level);
    let pools = Pools{
        level: mob_level,
        xp: 0,
        hit_points: Pool{ current: mob_hp, max: mob_hp },
        mana: Pool{ current: mob_mana, max: mob_mana }
    };
    eb = eb.with(pools);

    // skills
    let mut skills = Skills::default();
    if let Some(mobskills) = &mob_template.skills {
        for sk in mobskills.iter() {
            match sk.0.as_str() {
                "melee" => { skills.skills.insert(Skill::Melee, *sk.1); }
                "defence" => { skills.skills.insert(Skill::Defence, *sk.1); }
                "magic" => { skills.skills.insert(Skill::Magic, *sk.1); }
                _ => { rltk::console::log(format!("Unknown skill referenced: [{}]", sk.0)); }
            }
        }
    }
    eb = eb.with(skills);

    // natural ability
    if let Some(na) = &mob_template.natural {
        let mut nature = NaturalAttackDefence {
            armour_class: na.armour_class,
            attacks: Vec::new()
        };
        if let Some(attacks) = &na.attacks {
            for nattack in attacks.iter() {
                let (n, d, b) = parse_dice_string(&nattack.damage);
                let attack = NaturalAttack {
                    name: nattack.name.clone(),
                    hit_bonus: nattack.hit_bonus,
                    damage_n_dice: n,
                    damage_die_type: d,
                    damage_bonus: b
                };
                nature.attacks.push(attack);
            }
        }
        eb = eb.with(nature);
    }

    // visibility
    eb = eb.with(Viewshed{ 
        visible_tiles: Vec::new(),
        range: mob_template.vision_range,
        dirty: true
    });

    // loot
    if let Some(loot) = &mob_template.loot_table {
        eb = eb.with(LootTable{ table_name: loot.clone() });
    }

    // light
    if let Some(light) = &mob_template.light {
        eb = eb.with(LightSource{ range: light.range, colour: RGB::from_hex(&light.colour).expect("Bad colour") });
    }

    //faction
    if let Some(faction) = &mob_template.faction {
        eb = eb.with(Faction{ name: faction.clone() });
    } else {
        eb = eb.with(Faction{ name: "Mindless".to_string() })
    }

    let new_mob = eb.build();

    // equipment
    if let Some(wielding) = &mob_template.equipped {
        for tag in wielding.iter() {
            spawn_named_entity(raws, ecs, tag, SpawnType::Equipped{ by: new_mob });
        }
    }
    Some(new_mob)
}

pub fn spawn_named_prop(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    let prop_template = &raws.raws.props[raws.prop_index[key]];
    let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

    // spawn in the specified location
    eb = spawn_position(pos, eb, key, raws);

    // renderable
    if let Some(renderable) = &prop_template.renderable {
        eb = eb.with(get_renderable_component(renderable));
    }
    
    eb = eb.with(Name{ name: prop_template.name.clone() });

    if let Some(blocks_tile) = prop_template.blocks_tile {
        if blocks_tile { eb = eb.with(BlocksTile{}) };
    }
    if let Some(blocks_visibility) = prop_template.blocks_visibility {
        if blocks_visibility { eb = eb.with(BlocksVisibility{} )};
    }
    if let Some(door_open) = prop_template.door_open {
        eb = eb.with(Door{ open: door_open });
    }

    Some(eb.build())
}

pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> RandomTable {
    let available_options: Vec<&SpawnTableEntry> = raws.raws.spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt = rt.add(e.name.clone(), weight);
    }

    rt
}

pub fn get_item_drop(raws: &RawMaster, rng: &mut RandomNumberGenerator, table_name: &str) -> Option<String> {
    if raws.loot_index.contains_key(table_name) {
        let mut rt = RandomTable::new();
        let available_options = &raws.raws.loot_tables[raws.loot_index[table_name]];
        for item in available_options.drops.iter() {
            rt = rt.add(item.name.clone(), item.weight);
        }
        return rt.roll(rng);
    }
    None
}

pub fn faction_reaction(my_faction: &str, their_faction: &str, raws: &RawMaster) -> Reaction {
    if raws.faction_index.contains_key(my_faction) {
        let mf = &raws.faction_index[my_faction];
        if mf.contains_key(their_faction) {
            return mf[their_faction];
        } else if mf.contains_key("default") {
            return mf["default"];
        }
    }
    Reaction::Ignore
}

