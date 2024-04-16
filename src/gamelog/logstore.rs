use std::sync::Mutex;
use super::LogFragment;
use rltk::prelude::*;

lazy_static! {
    static ref LOG: Mutex<Vec<Vec<LogFragment>>> = Mutex::new(Vec::new());
}

pub fn append_entry(fragments: Vec<LogFragment>) {
    LOG.lock().unwrap().push(fragments);
}

pub fn clear_log() {
    LOG.lock().unwrap().clear();
}

pub fn print_log(console: &mut Box<dyn Console>, pos: Point) {
    let mut x = pos.x;
    let mut y = pos.y;
    LOG.lock().unwrap().iter().rev().take(6).for_each(|log| {
        log.iter().for_each(|frag| {
            console.print_color(x, y, frag.colour.to_rgba(1.0), RGBA::named(rltk::BLACK), &frag.text);
            x += frag.text.len() as i32;
            x += 1;
        });
        y += 1;
        x = pos.x;
    });
}

pub fn log_display() -> TextBuilder {
    let mut buffer = TextBuilder::empty();

    LOG.lock().unwrap().iter().rev().take(12).for_each(|log| {
        log.iter().for_each(|frag| {
            buffer.fg(frag.colour);
            buffer.line_wrap(&frag.text);
        });
        buffer.ln();
    });

    buffer
}

pub fn clone_log() -> Vec<Vec<LogFragment>> {
    LOG.lock().unwrap().clone()
}

pub fn restore_log(log: &mut Vec<Vec<LogFragment>>) {
    clear_log();
    LOG.lock().unwrap().append(log);
}
