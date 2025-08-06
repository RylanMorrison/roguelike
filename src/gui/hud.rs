use specs::prelude::*;
use rltk::prelude::*;
use super::{black, blue, box_gray, cyan, draw_requirement, draw_map_tooltips, gold, green, light_gray, orange, red, white, yellow};
use crate::{carry_capacity_lbs, player, player_xp_for_level, AbilityType, ActiveQuests, Attribute, Attributes, Consumable, Duration, Entity, Equipped, HungerClock, HungerState, InBackpack, Item, KnownAbilities, KnownAbility, Map, Name, Pools, Quest, Skill, Skills, StatusEffect};
use crate::raws;
use crate::gamelog;

fn draw_borders(ecs: &World, draw_batch: &mut DrawBatch) {
    let map = ecs.fetch::<Map>();

    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 119, 99), ColorPair::new(box_gray(), black())); // Overall box
    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 88, 82), ColorPair::new(box_gray(), black())); // Map box
    draw_batch.draw_hollow_box(Rect::with_size(0, 82, 119, 17), ColorPair::new(box_gray(), black())); // Log box
    draw_batch.draw_hollow_box(Rect::with_size(88, 0, 31, 56), ColorPair::new(box_gray(), black())); // Top-right panel

    draw_batch.set(Point::new(0, 82), ColorPair::new(box_gray(), black()), to_cp437('├'));
    draw_batch.set(Point::new(88, 56), ColorPair::new(box_gray(), black()), to_cp437('├'));
    draw_batch.set(Point::new(88, 0), ColorPair::new(box_gray(), black()), to_cp437('┬'));
    draw_batch.set(Point::new(88, 82), ColorPair::new(box_gray(), black()), to_cp437('┴'));
    draw_batch.set(Point::new(119, 56), ColorPair::new(box_gray(), black()), to_cp437('┤'));
    draw_batch.set(Point::new(119, 82), ColorPair::new(box_gray(), black()), to_cp437('┤'));

    // map name
    let name_length = map.name.len();
    let x_pos = (47 - (name_length / 2)) as i32;
    draw_batch.set(Point::new(x_pos, 0), ColorPair::new(box_gray(), black()), to_cp437('┤'));
    draw_batch.set(Point::new(x_pos + name_length as i32 + 1, 0), ColorPair::new(box_gray(), black()), to_cp437('├'));
    draw_batch.print_color(Point::new(x_pos+1, 0), &map.name, ColorPair::new(white(), black()));
}

