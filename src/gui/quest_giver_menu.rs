use specs::prelude::*;
use rltk::prelude::*;
use super::{white, black, yellow, green, cyan};
use crate::{dice_range, ActiveQuests, Name, Quest, QuestRequirement, QuestRequirementGoal, Quests, State};
use crate::gamelog;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum QuestGiverResult {
  NoResponse,
  Cancel,
  TakeOnQuest,
  TurnInQuest,
  ShowPreviousQuest,
  ShowNextQuest
}

pub fn draw_requirement(requirement: &QuestRequirement, draw_batch: &mut DrawBatch, y: i32) {
  let color = if requirement.complete {
    ColorPair::new(green(), black())
  } else {
    ColorPair::new(white(), black())
  };

  match requirement.requirement_goal {
    QuestRequirementGoal::KillCount => {
      if requirement.targets.len() > 1 {
        let mut text = format!("{}/{} {}", requirement.count, requirement.target_count, requirement.targets.first().unwrap());
        for target in requirement.targets.iter().skip(1) {
          text += format!("/{}", target).as_str();
        }
        text += " kills";

        draw_batch.print_color(Point::new(2, y), text, color);
      } else {
        draw_batch.print_color(
          Point::new(2, y),
          format!("{}/{} {} kills", requirement.count, requirement.target_count, requirement.targets.first().unwrap()),
          color
        );
      }
    }
    _ => {}
  }
}

pub fn show_quest_giver_menu(gs: &mut State, ctx: &mut Rltk, quest_giver: Entity, index: i32) -> QuestGiverResult {
  let quests = &gs.ecs.fetch::<Quests>().quests;
  let active_quests = &gs.ecs.fetch::<ActiveQuests>().quests;
  let current_quest = quests.get(index as usize).unwrap();
  let names = gs.ecs.read_storage::<Name>();
  let mut draw_batch = DrawBatch::new();
  let max_index = (quests.len() - 1) as i32;
  let mut current_active_quest: Option<Quest> = None;

  for quest in active_quests {
    if quest.name == current_quest.name {
      current_active_quest = Some(quest.clone());
      break;
    }
  }

  draw_batch.draw_box(Rect::with_size(0, 0, 99, 79), ColorPair::new(white(), black()));
  draw_batch.print_color(Point::new(2, 2),
    names.get(quest_giver).unwrap().name.clone(),
    ColorPair::new(yellow(), black())
  );
  draw_batch.print_color(Point::new(2, 6), current_quest.name.clone(), ColorPair::new(yellow(), black()));

  let mut y = 8;
  if current_active_quest.is_some() {
    draw_batch.print_color(Point::new(2, y), "ACTIVE", ColorPair::new(green(), black()));
    y += 2;
  }
  // TODO: text wrapping
  draw_batch.print_color(Point::new(2, y), current_quest.description.clone(), ColorPair::new(white(), black()));
  y += 4;

  if let Some(quest) = &current_active_quest {
    draw_batch.print_color(Point::new(2, y), "Requirements:", ColorPair::new(yellow(), black()));
    y += 2;

    for requirement in quest.requirements.iter() {
      draw_requirement(requirement, &mut draw_batch, y);
      y += 1;
    }

    y += 4;
  }

  draw_batch.print_color(Point::new(2, y), "Rewards:", ColorPair::new(yellow(), black()));
  y += 2;

  for reward in current_quest.rewards.iter() {
    if let Some(gold) = &reward.gold {
      draw_batch.print_color(
        Point::new(6, y),
        format!("Gold: {}", dice_range(&gold)),
        ColorPair::new(super::gold(), black())
      ); y += 2;
    }
    if let Some(xp) = &reward.xp {
      draw_batch.print_color(
        Point::new(6, y),
        format!("XP: {}", xp),
        ColorPair::new(cyan(), black())
      ); y += 2;
    }
  }

  y += 4;

  if let Some(quest) = &current_active_quest {
    if quest.is_complete() {
      draw_batch.print_color(Point::new(2, y), "(t)", ColorPair::new(yellow(), black()));
      draw_batch.print_color(Point::new(6, y), "Turn in", ColorPair::new(white(), black()));
    }
  } else if current_active_quest.is_none() {
    draw_batch.print_color(Point::new(2, y), "(t)", ColorPair::new(yellow(), black()));
    draw_batch.print_color(Point::new(6, y), "Take on", ColorPair::new(white(), black()));
  }

  if index > 0 {
    draw_batch.print_color(Point::new(57, 77), "(p)", ColorPair::new(yellow(), black()));
    draw_batch.print_color(Point::new(61, 77), "Previous quest", ColorPair::new(white(), black()));
  }
  if index < max_index {
    draw_batch.print_color(Point::new(81, 77), "(n)", ColorPair::new(yellow(), black()));
    draw_batch.print_color(Point::new(85, 77), "Next quest", ColorPair::new(white(), black()));
  }

  gamelog::clear_log();
  draw_batch.submit(5000).expect("Draw batch submission failed");

  match ctx.key {
    None => QuestGiverResult::NoResponse,
    Some(key) => {
      match key {
        VirtualKeyCode::Escape => QuestGiverResult::Cancel,
        VirtualKeyCode::T => {
          if let Some(quest) = &current_active_quest {
            if quest.is_complete() {
              QuestGiverResult::TurnInQuest
            } else {
              QuestGiverResult::NoResponse
            }
          } else {
            QuestGiverResult::TakeOnQuest
          }
        }
        VirtualKeyCode::P => {
          if index > 0 {
            QuestGiverResult::ShowPreviousQuest
          } else {
            QuestGiverResult::NoResponse
          }
        }
        VirtualKeyCode::N => {
          if index < max_index {
            QuestGiverResult::ShowNextQuest
          } else {
            QuestGiverResult::NoResponse
          }
        }
        _ => {
          QuestGiverResult::NoResponse
        }
      }
    }
  }
}
