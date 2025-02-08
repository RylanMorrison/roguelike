use rltk::prelude::*;
use super::{menu_option, menu_box};

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult { 
    NoResponse, 
    Cancel, 
    TeleportToExit,
    FullHeal,
    RevealMap,
    GodMode,
    LevelUp,
    MakeRich,
    QuestComplete,
    IncreaseAttributes
}

pub fn show_cheat_menu(ctx: &mut Rltk) -> CheatMenuResult {
    let mut draw_batch = DrawBatch::new();
    let count = 8;
    let mut y = (25 - (count / 2)) as i32;
    menu_box(&mut draw_batch, 15, y, 40, (count*2+3) as i32, "Cheating!");

    y += 1;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('T'), "Teleport to next level", None);
    y += 2;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('H'), "Heal to full", None);
    y += 2;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('R'), "Reveal the map", None);
    y += 2;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('G'), "God mode", None);
    y += 2;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('L'), "Level up", None);
    y += 2;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('M'), "Make rich", None);
    y += 2;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('Q'), "Quest complete", None);
    y += 2;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('A'), "Set attributes", None);

    draw_batch.submit(1000).expect("Draw batch submission failed");

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
                VirtualKeyCode::H => CheatMenuResult::FullHeal,
                VirtualKeyCode::R => CheatMenuResult::RevealMap,
                VirtualKeyCode::G => CheatMenuResult::GodMode,
                VirtualKeyCode::L => CheatMenuResult::LevelUp,
                VirtualKeyCode::M => CheatMenuResult::MakeRich,
                VirtualKeyCode::Q => CheatMenuResult::QuestComplete,
                VirtualKeyCode::A => CheatMenuResult::IncreaseAttributes,
                VirtualKeyCode::Escape => CheatMenuResult::Cancel,
                _ => CheatMenuResult::NoResponse
            }
        }
    }
}
