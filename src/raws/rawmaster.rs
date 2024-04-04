use std::collections::{HashMap, HashSet};
use specs::prelude::*;
use rltk::{RandomNumberGenerator, RGB};
use crate::components::*;
use super::{Raws, Reaction, RenderableData, SpawnTableEntry};
use crate::random_table::RandomTable;
use crate::{attr_bonus, npc_hp, mana_at_level};
use regex::Regex;
use specs::saveload::{MarkedBuilder, SimpleMarker};

/// Parse a dice string into its values. TODO move to gamesystem
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
    item_set_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
    spell_index: HashMap<String, usize>,
    loot_index: HashMap<String, usize>,
    faction_index: HashMap<String, HashMap<String, Reaction>>
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster{
            raws: Raws { 
                items: Vec::new(),
                item_sets: Vec::new(),
                item_class_colours: HashMap::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spells: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
                faction_table: Vec::new()
            },
            item_index: HashMap::new(),
            item_set_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
            spell_index: HashMap::new(),
            loot_index: HashMap::new(),
            faction_index: HashMap::new()
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        let mut used_names: HashSet<String> = HashSet::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!("WARNING - duplicate item name in raws [{}]", item.name));
            }
            if !self.raws.item_class_colours.contains_key(&item.class) {
                rltk::console::log(format!("WARNING - unknown item class in raws [{}]", &item.class));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i, item_set) in self.raws.item_sets.iter().enumerate() {
            if used_names.contains(&item_set.name) {
                rltk::console::log(format!("WARNING - duplicate item set name in raws [{}]", &item_set.name));
            }
            self.item_set_index.insert(item_set.name.clone(), i);
            used_names.insert(item_set.name.clone());
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!("WARNING - duplicate mob name in raws [{}]", mob.name));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!("WARNING - duplicate prop name in raws [{}]", prop.name));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }
        for (i, spell) in self.raws.spells.iter().enumerate() {
            self.spell_index.insert(spell.name.clone(), i);
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
            rltk::console::log(format!("WARNING - Unknown weapon slot type [{}]", slot));
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
            rltk::console::log(format!("WARNING - Unknown wearable slot type [{}]", slot));
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

fn get_renderable_component(renderable: &RenderableData, fg_override: Option<&String>) -> Renderable {
    Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: {
            if let Some(override_code) = fg_override  {
                RGB::from_hex(override_code).expect("Invalid RGB")
            } else if let Some(renderable_code) = &renderable.fg {
                RGB::from_hex(renderable_code).expect("Invalid RGB")
            } else {
                rltk::console::log("WARNING No foreground colour provided for renderable");
                RGB::named(rltk::WHITE)
            }
        },
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

macro_rules! apply_effects {
    ( $effects:expr, $eb:expr ) => {
        for effect in $effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "healing" => $eb = $eb.with(Healing{ heal_amount: effect.1.parse::<i32>().unwrap() }),
                "mana" => $eb = $eb.with(RestoresMana{ mana_amount: effect.1.parse::<i32>().unwrap() }),
                "ranged" => $eb = $eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }),
                "damage" => $eb = $eb.with(Damage{ damage: effect.1.to_string() }),
                "area_of_effect" => $eb = $eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }),
                "confusion" => {
                    $eb = $eb.with(Confusion{});
                    $eb = $eb.with(Duration{ turns: effect.1.parse::<i32>().unwrap() });
                }
                "magic_mapping" => $eb = $eb.with(MagicMapping{}),
                "town_portal" => $eb = $eb.with(TownPortal{}),
                "food" => $eb = $eb.with(Food{}),
                "single_activation" => $eb = $eb.with(SingleActivation{}),
                "particle_line" => $eb = $eb.with(parse_particle_line(&effect.1)),
                "particle" => $eb = $eb.with(parse_particle(&effect.1)),
                "duration" => $eb = $eb.with(Duration{ turns: effect.1.parse::<i32>().unwrap() }),
                "teach_spell" => $eb = $eb.with(TeachesSpell{ spell: effect.1.to_string() }),
                "slow" => $eb = $eb.with(Slow{ initiative_penalty: effect.1.parse::<f32>().unwrap() }),
                "damage_over_time" => $eb = $eb.with(DamageOverTime{ damage: effect.1.parse::<i32>().unwrap() }),
                _ => rltk::console::log(format!("WARNING - Effect not implemented: {}", effect_name))
            }
        }
    };
}

