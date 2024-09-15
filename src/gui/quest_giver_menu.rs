use specs::prelude::*;
use rltk::prelude::*;
use super::{menu_box, white, black, yellow};
use crate::{State, QuestGiver, Quest, Quests, ActiveQuests, dice_range};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum QuestGiverMode { TakeOn, TurnIn }

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum QuestGiverResult {
  NoResponse,
  Cancel,
  TakeOnQuest,
  TurnInQuest,
  TakeOnQuestMode,
  TurnInQuestMode
}


pub fn show_quest_giver_menu(gs: &mut State, ctx: &mut Rltk, quest_giver: Entity, mode: QuestGiverMode) -> (QuestGiverResult, Option<Quest>) {
  match mode {
    QuestGiverMode::TakeOn => quest_take_on_menu(gs, ctx),
    QuestGiverMode::TurnIn => quest_turn_in_menu(gs, ctx)
  }
}

fn quest_take_on_menu(gs: &mut State, ctx: &mut Rltk) -> (QuestGiverResult, Option<Quest>) {
  let quests = &gs.ecs.fetch::<Quests>().quests;
  let mut draw_batch = DrawBatch::new();

  let count = quests.len() as i32;
  let mut y = 25 - (count / 2);
  menu_box(&mut draw_batch, 10, y, count*2+3, "Take on which quest? (SPACE to switch to turn in mode)");

  let mut j = 0;
  for quest in quests.iter() {
    draw_batch.set(Point::new(13, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
    draw_batch.set(Point::new(14, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
    draw_batch.set(Point::new(15, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

    draw_batch.print_color(Point::new(18, y), quest.name.clone(), ColorPair::new(white(), black()));
    y += 1;
    
    if let Some(gold) = &quest.reward.gold {
      draw_batch.print_color(Point::new(20, y), format!("Gold: {}", dice_range(&gold)), ColorPair::new(super::gold(), black()));
      y += 1;
    }

    y += 2;
    j += 1;
  }

  draw_batch.submit(1000).expect("Draw batch submission failed");

  match ctx.key {
    None => (QuestGiverResult::NoResponse, None),
    Some(key) => {
      match key {
        VirtualKeyCode::Space => (QuestGiverResult::TurnInQuestMode, None),
        VirtualKeyCode::Escape => (QuestGiverResult::Cancel, None),
        _ => {
          let selection = rltk::letter_to_option(key);
          if selection > -1 && selection < count {
            return (QuestGiverResult::TakeOnQuest, Some(quests[selection as usize].clone()))
          }
          (QuestGiverResult::NoResponse, None)
        }
      }
    }
  }
}

fn quest_turn_in_menu(gs: &mut State, ctx: &mut Rltk) -> (QuestGiverResult, Option<Quest>) {
  let active_quests = &gs.ecs.fetch::<ActiveQuests>().quests;
  let mut draw_batch = DrawBatch::new();

  let mut complete_quests: Vec<&Quest> = Vec::new();
  for quest in active_quests {
    if quest.is_complete() {
      complete_quests.push(quest);
    }
  }

  let count = complete_quests.len() as i32;
  let mut y = 25 - (count / 2);
  menu_box(&mut draw_batch, 10, y, count*2+3, "Turn in which quest? (SPACE to switch to take on mode)");

  let mut j = 0;
  for quest in complete_quests.iter() {
    draw_batch.set(Point::new(13, y), ColorPair::new(white(), black()), rltk::to_cp437('('));
    draw_batch.set(Point::new(14, y), ColorPair::new(yellow(), black()), 97+j as rltk::FontCharType);
    draw_batch.set(Point::new(15, y), ColorPair::new(white(), black()), rltk::to_cp437(')'));

    draw_batch.print_color(Point::new(18, y), quest.name.clone(), ColorPair::new(white(), black()));
    y += 1;

    if let Some(gold) = &quest.reward.gold {
      draw_batch.print_color(Point::new(20, y), format!("Gold: {}", dice_range(&gold)), ColorPair::new(super::gold(), black()));
      y += 1;
    }

    y += 2;
    j += 1;
  }

  draw_batch.submit(1000).expect("Draw batch submission failed");

  match ctx.key {
    None => (QuestGiverResult::NoResponse, None),
    Some(key) => {
      match key {
        VirtualKeyCode::Space => (QuestGiverResult::TakeOnQuestMode, None),
        VirtualKeyCode::Escape => (QuestGiverResult::Cancel, None),
        _ => {
          let selection = rltk::letter_to_option(key);
          if selection > -1 && selection < count {
            return (QuestGiverResult::TurnInQuest, Some(complete_quests[selection as usize].clone()))
          }
          (QuestGiverResult::NoResponse, None)
        }
      }
    }
  }
}

