use specs::prelude::*;
use super::{Attributes, Skills, WantsToMelee, Name, Position,
    HungerClock, HungerState, Pools, Equipped, Weapon, AreaOfEffect,
    EquipmentSlot, WeaponAttribute, Wearable, NaturalAttackDefence, Map};
use super::effects::{add_effect, aoe_tiles, EffectType, Targets};
use rltk::{RandomNumberGenerator, RGB, Point};
use crate::gamelog;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, Weapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, NaturalAttackDefence>,
        ReadStorage<'a, AreaOfEffect>,
        WriteExpect<'a, Map>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_melees, names, attributes, 
            skills, pools, positions, hunger_clock, mut rng, 
            equipped_items, melee_weapons, wearables, natural,
            area_of_effect, map) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in (&entities, &wants_melees, &names, &attributes, &skills, &pools).join() {
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            if attacker_pools.hit_points.current <= 0 || target_pools.hit_points.current <= 0 {
                continue; // skip if attacker or defender are dead
            }

            // default to unarmed
            let mut weapon_info = Weapon {
                range: None,
                attribute: crate::WeaponAttribute::Strength,
                hit_bonus: 0,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0,
                proc_chance: None,
                proc_target: None
            };

            // natural attack ability of attacker
            if let Some(natural) = natural.get(entity) {
                if !natural.attacks.is_empty() {
                    let attack_index = if natural.attacks.len() == 1 {
                        0
                    } else {
                        rng.roll_dice(1, natural.attacks.len() as i32) as usize - 1
                    };
                    weapon_info.hit_bonus = natural.attacks[attack_index].hit_bonus;
                    weapon_info.damage_n_dice = natural.attacks[attack_index].damage_n_dice;
                    weapon_info.damage_die_type = natural.attacks[attack_index].damage_die_type;
                    weapon_info.damage_bonus = natural.attacks[attack_index].damage_bonus;
                }
            }

            // weapon attack ability of attacker
            let mut weapon_entity: Option<Entity> = None;
            for (weapon, wielded, melee) in (&entities, &equipped_items, &melee_weapons).join() {
                if wielded.owner == entity && (wielded.slot == EquipmentSlot::MainHand || wielded.slot == EquipmentSlot::TwoHanded) {
                    weapon_info = melee.clone();
                    weapon_entity = Some(weapon);
                }
            }

            // calculate attacker hit roll
            let target_name = names.get(wants_melee.target).unwrap();
            let natural_roll = rng.roll_dice(1, 20);
            let attribute_hit_bonus = if weapon_info.attribute == WeaponAttribute::Strength {
                attacker_attributes.strength.bonus
            } else {
                attacker_attributes.dexterity.bonus
            };
            let skill_hit_bonus = &attacker_skills.melee.bonus();
            let weapon_hit_bonus = weapon_info.hit_bonus;
            let mut status_hit_bonus = 0;
            if let Some(hc) = hunger_clock.get(entity) {
                if hc.state == HungerState::WellFed {
                    status_hit_bonus += 1;
                }
            }
            let modified_hit_roll = natural_roll + attribute_hit_bonus + skill_hit_bonus
                + weapon_hit_bonus + status_hit_bonus;
            
            // natural defence ability of defender
            let base_armour_class = match natural.get(wants_melee.target) {
                None => 10,
                Some(nature) => nature.armour_class.unwrap_or(10)
            };

            // defence from any armour defender is wearing
            let mut armour_item_bonus = 0.0;
            for (wielded, armour) in (&equipped_items, &wearables).join() {
                if wielded.owner == wants_melee.target {
                    armour_item_bonus += armour.armour_class;
                }
            }
            
            // calculate armour class of defender
            let armour_dexterity_bonus = target_attributes.dexterity.bonus;
            // each point in defence gives 0.1 armour class
            // TODO make this scale equipped armour instead of a flat bonus
            let armour_skill_bonus = target_skills.defence.bonus() as f32 * 0.1;
            let total_armour_bonus = (armour_item_bonus + armour_skill_bonus) as i32;
            let armour_class = base_armour_class + armour_dexterity_bonus + total_armour_bonus;

            if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armour_class) {
                // TODO: critical hits
                // hit
                let base_damage = rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                let attr_damage_bonus = if weapon_info.attribute == WeaponAttribute::Strength {
                    attacker_attributes.strength.bonus
                } else {
                    attacker_attributes.dexterity.bonus
                };
                let skill_damage_bonus = &attacker_skills.melee.bonus();
                let weapon_damage_bonus = weapon_info.damage_bonus;
                
                let damage = i32::max(0, base_damage + attr_damage_bonus + skill_damage_bonus
                    + weapon_damage_bonus);
                add_effect(
                    Some(entity),
                    EffectType::Damage{ amount: damage },
                    Targets::Single{ target: wants_melee.target }
                );
                
                // indicate that damage was done
                gamelog::Logger::new()
                    .character_name(&name.name)
                    .append("hits")
                    .character_name(&target_name.name)
                    .append("dealing")
                    .damage(damage)
                    .append("damage.")
                    .log();

                if positions.get(wants_melee.target).is_some() {
                    add_effect(
                        None, 
                        EffectType::Particle {
                            glyph: rltk::to_cp437('‼'),
                            fg: RGB::named(rltk::ORANGE),
                            bg: RGB::named(rltk::BLACK),
                            lifespan: 200.0
                        },
                        Targets::Single{ target: wants_melee.target }
                    );
                }

                // proc effects
                if let Some(chance) = &weapon_info.proc_chance {
                    if rng.roll_dice(1, 100) <= (chance * 100.0) as i32 {
                        let mut effect_target = Targets::Single { target: wants_melee.target };
                        if weapon_info.proc_target.unwrap() == "Self" {
                            effect_target = Targets::Single{ target: entity }
                        } else if weapon_entity.is_some() {
                            // check for area effects
                            if let Some(aoe) = area_of_effect.get(weapon_entity.unwrap()) {
                                if let Some(pos) = positions.get(wants_melee.target) {
                                    // TODO remove effect creator from target list
                                    effect_target = Targets::Tiles{ tiles: aoe_tiles(&*map, Point{ x: pos.x, y: pos.y }, aoe.radius) }
                                }
                            }
                        }
                        add_effect(
                            Some(entity),
                            EffectType::ItemUse{ item: weapon_entity.unwrap() },
                            effect_target
                        );
                    }
                }
            } else if natural_roll == 1 {
                // critical miss
                gamelog::Logger::new()
                    .character_name(&name.name)
                    .append("completely misses")
                    .character_name(&target_name.name)
                    .append("!")
                    .log();
                if positions.get(wants_melee.target).is_some() {
                    add_effect(
                        None, 
                        EffectType::Particle {
                            glyph: rltk::to_cp437('‼'),
                            fg: RGB::named(rltk::BLUE),
                            bg: RGB::named(rltk::BLACK),
                            lifespan: 200.0
                        },
                        Targets::Single{ target: wants_melee.target }
                    );
                }
            } else {
                // miss
                gamelog::Logger::new()
                    .character_name(&target_name.name)
                    .append("evades attack from")
                    .character_name(&name.name)
                    .log();
                if positions.get(wants_melee.target).is_some() {
                    add_effect(
                        None, 
                        EffectType::Particle {
                            glyph: rltk::to_cp437('‼'),
                            fg: RGB::named(rltk::CYAN),
                            bg: RGB::named(rltk::BLACK),
                            lifespan: 200.0
                        },
                        Targets::Single{ target: wants_melee.target }
                    );
                }
            }
        }
        wants_melees.clear();
    }
}
