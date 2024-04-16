use specs::prelude::*;
use rltk::prelude::*;
use super::{green, white, black, yellow};
use crate::{State, Attributes, Skills, PendingLevelUp};

pub enum LevelUpMenuResult {
    NoResponse,
    AssignedAttribute,
    AssignedSkill,
    Done
}

fn draw_level_choice(draw_batch: &mut DrawBatch, y: i32, name: &str, value: &i32, selection: &str, selected: bool) {
    let colour = if selected { green() } else { white() };
    let mod_value = if selected { value + 1 } else { *value };

    draw_batch.print_color(Point::new(22, y), name, ColorPair::new(colour, black()));
    draw_batch.print_color(Point::new(39, y), mod_value, ColorPair::new(colour, black()));
    draw_batch.print_color(Point::new(47, y), selection, ColorPair::new(yellow(), black()));
}

pub fn show_levelup_menu(gs: &mut State, ctx: &mut Rltk, attribute_points: i32, skill_points: i32) -> LevelUpMenuResult {
    let player = gs.ecs.fetch::<Entity>();
    let attributes = gs.ecs.read_storage::<Attributes>();
    let player_attributes = attributes.get(*player).unwrap();
    let skills = gs.ecs.read_storage::<Skills>();
    let player_skills = skills.get(*player).unwrap();
    let mut pending_level_ups = gs.ecs.write_storage::<PendingLevelUp>();
    let level_up = pending_level_ups.get_mut(*player).unwrap();
    let mut draw_batch = DrawBatch::new();
    
    draw_batch.draw_box(Rect::with_size(12, 25, 51, 20), ColorPair::new(white(), black()));
    draw_batch.print_color(Point::new(15, 25), "Level Up", ColorPair::new(yellow(), black()));
    draw_batch.print_color(Point::new(15, 27), "Pick one attribute and two skills to improve", ColorPair::new(yellow(), black()));

    draw_level_choice(&mut draw_batch, 29, "Strength", &player_attributes.strength.base, "(a)", level_up.attributes.strength.base > player_attributes.strength.base);
    draw_level_choice(&mut draw_batch, 31, "Dexterity", &player_attributes.dexterity.base, "(b)", level_up.attributes.dexterity.base > player_attributes.dexterity.base);
    draw_level_choice(&mut draw_batch, 33, "Constitution", &player_attributes.constitution.base, "(c)", level_up.attributes.constitution.base > player_attributes.constitution.base);
    draw_level_choice(&mut draw_batch, 35, "Intelligence", &player_attributes.intelligence.base, "(d)", level_up.attributes.intelligence.base > player_attributes.intelligence.base);

    draw_level_choice(&mut draw_batch, 38, "Melee", &player_skills.melee.base, "(e)", level_up.skills.melee.base > player_skills.melee.base);
    draw_level_choice(&mut draw_batch, 40, "Defence", &player_skills.defence.base, "(f)", level_up.skills.defence.base > player_skills.defence.base);
    draw_level_choice(&mut draw_batch, 42, "Magic", &player_skills.magic.base, "(g)", level_up.skills.magic.base > player_skills.magic.base);
    
    draw_batch.print_color(Point::new(15, 45), "ENTER when done", ColorPair::new(yellow(), black()));

    draw_batch.submit(1000).expect("Draw batch submission failed");

    match ctx.key {
        None => {},
        Some(key) => {
            match key {
                VirtualKeyCode::A => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.strength.base = player_attributes.strength.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                },
                VirtualKeyCode::B => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.dexterity.base = player_attributes.dexterity.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                },
                VirtualKeyCode::C => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.constitution.base = player_attributes.constitution.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                }
                VirtualKeyCode::D => {
                    if attribute_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.attributes.intelligence.base = player_attributes.intelligence.base + 1;
                        return LevelUpMenuResult::AssignedAttribute;
                    }
                },
                VirtualKeyCode::E => {
                    if level_up.skills.melee.base > player_skills.melee.base || skill_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.skills.melee.base = player_skills.melee.base + 1;
                        return LevelUpMenuResult::AssignedSkill;
                    }
                },
                VirtualKeyCode::F => {
                    if level_up.skills.defence.base > player_skills.defence.base || skill_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.skills.defence.base = player_skills.defence.base + 1;
                        return LevelUpMenuResult::AssignedSkill;
                    }
                },
                VirtualKeyCode::G => {
                    if level_up.skills.magic.base > player_skills.magic.base || skill_points == 0 {
                        return LevelUpMenuResult::NoResponse;
                    } else {
                        level_up.skills.magic.base = player_skills.magic.base + 1;
                        return LevelUpMenuResult::AssignedSkill;
                    }
                },
                VirtualKeyCode::Return => {
                    if attribute_points == 0 && skill_points == 0 {
                        return LevelUpMenuResult::Done;
                    } else {
                        return LevelUpMenuResult::NoResponse;
                    }
                }
                _ => {}
            }
        }
    }

    LevelUpMenuResult::NoResponse
}
