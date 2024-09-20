use specs::prelude::*;
use rltk::prelude::*;
use super::{green, white, black, magenta, yellow, gold};
use crate::{CharacterClass, ClassPassive, ClassPassiveLevel, WantsToLevelUp, State, AttributeBonus, SkillBonus};
use std::collections::{BTreeMap, HashMap};

pub enum LevelUpMenuResult {
    NoResponse,
    SelectedPassive,
    DeselectedPassive,
    Done
}

pub fn show_levelup_menu(gs: &mut State, ctx: &mut Rltk) -> LevelUpMenuResult {
    let player = gs.ecs.fetch::<Entity>();
    let mut level_ups = gs.ecs.write_storage::<WantsToLevelUp>();
    let character_classes = gs.ecs.read_storage::<CharacterClass>();
    let player_class = character_classes.get(*player).unwrap();
    let level_up = level_ups.get_mut(*player).unwrap();
    let mut draw_batch = DrawBatch::new();
    
    draw_batch.draw_box(Rect::with_size(0, 0, 99, 79), ColorPair::new(white(), black()));
    draw_batch.print_color(Point::new(2, 2), "Level Up", ColorPair::new(yellow(), black()));
    draw_batch.print_color(
        Point::new(2, 4),
        "Pick a passive ability to learn/improve",
        ColorPair::new(yellow(), black())
    );

    let mut y = 8;
    let mut j = 0;
    let mut passive_selections: HashMap<String, String> = HashMap::new();
    for (name, passive) in player_class.passives.iter() {
        let selection = rltk::to_char(97 + j);
        passive_selections.insert(selection.to_string(), name.clone());

        draw_passive_choice(&mut draw_batch, &mut y, passive, format!("({})", selection), passive_selected(level_up, passive));
        j += 1;
    }
    draw_batch.print_color(Point::new(82, 77), "ENTER when done", ColorPair::new(yellow(), black()));

    draw_batch.submit(5000).expect("Draw batch submission failed");

    match ctx.key {
        None => {},
        Some(key) => {
            match key {
                VirtualKeyCode::A => {
                    return handle_selection("a", passive_selections, level_up, player_class);
                },
                VirtualKeyCode::B => {
                    return handle_selection("b", passive_selections, level_up, player_class);
                },
                VirtualKeyCode::C => {
                    return handle_selection("c", passive_selections, level_up, player_class);
                }
                VirtualKeyCode::D => {
                    if passive_selections.len() >= 4 {
                        return handle_selection("d", passive_selections, level_up, player_class);
                    }
                }
                VirtualKeyCode::Return => {
                    let mut selection_made = false;
                    for (_name, passive) in player_class.passives.iter() {
                        if passive_selected(level_up, passive) { selection_made = true; }
                    }

                    if selection_made {
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

fn draw_passive_choice(draw_batch: &mut DrawBatch, y: &mut i32, passive: &ClassPassive, selection: String, selected: bool) {
    let colour = if selected { green() } else if passive.is_max_level() { magenta() } else { white() };
    let display_level_int = if selected { passive.current_level + 1 } else { passive.current_level };

    draw_batch.print_color(Point::new(4, *y), selection, ColorPair::new(yellow(), black()));
    draw_batch.print_color(Point::new(8, *y), passive.name.clone(), ColorPair::new(gold(), black()));
    *y += 2;
    draw_batch.print_color(Point::new(4, *y), passive.description.clone(), ColorPair::new(yellow(), black()));
    *y += 2;
    draw_batch.print_color(Point::new(4, *y), format!("Level: {}",
        if passive.is_max_level() { "Max".to_string() } else { display_level_int.to_string() }),
        ColorPair::new(colour, black())
    );
    *y += 2;

    let level_to_show: Option<ClassPassiveLevel> = cumulative_level(passive, display_level_int);
    if let Some(display_level) = level_to_show {
        if let Some(attribute_bonus) = &display_level.attribute_bonus {
            draw_batch.print_color(Point::new(4, *y), "Attribute bonuses:", ColorPair::new(colour, black()));
            *y += 1;
            if let Some(strength_bonus) = &attribute_bonus.strength {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Strength ({})", strength_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            if let Some(dexterity_bonus) = &attribute_bonus.dexterity {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Dexterity ({})", dexterity_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            if let Some(constitution_bonus) = &attribute_bonus.constitution {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Constitution ({})", constitution_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            if let Some(intelligence_bonus) = &attribute_bonus.intelligence {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Intelligence ({})", intelligence_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            *y += 1;
        }
        if let Some(skill_bonus) = &display_level.skill_bonus {
            draw_batch.print_color(Point::new(4, *y), "Skill bonuses:", ColorPair::new(colour, black()));
            *y += 1;
            if let Some(melee_bonus) = &skill_bonus.melee {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Melee ({})", melee_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            if let Some(defence_bonus) = &skill_bonus.defence {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Defence ({})", defence_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            if let Some(ranged_bonus) = &skill_bonus.ranged {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Ranged ({})", ranged_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            if let Some(magic_bonus) = &skill_bonus.magic {
                draw_batch.print_color(
                    Point::new(4, *y),
                    format!("Magic ({})", magic_bonus),
                    ColorPair::new(colour, black())
                ); *y += 1;
            }
            *y += 1;
        }
        if let Some(learn_ability) = &display_level.learn_ability {
            draw_batch.print_color(
                Point::new(4, *y),
                format!("Learn ability: {}", learn_ability),
                ColorPair::new(colour, black())
            ); *y += 1;
        }
        if let Some(level_ability) = &display_level.level_ability {
            draw_batch.print_color(
                Point::new(4, *y),
                format!("Improve ability: {}", level_ability),
                ColorPair::new(colour, black())
            ); *y += 1;
        }
    }
    *y += 2;
}

fn cumulative_level(passive: &ClassPassive, display_level: i32) -> Option<ClassPassiveLevel> {
    if display_level == 0 { return None }
    if display_level == 1 { return Some(passive.levels[&1].clone()); }

    let mut attribute_bonus: Option<AttributeBonus> = passive.levels[&1].attribute_bonus.clone();
    let mut skill_bonus: Option<SkillBonus> = passive.levels[&1].skill_bonus.clone();
    // TODO: combine learn/level abilities
    let learn_ability: Option<String> = passive.levels[&display_level].learn_ability.clone();
    let level_ability: Option<String> = passive.levels[&display_level].level_ability.clone();

    for i in 2..=display_level {
        if attribute_bonus.is_some() {
            attribute_bonus.as_mut().unwrap().combine(passive.levels[&i].attribute_bonus.as_ref());
        }
        if skill_bonus.is_some() {
            skill_bonus.as_mut().unwrap().combine(passive.levels[&i].skill_bonus.as_ref());
        }
    }

    Some(ClassPassiveLevel { attribute_bonus, skill_bonus, learn_ability, level_ability })
}

fn passive_selected(level_up: &mut WantsToLevelUp, passive: &ClassPassive) -> bool {
    for (name, pass) in level_up.passives.iter() {
        if name.as_str() == passive.name.as_str() {
            return pass.current_level != passive.current_level
        }
    }
    false
}

fn get_selected_passive<'a>(level_up: &'a mut WantsToLevelUp, current_passives: &BTreeMap<String, ClassPassive>) -> Option<&'a mut ClassPassive> {
    for (name, passive) in level_up.passives.iter_mut() {
        if current_passives[name].current_level != passive.current_level { return Some(passive); }
    }
    None
}

fn handle_selection(selection: &str, passive_selections: HashMap<String, String>, level_up: &mut WantsToLevelUp, player_class: &CharacterClass) -> LevelUpMenuResult {
    let passive_name = &passive_selections[selection];
    let passive = &player_class.passives[passive_name];

    if passive_selected(level_up, &passive) {
        let level_up_passive = level_up.passives.get_mut(passive_name).unwrap();
        level_up_passive.current_level -= 1;

        return LevelUpMenuResult::DeselectedPassive;
    } else {
        if passive.is_max_level() { return LevelUpMenuResult::NoResponse; }

        if let Some(currently_selected) = get_selected_passive(level_up, &player_class.passives) {
            currently_selected.current_level -= 1;
        }
        let level_up_passive = level_up.passives.get_mut(passive_name).unwrap();
        level_up_passive.current_level += 1;

        return LevelUpMenuResult::SelectedPassive;
    }
}
