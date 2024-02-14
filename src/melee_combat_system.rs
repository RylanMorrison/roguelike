use specs::prelude::*;
use super::{CombatStats, WantsToMelee, Name, SufferDamage, gamelog::GameLog, 
    Equipped, DefenceBonus, MeleePowerBonus, ParticleBuilder, Position, 
    HungerState, HungerClock};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenceBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_melee, 
            names, combat_stats, mut suffer_damage,
            melee_bonuses, defence_bonuses, equipped,
            mut particle_builder, positions, hunger_clocks) = data;

        for (entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in (&entities, &melee_bonuses, &equipped).join() {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                    let hunger = hunger_clocks.get(entity);
                    if let Some(hunger) = hunger {
                        if hunger.state == HungerState::WellFed {
                            offensive_bonus += 1;
                        }
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();
                    let mut defensive_bonus = 0;

                    for (_item_entity, defence_bonus, equipped_by) in (&entities, &defence_bonuses, &equipped).join() {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defence_bonus.defence;
                        }
                    }

                    let damage = i32::max(0, (stats.power + offensive_bonus) - (target_stats.defence + defensive_bonus));
                    if damage == 0 {
                        log.entries.push(format!("{} is unable to hurt {}.", &name.name, &target_name.name));
                    } else {
                        log.entries.push(format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut suffer_damage, wants_melee.target, damage);
                        let pos = positions.get(wants_melee.target);
                        if let Some(pos) = pos {
                            particle_builder.add_request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                        }
                    }
                }
            }
        }
        wants_melee.clear();
    }
}