fn draw_stats(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity, y: &mut i32) {
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();

    let health = format!("{}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
    let mana = format!("{}/{}", player_pools.mana.current, player_pools.mana.max);
    let level = format!("Level: {}", player_pools.level);

    draw_batch.print_color(Point::new(90, *y), "Health: ", ColorPair::new(white(), black()));
    draw_batch.bar_horizontal(
        Point::new(100, *y),
        18,
        player_pools.hit_points.current,
        player_pools.hit_points.max,
        ColorPair::new(red(), black())
    );
    draw_batch.print_color(Point::new(106, *y), &health, ColorPair::new(white(), black()));

    *y += 1;
    draw_batch.print_color(Point::new(90, *y), "Mana: ", ColorPair::new(white(), black()));
    draw_batch.bar_horizontal(
        Point::new(100, *y),
        18,
        player_pools.mana.current,
        player_pools.mana.max,
        ColorPair::new(blue(), black())
    );
    draw_batch.print_color(Point::new(106, *y), &mana, ColorPair::new(white(), black()));

    *y += 1;
    draw_batch.print_color(Point::new(90, *y), &level, ColorPair::new(white(), black()));
    draw_batch.bar_horizontal(
        Point::new(100, *y),
        18,
        player_pools.xp,
        player_xp_for_level(player_pools.level),
        ColorPair::new(gold(), black())
    );

    *y += 1;
}

fn draw_gear_stats(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity, y: &mut i32) {
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let attributes = ecs.read_storage::<Attributes>();
    let player_attributes = attributes.get(*player_entity).unwrap();

    *y += 1;
    draw_batch.print_color(Point::new(90, *y), "Armour Class:", ColorPair::new(light_gray(), black()));
    draw_batch.print_color(Point::new(107, *y), player_pools.total_armour_class, ColorPair::new(white(), black()));

    *y += 1;
    draw_batch.print_color(Point::new(90, *y), "Base Damage:", ColorPair::new(light_gray(), black()));
    draw_batch.print_color(Point::new(107, *y), player_pools.base_damage.clone(), ColorPair::new(white(), black()));

    // weight
    *y += 2;
    let weight = player_pools.total_weight;
    let capacity = carry_capacity_lbs(&player_attributes.strength);
    let colour = if weight > capacity { red() } else { white() };
    draw_batch.print_color(
        Point::new(90, *y),
        &format!("{:0} lbs ({} lbs max)", weight, capacity),
        ColorPair::new(colour, black())
    );

    *y += 1;
    draw_batch.print_color(
        Point::new(90, *y),
        &format!("Initiative Penalty: {:.0}", player_pools.initiative_penalty.total()),
        ColorPair::new(white(), black())
    );

    // gold
    *y += 2;
    draw_batch.print_color(
        Point::new(90, *y),
        &format!("Gold: {}", player_pools.gold),
        ColorPair::new(gold(), black())
    );

    *y += 1;
}

fn draw_attributes(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity, y: &mut i32) {
    let attributes = ecs.read_storage::<Attributes>();
    let player_attributes = attributes.get(*player_entity).unwrap();

    *y += 1;
    draw_attribute("Strength:", &player_attributes.strength, *y, draw_batch);
    *y += 1;
    draw_attribute("Dexterity:", &player_attributes.dexterity, *y, draw_batch);
    *y += 1;
    draw_attribute("Constitution:", &player_attributes.constitution, *y, draw_batch);
    *y += 1;
    draw_attribute("Intelligence:", &player_attributes.intelligence, *y, draw_batch);
    *y += 1;
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, draw_batch: &mut DrawBatch) {
    draw_batch.print_color(Point::new(90, y), name, ColorPair::new(light_gray(), black()));

    let modified_colour: RGB = if attribute.total_modifiers() < 0 {
        red()
    } else if attribute.total_modifiers() == 0 {
        white()
    } else {
        green()
    };
    draw_batch.print_color(
        Point::new(107, y),
        &format!("{}", attribute.base + attribute.total_modifiers()),
        ColorPair::new(modified_colour, black())
    );

    let bonus_colour: RGB = if attribute.bonus < 0 {
        red()
    } else if attribute.bonus == 0 {
        white()
    } else {
        draw_batch.set(Point::new(112, y), ColorPair::new(green(), black()), rltk::to_cp437('+'));
        green()
    };
    draw_batch.print_color(Point::new(113, y), &format!("{}", attribute.bonus), ColorPair::new(bonus_colour, black()));
}

fn draw_skills(ecs: &World, draw_batch: &mut DrawBatch, player: &Entity, y: &mut i32) {
    let skills = ecs.read_storage::<Skills>();
    let player_skills = &skills.get(*player).unwrap();

    *y += 1;
    draw_skill("Melee:", &player_skills.melee, *y, draw_batch);
    *y += 1;
    draw_skill("Defence:", &player_skills.defence, *y, draw_batch);
    *y += 1;
    draw_skill("Ranged", &player_skills.ranged, *y, draw_batch);
    *y += 1;
    draw_skill("Magic:", &player_skills.magic, *y, draw_batch);
    *y += 1;
}

fn draw_skill(name: &str, skill: &Skill, y: i32, draw_batch: &mut DrawBatch) {
    draw_batch.print_color(Point::new(90, y), name, ColorPair::new(light_gray(), black()));
    let colour = if skill.total_modifiers() > 0 {
        green()
    } else if skill.total_modifiers() == 0 {
        white()
    } else {
        red()
    };
    draw_batch.print_color(Point::new(107, y), skill.bonus(), ColorPair::new(colour, black()));
}

fn draw_equipment(ecs: &World, draw_batch: &mut DrawBatch, player: &Entity, y: &mut i32) {
    let equipped = ecs.read_storage::<Equipped>();
    let items = ecs.read_storage::<Item>();
    *y += 1;
    for (item, equipment) in (&items, &equipped).join() {
        if equipment.owner == *player {
            draw_batch.print_color(
                Point::new(90, *y),
                item.full_name(),
                ColorPair::new(raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black())
            );
            *y += 1;
        }
    }
}

fn draw_consumables(ecs: &World, draw_batch: &mut DrawBatch, player: &Entity, y: &mut i32) {
    let backpacks = ecs.read_storage::<InBackpack>();
    let items = ecs.read_storage::<Item>();
    let consumables = ecs.read_storage::<Consumable>();

    *y += 1;
    let mut index = 1;
    for (carried_by, item, consumable) in (&backpacks, &items, &consumables).join() {
        if carried_by.owner == *player && index < 10 {
            draw_batch.print_color(Point::new(90, *y), &format!("↑{}", index), ColorPair::new(yellow(), black()));
            let mut display_name = item.full_name();
            if consumable.max_charges > 1 {
                display_name = format!("{} ({})", item.full_name(), consumable.charges);
            }
            draw_batch.print_color(
                Point::new(93, *y),
                display_name,
                ColorPair::new(raws::get_item_colour(&item, &raws::RAWS.lock().unwrap()), black())
            );
            *y += 1;
            index += 1;
        }
    }
    *y += 1;
}

fn draw_abilities(ecs: &World, draw_batch: &mut DrawBatch, player: &Entity, y: &mut i32) {
    *y += 1;
    let known_ability_lists = ecs.read_storage::<KnownAbilities>();
    let player_abilities = &known_ability_lists.get(*player).unwrap().abilities;
    let all_known_abilities = ecs.read_storage::<KnownAbility>();
    let mut index = 1;
    for ability_entity in player_abilities.iter() {
        let known_ability = all_known_abilities.get(*ability_entity).unwrap();
        if known_ability.ability_type == AbilityType::Active {
            draw_batch.print_color(Point::new(90, *y), &format!("^{}", index), ColorPair::new(cyan(), black()));
            draw_batch.print_color(
                Point::new(93, *y),
                &format!("{} ({})", known_ability.name, known_ability.mana_cost),
                ColorPair::new(cyan(), black())
            );
            index += 1;
            *y += 1;
        }
    }
}

fn draw_status_effects(ecs: &World, draw_batch: &mut DrawBatch, player: &Entity) {
    let mut y = 80;
    let names = ecs.read_storage::<Name>();
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player).unwrap();
    match hc.state {
        HungerState::WellFed => {
            draw_batch.print_color(Point::new(90, y), "Well Fed", ColorPair::new(green(), black()));
            y -= 1;
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            draw_batch.print_color(Point::new(90, y), "Hungry", ColorPair::new(orange(), black()));
            y -= 1;
        }
        HungerState::Starving => {
            draw_batch.print_color(Point::new(90, y), "Starving", ColorPair::new(red(), black()));
            y -= 1;
        }
    }
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    for (status, duration, name) in (&statuses, &durations, &names).join() {
        let fg = if status.is_debuff { red() } else { green() };
        if status.target == *player {
            draw_batch.print_color(
                Point::new(90, y),
                &format!("{} ({})", name.name, duration.turns),
                ColorPair::new(fg, black())
            );
            y -= 1;
        }
    }
}

