use specs::prelude::*;
use super::{Attributes, Skills, WantsToMelee, Name, SufferDamage, gamelog::GameLog,
    particle_system::ParticleBuilder, Position, HungerClock, HungerState, Pools, skill_bonus,
    Skill, Equipped, MeleeWeapon, EquipmentSlot, WeaponAttribute, Wearable, NaturalAttackDefence};
use rltk::RandomNumberGenerator;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        ReadStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, NaturalAttackDefence>,
        ReadExpect<'a, Entity>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut gamelog, mut wants_melee, names, attributes, skills,
            pools, mut inflict_damage, mut particle_builder, positions, hunger_clock, 
            mut rng, equipped_items, melee_weapons, wearables, natural, player_entity) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in (&entities, &wants_melee, &names, &attributes, &skills, &pools).join() {
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            if attacker_pools.hit_points.current <= 0 || target_pools.hit_points.current <= 0 {
                continue; // skip if attacker or defender are dead
            }

            // default to unarmed
            let mut weapon_info = MeleeWeapon {
                attribute: crate::WeaponAttribute::Strength,
                hit_bonus: 0,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0
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
            for (wielded, melee) in (&equipped_items, &melee_weapons).join() {
                if wielded.owner == entity && wielded.slot == EquipmentSlot::MainHand {
                    weapon_info = melee.clone();
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
            let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
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
            let mut armour_item_bonus_f = 0.0;
            for (wielded, armour) in (&equipped_items, &wearables).join() {
                if wielded.owner == wants_melee.target {
                    armour_item_bonus_f += armour.armour_class;
                }
            }
            
            // calculate armour class of defender
            let armour_dexterity_bonus = target_attributes.dexterity.bonus;
            let armour_skill_bonus = skill_bonus(Skill::Defence, &*target_skills);
            let armour_item_bonus = armour_item_bonus_f as i32;
            let armour_class = base_armour_class + armour_dexterity_bonus + armour_skill_bonus
                + armour_item_bonus;

            if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armour_class) {
                // TODO: critical hits
                // hit
                let base_damage = rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                let attr_damage_bonus = if weapon_info.attribute == WeaponAttribute::Strength {
                    attacker_attributes.strength.bonus
                } else {
                    attacker_attributes.dexterity.bonus
                };
                let skill_damage_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_damage_bonus = weapon_info.damage_bonus;
                
                let damage = i32::max(0, base_damage + attr_damage_bonus + skill_damage_bonus
                    + weapon_damage_bonus);
                SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage, entity == *player_entity);
                
                // indicate that damage was done
                gamelog.entries.push(format!("{} hits {}, dealing {} damage.", &name.name, &target_name.name, damage));
                if let Some(pos) = positions.get(wants_melee.target) {
                    particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                }
            } else if natural_roll == 1 {
                // critical miss
                gamelog.entries.push(format!("{} completely misses {}!", name.name, target_name.name));
                if let Some(pos) = positions.get(wants_melee.target) {
                    particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::BLUE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                }
            } else {
                // miss
                gamelog.entries.push(format!("{} evades {}'s attack.", target_name.name, name.name));
                if let Some(pos) = positions.get(wants_melee.target) {
                    particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::CYAN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                }
            }
        }
        wants_melee.clear();
    }
}