pub fn spawn_named_item(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    let item_template = &raws.raws.items[raws.item_index[key]];
    let item_class_colours = &raws.raws.item_class_colours;
    let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

    // spawn in the specified location
    eb = spawn_position(pos, eb, key, raws);

    // renderable
    if let Some(renderable) = &item_template.renderable {
        eb = eb.with(get_renderable_component(renderable, item_class_colours.get(&item_template.class)));
    }

    eb = eb.with(Name{ name: item_template.name.clone() });
    eb = eb.with(Item{
        initiative_penalty: item_template.initiative_penalty.unwrap_or(0.0),
        weight_lbs: item_template.weight_lbs.unwrap_or(0.0),
        base_value: item_template.base_value.unwrap_or(0),
        class: {
            let class_name = item_template.class.as_str();
            match class_name {
                "common" => ItemClass::Common,
                "rare" => ItemClass::Rare,
                "legendary" => ItemClass::Legendary,
                "set" => ItemClass::Set,
                "unique" => ItemClass::Unique,
                _ => {
                    rltk::console::log(format!("WARNING - Unknown item class: {}", class_name));
                    ItemClass::Common
                }
            }
        }
    });

    // consumables
    if let Some(consumable) = &item_template.consumable {
        let max_charges = consumable.charges.unwrap_or(1);
        eb = eb.with(Consumable{ max_charges, charges: max_charges });
        apply_effects!(consumable.effects, eb);
    }

    // equipment
    if let Some(weapon) = &item_template.weapon {
        eb = eb.with(Equippable{ slot: string_to_weapon_slot(&weapon.slot) });
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
            hit_bonus: weapon.hit_bonus,
            proc_chance: weapon.proc_chance,
            proc_target: weapon.proc_target.clone()
        };
        eb = eb.with(wpn);
        if let Some(proc_effects) = &weapon.proc_effects {
            apply_effects!(proc_effects, eb);
        }
    }
    if let Some(wearable) = &item_template.wearable {
        let slot = string_to_wearable_slot(&wearable.slot);
        eb = eb.with(Equippable{ slot });
        eb = eb.with(Wearable{ slot, armour_class: wearable.armour_class });
    }

    // attribute bonuses
    if let Some(bonus) = &item_template.attribute_bonuses {
        eb = eb.with(AttributeBonus{
            strength: bonus.strength,
            dexterity: bonus.dexterity,
            constitution: bonus.constitution,
            intelligence: bonus.intelligence
        });
    }

    // skill bonuses
    if let Some(bonus) = &item_template.skill_bonuses {
        eb = eb.with(SkillBonus{
            melee: bonus.melee,
            defence: bonus.defence,
            magic: bonus.magic
        });
    }

    // item sets
    if let Some(set_name) = &item_template.set_name {
        eb = eb.with(PartOfSet{ set_name: set_name.to_string() });
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
        eb = eb.with(get_renderable_component(renderable, None));
        if renderable.x_size.is_some() || renderable.y_size.is_some() {
            eb = eb.with(TileSize{ x: renderable.x_size.unwrap_or(1), y: renderable.y_size.unwrap_or(1) });
        }
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
        mana: Pool{ current: mob_mana, max: mob_mana },
        total_weight: 0.0,
        total_initiative_penalty: 0.0,
        gold: 
        if let Some(gold) = &mob_template.gold {
            let mut rng = RandomNumberGenerator::new();
            let (n, d, b) = parse_dice_string(&gold);
            rng.roll_dice(n, d) + b
        } else {
            0
        },
        total_armour_class: 10, // only used by player for now
        base_damage: "1d4".to_string(), // only used by player for now
        god_mode: false
    };
    eb = eb.with(pools);

    // skills
    let mut skills = Skills::default();
    if let Some(mobskills) = &mob_template.skills {
        for sk in mobskills.iter() {
            match sk.0.as_str() {
                "melee" => { skills.melee.base = *sk.1; }
                "defence" => { skills.defence.base = *sk.1; }
                "magic" => { skills.magic.base = *sk.1; }
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

    // special abilities
    if let Some(ability_list) = &mob_template.abilities {
        let mut sabilities = SpecialAbilities{ abilities: Vec::new() };
        for ability in ability_list.iter() {
            sabilities.abilities.push(
                SpecialAbility{
                    spell: ability.spell.clone(),
                    chance: ability.chance,
                    range: ability.range,
                    min_range: ability.min_range
                }
            );
        }
        eb = eb.with(sabilities);
    }

    // visibility
    eb = eb.with(Viewshed{ 
        visible_tiles: Vec::new(),
        range: mob_template.vision_range,
        dirty: true
    });

    eb = eb.with(EquipmentChanged{});

    // loot
    if let Some(loot) = &mob_template.loot_table {
        eb = eb.with(LootTable{ table_name: loot.clone() });
    }

    if let Some(vendor) = &mob_template.vendor {
        eb = eb.with(Vendor{ categories: vendor.clone() });
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
        eb = eb.with(get_renderable_component(renderable, None));
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
    if let Some(entry_trigger) = &prop_template.entry_trigger {
        eb = eb.with(EntryTrigger{});
        apply_effects!(entry_trigger.effects, eb);
    }
    if let Some(light) = &prop_template.light {
        eb = eb.with(LightSource{ range: light.range, colour: RGB::from_hex(&light.colour).expect("Bad colour") });
        eb = eb.with(Viewshed{ range: light.range, dirty: true, visible_tiles: Vec::new() });
    }

    Some(eb.build())
}

pub fn spawn_named_spell(raws: &RawMaster, ecs: &mut World, key: &str) -> Option<Entity> {
    if raws.spell_index.contains_key(key) {
        let spell_template = &raws.raws.spells[raws.spell_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = eb.with(Spell{ mana_cost: spell_template.mana_cost });
        eb = eb.with(Name{ name: spell_template.name.clone() });
        apply_effects!(spell_template.effects, eb);

        return Some(eb.build());
    }
    None
}

pub fn store_all_item_sets(ecs: &mut World) {
    let raws = &super::RAWS.lock().unwrap();
    for item_set in raws.raws.item_sets.iter() {
        store_named_item_set(raws, ecs, &item_set.name);
    }
}

pub fn store_named_item_set(raws: &RawMaster, ecs: &mut World, key: &str) {
    if raws.item_set_index.contains_key(key) {
        let item_set_template = &raws.raws.item_sets[raws.item_set_index[key]];

        let mut item_sets = ecs.fetch_mut::<ItemSets>();
        let mut set_bonuses: HashMap<i32, ItemSetBonus> = HashMap::new();
        for set_bonus in item_set_template.set_bonuses.iter() {
            let required_pieces = set_bonus.required_pieces;
            let mut attribute_bonus: Option<AttributeBonus> = None;
            if let Some(attr_bonus) = &set_bonus.attribute_bonuses {
                attribute_bonus = Some(AttributeBonus{
                    strength: attr_bonus.strength,
                    dexterity: attr_bonus.dexterity,
                    constitution: attr_bonus.constitution,
                    intelligence: attr_bonus.intelligence
                });
            }
            let mut skill_bonus: Option<SkillBonus> = None;
            if let Some(sk_bonus) = &set_bonus.skill_bonuses {
                skill_bonus = Some(SkillBonus{
                    melee: sk_bonus.melee,
                    defence: sk_bonus.defence,
                    magic: sk_bonus.magic
                });
            }
            set_bonuses.insert(required_pieces, ItemSetBonus{ attribute_bonus, skill_bonus });
        }
        let item_set = ItemSet{ total_pieces: item_set_template.total_pieces, set_bonuses };
        item_sets.item_sets.insert(item_set_template.name.clone(), item_set);
    }
}

pub fn spawn_all_spells(ecs: &mut World) {
    let raws = &super::RAWS.lock().unwrap();
    for spell in raws.raws.spells.iter() {
        spawn_named_spell(raws, ecs, &spell.name);
    }
}

pub fn find_spell_entity(ecs: &World, name: &str) -> Option<Entity> {
    let names: Storage<'_, Name, specs::shred::Fetch<'_, specs::storage::MaskedStorage<Name>>> = ecs.read_storage::<Name>();
    let spells = ecs.read_storage::<Spell>();
    let entities = ecs.entities();

    for (entity, sname, _spell) in (&entities, &names, &spells).join() {
        if name == sname.name {
            return Some(entity);
        }
    }
    None
}

pub fn find_spell_entity_by_name(name: &str, names: &ReadStorage::<Name>, spells: &ReadStorage::<Spell>, entities: &Entities) -> Option<Entity> {
    for (entity, sname, _spell) in (entities, names, spells).join() {
        if name == sname.name {
            return Some(entity);
        }
    }
    None
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
            weight += depth - e.min_depth;
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

pub fn get_vendor_items(categories: &[String], raws: &RawMaster) -> Vec<(String, i32, RGB)> {
    let mut result: Vec<(String, i32, RGB)> = Vec::new();
    for item in raws.raws.items.iter() {
        if let Some(cat) = &item.vendor_category {
            if categories.contains(cat) && item.base_value.is_some() {
                result.push((
                    item.name.clone(),
                    item.base_value.unwrap(),
                    get_item_class_colour(item.class.as_str(), raws)
                ));
            }
        }
    }
    result.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
    result
}

pub fn get_item_colour(item: &Item, raws: &RawMaster) -> RGB {
    let class_string = match item.class {
        ItemClass::Common => "common",
        ItemClass::Rare => "rare",
        ItemClass::Legendary => "legendary",
        ItemClass::Set => "set",
        ItemClass::Unique => "unique"
    };
    get_item_class_colour(class_string, raws)
}

fn get_item_class_colour(class_string: &str, raws: &RawMaster) -> RGB {
    let colour = raws.raws.item_class_colours.get(class_string);
    RGB::from_hex(colour.unwrap()).expect("Invalid RGB")
}

fn parse_particle_line(token_string: &str) -> SpawnParticleLine {
    let tokens: Vec<_> = token_string.split(';').collect();
    SpawnParticleLine{
        glyph: rltk::to_cp437(tokens[0].chars().next().unwrap()),
        colour: RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap()
    }
}

fn parse_particle(token_string: &str) -> SpawnParticleBurst {
    let tokens: Vec<_> = token_string.split(';').collect();
    SpawnParticleBurst{
        glyph: rltk::to_cp437(tokens[0].chars().next().unwrap()),
        colour: RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap()
    }
}


