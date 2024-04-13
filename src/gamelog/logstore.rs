use std::sync::Mutex;
use super::LogFragment;
use rltk::prelude::*;

lazy_static! {
    static ref LOG: Mutex<Vec<Vec<LogFragment>>> = Mutex::new(Vec::new());
}

pub fn append_fragment(fragment: LogFragment) {
    LOG.lock().unwrap().push(vec![fragment]);
}

pub fn append_entry(fragments: Vec<LogFragment>) {
    LOG.lock().unwrap().push(fragments);
}

pub fn clear_log() {
    LOG.lock().unwrap().clear();
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
    LOG.lock().unwrap().clear();
    LOG.lock().unwrap().append(log);
}