fn quest_box_height(quests: &Vec<Quest>) -> i32 {
    let mut height = 0;
    for quest in quests.iter() {
        height += quest.requirements.len() + 3;
    }
    height as i32
}

fn draw_quests(ecs: &World, draw_batch: &mut DrawBatch) {
    let active_quests = &ecs.fetch::<ActiveQuests>().quests;
    if active_quests.len() < 1 { return; }

    let mut y = 2;
    draw_batch.draw_box(
        Rect::with_size(0, 0, 40,
            quest_box_height(active_quests)),
            ColorPair::new(box_gray(), black())
        );
    for quest in active_quests.iter() {
        draw_batch.print_color(
            Point::new(2, y),
            quest.name.clone(),
            if quest.is_complete() {
                ColorPair::new(green(), black())
            } else {
                ColorPair::new(yellow(), black())
            }
        );
        y += 1;

        let requirements = &quest.requirements;
        for requirement in requirements.iter() {
            draw_requirement(requirement, draw_batch, 4, y);
            y += 1;
        }
        y += 1;
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();
    let player_entity = ecs.fetch::<Entity>();

    draw_borders(ecs, &mut draw_batch);

    let mut y = 2;
    draw_stats(ecs, &mut draw_batch, &player_entity, &mut y);
    draw_attributes(ecs, &mut draw_batch, &player_entity, &mut y);
    draw_skills(ecs, &mut draw_batch, &player_entity, &mut y);
    draw_gear_stats(ecs, &mut draw_batch, &player_entity, &mut y);

    draw_equipment(ecs, &mut draw_batch, &player_entity, &mut y);
    draw_consumables(ecs, &mut draw_batch, &player_entity, &mut y);
    draw_abilities(ecs, &mut draw_batch, &player_entity, &mut y);
    draw_status_effects(ecs, &mut draw_batch, &player_entity);
    draw_quests(ecs, &mut draw_batch);

    gamelog::print_log(&mut rltk::BACKEND_INTERNAL.lock().consoles[1].console, Point::new(2, 35));
    draw_map_tooltips(ecs, ctx);

    draw_batch.submit(3000).expect("Draw batch submission failed");
}
